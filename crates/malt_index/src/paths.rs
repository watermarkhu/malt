//! The folders on the search path: `[index] paths` in the working directory's
//! `.malt.toml`, the folder of the linted file, and extra folders.
//!
//! Without `paths` the search path is the working directory. With `paths`, the listed
//! folders replace the working directory (list `"."` to keep it as well). The folder of
//! the linted file and the extra folders are always added, first, as MATLAB searches the
//! current folder before the path.
//!
//! Pattern rules (genpath's, as MATLAB reads them):
//! - `lib/**` adds `lib` and every plain subfolder of it, recursively. `+pkg`, `@cls`,
//!   `private` and hidden folders are never added, and nothing below them is.
//! - Any other pattern (`lib`, `src/*`, `lib?`, `[ab]x`) adds the matching folders only,
//!   not recursively. `*`, `?` and `[...]` match within one path segment; `**` in the
//!   middle of a pattern matches any number of folder levels.
//! - Relative patterns resolve against the working directory. Recursion does not follow
//!   symbolic links (it cannot loop); a link named by a pattern is followed.
//!
//! The result is deduplicated. Its order is the search order: the file's folder, the
//! extra folders in the order given, then the configured folders sorted by path.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::IndexError;

/// The configuration file read from the working directory.
pub const CONFIG_FILE: &str = ".malt.toml";

/// The `[index]` table of `.malt.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PathConfig {
    /// Whether the linter uses the index (default: no). The index only adds
    /// findings that MATLAB's Code Analyzer does not make; this crate ignores it.
    pub enabled: bool,
    /// Folders to search: plain folders or glob patterns (see the module docs).
    /// Empty means the working directory alone.
    pub paths: Vec<String>,
}

/// The parts of `.malt.toml` that this module reads. Other tables are ignored.
#[derive(Deserialize, Default)]
struct ConfigFile {
    #[serde(default)]
    index: PathConfig,
}

fn parse(content: &str) -> Result<PathConfig, toml::de::Error> {
    Ok(toml::from_str::<ConfigFile>(content)?.index)
}

impl PathConfig {
    /// Parse the `[index]` table of a `.malt.toml`'s content. Other tables are ignored.
    pub fn from_toml(content: &str) -> Result<Self, IndexError> {
        parse(content).map_err(|err| IndexError::Config(err.to_string()))
    }

    /// Read `dir/.malt.toml`. A missing file is the empty configuration.
    pub fn load(dir: &Path) -> Result<Self, IndexError> {
        let path = dir.join(CONFIG_FILE);
        match fs::read_to_string(&path) {
            Ok(content) => parse(&content)
                .map_err(|err| IndexError::Config(format!("{}: {err}", path.display()))),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(IndexError::io(&path, &err)),
        }
    }
}

/// The folders to search, in search order, and the warnings about what was skipped:
/// missing folders, patterns that match nothing, and package, class and `private`
/// folders named explicitly. Never panics; a folder that cannot be read is skipped.
pub fn resolve_paths(
    config: &PathConfig,
    cwd: &Path,
    file: Option<&Path>,
    extra_dirs: &[PathBuf],
) -> (Vec<PathBuf>, Vec<String>) {
    let mut found = Found::default();
    let mut warnings = Vec::new();

    if let Some(parent) = file.and_then(Path::parent) {
        let folder = cwd.join(parent);
        if folder.is_dir() {
            found.push(&folder);
        } else {
            warnings.push(format!(
                "{}: folder of the linted file not found",
                folder.display()
            ));
        }
    }
    for dir in extra_dirs {
        let folder = cwd.join(dir);
        if folder.is_dir() {
            found.push(&folder);
        } else {
            warnings.push(format!("{}: extra folder not found", folder.display()));
        }
    }

    if config.paths.is_empty() {
        if cwd.is_dir() {
            found.push(cwd);
        } else {
            warnings.push(format!("{}: working directory not found", cwd.display()));
        }
    } else {
        let mut configured = BTreeSet::new();
        for pattern in &config.paths {
            expand(pattern, cwd, &mut configured, &mut warnings);
        }
        for dir in &configured {
            found.push(dir);
        }
    }

    (found.dirs, warnings)
}

/// The folders found so far, deduplicated by their canonical path, in the order added.
#[derive(Default)]
struct Found {
    seen: BTreeSet<PathBuf>,
    dirs: Vec<PathBuf>,
}

impl Found {
    fn push(&mut self, dir: &Path) {
        let canon = canonical(dir);
        if self.seen.insert(canon.clone()) {
            self.dirs.push(canon);
        }
    }
}

