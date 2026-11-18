//! The path configuration: `[index] paths` in `.malt.toml`, the rules for globs (plain
//! folders, `**` recursion, `*`/`?`/`[...]`), missing folders, deduplication, order, and
//! the config parse errors. Each test builds its own project under the system temp
//! directory and passes its working directory explicitly.

use std::fs;
use std::path::{Path, PathBuf};

use malt_index::{resolve_paths, Index, IndexError, ParserConfig, PathConfig};

/// A fresh project folder with the given files (created with their parents).
fn project(name: &str, files: &[&str]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("malt-paths-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    for rel in files {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "function y = f(x)\ny = x;\nend\n").unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    malt_index::fsutil::canonicalize(&root).unwrap()
}

/// The configuration with the given `paths`.
fn config(paths: &[&str]) -> PathConfig {
    PathConfig {
        paths: paths.iter().map(|p| p.to_string()).collect(),
        ..PathConfig::default()
    }
}

/// The resolved folders, relative to the project root, for readable assertions.
fn rel(root: &Path, dirs: &[PathBuf]) -> Vec<String> {
    dirs.iter()
        .map(|d| {
            d.strip_prefix(root)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| d.display().to_string())
                // `/` on every platform, so the expectations read the same everywhere.
                .replace('\\', "/")
        })
        .map(|s| if s.is_empty() { ".".to_string() } else { s })
        .collect()
}

fn resolve(root: &Path, paths: &[&str]) -> (Vec<String>, Vec<String>) {
    let (dirs, warnings) = resolve_paths(&config(paths), root, None, &[]);
    (rel(root, &dirs), warnings)
}

const FILES: &[&str] = &[
    "lib/a.m",
    "lib/sub/b.m",
    "lib/sub/deeper/c.m",
    "lib/+pkg/p.m",
    "lib/+pkg/+inner/q.m",
    "lib/@cls/cls.m",
    "lib/@cls/method.m",
    "lib/private/secret.m",
    "lib/sub/private/hidden.m",
    "lib/.cache/x.m",
    "lib/sub/+pkg2/r.m",
    "src/one/x.m",
    "src/two/y.m",
    "src/+pkgs/z.m",
    "src/.git/g.m",
    "src/loose.m",
    "other/o.m",
];

// ---- default and configuration parsing ----

#[test]
fn without_config_the_working_directory_is_the_path() {
    let root = project("default", FILES);
    let (dirs, warnings) = resolve_paths(&PathConfig::default(), &root, None, &[]);
    assert_eq!(rel(&root, &dirs), vec!["."]);
    assert!(warnings.is_empty());
}

#[test]
fn the_file_folder_comes_first_then_extra_dirs_then_the_working_directory() {
    let root = project("file-first", FILES);
    let file = root.join("other/o.m");
    let extra = vec![PathBuf::from("src/one")];
    let (dirs, warnings) = resolve_paths(&PathConfig::default(), &root, Some(&file), &extra);
    assert_eq!(rel(&root, &dirs), vec!["other", "src/one", "."]);
    assert!(warnings.is_empty());
}

#[test]
fn a_file_in_the_working_directory_is_not_listed_twice() {
    let root = project("dedup-file", FILES);
    let file = root.join("lib/a.m");
    let (dirs, _) = resolve_paths(&config(&["lib"]), &root, Some(&file), &[]);
    assert_eq!(rel(&root, &dirs), vec!["lib"]);
}

#[test]
fn missing_extra_and_file_folders_are_warnings() {
    let root = project("missing-file", FILES);
    let file = root.join("gone/x.m");
    let extra = vec![PathBuf::from("nowhere")];
    let (dirs, warnings) = resolve_paths(&PathConfig::default(), &root, Some(&file), &extra);
    assert_eq!(rel(&root, &dirs), vec!["."]);
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(warnings[0].contains("gone"), "{warnings:?}");
    assert!(warnings[1].contains("nowhere"), "{warnings:?}");
}

#[test]
fn toml_index_table_is_parsed_and_other_tables_are_ignored() {
    let content = r#"
        [lint]
        exclude = ["vendor/**"]
        [lint.rules]
        NOSEMI = "info"

        [index]
        paths = ["lib/**", "src/*"]
    "#;
    let parsed = PathConfig::from_toml(content).unwrap();
    assert_eq!(parsed.paths, vec!["lib/**", "src/*"]);
}

