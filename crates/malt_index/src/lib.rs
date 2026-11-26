//! A native index of the MATLAB path: the functions, classes, scripts, live scripts,
//! packages (`+pkg`), class folders (`@cls`), plain folders and `private` folders that
//! MATLAB resolves names against. It is the Rust core of malt's path index, and it has no
//! linter dependency.
//!
//! - [`Index`] lists a search path up front and parses files lazily.
//! - [`parse_file`] and [`parse_source`] turn one file into an [`Entity`].
//! - [`parse_live_file`], [`parse_live_text_file`] and [`parse_live_bytes`] read live
//!   scripts (`.mlx` and plain-text live code).
//! - [`Sources`] reads the source lines of files, cached by path.
//! - [`dump::dump`] writes the index as JSON.
//!
//! How MATLAB is read, the entities and the JSON schema are in the documentation at
//! <https://watermarkhu.nl/malt/api/>.

pub mod comments;
pub mod config;
pub mod dump;
pub mod error;
pub mod expr;
pub mod folder;
pub mod fsutil;
pub mod hierarchy;
pub mod index;
pub mod live;
pub mod model;
pub mod parse;
pub mod paths;
pub mod source;

pub use config::ParserConfig;
pub use error::IndexError;
pub use expr::Expr;
pub use hierarchy::{Ancestor, Inherited};
pub use index::{EntryId, Index};
pub use live::{parse_live_bytes, parse_live_file, parse_live_text_file};
pub use model::{
    Argument, ArgumentKind, Detail, Docstring, Entity, FunctionDetail, Kind, LiveSection,
    PropertyDetail, SectionKind,
};
pub use parse::{parse_file, parse_source};
pub use paths::{resolve_paths, PathConfig};
pub use source::Sources;
