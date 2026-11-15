//! Parser configuration: where the docstring of a declaration may sit.

use serde::{Deserialize, Serialize};

/// Which comment block belongs to a declaration.
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
}