#[test]
fn toml_without_index_table_is_the_empty_configuration() {
    let parsed = PathConfig::from_toml("[lint]\npreset = \"all\"\n").unwrap();
    assert_eq!(parsed, PathConfig::default());
    assert_eq!(PathConfig::from_toml("").unwrap(), PathConfig::default());
}

#[test]
fn invalid_index_tables_are_errors_not_panics() {
    for content in [
        "[index]\npaths = \"lib\"\n",
        "[index]\npaths = [1, 2]\n",
        "[index]\npaht = [\"lib\"]\n",
        "index = 3\n",
        "[index\npaths = []\n",
        "this is not toml",
    ] {
        let err = PathConfig::from_toml(content).unwrap_err();
        assert!(matches!(err, IndexError::Config(_)), "{content:?}: {err:?}");
        assert!(err.to_string().starts_with("invalid .malt.toml:"));
    }
}

#[test]
fn load_reads_the_working_directory_config() {
    let root = project("load", &["lib/a.m"]);
    assert_eq!(PathConfig::load(&root).unwrap(), PathConfig::default());

    fs::write(root.join(".malt.toml"), "[index]\npaths = [\"lib\"]\n").unwrap();
    assert_eq!(PathConfig::load(&root).unwrap(), config(&["lib"]));

    fs::write(root.join(".malt.toml"), "[index]\npaths = 7\n").unwrap();
    let err = PathConfig::load(&root).unwrap_err();
    assert!(err.to_string().contains(".malt.toml"), "{err}");
}

// ---- glob rules ----

#[test]
fn a_plain_folder_is_not_recursive() {
    let root = project("plain", FILES);
    let (dirs, warnings) = resolve(&root, &["lib"]);
    assert_eq!(dirs, vec!["lib"]);
    assert!(warnings.is_empty());
}

#[test]
fn a_trailing_double_star_is_genpath() {
    let root = project("genpath", FILES);
    let (dirs, warnings) = resolve(&root, &["lib/**"]);
    // lib and its plain subfolders; +pkg, @cls, private and hidden folders (and what
    // lies below them) are skipped, the sub-package sits under a skipped folder too.
    assert_eq!(dirs, vec!["lib", "lib/sub", "lib/sub/deeper"]);
    assert!(warnings.is_empty());
}

#[test]
fn double_star_alone_is_the_working_directory_and_its_plain_subfolders() {
    let root = project("star-alone", FILES);
    let (dirs, _) = resolve(&root, &["**"]);
    assert_eq!(
        dirs,
        vec![
            ".",
            "lib",
            "lib/sub",
            "lib/sub/deeper",
            "other",
            "src",
            "src/one",
            "src/two"
        ]
    );
}

#[test]
fn a_single_star_matches_direct_subfolders_only() {
    let root = project("single-star", FILES);
    let (dirs, warnings) = resolve(&root, &["src/*"]);
    // +pkgs and .git are never on the path, and a file (loose.m) is not a folder.
    assert_eq!(dirs, vec!["src/one", "src/two"]);
    assert!(warnings.is_empty(), "{warnings:?}");
}

#[test]
fn question_mark_and_character_classes_match_within_a_segment() {
    let root = project("classes", FILES);
    let (dirs, _) = resolve(&root, &["src/on?", "src/[t]wo", "src/[!t]wo"]);
    assert_eq!(dirs, vec!["src/one", "src/two"]);
}

#[test]
fn a_double_star_in_the_middle_matches_any_depth() {
    let root = project("middle", FILES);
    let (dirs, _) = resolve(&root, &["lib/**/deeper"]);
    assert_eq!(dirs, vec!["lib/sub/deeper"]);
}

#[test]
fn explicit_package_class_and_private_folders_warn_and_are_skipped() {
    let root = project("explicit-skip", FILES);
    let (dirs, warnings) = resolve(
        &root,
        &["lib/+pkg", "lib/@cls", "lib/private", "lib/.cache"],
    );
    assert!(dirs.is_empty(), "{dirs:?}");
    assert_eq!(warnings.len(), 4, "{warnings:?}");
    assert!(
        warnings.iter().all(|w| w.contains("not on the path")),
        "{warnings:?}"
    );
}

