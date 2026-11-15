//! The search path of a run, as the linter's [`ProjectIndex`]: the classes and
//! functions that MATLAB resolves names against, read by `malt-index`.
//!
//! It is off unless the user turns it on (`--index`, or `[index] enabled = true`):
//! MATLAB's Code Analyzer reads one file at a time, so the index only adds findings
//! the Code Analyzer does not make (see `docs/VERIFY.md`, 'Path index, stage 3.4').
//!
//! A file's own folder is searched first, as MATLAB does, so the index is built per
//! folder (`Index::from_config` with the file) and kept for the rest of the run.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{env, fs};

use anyhow::{Context, Result};
use malt_core::ProjectIndex;
use malt_index::{Index, Kind, ParserConfig, PathConfig};

/// The search path of a run: the `[index]` settings, the working directory, and
/// one index per folder of the files linted (each with that folder first).
pub struct PathIndex {
    settings: PathConfig,
    cwd: PathBuf,
    parser: ParserConfig,
    by_folder: Mutex<HashMap<PathBuf, Arc<Index>>>,
}

/// The `[index]` table of the config file `--config` names, or of the working
/// directory's `.malt.toml` when it names none.
pub fn load_settings(explicit: Option<&Path>) -> Result<PathConfig> {
    match explicit {
        Some(path) => {
            let content = fs::read_to_string(path)
                .with_context(|| format!("failed to read config: {}", path.display()))?;
            PathConfig::from_toml(&content)
                .with_context(|| format!("failed to parse config: {}", path.display()))
        }
        None => PathConfig::load(Path::new("."))
            .context("failed to read the [index] table of .malt.toml"),
    }
}

/// The search path for `settings` in the working directory, and the warnings about
/// the configured folders (missing folders, patterns that match nothing).
pub fn open(settings: PathConfig) -> Result<(PathIndex, Vec<String>)> {
    let cwd = env::current_dir().context("cannot read the working directory")?;
    let (_, warnings) = Index::from_config(&settings, &cwd, None, &[], &ParserConfig::default());
    let index = PathIndex {
        settings,
        cwd,
        parser: ParserConfig::default(),
        by_folder: Mutex::new(HashMap::new()),
    };
    Ok((index, warnings))
}

impl PathIndex {
    /// The index that searches the folder of `file` first, built once per folder.
    fn index_for(&self, file: &Path) -> Option<Arc<Index>> {
        let file = normal_absolute(file)?;
        let folder = file.parent()?.to_path_buf();
        let mut by_folder = self.by_folder.lock().ok()?;
        let index = by_folder.entry(folder).or_insert_with(|| {
            let (index, _) =
                Index::from_config(&self.settings, &self.cwd, Some(&file), &[], &self.parser);
            Arc::new(index)
        });
        Some(Arc::clone(index))
    }
}

impl ProjectIndex for PathIndex {
    /// `handle` is a handle class. Any other class is looked up from the file's
    /// folder, then the path. It is a handle class when `handle` is among its bases
    /// (also through a project base). It is a value class when every base in its
    /// hierarchy is a project class, and none is `handle`. Anything else (a name
    /// that is not on the path, or a base that is not) is unknown.
    fn is_handle_class(&self, name: &str, from: &Path) -> Option<bool> {
        if name == "handle" {
            return Some(true);
        }
        let index = self.index_for(from)?;
        let folder = normal_absolute(from)?.parent()?.to_path_buf();
        let class = index.get_from(name, &folder).ok()??;
        if class.kind != Kind::Class {
            return None;
        }
        let mut handle = false;
        for ancestor in index.mro(&class).ok()?.iter().skip(1) {
            match &ancestor.entity {
                None if ancestor.name == "handle" => handle = true,
                None => return None,
                Some(entity) if entity.kind != Kind::Class => return None,
                Some(_) => {}
            }
        }
        Some(handle)
    }
}

