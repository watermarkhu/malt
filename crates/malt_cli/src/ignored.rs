//! `ignoredFiles` of a MATLAB Code Analyzer configuration: the files of a run that the
//! configuration skips.
//!
//! The patterns are relative to the folder that holds the `resources` folder of the
//! configuration (the folder of the file itself, when it is not in a `resources`
//! folder). A pattern has one segment per folder level and must match a file's path
//! exactly in depth: `*` and `?` match within one name, so `sub/*.m` matches the files
//! of `sub` but not of folders below it.

use std::env;
use std::path::{Component, Path, PathBuf};

/// The patterns of one configuration, anchored at its folder.
pub struct IgnoredFiles {
    base: PathBuf,
    patterns: Vec<Vec<String>>,
}

impl IgnoredFiles {
    /// The matcher for `patterns` of the configuration at `config`, or `None` when
    /// there are no patterns.
    pub fn new(config: &Path, patterns: Vec<String>) -> Option<Self> {
        if patterns.is_empty() {
            return None;
        }
        let folder = config.parent().unwrap_or(Path::new(""));
        let base = if folder.file_name().is_some_and(|name| name == "resources") {
            folder.parent().unwrap_or(Path::new(""))
        } else {
            folder
        };
        let patterns = patterns
            .iter()
            .map(|pattern| {
                pattern
                    .split('/')
                    .filter(|segment| !segment.is_empty() && *segment != ".")
                    .map(str::to_string)
                    .collect()
            })
            .collect();
        Some(Self {
            base: absolute(base)?,
            patterns,
        })
    }

    /// Whether `file` is one of the ignored files.
    pub fn matches(&self, file: &Path) -> bool {
        let Some(file) = absolute(file) else {
            return false;
        };
        let Ok(relative) = file.strip_prefix(&self.base) else {
            return false;
        };
        let names: Vec<String> = relative
            .components()
            .filter_map(|component| match component {
                Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect();
        self.patterns
            .iter()
            .any(|pattern| segments_match(pattern, &names))
    }
}

fn absolute(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        env::current_dir().ok().map(|cwd| cwd.join(path))
    }
}

fn segments_match(pattern: &[String], names: &[String]) -> bool {
    pattern.len() == names.len()
        && pattern
            .iter()
            .zip(names)
            .all(|(segment, name)| glob(&chars(segment), &chars(name)))
}

fn chars(text: &str) -> Vec<char> {
    text.chars().collect()
}

/// Whether `name` matches `pattern`, where `*` is any run of characters and `?` one.
fn glob(pattern: &[char], name: &[char]) -> bool {
    match pattern.first() {
        None => name.is_empty(),
        Some('*') => glob(&pattern[1..], name) || (!name.is_empty() && glob(pattern, &name[1..])),
        Some('?') => !name.is_empty() && glob(&pattern[1..], &name[1..]),
        Some(c) => name.first() == Some(c) && glob(&pattern[1..], &name[1..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_matches_within_one_name() {
        assert!(glob(&chars("*.m"), &chars("a.m")));
        assert!(!glob(&chars("*.m"), &chars("a.txt")));
        assert!(glob(&chars("s?.m"), &chars("s1.m")));
        assert!(!glob(&chars("s?.m"), &chars("s12.m")));
    }

    #[test]
    fn segments_must_match_in_depth() {
        let names = vec!["sub".to_string(), "a.m".to_string()];
        assert!(segments_match(&["sub".into(), "*.m".into()], &names));
        assert!(!segments_match(&["*.m".into()], &names));
        assert!(!segments_match(&["a.m".into()], &names));
    }
}
