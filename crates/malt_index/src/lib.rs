//! A native index of the MATLAB path: the functions, classes, scripts, packages
//! (`+pkg`), class folders (`@cls`) and `private` folders that MATLAB resolves names
//! against. It is the Rust core of the replacement for the Python library maxx; it
//! has no linter dependency.
//!
//! - [`Index`] lists a search path up front and parses files lazily.
//! - [`parse_file`] and [`parse_source`] turn one file into an [`Entity`].
//! - [`dump::dump`] writes the index in the schema of `maxx_dump.py`.
//!
//! The behaviour, the schema and the differences from maxx are in the crate's README.

pub mod comments;
pub mod config;
pub mod dump;
pub mod error;
pub mod index;
pub mod model;
pub mod parse;
pub mod paths;

pub use config::ParserConfig;
pub use error::IndexError;
pub use index::{EntryId, Index};
pub use model::{
    Argument, ArgumentKind, Detail, Docstring, Entity, FunctionDetail, Kind, PropertyDetail,
};
pub use parse::{parse_file, parse_source};
pub use paths::{resolve_paths, PathConfig};
