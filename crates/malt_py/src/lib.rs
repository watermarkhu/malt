//! Python bindings for `malt-index`: the MATLAB search path, the functions, classes,
//! properties, arguments and live scripts it defines, as the module `malt_index`.
//!
//! This layer only converts types, raises errors and forwards log records. The index, the
//! parser, the path rules, the live-script readers and the class hierarchy are all in
//! `malt-index`. The API (`PathsCollection`, `LinesCollection`, `Object`, `Class`, `Folder`,
//! `Namespace`, `Script`, `LiveScript`, `Function`, `Property`, `Enumeration`, `Expr`,
//! `ParserConfig`, the exceptions) is documented at <https://watermarkhu.nl/malt/api/python>.

mod collection;
mod config;
mod lines;
mod logging;
mod objects;

use std::path::{Path, PathBuf};

use malt_index::{Index, IndexError, ParserConfig as IndexParserConfig};
use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use collection::PathsCollection;
use config::ParserConfig;
use objects::{Argument, Class, Enumeration, Expr, Function, LiveScript, LiveScriptSection};
use objects::{Folder, Namespace, Object, Property, Script};

create_exception!(
    malt_index,
    FilePathError,
    pyo3::exceptions::PyException,
    "A file or folder cannot be read."
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
    "The superclasses of a class lead back to the class."
);

/// The Python exception for an error of the index.
pub(crate) fn index_error(err: IndexError) -> PyErr {
    match err {
        IndexError::Io { .. } => FilePathError::new_err(err.to_string()),
        IndexError::Format { .. } => PyValueError::new_err(err.to_string()),
        IndexError::InvalidIdentifier(_) => NameResolutionError::new_err(err.to_string()),
        IndexError::CyclicBase(_) => cyclic_alias_error(err),
        IndexError::Grammar(_) => PyRuntimeError::new_err(err.to_string()),
        IndexError::Config(_) => PyValueError::new_err(err.to_string()),
    }
}

/// The `CyclicAliasError` for a cycle of superclasses.
pub(crate) fn cyclic_alias_error(err: IndexError) -> PyErr {
    CyclicAliasError::new_err(err.to_string())
}

/// A collection over one file's folder, so that the objects of a file parsed alone can
/// look up their superclasses there. The folder is not parsed beyond its listing.
fn standalone(
    py: Python<'_>,
    entity: malt_index::Entity,
    path: &Path,
    config: IndexParserConfig,
) -> PyResult<Py<PyAny>> {
    let mut index = Index::new(config);
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        // An unreadable folder only loses the superclasses: the file itself is parsed.
        let _ = index.addpath(dir, true, false);
    }
    let collection = Py::new(py, PathsCollection::from_index(index)?)?;
    let name = entity.name.clone();
    objects::wrap(py, entity, name, collection, None, false)
}

/// Parse one MATLAB file. Returns its object (a `Function`, `Class`, `Script` or
/// `LiveScript`, as the file declares), with no parent. A `.mlx` is read as a live script.
/// Raises `FilePathError` when the file cannot be read.
#[pyfunction]
#[pyo3(signature = (path, parser_config=None))]
fn parse_file(
    py: Python<'_>,
    path: PathBuf,
    parser_config: Option<PyRef<'_, ParserConfig>>,
) -> PyResult<Py<PyAny>> {
    let config = parser_config.map(|c| c.to_index()).unwrap_or_default();
    let entity = malt_index::parse_file(&path, &config).map_err(index_error)?;
    standalone(py, entity, &path, config)
}

/// Parse MATLAB source text, as the file `path` (the name comes from `path`; the file is
/// not read). Returns the object, as `parse_file` does.
#[pyfunction]
#[pyo3(signature = (source, path, parser_config=None))]
fn parse_source(
    py: Python<'_>,
    source: &str,
    path: PathBuf,
    parser_config: Option<PyRef<'_, ParserConfig>>,
) -> PyResult<Py<PyAny>> {
    let config = parser_config.map(|c| c.to_index()).unwrap_or_default();
    let entity =
        malt_index::parse_source(&path, source.as_bytes(), &config).map_err(index_error)?;
    standalone(py, entity, &path, config)
}

