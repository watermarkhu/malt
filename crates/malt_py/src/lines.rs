//! `LinesCollection`: the source lines of the files a collection reads, by path.

use std::path::PathBuf;
use std::sync::Arc;

use malt_index::Sources;
use pyo3::prelude::*;

use crate::index_error;

/// The source lines of files, read from disk on first use and cached by path (the
/// collection of maxx's `lines_collection`). Lines are split on `\n`, a `\r` before it is
/// removed, and a final newline adds no empty line.
#[pyclass(module = "malt_index", frozen)]
pub struct LinesCollection {
    sources: Arc<Sources>,
}

/// The Python object over a source cache.
pub(crate) fn wrap(py: Python<'_>, sources: Arc<Sources>) -> PyResult<Py<LinesCollection>> {
    Py::new(py, LinesCollection { sources })
}

#[pymethods]
impl LinesCollection {
    /// The lines of a file (a `str` or a `pathlib.Path`). Raises `FilePathError` when the
    /// file cannot be read.
    fn lines(&self, path: PathBuf) -> PyResult<Vec<String>> {
        self.sources
            .lines(&path)
            .map(|lines| lines.to_vec())
            .map_err(index_error)
    }

    fn __getitem__(&self, path: PathBuf) -> PyResult<Vec<String>> {
        self.lines(path)
    }

    /// Whether the lines of a file have been read (by `lines`, or an object's `lines`).
    fn __contains__(&self, path: PathBuf) -> bool {
        self.sources.is_cached(path)
    }

    /// The paths whose lines have been read, sorted.
    fn keys(&self) -> Vec<String> {
        self.sources
            .cached_paths()
            .into_iter()
            .map(|p| p.display().to_string())
            .collect()
    }
}