#[test]
fn glob_matches_of_skipped_folders_are_silent() {
    let root = project("glob-skip", FILES);
    let (dirs, warnings) = resolve(&root, &["src/*"]);
    assert_eq!(dirs.len(), 2);
    assert!(warnings.is_empty(), "{warnings:?}");
}

#[test]
fn a_pattern_that_matches_nothing_warns_and_the_rest_still_resolves() {
    let root = project("missing", FILES);
    let (dirs, warnings) = resolve(&root, &["missing", "lib", "nope/**", "src/x*"]);
    assert_eq!(dirs, vec!["lib"]);
    assert_eq!(warnings.len(), 3, "{warnings:?}");
    assert!(warnings[0].contains("missing"), "{warnings:?}");
    assert!(warnings[1].contains("nope/**"), "{warnings:?}");
    assert!(warnings[2].contains("src/x*"), "{warnings:?}");
}

#[test]
fn empty_entries_warn() {
    let root = project("empty-entry", FILES);
    let (dirs, warnings) = resolve(&root, &["  "]);
    assert!(dirs.is_empty());
    assert_eq!(warnings.len(), 1, "{warnings:?}");
}

#[test]
fn an_absolute_pattern_is_not_resolved_against_the_working_directory() {
    let root = project("absolute", FILES);
    let other = project("absolute-other", FILES);
    let pattern = other.join("lib/**").display().to_string();
    let (dirs, warnings) = resolve_paths(&config(&[&pattern]), &root, None, &[]);
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(rel(&other, &dirs), vec!["lib", "lib/sub", "lib/sub/deeper"]);
}

// ---- deduplication and order ----

#[test]
fn overlapping_entries_are_deduplicated() {
    let root = project("dedup", FILES);
    let (dirs, warnings) = resolve(
        &root,
        &["lib", "lib/**", "./lib", "lib/sub/", "lib/sub/deeper", "."],
    );
    assert_eq!(dirs, vec![".", "lib", "lib/sub", "lib/sub/deeper"]);
    assert!(warnings.is_empty(), "{warnings:?}");
}

#[test]
fn the_configured_folders_are_sorted_whatever_the_order_of_the_entries() {
    let root = project("order", FILES);
    let (forward, _) = resolve(&root, &["src/two", "other", "lib", "src/one"]);
    let (backward, _) = resolve(&root, &["src/one", "lib", "other", "src/two"]);
    assert_eq!(forward, vec!["lib", "other", "src/one", "src/two"]);
    assert_eq!(forward, backward);
}

#[test]
fn the_extra_dirs_keep_their_given_order_ahead_of_the_configured_folders() {
    let root = project("extra-order", FILES);
    let extra = vec![PathBuf::from("src/two"), PathBuf::from("other")];
    let (dirs, _) = resolve_paths(&config(&["lib"]), &root, None, &extra);
    assert_eq!(rel(&root, &dirs), vec!["src/two", "other", "lib"]);
}

// ---- Index::from_config ----

#[test]
fn from_config_indexes_the_resolved_folders() {
    let root = project("from-config", FILES);
    let file = root.join("src/one/x.m");
    let (index, warnings) = Index::from_config(
        &config(&["lib/**", "missing"]),
        &root,
        Some(&file),
        &[],
        &ParserConfig::default(),
    );
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("missing"));
    let names = index.member_names();
    // The file's folder, then lib and its plain subfolders; the package and class
    // folders of lib are only reached through lib itself.
    assert!(names.contains(&"x"), "{names:?}");
    assert!(names.contains(&"a"), "{names:?}");
    assert!(names.contains(&"b"), "{names:?}");
    assert!(names.contains(&"c"), "{names:?}");
    assert!(names.contains(&"+pkg"), "{names:?}");
    assert!(!names.contains(&"hidden"), "{names:?}");
    assert!(!names.contains(&"r"), "{names:?}");
    assert!(!names.contains(&"y"), "{names:?}");
}

#[test]
fn from_config_with_a_missing_entry_still_builds_an_index() {
    let root = project("from-config-empty", FILES);
    let (index, warnings) = Index::from_config(
        &config(&["gone/**"]),
        &root,
        None,
        &[],
        &ParserConfig::default(),
    );
    assert!(index.search_path().is_empty());
    assert_eq!(warnings.len(), 1, "{warnings:?}");
}
