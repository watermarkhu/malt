//! Errors of the index. Parsing never fails on MATLAB syntax errors: tree-sitter
//! recovers and the parts that are not understood are skipped.

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
    /// The identifier has an empty part (`pkg..fn` or an empty string).
    #[error("invalid identifier {0:?}")]
    InvalidIdentifier(String),
    /// The parser could not be set up (the grammar version does not match).
    #[error("tree-sitter could not load the MATLAB grammar: {0}")]
    Grammar(String),
}

impl IndexError {
    pub(crate) fn io(path: &std::path::Path, err: &std::io::Error) -> Self {
        IndexError::Io {
            path: path.to_path_buf(),
            message: err.to_string(),
        }
    }
}
