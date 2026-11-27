//! Finding the files of a run.
//!
//! A path named on the command line is a file or a folder. A file is linted as it is. A
//! folder is searched, recursively, for the files that `[lint] include` picks up, minus
//! what `exclude` (and `extend_exclude`) skips and, with `respect_gitignore`, what the
//! `.gitignore` files of the tree ignore. Without any path, the current folder is the one
//! searched.
//!
//! Patterns are gitignore patterns (`vendor/`, `**/generated`, `test/*.m`) anchored at the
//! folder of the configuration file, which is the current folder without one.

use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::WalkBuilder;
use malt_core::Config;

/// The files of a run, and what the search found along the way.
#[derive(Debug, Default)]
pub struct Found {
    /// The files to lint, in the order given, the files of a folder in path order.
    pub files: Vec<PathBuf>,
    /// The folders that were searched.
    pub searched: Vec<PathBuf>,
    /// Entries that could not be read (a broken link, a folder without permission). They
    /// are skipped, and the rest of the search goes on.
    pub warnings: Vec<String>,
}

/// Find the files of a run: `paths` (the current folder when empty), searched and
/// filtered by `config`, with patterns anchored at `anchor`.
pub fn discover(paths: &[PathBuf], config: &Config, anchor: &Path) -> Result<Found> {
    let anchor = absolute(anchor)?;
    let exclude = matcher(&anchor, &config.exclude)?;
    let include = matcher(&anchor, &config.include)?;
    let default = [PathBuf::from(".")];
    let paths = if paths.is_empty() { &default } else { paths };

    let mut found = Found::default();
    let mut seen = HashSet::new();
    let mut add = |file: PathBuf, found: &mut Found| {
        if seen.insert(file.clone()) {
            found.files.push(file);
        }
    };
    for path in paths {
        if path.is_dir() {
            found.searched.push(path.clone());
            let (files, warnings) = search(path, config, &anchor, &exclude, &include);
            found.warnings.extend(warnings);
            for file in files {
                add(file, &mut found);
            }
        } else if config.force_exclude && excluded(&exclude, &anchor, path, false, true) {
            continue;
        } else {
            add(path.clone(), &mut found);
        }
    }
    Ok(found)
}

/// The files of one folder, in path order.
fn search(
    root: &Path,
    config: &Config,
    anchor: &Path,
    exclude: &Gitignore,
    include: &Gitignore,
) -> (Vec<PathBuf>, Vec<String>) {
    let mut walk = WalkBuilder::new(root);
    walk.hidden(false)
        .parents(config.respect_gitignore)
        .git_ignore(config.respect_gitignore)
        .git_exclude(config.respect_gitignore)
        .git_global(false)
        .ignore(false)
        .require_git(false)
        .sort_by_file_path(|a, b| a.cmp(b));
    let anchor = anchor.to_path_buf();
    let skip = exclude.clone();
    // The walk prunes an excluded folder, so only the entry itself is matched, and the
    // folder that was named is never skipped.
    walk.filter_entry(move |entry| {
        let is_dir = entry.file_type().is_some_and(|t| t.is_dir());
        entry.depth() == 0 || !excluded(&skip, &anchor, entry.path(), is_dir, false)
    });

    let mut files = Vec::new();
    let mut warnings = Vec::new();
    for entry in walk.build() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                warnings.push(format!("skipped an unreadable entry: {err}"));
                continue;
            }
        };
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        if !included(include, entry.path()) {
            continue;
        }
        files.push(tidy(entry.path()));
    }
    (files, warnings)
}

/// A gitignore matcher for `patterns`, anchored at `anchor`.
fn matcher(anchor: &Path, patterns: &[String]) -> Result<Gitignore> {
    let mut builder = GitignoreBuilder::new(anchor);
    for pattern in patterns {
        builder
            .add_line(None, pattern)
            .with_context(|| format!("invalid pattern in the configuration: {pattern}"))?;
    }
    builder
        .build()
        .context("invalid patterns in the configuration")
}

/// Whether `path` matches an exclude pattern; with `parents`, also when a folder above it
/// (below the anchor) does.
fn excluded(exclude: &Gitignore, anchor: &Path, path: &Path, is_dir: bool, parents: bool) -> bool {
    let Some(relative) = relative_to(anchor, path) else {
        return false;
    };
    if relative.as_os_str().is_empty() {
        return false;
    }
    if parents {
        exclude
            .matched_path_or_any_parents(&relative, is_dir)
            .is_ignore()
    } else {
        exclude.matched(&relative, is_dir).is_ignore()
    }
}

/// Whether a file matches an include pattern: by its name for a pattern without a slash
/// (`*.m`), by its path from the anchor otherwise.
fn included(include: &Gitignore, path: &Path) -> bool {
    let name = path.file_name().map(Path::new);
    name.is_some_and(|name| include.matched(name, false).is_ignore())
        || include.matched(path, false).is_ignore()
}

/// `path` below `anchor`, as a relative path; `None` when it is not below it.
fn relative_to(anchor: &Path, path: &Path) -> Option<PathBuf> {
    let absolute = absolute(path).ok()?;
    absolute.strip_prefix(anchor).ok().map(Path::to_path_buf)
}

fn absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()
            .context("cannot read the current folder")?
            .join(path))
    }
}

