//! Finding the files of a run.
//!
//! A path named by the user is a file or a folder. A file is linted as it is. A folder is
//! searched, recursively, for the files that `[lint] include` picks up, minus what
//! `exclude` (and `extend_exclude`) skips and, with `respect_gitignore`, what the
//! `.gitignore` files of the tree ignore.
//!
//! The patterns are gitignore patterns (`vendor/`, `**/generated`, `test/*.m`), matched
//! against paths relative to one anchor folder (the one of the configuration file). The
//! search reads the file system only through [`Fs`], so the command line (`std::fs`) and the
//! WebAssembly package (callbacks into Node) run the same code.
//!
//! All paths here are relative to the anchor and use `/`.

use std::collections::HashSet;

use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::config::Config;

/// What a path is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A regular file.
    File,
    /// A folder.
    Dir,
    /// Anything else: a symbolic link, a socket. It is skipped.
    Other,
}

/// The file system the search reads, with paths relative to the anchor.
pub trait Fs {
    /// What `path` is, or `None` when it does not exist.
    fn kind(&self, path: &str) -> Option<Kind>;
    /// The entries of the folder `dir` (`.` is the anchor): name and kind, in any order.
    fn list(&self, dir: &str) -> Result<Vec<(String, Kind)>, String>;
    /// The bytes of a file, or `None` when it cannot be read.
    fn read(&self, path: &str) -> Option<Vec<u8>>;
}

/// The files of a run, and what the search found along the way.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Found {
    /// The files to lint, in the order the paths were named, the files of a folder in path
    /// order. A named path that does not exist is in its place too, so that linting it reports
    /// it where it was named.
    pub files: Vec<String>,
    /// The named paths that do not exist (they are also in `files`).
    pub missing: Vec<String>,
    /// The folders that were searched.
    pub searched: Vec<String>,
    /// Entries that could not be read (a folder without permission). They are skipped, and
    /// the rest of the search goes on.
    pub warnings: Vec<String>,
}

/// A path as the search spells it: forward slashes, no leading `./`, no trailing `/`, and `.`
/// for the anchor itself.
pub fn normalize(path: &str) -> String {
    let path = path.replace('\\', "/");
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|p| *p != "..") => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

/// The matchers of a search: one for `config.exclude`, one for `config.include`.
struct Matchers {
    exclude: Gitignore,
    include: Gitignore,
}