/// The absolute form of `path`, with `.` and `..` resolved lexically (symbolic links
/// are kept, as MATLAB keeps them). Folders are stored in the index in this form.
fn normal_absolute(path: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for part in std::path::absolute(path).ok()?.components() {
        match part {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A project in a temporary folder with the given `(name, source)` files.
    fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (name, source) in files {
            let path = dir.path().join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, source).unwrap();
        }
        dir
    }

    /// The index of `dir` as the working directory `dir` with no `[index]` settings.
    fn index_in(dir: &Path) -> PathIndex {
        let settings = PathConfig::default();
        let cwd = fs::canonicalize(dir).unwrap();
        PathIndex {
            settings,
            cwd,
            parser: ParserConfig::default(),
            by_folder: Mutex::new(HashMap::new()),
        }
    }

    const VALUE_BASE: &str = "classdef base_v\nend\n";
    const HANDLE_BASE: &str = "classdef base_h < handle\nend\n";

    #[test]
    fn handle_is_a_handle_class_without_any_file() {
        let dir = project(&[]);
        let index = index_in(dir.path());
        assert_eq!(
            index.is_handle_class("handle", &dir.path().join("a.m")),
            Some(true)
        );
    }

    #[test]
    fn value_base_in_the_file_folder_is_not_handle() {
        let dir = project(&[
            ("base_v.m", VALUE_BASE),
            ("sub.m", "classdef sub < base_v\nend\n"),
        ]);
        let index = index_in(dir.path());
        let from = dir.path().join("sub.m");
        assert_eq!(index.is_handle_class("base_v", &from), Some(false));
    }

    #[test]
    fn handle_base_is_handle() {
        let dir = project(&[("base_h.m", HANDLE_BASE)]);
        let index = index_in(dir.path());
        let from = dir.path().join("sub.m");
        assert_eq!(index.is_handle_class("base_h", &from), Some(true));
    }

    #[test]
    fn handle_through_a_project_base_is_handle() {
        // sub_h < base_h < handle: the handle is reached through two project classes.
        let dir = project(&[
            ("base_h.m", HANDLE_BASE),
            ("sub_h.m", "classdef sub_h < base_h\nend\n"),
        ]);
        let index = index_in(dir.path());
        let from = dir.path().join("user.m");
        assert_eq!(index.is_handle_class("sub_h", &from), Some(true));
    }

    #[test]
    fn value_class_with_a_value_chain_is_not_handle() {
        // sub_v < base_v, base_v has no superclass: the whole chain is value classes.
        let dir = project(&[
            ("base_v.m", VALUE_BASE),
            ("sub_v.m", "classdef sub_v < base_v\nend\n"),
        ]);
        let index = index_in(dir.path());
        let from = dir.path().join("user.m");
        assert_eq!(index.is_handle_class("sub_v", &from), Some(false));
    }

    #[test]
    fn a_name_not_on_the_path_is_unknown() {
        let dir = project(&[]);
        let index = index_in(dir.path());
        let from = dir.path().join("sub.m");
        assert_eq!(index.is_handle_class("no_such_base", &from), None);
        // A MathWorks class such as matlab.mixin.SetGet is not on the path either.
        assert_eq!(index.is_handle_class("matlab.mixin.SetGet", &from), None);
    }

    #[test]
    fn a_value_class_with_an_unknown_base_is_unknown() {
        // base_v < no_such_base: the chain is not fully known, so it is not known to be a value class.
        let dir = project(&[
            ("base_v.m", "classdef base_v < no_such_base\nend\n"),
            ("sub.m", "classdef sub < base_v\nend\n"),
        ]);
        let index = index_in(dir.path());
        let from = dir.path().join("user.m");
        assert_eq!(index.is_handle_class("base_v", &from), None);
        assert_eq!(index.is_handle_class("sub", &from), None);
    }

    #[test]
    fn a_base_on_the_path_but_not_in_the_file_folder_resolves() {
        // The file lives outside the search path: its base is found in its own folder.
        let dir = project(&[
            ("src/base_v.m", VALUE_BASE),
            ("lib/user.m", "function y = user()\ny = 1;\nend\n"),
        ]);
        let index = index_in(dir.path());
        let from = dir.path().join("src/sub.m");
        assert_eq!(index.is_handle_class("base_v", &from), Some(false));
    }

    #[test]
    fn a_base_on_the_search_path_resolves_from_another_folder() {
        // lib is on the search path (configured), the file is in src.
        let dir = project(&[
            ("lib/base_v.m", VALUE_BASE),
            ("src/sub.m", "classdef sub < base_v\nend\n"),
        ]);
        let cwd = fs::canonicalize(dir.path()).unwrap();
        let settings = PathConfig {
            paths: vec!["lib".to_string()],
            ..PathConfig::default()
        };
        let index = PathIndex {
            settings,
            cwd,
            parser: ParserConfig::default(),
            by_folder: Mutex::new(HashMap::new()),
        };
        let from = dir.path().join("src/sub.m");
        assert_eq!(index.is_handle_class("base_v", &from), Some(false));
    }

    #[test]
    fn a_function_is_not_a_class() {
        let dir = project(&[("helper.m", "function y = helper(x)\ny = x;\nend\n")]);
        let index = index_in(dir.path());
        let from = dir.path().join("sub.m");
        assert_eq!(index.is_handle_class("helper", &from), None);
    }

    #[test]
    fn a_relative_file_path_resolves_like_an_absolute_one() {
        let dir = project(&[
            ("base_v.m", VALUE_BASE),
            ("sub.m", "classdef sub < base_v\nend\n"),
        ]);
        let index = index_in(dir.path());
        let absolute = dir.path().join("sub.m");
        // A file given relative to the working directory (the temporary folder).
        let relative = absolute
            .strip_prefix(dir.path())
            .map(Path::to_path_buf)
            .unwrap();
        let cwd = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();
        let answer = index.is_handle_class("base_v", &relative);
        env::set_current_dir(cwd).unwrap();
        assert_eq!(answer, Some(false));
    }

    #[test]
    fn normal_absolute_resolves_dot_dot() {
        let path = Path::new("/a/b/../c/./d.m");
        assert_eq!(normal_absolute(path), Some(PathBuf::from("/a/c/d.m")));
    }

    #[test]
    fn load_settings_reads_the_index_table_of_the_named_config() {
        let dir = project(&[(
            "cfg.toml",
            "[lint]\npreset = \"all\"\n\n[index]\nenabled = true\npaths = [\"lib/**\"]\n",
        )]);
        let settings = load_settings(Some(&dir.path().join("cfg.toml"))).unwrap();
        assert!(settings.enabled);
        assert_eq!(settings.paths, vec!["lib/**".to_string()]);
    }

    #[test]
    fn load_settings_rejects_an_unknown_key_in_the_index_table() {
        let dir = project(&[("cfg.toml", "[index]\nenable = true\n")]);
        let err = load_settings(Some(&dir.path().join("cfg.toml"))).unwrap_err();
        assert!(format!("{err:#}").contains("enable"), "{err:#}");
    }
}
