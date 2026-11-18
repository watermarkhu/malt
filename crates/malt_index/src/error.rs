//! Errors of the index. Parsing MATLAB syntax never fails: tree-sitter recovers and the
//! parts that are not understood are skipped. A file that is not a readable live script
//! (a `.mlx` that is not a zip archive, or has no document) is a [`IndexError::Format`].

use std::path::PathBuf;

use thiserror::Error;

/// An error from reading the path or a file.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IndexError {
    /// A file or folder could not be read.
    #[error("cannot read {path}: {message}")]
    Io {
        /// The path that failed.
        path: PathBuf,
        /// The underlying error, as text (keeps the error cloneable for the cache).
        message: String,
    },
    /// A file's contents are not in the format its extension promises (a `.mlx` that is
    /// not a zip archive, or one without `matlab/document.xml`).
    #[error("{path}: {message}")]
    Format {
        /// The file.
        path: PathBuf,
        /// What is wrong with it.
        message: String,
    },
    /// The identifier has an empty part (`pkg..fn` or an empty string).
    #[error("invalid identifier {0:?}")]
    InvalidIdentifier(String),
    /// The superclasses of a class lead back to the class (`A < B`, `B < A`).
    #[error("cyclic superclasses: {0}")]
    CyclicBase(String),
    /// The parser could not be set up (the grammar version does not match).
    #[error("tree-sitter could not load the MATLAB grammar: {0}")]
    Grammar(String),
    /// The `.malt.toml` is not valid TOML, or its `[index]` table is not valid.
    #[error("invalid .malt.toml: {0}")]
    Config(String),
}

impl IndexError {
    pub(crate) fn io(path: &std::path::Path, err: &std::io::Error) -> Self {
        IndexError::Io {
            path: path.to_path_buf(),
            message: err.to_string(),
        }
    }
}
