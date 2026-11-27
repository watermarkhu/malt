//! Finding the files of a run on the command line: the shared search of
//! [`malt_core::files`] over the real file system.
//!
//! A path named on the command line is a file or a folder. Without any path, the current
//! folder is the one searched. Patterns are anchored at the folder of the configuration file,
//! which is the current folder without one.

use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use malt_core::files::{self, Fs, Kind};
use malt_core::Config;

/// The files of a run, and what the search found along the way.
#[derive(Debug, Default)]
pub struct Found {
    /// The files to lint, in the order given, the files of a folder in path order.
    pub files: Vec<PathBuf>,
    /// The folders that were searched.
    pub searched: Vec<PathBuf>,
    /// Entries that could not be read (a folder without permission). They are skipped, and
    /// the rest of the search goes on.
    pub warnings: Vec<String>,
}

/// The real file system under the anchor folder.
struct StdFs {
    root: PathBuf,
}

impl StdFs {
    fn real(&self, path: &str) -> PathBuf {
        if path == "." {
            self.root.clone()
        } else {
            self.root.join(path)
        }
    }
}

impl Fs for StdFs {
    fn kind(&self, path: &str) -> Option<Kind> {
        // A path that was named follows links, as `path.is_dir()` does.
        let meta = fs::metadata(self.real(path)).ok()?;
        Some(if meta.is_dir() {
            Kind::Dir
        } else if meta.is_file() {
            Kind::File
        } else {
            Kind::Other
        })
    }

    fn list(&self, dir: &str) -> Result<Vec<(String, Kind)>, String> {
        let entries = fs::read_dir(self.real(dir)).map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = match entry.file_type() {
                Ok(t) if t.is_dir() => Kind::Dir,
                Ok(t) if t.is_file() => Kind::File,
                _ => Kind::Other,
            };
            out.push((entry.file_name().to_string_lossy().into_owned(), kind));
        }
        Ok(out)
    }

    fn read(&self, path: &str) -> Option<Vec<u8>> {
        fs::read(self.real(path)).ok()
    }
}

/// Find the files of a run: `paths` (the current folder when empty), searched and
/// filtered by `config`, with patterns anchored at `anchor`.
pub fn discover(paths: &[PathBuf], config: &Config, anchor: &Path) -> Result<Found> {
    files::validate_patterns(config).map_err(anyhow::Error::msg)?;
    let cwd = env::current_dir().context("cannot read the current folder")?;
    let anchor = absolute(anchor, &cwd);
    let stdfs = StdFs {
        root: anchor.clone(),
    };

    // The search works on paths relative to the anchor; the files come back relative to the
    // current folder, and a file that was named is printed as it was named.
    let named: Vec<(String, PathBuf)> = if paths.is_empty() {
        vec![(String::from("."), PathBuf::from("."))]
    } else {
        paths
            .iter()
            .map(|p| (relative(&anchor, &absolute(p, &cwd)), p.clone()))
            .collect()
    };
    let virtual_paths: Vec<String> = named.iter().map(|(v, _)| v.clone()).collect();
    let found = files::discover(&stdfs, &virtual_paths, config);

    let map = |virtual_path: &str| -> PathBuf {
        match named
            .iter()
            .find(|(v, _)| files::normalize(v) == virtual_path)
        {
            Some((_, original)) => original.clone(),
            None => {
                let real = stdfs.real(virtual_path);
                tidy(real.strip_prefix(&cwd).unwrap_or(&real))
            }
        }
    };
    let mut result = Found {
        files: found.files.iter().map(|f| map(f)).collect(),
        searched: found
            .searched
            .iter()
            .map(|s| {
                named
                    .iter()
                    .find(|(v, _)| files::normalize(v) == *s)
                    .map_or_else(|| map(s), |(_, original)| original.clone())
            })
            .collect(),
        warnings: found.warnings,
    };
    // A named path that does not exist is linted as it is, which reports it as NOFIL.
    for missing in &found.missing {
        let original = named
            .iter()
            .find(|(v, _)| files::normalize(v) == *missing)
            .map_or_else(|| PathBuf::from(missing), |(_, original)| original.clone());
        if !result.files.contains(&original) {
            result.files.push(original);
        }
    }
    Ok(result)
}

fn absolute(path: &Path, cwd: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

/// `path` relative to `anchor`, with `/` and `..` where it is not below it.
fn relative(anchor: &Path, path: &Path) -> String {
    let anchor: Vec<Component> = anchor.components().collect();
    let path: Vec<Component> = path.components().collect();
    let common = anchor.iter().zip(&path).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = Vec::new();
    for _ in common..anchor.len() {
        parts.push("..".to_string());
    }
    for part in &path[common..] {
        parts.push(part.as_os_str().to_string_lossy().into_owned());
    }
    files::normalize(&parts.join("/"))
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