/// Read a live script: a `.mlx` (binary, or plain text when it is not a zip archive), or
/// with `plaintext` a file read as plain-text live code (R2025a `.m`), whatever its name.
/// Returns a `LiveScript` object. Raises `FilePathError` or `ValueError`.
#[pyfunction]
#[pyo3(signature = (path, plaintext=false))]
fn parse_live_script(py: Python<'_>, path: PathBuf, plaintext: bool) -> PyResult<Py<PyAny>> {
    let entity = if plaintext {
        malt_index::parse_live_text_file(&path)
    } else {
        malt_index::parse_live_file(&path)
    }
    .map_err(index_error)?;
    let config = IndexParserConfig::default();
    standalone(py, entity, &path, config)
}

/// An enumeration of strings (`str`-valued), in the module.
fn str_enum<'py>(
    py: Python<'py>,
    name: &str,
    members: &[(&str, &str)],
) -> PyResult<Bound<'py, PyAny>> {
    let items = PyList::new(py, members.iter().map(|(member, value)| (*member, *value)))?;
    let kwargs = PyDict::new(py);
    kwargs.set_item("type", py.import("builtins")?.getattr("str")?)?;
    kwargs.set_item("module", "malt_index")?;
    py.import("enum")?
        .getattr("Enum")?
        .call((name, items), Some(&kwargs))
}

#[pymodule]
#[pyo3(name = "malt_index")]
fn malt_index_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    logging::install(py)?;

    m.add_class::<ParserConfig>()?;
    m.add_class::<PathsCollection>()?;
    m.add_class::<lines::LinesCollection>()?;
    m.add_class::<Object>()?;
    m.add_class::<Folder>()?;
    m.add_class::<Namespace>()?;
    m.add_class::<Script>()?;
    m.add_class::<LiveScript>()?;
    m.add_class::<LiveScriptSection>()?;
    m.add_class::<Class>()?;
    m.add_class::<Function>()?;
    m.add_class::<Property>()?;
    m.add_class::<Enumeration>()?;
    m.add_class::<Argument>()?;
    m.add_class::<Expr>()?;
    m.add_function(wrap_pyfunction!(parse_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_source, m)?)?;
    m.add_function(wrap_pyfunction!(parse_live_script, m)?)?;

    m.add(
        "Kind",
        str_enum(
            py,
            "Kind",
            &[
                ("FOLDER", "folder"),
                ("NAMESPACE", "namespace"),
                ("CLASS", "class"),
                ("ENUMERATION", "enumeration"),
                ("FUNCTION", "function"),
                ("SCRIPT", "script"),
                ("LIVE_SCRIPT", "live_script"),
                ("PROPERTY", "property"),
            ],
        )?,
    )?;
    m.add(
        "ArgumentKind",
        str_enum(
            py,
            "ArgumentKind",
            &[
                ("positional_only", "positional-only"),
                ("optional", "optional"),
                ("keyword_only", "keyword-only"),
                ("varargin", "varargin"),
            ],
        )?,
    )?;
    m.add(
        "AccessKind",
        str_enum(
            py,
            "AccessKind",
            &[
                ("public", "public"),
                ("protected", "protected"),
                ("private", "private"),
                ("immutable", "immutable"),
            ],
        )?,
    )?;
    m.add(
        "SectionKind",
        str_enum(py, "SectionKind", &[("TEXT", "text"), ("CODE", "code")])?,
    )?;

    m.add("FilePathError", py.get_type::<FilePathError>())?;
    m.add("NameResolutionError", py.get_type::<NameResolutionError>())?;
    m.add("CyclicAliasError", py.get_type::<CyclicAliasError>())?;
    Ok(())
}
