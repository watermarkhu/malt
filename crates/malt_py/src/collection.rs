//! `PathsCollection`: the MATLAB search path as a Python object.

use std::path::PathBuf;

use malt_index::Index;
use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::config::ParserConfig;
use crate::objects::wrap;
use crate::{index_error, NameResolutionError};

/// The MATLAB search path: the folders in precedence order, and what they define.
///
/// `paths` are folders, read as written (plain folders, not globs). A relative path
/// resolves against `working_directory` (the current directory by default). With
/// `recursive`, the plain subfolders of each path join too, as MATLAB's `genpath` does.
#[pyclass(module = "malt_index")]
pub struct PathsCollection {
    index: Index,
    working_directory: PathBuf,
}

#[pymethods]
impl PathsCollection {
    #[new]
    #[pyo3(signature = (paths, recursive=false, working_directory=None, parser_config=None))]
    fn new(
        paths: Vec<PathBuf>,
        recursive: bool,
        working_directory: Option<PathBuf>,
        parser_config: Option<PyRef<'_, ParserConfig>>,
    ) -> PyResult<Self> {
        let current = std::env::current_dir()?;
        let working_directory = match working_directory {
            Some(dir) => current.join(dir),
            None => current,
        };
        let config = parser_config.map(|c| c.to_index()).unwrap_or_default();
        let mut index = Index::new(config);
        for path in paths {
            index
                .addpath(working_directory.join(path), true, recursive)
                .map_err(index_error)?;
        }
        Ok(Self {
            index,
            working_directory,
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
            let object = wrap(py, entity, path, collection.clone_ref(py), None)?;
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

    /// The object of `identifier` (`fn`, `Cls`, `pkg.fn`, `+pkg.Cls`, `Cls.method`), or
    /// `None` when nothing on the path matches.
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
    /// The index behind the collection (for the objects of this crate).
    pub(crate) fn index(&self) -> &Index {
        &self.index
    }
}

/// The object for `identifier`, looked up from `dir` (relative to the working directory)
/// when it is given. The object's path is the identifier without a leading `+`.
fn lookup(
    slf: &Bound<'_, PathsCollection>,
    identifier: &str,
    dir: Option<PathBuf>,
) -> PyResult<Option<Py<PyAny>>> {
    let py = slf.py();
    let found = {
        let this = slf.borrow();
        match dir {
            Some(dir) => this
                .index
                .get_from(identifier, &this.working_directory.join(dir)),
            None => this.index.get(identifier),
        }
        .map_err(index_error)?
    };
    let path = identifier.trim_start_matches('+').to_string();
    found
        .map(|entity| wrap(py, entity, path, slf.clone().unbind(), None))
        .transpose()
}
