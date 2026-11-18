//! A native index of the MATLAB path: the functions, classes, scripts, live scripts,
//! packages (`+pkg`), class folders (`@cls`), plain folders and `private` folders that
//! MATLAB resolves names against. It is the Rust core of the replacement for the Python
//! library maxx; it has no linter dependency.
//!
//! - [`Index`] lists a search path up front and parses files lazily.
//! - [`parse_file`] and [`parse_source`] turn one file into an [`Entity`].
//! - [`parse_live_file`], [`parse_live_text_file`] and [`parse_live_bytes`] read live
//!   scripts (`.mlx` and plain-text live code).
//! - [`Sources`] reads the source lines of files, cached by path.
//! - [`dump::dump`] writes the index in the schema of `maxx_dump.py`.
//!
//! The behaviour, the schema and the differences from maxx are in the crate's README.

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
