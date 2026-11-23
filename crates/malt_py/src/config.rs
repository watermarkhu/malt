//! `ParserConfig`: which comment block documents a declaration, and whether `.mlx` files on
//! the path are live scripts (see malt-index).

use malt_index::ParserConfig as IndexParserConfig;
use pyo3::prelude::*;

/// Parser options. By default a comment block after a property, argument or enumeration
/// member documents the declaration before it (as maxx does); with the flag set, the block
/// before the declaration documents it. `parse_live_scripts` makes `.mlx` files on the path
/// live scripts (off by default, as maxx).
#[pyclass(module = "malt_index")]
pub struct ParserConfig {
    /// Comment blocks before a property document that property (default: after).
    #[pyo3(get, set)]
    pub docstring_before_properties: bool,
    /// Comment blocks before an argument document that argument (default: after).
    #[pyo3(get, set)]
    pub docstring_before_arguments: bool,
    /// Comment blocks before an enumeration member document that member (default: after).
    #[pyo3(get, set)]
    pub docstring_before_enumerations: bool,
    /// Whether `.mlx` files are live scripts.
    #[pyo3(get, set)]
    pub parse_live_scripts: bool,
}

#[pymethods]
impl ParserConfig {
    #[new]
    #[pyo3(signature = (
        docstring_before_properties=false,
        docstring_before_arguments=false,
        docstring_before_enumerations=false,
        parse_live_scripts=false,
    ))]
    fn new(
        docstring_before_properties: bool,
        docstring_before_arguments: bool,
        docstring_before_enumerations: bool,
        parse_live_scripts: bool,
    ) -> Self {
        Self {
            docstring_before_properties,
            docstring_before_arguments,
            docstring_before_enumerations,
            parse_live_scripts,
        }
    }
}

impl ParserConfig {
    /// The configuration of the index.
    pub(crate) fn to_index(&self) -> IndexParserConfig {
        IndexParserConfig {
            docstring_before_properties: self.docstring_before_properties,
            docstring_before_arguments: self.docstring_before_arguments,
            docstring_before_enumerations: self.docstring_before_enumerations,
            parse_live_scripts: self.parse_live_scripts,
        }
    }
}
