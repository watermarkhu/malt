//! Python bindings for `malt-index`: the MATLAB search path and the functions, classes,
//! properties and arguments it defines, as the module `malt_index`.
//!
//! This layer only converts types and raises errors. The index, the parser, the path
//! rules and the class hierarchy are all in `malt-index`. The names follow maxx's
//! (`PathsCollection`, `Object`, `Class`, `Function`, `Property`, `Argument`,
//! `ParserConfig`, the exceptions) where the first API has them; the README lists what
//! is still missing.

mod collection;
mod config;
mod objects;

use malt_index::IndexError;
use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

create_exception!(
    malt_index,
    FilePathError,
    pyo3::exceptions::PyException,
    "A folder on the search path cannot be read."
);
create_exception!(
    malt_index,
    NameResolutionError,
    pyo3::exceptions::PyException,
    "A name is not on the search path."
);
create_exception!(
    malt_index,
    CyclicAliasError,
    pyo3::exceptions::PyException,
    "An alias refers to itself. Aliases are not provided yet, so this is never raised."
);

/// The Python exception for an error of the index.
pub(crate) fn index_error(err: IndexError) -> PyErr {
    match err {
        IndexError::Io { .. } => FilePathError::new_err(err.to_string()),
        IndexError::InvalidIdentifier(_) => NameResolutionError::new_err(err.to_string()),
        IndexError::Grammar(_) => PyRuntimeError::new_err(err.to_string()),
        IndexError::Config(_) => PyValueError::new_err(err.to_string()),
    }
}

#[pymodule]
#[pyo3(name = "malt_index")]
fn malt_index_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<config::ParserConfig>()?;
    m.add_class::<collection::PathsCollection>()?;
    m.add_class::<objects::Object>()?;
    m.add_class::<objects::Class>()?;
    m.add_class::<objects::Function>()?;
    m.add_class::<objects::Property>()?;
    m.add_class::<objects::Argument>()?;

    let py = m.py();
    m.add("FilePathError", py.get_type::<FilePathError>())?;
    m.add("NameResolutionError", py.get_type::<NameResolutionError>())?;
    m.add("CyclicAliasError", py.get_type::<CyclicAliasError>())?;
    Ok(())
}
