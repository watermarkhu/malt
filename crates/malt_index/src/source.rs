//! The source lines of files, read once and cached by path: the lines collection of maxx.
//!
//! Lines are split on `\n`, a `\r` before it is removed, and a final newline does not
//! start an extra empty line (maxx keeps that empty line). Text that is not UTF-8 is
//! read as Latin-1, as the parser reads it.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use crate::error::IndexError;
use crate::live::decode;

/// The lines of the files read so far. Shared by reference: `Sources` is `Send + Sync`.
#[derive(Debug, Default)]
pub struct Sources {
    cache: Mutex<HashMap<PathBuf, Arc<Vec<String>>>>,
}

impl Sources {
    /// An empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// The lines of a file, read from disk on first use. A file that cannot be read is an
    /// [`IndexError::Io`] and is not cached.
    pub fn lines(&self, path: impl AsRef<Path>) -> Result<Arc<Vec<String>>, IndexError> {
        let path = path.as_ref();
        if let Some(lines) = self.locked().get(path) {
            return Ok(Arc::clone(lines));
        }
        let bytes = fs::read(path).map_err(|e| IndexError::io(path, &e))?;
        let lines = Arc::new(split_lines(&bytes));
        self.locked().insert(path.to_path_buf(), Arc::clone(&lines));
        Ok(lines)
    }

    /// Whether the lines of a file are cached (read by an earlier call).
    pub fn is_cached(&self, path: impl AsRef<Path>) -> bool {
        self.locked().contains_key(path.as_ref())
    }

    /// The cached paths, sorted.
    pub fn cached_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = self.locked().keys().cloned().collect();
        paths.sort();
        paths
    }

    fn locked(&self) -> std::sync::MutexGuard<'_, HashMap<PathBuf, Arc<Vec<String>>>> {
        self.cache.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The lines of some bytes, as [`Sources::lines`] reads them.
pub fn split_lines(bytes: &[u8]) -> Vec<String> {
    let text = decode(bytes);
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
        .collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::split_lines;

    #[test]
    fn lines_drop_the_final_newline_and_the_carriage_return() {
        assert_eq!(split_lines(b"a\r\nb\n"), ["a", "b"]);
        assert_eq!(split_lines(b"a\n\nb"), ["a", "", "b"]);
        assert!(split_lines(b"").is_empty());
        assert_eq!(split_lines(b"\xe9t\xe9\n"), ["\u{e9}t\u{e9}"]);
    }
}
