//! Parser configuration: where the docstring of a declaration may sit, and whether
//! live scripts on the path are read.

use serde::{Deserialize, Serialize};

/// Which comment block belongs to a declaration, and what the path holds.
///
/// By default (`false`) a comment block after a property, argument or enumeration
/// documents the declaration before it, as maxx does. With `true` the block before
/// the declaration documents it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ParserConfig {
    /// Comment blocks before a property document that property (default: after).
    pub docstring_before_properties: bool,
    /// Comment blocks before an argument document that argument (default: after).
    pub docstring_before_arguments: bool,
    /// Comment blocks before an enumeration member document that member (default: after).
    pub docstring_before_enumerations: bool,
    /// Whether `.mlx` files on the path are live scripts (default: no, as maxx).
    /// Plain-text live code is never picked up from its `.m` file: it is read with
    /// [`crate::parse_live_text_file`] when the caller knows it is one.
    pub parse_live_scripts: bool,
}
