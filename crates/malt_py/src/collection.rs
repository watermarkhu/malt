//! `PathsCollection`: the MATLAB search path as a Python object.

use std::path::PathBuf;
use std::sync::Arc;

use malt_index::{Index, Kind, Sources};
use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::config::ParserConfig;
use crate::lines::LinesCollection;
use crate::objects::wrap;
use crate::{index_error, NameResolutionError};

/// The MATLAB search path: the folders in precedence order, and what they define.
///
/// `paths` are folders, relative paths resolve against `working_directory` (the current
/// directory by default). With `recursive`, the plain subfolders of each path join too, as
/// MATLAB's `genpath` does. `parse_live_scripts` makes `.mlx` files live scripts.
#[pyclass(module = "malt_index")]
pub struct PathsCollection {
    index: Index,
    working_directory: PathBuf,
    sources: Arc<Sources>,
}

#[pymethods]
impl PathsCollection {
    #[new]
    #[pyo3(signature = (paths, recursive=false, working_directory=None, parser_config=None, parse_live_scripts=false))]
    fn new(
        paths: Vec<PathBuf>,
        recursive: bool,
        working_directory: Option<PathBuf>,
        parser_config: Option<PyRef<'_, ParserConfig>>,
        parse_live_scripts: bool,
    ) -> PyResult<Self> {
        let current = std::env::current_dir()?;
        let working_directory = match working_directory {
            Some(dir) => current.join(dir),
            None => current,
        };
        let mut config = parser_config.map(|c| c.to_index()).unwrap_or_default();
        config.parse_live_scripts |= parse_live_scripts;
        let mut index = Index::new(config);
        for path in paths {
            index
                .addpath(working_directory.join(path), true, recursive)
                .map_err(index_error)?;
        }
        Ok(Self {
            index,
            working_directory,
            sources: Arc::new(Sources::new()),
        })
    }

    /// The top-level names, as maxx's collection lists them: each top-level name, the
    /// members of packages under dotted names (`pkg.fn`), and the methods of class folders
    /// under `Cls.method`. A package is keyed `+pkg`.
    #[getter]
    fn members<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        let py = slf.py();
        let entries = slf.borrow().index.members().map_err(index_error)?;
        let collection = slf.clone().unbind();
        let dict = PyDict::new(py);
        for (name, entity) in entries {
            let path = name.trim_start_matches('+').to_string();
            let object = wrap(py, entity, path, collection.clone_ref(py), None, false)?;
            dict.set_item(name, object)?;
        }
        Ok(dict)
    }

    /// The folders on the search path, in precedence order.
    #[getter]
    fn search_path(&self) -> Vec<String> {
        self.index
            .search_path()
            .iter()
            .map(|p| p.display().to_string())
            .collect()
    }

    /// The source lines of the files this collection's objects came from.
    #[getter]
    fn lines_collection(&self, py: Python<'_>) -> PyResult<Py<LinesCollection>> {
        crate::lines::wrap(py, Arc::clone(&self.sources))
    }

    /// The object of `identifier` (`fn`, `Cls`, `pkg.fn`, `+pkg.Cls`, `Cls.method`), or a
    /// path (`sub/fn.m`, relative to the working directory, or absolute) for a folder on the
    /// path, or below one, or for a file in such a folder. `None` when nothing matches.
    fn get(slf: &Bound<'_, Self>, identifier: &str) -> PyResult<Option<Py<PyAny>>> {
        lookup(slf, identifier, None)
    }

    /// Like `get`, but a plain name first looks at the files next to `from_dir` and in its
    /// `private` folder, as MATLAB does for a file in that folder. Raises
    /// `NameResolutionError` when nothing matches.
    #[pyo3(signature = (identifier, from_dir=None))]
    fn resolve(
        slf: &Bound<'_, Self>,
        identifier: &str,
        from_dir: Option<PathBuf>,
    ) -> PyResult<Py<PyAny>> {
        lookup(slf, identifier, from_dir)?.ok_or_else(|| {
            NameResolutionError::new_err(format!("{identifier} is not on the search path"))
        })
    }

    fn __getitem__(slf: &Bound<'_, Self>, identifier: &str) -> PyResult<Py<PyAny>> {
        lookup(slf, identifier, None)?.ok_or_else(|| PyKeyError::new_err(identifier.to_string()))
    }

    fn __contains__(slf: &Bound<'_, Self>, identifier: &str) -> PyResult<bool> {
        Ok(lookup(slf, identifier, None)?.is_some())
    }

    /// Add a folder to the front of the search path, as MATLAB's `addpath` does, or to the
    /// end with `to_end`. With `recursive`, its plain subfolders join too. A folder that is
    /// already on the path moves to the new position. Raises `FilePathError` when the
    /// folder cannot be read.
    #[pyo3(signature = (path, *, to_end=false, recursive=false))]
    fn addpath(&mut self, path: PathBuf, to_end: bool, recursive: bool) -> PyResult<()> {
        self.index
            .addpath(self.working_directory.join(path), to_end, recursive)
            .map_err(index_error)
    }

    /// Remove a folder from the search path, and with `recursive` the folders below it.
    /// Returns whether it was on the path.
    #[pyo3(signature = (path, recursive=false))]
    fn rmpath(&mut self, path: PathBuf, recursive: bool) -> bool {
        self.index
            .rmpath(self.working_directory.join(path), recursive)
    }
}

impl PathsCollection {
    /// A collection over `index`, relative paths resolving against the current directory.
    pub(crate) fn from_index(index: Index) -> PyResult<Self> {
        Ok(Self {
            index,
            working_directory: std::env::current_dir()?,
            sources: Arc::new(Sources::new()),
        })
    }

    /// The index behind the collection (for the objects of this crate).
    pub(crate) fn index(&self) -> &Index {
        &self.index
    }

    /// The source-line cache behind the collection.
    pub(crate) fn sources(&self) -> Arc<Sources> {
        Arc::clone(&self.sources)
    }
}

/// Whether `identifier` is a file or folder path (it has a separator), not a name:
/// `/` everywhere, and `\` too on Windows (`\` is a name character nowhere else).
fn is_path_identifier(identifier: &str) -> bool {
    identifier.contains('/') || (cfg!(windows) && identifier.contains('\\'))
}

/// The object for `identifier`, looked up from `dir` (relative to the working directory)
/// when it is given. The object's path is the identifier without a leading `+`. A path
/// lookup (an identifier with `/`) is resolved as maxx does: an absolute path stays as it
/// is, a relative one joins the working directory. A folder's path is `/name`, and a file's
/// is the identifier as written.
fn lookup(
    slf: &Bound<'_, PathsCollection>,
    identifier: &str,
    dir: Option<PathBuf>,
) -> PyResult<Option<Py<PyAny>>> {
    let py = slf.py();
    let found = {
        let this = slf.borrow();
        if is_path_identifier(identifier) {
            let candidate = this.working_directory.join(identifier);
            this.index
                .get_path(&candidate)
                .map_err(index_error)?
                .map(|entity| {
                    let path = if entity.kind == Kind::Folder {
                        format!("/{}", entity.name)
                    } else {
                        identifier.to_string()
                    };
                    (entity, path)
                })
        } else {
            let entity = match dir {
                Some(dir) => this
                    .index
                    .get_from(identifier, &this.working_directory.join(dir)),
                None => this.index.get(identifier),
            }
            .map_err(index_error)?;
            entity.map(|e| (e, identifier.trim_start_matches('+').to_string()))
        }
    };
    found
        .map(|(entity, path)| wrap(py, entity, path, slf.clone().unbind(), None, false))
        .transpose()
}