fn matcher(patterns: &[String]) -> Gitignore {
    let mut builder = GitignoreBuilder::new("");
    for pattern in patterns {
        // An invalid pattern is reported by [`validate_patterns`]; here it matches nothing.
        let _ = builder.add_line(None, pattern);
    }
    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

/// Check the `include` and `exclude` patterns of `config`: the first one that is not a valid
/// gitignore pattern is an error naming it.
pub fn validate_patterns(config: &Config) -> Result<(), String> {
    let mut builder = GitignoreBuilder::new("");
    for pattern in config.include.iter().chain(config.exclude.iter()) {
        builder
            .add_line(None, pattern)
            .map_err(|_| format!("invalid pattern in the configuration: {pattern}"))?;
    }
    Ok(())
}

/// The `.gitignore` of `dir`, as a matcher anchored at `dir`, when there is one.
fn gitignore_of(fs: &dyn Fs, dir: &str) -> Option<Gitignore> {
    let file = if dir == "." {
        ".gitignore".to_string()
    } else {
        format!("{dir}/.gitignore")
    };
    let bytes = fs.read(&file)?;
    let text = String::from_utf8_lossy(&bytes);
    let root = if dir == "." { "" } else { dir };
    let mut builder = GitignoreBuilder::new(root);
    for line in text.lines() {
        let _ = builder.add_line(None, line);
    }
    builder.build().ok()
}

/// Find the files for the named `paths` under `config`. Without any path, the anchor is
/// searched.
pub fn discover(fs: &dyn Fs, paths: &[String], config: &Config) -> Found {
    let matchers = Matchers {
        exclude: matcher(&config.exclude),
        include: matcher(&config.include),
    };
    let default = [".".to_string()];
    let paths = if paths.is_empty() { &default } else { paths };

    let mut found = Found::default();
    let mut seen = HashSet::new();
    for named in paths {
        let path = normalize(named);
        match fs.kind(&path) {
            Some(Kind::Dir) => {
                found.searched.push(path.clone());
                let stack = parents_gitignores(fs, &path, config);
                walk(fs, &path, config, &matchers, &stack, &mut found, &mut seen);
            }
            None => {
                found.missing.push(path.clone());
                if seen.insert(path.clone()) {
                    found.files.push(path);
                }
            }
            Some(_) => {
                // A named file is linted as it is, unless `force_exclude` applies the
                // patterns to it, or a folder above it, too.
                let excluded = config.force_exclude
                    && matchers
                        .exclude
                        .matched_path_or_any_parents(&path, false)
                        .is_ignore();
                if !excluded && seen.insert(path.clone()) {
                    found.files.push(path);
                }
            }
        }
    }
    found
}

/// The `.gitignore` files of the folders above `root`, up to the anchor, outermost first.
fn parents_gitignores(fs: &dyn Fs, root: &str, config: &Config) -> Vec<Gitignore> {
    if !config.respect_gitignore || root == "." {
        return Vec::new();
    }
    let mut dirs = vec![".".to_string()];
    let parts: Vec<&str> = root.split('/').collect();
    for end in 1..parts.len() {
        dirs.push(parts[..end].join("/"));
    }
    dirs.iter()
        .filter_map(|dir| gitignore_of(fs, dir))
        .collect()
}

fn walk(
    fs: &dyn Fs,
    dir: &str,
    config: &Config,
    matchers: &Matchers,
    stack: &[Gitignore],
    found: &mut Found,
    seen: &mut HashSet<String>,
) {
    let mut entries = match fs.list(dir) {
        Ok(entries) => entries,
        Err(error) => {
            found
                .warnings
                .push(format!("skipped an unreadable folder: {dir} ({error})"));
            return;
        }
    };
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut local = stack.to_vec();
    if config.respect_gitignore {
        if let Some(own) = gitignore_of(fs, dir) {
            local.push(own);
        }
    }
    for (name, kind) in entries {
        let is_dir = match kind {
            Kind::Dir => true,
            Kind::File => false,
            Kind::Other => continue,
        };
        let child = if dir == "." {
            name
        } else {
            format!("{dir}/{name}")
        };
        // Only the entry itself is matched: an excluded folder is not entered, and the
        // folder that was named is never skipped.
        if matchers.exclude.matched(&child, is_dir).is_ignore() {
            continue;
        }
        if local
            .iter()
            .any(|gitignore| gitignore.matched(&child, is_dir).is_ignore())
        {
            continue;
        }
        if is_dir {
            walk(fs, &child, config, matchers, &local, found, seen);
        } else if matchers.include.matched(&child, false).is_ignore() && seen.insert(child.clone())
        {
            found.files.push(child);
        }
    }
}

/// An in-memory [`Fs`], for tests: a list of files, and the folders they imply.
#[cfg(any(test, feature = "test-util"))]
pub struct MemFs {
    files: Vec<(String, Vec<u8>)>,
    others: Vec<String>,
}

#[cfg(any(test, feature = "test-util"))]
impl MemFs {
    /// A file system of `files`, each a path and its text.
    pub fn new(files: &[(&str, &str)]) -> Self {
        Self {
            files: files
                .iter()
                .map(|(p, c)| (normalize(p), c.as_bytes().to_vec()))
                .collect(),
            others: Vec::new(),
        }
    }

    /// Add a file of raw bytes.
    pub fn with_bytes(mut self, path: &str, bytes: &[u8]) -> Self {
        self.files.push((normalize(path), bytes.to_vec()));
        self
    }

    /// Add an entry that is neither file nor folder (a link).
    pub fn with_other(mut self, path: &str) -> Self {
        self.others.push(normalize(path));
        self
    }
}

#[cfg(any(test, feature = "test-util"))]
impl Fs for MemFs {
    fn kind(&self, path: &str) -> Option<Kind> {
        let path = normalize(path);
        if path == "."
            || self
                .files
                .iter()
                .any(|(f, _)| f.starts_with(&format!("{path}/")))
        {
            return Some(Kind::Dir);
        }
        if self.others.contains(&path) {
            return Some(Kind::Other);
        }
        self.files
            .iter()
            .any(|(f, _)| *f == path)
            .then_some(Kind::File)
    }

    fn list(&self, dir: &str) -> Result<Vec<(String, Kind)>, String> {
        let dir = normalize(dir);
        let prefix = if dir == "." {
            String::new()
        } else {
            format!("{dir}/")
        };
        let mut entries: Vec<(String, Kind)> = Vec::new();
        for path in self.files.iter().map(|(f, _)| f).chain(self.others.iter()) {
            let Some(rest) = path.strip_prefix(&prefix) else {
                continue;
            };
            let (name, kind) = match rest.split_once('/') {
                Some((name, _)) => (name, Kind::Dir),
                None if self.others.contains(path) => (rest, Kind::Other),
                None => (rest, Kind::File),
            };
            if !entries.iter().any(|(n, _)| n == name) {
                entries.push((name.to_string(), kind));
            }
        }
        Ok(entries)
    }

    fn read(&self, path: &str) -> Option<Vec<u8>> {
        let path = normalize(path);
        self.files
            .iter()
            .find(|(f, _)| *f == path)
            .map(|(_, c)| c.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(fs: &MemFs, paths: &[&str], config: &Config) -> Vec<String> {
        let paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
        discover(fs, &paths, config).files
    }

    fn toml(text: &str) -> Config {
        Config::from_toml(text).unwrap()
    }

    #[test]
    fn normalize_cleans_paths() {
        assert_eq!(normalize("./a/./b/"), "a/b");
        assert_eq!(normalize("a\\b.m"), "a/b.m");
        assert_eq!(normalize("a/../b"), "b");
        assert_eq!(normalize("../x"), "../x");
        assert_eq!(normalize(""), ".");
        assert_eq!(normalize("./"), ".");
    }

    #[test]
    fn a_folder_gives_its_m_files_in_path_order() {
        let fs = MemFs::new(&[
            ("b.m", ""),
            ("a.m", ""),
            ("sub/c.m", ""),
            ("notes.txt", ""),
            ("data.mat", ""),
        ]);
        assert_eq!(
            found(&fs, &["."], &Config::default()),
            ["a.m", "b.m", "sub/c.m"]
        );
        assert_eq!(
            found(&fs, &[], &Config::default()),
            found(&fs, &["."], &Config::default())
        );
    }

    #[test]
    fn a_file_is_linted_whatever_its_name_and_once() {
        let fs = MemFs::new(&[("a.txt", ""), ("b.m", "")]);
        assert_eq!(
            found(&fs, &["a.txt", "b.m", "./b.m"], &Config::default()),
            ["a.txt", "b.m"]
        );
    }

    #[test]
    fn a_missing_path_is_reported() {
        let fs = MemFs::new(&[("a.m", "")]);
        let result = discover(&fs, &["nope.m".into(), "a.m".into()], &Config::default());
        // In the order named, so that the missing file is reported where it was named.
        assert_eq!(result.files, ["nope.m", "a.m"]);
        assert_eq!(result.missing, ["nope.m"]);
    }

    #[test]
    fn the_default_excludes_skip_version_control_and_dependencies() {
        let fs = MemFs::new(&[
            ("a.m", ""),
            (".git/h.m", ""),
            ("node_modules/p/y.m", ""),
            ("slprj/z.m", ""),
        ]);
        assert_eq!(found(&fs, &["."], &Config::default()), ["a.m"]);
    }

    #[test]
    fn exclude_replaces_the_defaults_and_extend_exclude_adds() {
        let fs = MemFs::new(&[
            ("a.m", ""),
            ("vendor/v.m", ""),
            ("gen/g.m", ""),
            ("node_modules/n.m", ""),
        ]);
        assert_eq!(
            found(&fs, &["."], &toml("[lint]\nexclude = [\"vendor/\"]\n")),
            ["a.m", "gen/g.m", "node_modules/n.m"]
        );
        assert_eq!(
            found(&fs, &["."], &toml("[lint]\nextend_exclude = [\"gen/\"]\n")),
            ["a.m", "vendor/v.m"]
        );
    }

    #[test]
    fn exclude_patterns_follow_gitignore_rules() {
        let fs = MemFs::new(&[
            ("a.m", ""),
            ("test/t.m", ""),
            ("test/deep/d.m", ""),
            ("src/test/s.m", ""),
            ("x_old.m", ""),
        ]);
        let config = toml("[lint]\nexclude = [\"/test/\", \"*_old.m\"]\n");
        assert_eq!(found(&fs, &["."], &config), ["a.m", "src/test/s.m"]);
    }

    #[test]
    fn include_picks_other_files() {
        let fs = MemFs::new(&[("a.m", ""), ("b.mlx", ""), ("sub/c.mlx", "")]);
        let config = toml("[lint]\ninclude = [\"*.mlx\"]\n");
        assert_eq!(found(&fs, &["."], &config), ["b.mlx", "sub/c.mlx"]);
    }

    #[test]
    fn gitignore_files_are_respected_nested_for_their_folder_unless_switched_off() {
        let fs = MemFs::new(&[
            ("a.m", ""),
            ("build/b.m", ""),
            ("c.m", ""),
            ("sub/d.m", ""),
            ("sub/e.m", ""),
            (".gitignore", "build/\nc.m\n"),
            ("sub/.gitignore", "d.m\n"),
        ]);
        assert_eq!(found(&fs, &["."], &Config::default()), ["a.m", "sub/e.m"]);
        let off = toml("[lint]\nrespect_gitignore = false\n");
        assert_eq!(
            found(&fs, &["."], &off),
            ["a.m", "build/b.m", "c.m", "sub/d.m", "sub/e.m"]
        );
    }

    #[test]
    fn the_gitignore_above_a_named_folder_applies_to_it() {
        let fs = MemFs::new(&[
            ("src/a.m", ""),
            ("src/gen/g.m", ""),
            (".gitignore", "gen/\n"),
        ]);
        assert_eq!(found(&fs, &["src"], &Config::default()), ["src/a.m"]);
    }

    #[test]
    fn force_exclude_applies_the_patterns_to_named_files() {
        let fs = MemFs::new(&[("a.m", ""), ("vendor/v.m", "")]);
        let named = ["a.m", "vendor/v.m"];
        let plain = toml("[lint]\nexclude = [\"vendor/\"]\n");
        assert_eq!(found(&fs, &named, &plain), ["a.m", "vendor/v.m"]);
        let forced = toml("[lint]\nexclude = [\"vendor/\"]\nforce_exclude = true\n");
        assert_eq!(found(&fs, &named, &forced), ["a.m"]);
    }

    #[test]
    fn a_named_folder_below_an_excluded_one_is_still_searched() {
        let fs = MemFs::new(&[("vendor/lib/l.m", ""), ("vendor/other/o.m", "")]);
        let config = toml("[lint]\nexclude = [\"vendor/\"]\n");
        assert_eq!(found(&fs, &["vendor/lib"], &config), ["vendor/lib/l.m"]);
    }

    #[test]
    fn an_invalid_pattern_is_an_error() {
        let config = Config {
            exclude: vec!["{a,".into()],
            ..Config::default()
        };
        let error = validate_patterns(&config).unwrap_err();
        assert!(error.contains("{a,"), "{error}");
        assert!(validate_patterns(&Config::default()).is_ok());
    }

    #[test]
    fn links_are_not_followed() {
        let fs = MemFs::new(&[("a.m", "")]).with_other("link");
        assert_eq!(found(&fs, &["."], &Config::default()), ["a.m"]);
    }

    #[test]
    fn an_empty_folder_finds_nothing() {
        let fs = MemFs::new(&[]);
        let result = discover(&fs, &[], &Config::default());
        assert!(result.files.is_empty() && result.missing.is_empty());
        assert_eq!(result.searched, ["."]);
    }

    #[test]
    fn an_unreadable_folder_is_a_warning() {
        struct Broken;
        impl Fs for Broken {
            fn kind(&self, _: &str) -> Option<Kind> {
                Some(Kind::Dir)
            }
            fn list(&self, _: &str) -> Result<Vec<(String, Kind)>, String> {
                Err("EACCES".into())
            }
            fn read(&self, _: &str) -> Option<Vec<u8>> {
                None
            }
        }
        let result = discover(&Broken, &[], &Config::default());
        assert!(result.files.is_empty());
        assert!(
            result.warnings[0].contains("EACCES"),
            "{:?}",
            result.warnings
        );
    }
}
