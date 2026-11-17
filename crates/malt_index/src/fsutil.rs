//! File system helpers shared by the index.

use std::io;
use std::path::{Path, PathBuf};

/// `fs::canonicalize` with the Windows verbatim prefix removed again.
///
/// On Windows `canonicalize` returns `\\?\C:\dir` (and `\\?\UNC\server\share`), which
/// is the same file as `C:\dir` but does not compare equal to it and is not what a user
/// or MATLAB writes. The prefix is dropped when the plain form means the same.
pub fn canonicalize(path: &Path) -> io::Result<PathBuf> {
    let canonical = std::fs::canonicalize(path)?;
    Ok(match canonical.to_str() {
        Some(text) => PathBuf::from(strip_verbatim(text)),
        None => canonical,
    })
}

/// `text` without a Windows verbatim prefix: `\\?\C:\a` becomes `C:\a` and
/// `\\?\UNC\host\share` becomes `\\host\share`. Other text is returned unchanged.
pub fn strip_verbatim(text: &str) -> String {
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{rest}");
    }
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        let bytes = rest.as_bytes();
        if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
            return rest.to_string();
        }
    }
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drive_path_loses_the_verbatim_prefix() {
        assert_eq!(strip_verbatim(r"\\?\C:\Users\a"), r"C:\Users\a");
    }

    #[test]
    fn a_unc_path_keeps_its_share() {
        assert_eq!(strip_verbatim(r"\\?\UNC\host\share\a"), r"\\host\share\a");
    }

    #[test]
    fn other_paths_are_unchanged() {
        for text in ["/usr/lib", r"C:\a", r"\\?\Volume{1234}\a", ""] {
            assert_eq!(strip_verbatim(text), text);
        }
    }
}