/// A path without a leading `./`.
fn tidy(path: &Path) -> PathBuf {
    path.strip_prefix(".").unwrap_or(path).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn project(files: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for file in files {
            let path = dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "x = 1;\n").unwrap();
        }
        dir
    }

    fn names(dir: &Path, found: &Found) -> Vec<String> {
        found
            .files
            .iter()
            .map(|f| {
                f.strip_prefix(dir)
                    .unwrap_or(f)
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    }

    fn run(dir: &Path, paths: &[&str], config: &Config) -> Vec<String> {
        let paths: Vec<PathBuf> = paths.iter().map(|p| dir.join(p)).collect();
        let found = discover(&paths, config, dir).unwrap();
        names(dir, &found)
    }

    #[test]
    fn a_folder_gives_its_m_files_in_path_order() {
        let dir = project(&["b.m", "a.m", "sub/c.m", "notes.txt", "data.mat"]);
        let files = run(dir.path(), &["."], &Config::default());
        assert_eq!(files, ["a.m", "b.m", "sub/c.m"].map(String::from));
    }

    #[test]
    fn a_file_is_linted_whatever_its_name() {
        let dir = project(&["a.txt", "b.m"]);
        let files = run(dir.path(), &["a.txt", "b.m", "b.m"], &Config::default());
        assert_eq!(files, ["a.txt", "b.m"].map(String::from));
    }

    #[test]
    fn the_default_excludes_skip_version_control_and_dependencies() {
        let dir = project(&["a.m", ".git/hooks/x.m", "node_modules/p/y.m", "slprj/z.m"]);
        assert_eq!(
            run(dir.path(), &["."], &Config::default()),
            ["a.m"].map(String::from)
        );
    }

    #[test]
    fn exclude_replaces_the_defaults_and_extend_exclude_adds() {
        let dir = project(&["a.m", "vendor/v.m", "gen/g.m", "node_modules/n.m"]);
        let config = Config::from_toml("[lint]\nexclude = [\"vendor/\"]\n").unwrap();
        assert_eq!(
            run(dir.path(), &["."], &config),
            ["a.m", "gen/g.m", "node_modules/n.m"].map(String::from)
        );
        let config = Config::from_toml("[lint]\nextend_exclude = [\"gen/\"]\n").unwrap();
        assert_eq!(
            run(dir.path(), &["."], &config),
            ["a.m", "vendor/v.m"].map(String::from)
        );
    }

    #[test]
    fn exclude_patterns_follow_gitignore_rules() {
        let dir = project(&[
            "a.m",
            "test/t.m",
            "test/deep/d.m",
            "src/test/s.m",
            "x_old.m",
        ]);
        let config = Config::from_toml("[lint]\nexclude = [\"/test/\", \"*_old.m\"]\n").unwrap();
        assert_eq!(
            run(dir.path(), &["."], &config),
            ["a.m", "src/test/s.m"].map(String::from)
        );
    }

    #[test]
    fn include_picks_other_files() {
        let dir = project(&["a.m", "b.mlx", "sub/c.mlx"]);
        let config = Config::from_toml("[lint]\ninclude = [\"*.mlx\"]\n").unwrap();
        assert_eq!(
            run(dir.path(), &["."], &config),
            ["b.mlx", "sub/c.mlx"].map(String::from)
        );
    }

    #[test]
    fn gitignore_is_respected_unless_switched_off() {
        let dir = project(&["a.m", "build/b.m", "c.m"]);
        fs::write(dir.path().join(".gitignore"), "build/\nc.m\n").unwrap();
        assert_eq!(
            run(dir.path(), &["."], &Config::default()),
            ["a.m"].map(String::from)
        );
        let config = Config::from_toml("[lint]\nrespect_gitignore = false\n").unwrap();
        assert_eq!(
            run(dir.path(), &["."], &config),
            ["a.m", "build/b.m", "c.m"].map(String::from)
        );
    }

    #[test]
    fn force_exclude_applies_the_patterns_to_named_files() {
        let dir = project(&["a.m", "vendor/v.m"]);
        let named = ["a.m", "vendor/v.m"];
        let config = Config::from_toml("[lint]\nexclude = [\"vendor/\"]\n").unwrap();
        assert_eq!(
            run(dir.path(), &named, &config),
            ["a.m", "vendor/v.m"].map(String::from)
        );
        let config =
            Config::from_toml("[lint]\nexclude = [\"vendor/\"]\nforce_exclude = true\n").unwrap();
        assert_eq!(run(dir.path(), &named, &config), ["a.m"].map(String::from));
    }

    #[test]
    fn a_named_folder_below_an_excluded_one_is_still_searched() {
        // Naming `vendor/lib` is asking for it: only what the walk meets is filtered.
        let dir = project(&["vendor/lib/l.m", "vendor/other/o.m"]);
        let config = Config::from_toml("[lint]\nexclude = [\"vendor/\"]\n").unwrap();
        let files = run(dir.path(), &["vendor/lib"], &config);
        assert_eq!(files, ["vendor/lib/l.m"].map(String::from));
    }

    #[test]
    fn an_invalid_pattern_is_an_error() {
        let dir = project(&["a.m"]);
        let config = Config {
            exclude: vec!["{a,".into()],
            ..Config::default()
        };
        let result = discover(&[dir.path().to_path_buf()], &config, dir.path());
        assert!(result.is_err());
    }
}