/// Add the folders one `[index] paths` entry names to `out`.
fn expand(pattern: &str, cwd: &Path, out: &mut BTreeSet<PathBuf>, warnings: &mut Vec<String>) {
    let raw = pattern.trim();
    if raw.is_empty() {
        warnings.push("empty entry in [index] paths ignored".to_string());
        return;
    }
    let (recursive, base) = if raw == "**" {
        (true, "")
    } else if let Some(base) = raw.strip_suffix("/**") {
        (true, base)
    } else {
        (false, raw)
    };

    let start = if Path::new(base).has_root() {
        PathBuf::from("/")
    } else {
        cwd.to_path_buf()
    };
    let segments: Vec<&str> = base
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();
    let is_glob = base.contains(['*', '?', '[']);

    let mut matches = Vec::new();
    match_segments(&start, &segments, &mut matches);
    if matches.is_empty() {
        warnings.push(format!("{raw}: matches no folder"));
        return;
    }
    for dir in matches {
        let plain = segments.is_empty() || is_plain_name(&name_of(&dir));
        if !plain {
            if !is_glob {
                warnings.push(format!(
                    "{raw}: package, class and private folders are not on the path"
                ));
            }
            continue;
        }
        if recursive {
            genpath(&dir, out);
        } else {
            out.insert(canonical(&dir));
        }
    }
}

/// Every folder that the segments name under `dir`. `**` matches zero or more levels;
/// a segment with `*`, `?` or `[` matches names within the folder (hidden names only
/// when the segment starts with a dot).
fn match_segments(dir: &Path, segments: &[&str], out: &mut Vec<PathBuf>) {
    let Some((first, rest)) = segments.split_first() else {
        out.push(dir.to_path_buf());
        return;
    };
    if *first == "**" {
        match_segments(dir, rest, out);
        for sub in plain_subdirs(dir) {
            match_segments(&sub, segments, out);
        }
    } else if is_glob(first) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') && !first.starts_with('.') {
                continue;
            }
            let pattern: Vec<char> = first.chars().collect();
            let name_chars: Vec<char> = name.chars().collect();
            if glob_match(&pattern, &name_chars) && entry.path().is_dir() {
                match_segments(&entry.path(), rest, out);
            }
        }
    } else {
        let next = dir.join(first);
        if next.is_dir() {
            match_segments(&next, rest, out);
        }
    }
}

/// `dir` and every plain subfolder of it, recursively (genpath's rule).
fn genpath(dir: &Path, out: &mut BTreeSet<PathBuf>) {
    out.insert(canonical(dir));
    for sub in plain_subdirs(dir) {
        genpath(&sub, out);
    }
}

/// The subfolders of `dir` that can be on the path, sorted. Symbolic links are not
/// followed, so recursion ends.
fn plain_subdirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut subs: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|t| t.is_dir()))
        .map(|entry| entry.path())
        .filter(|path| is_plain_name(&name_of(path)))
        .collect();
    subs.sort();
    subs
}

/// Whether a folder of this name can be on the path: not a package (`+`), a class
/// folder (`@`), `private` or hidden.
fn is_plain_name(name: &str) -> bool {
    !(name.starts_with(['.', '+', '@']) || name == "private")
}

fn is_glob(segment: &str) -> bool {
    segment.contains(['*', '?', '['])
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn canonical(path: &Path) -> PathBuf {
    crate::fsutil::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Match one name against a segment pattern: `*` (any run), `?` (one character) and
/// `[...]` (a set, with a range `a-z`, negated by a leading `!` or `^`).
fn glob_match(pattern: &[char], name: &[char]) -> bool {
    match pattern.split_first() {
        None => name.is_empty(),
        Some((&'*', rest)) => (0..=name.len()).any(|skip| glob_match(rest, &name[skip..])),
        Some((&'?', rest)) => name
            .split_first()
            .is_some_and(|(_, tail)| glob_match(rest, tail)),
        Some((&'[', rest)) => {
            let Some((&c, tail)) = name.split_first() else {
                return false;
            };
            match rest.iter().position(|&p| p == ']') {
                Some(end) => class_matches(&rest[..end], c) && glob_match(&rest[end + 1..], tail),
                None => c == '[' && glob_match(rest, tail),
            }
        }
        Some((&literal, rest)) => name
            .split_first()
            .is_some_and(|(&c, tail)| c == literal && glob_match(rest, tail)),
    }
}

/// Whether `c` is in a `[...]` set (the characters between the brackets).
fn class_matches(set: &[char], c: char) -> bool {
    let (negated, set) = match set.first() {
        Some('!' | '^') => (true, &set[1..]),
        _ => (false, set),
    };
    let mut hit = false;
    let mut i = 0;
    while i < set.len() {
        if i + 2 < set.len() && set[i + 1] == '-' {
            hit |= (set[i]..=set[i + 2]).contains(&c);
            i += 3;
        } else {
            hit |= set[i] == c;
            i += 1;
        }
    }
    hit != negated
}
