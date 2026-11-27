//! Which files a run lints: no path means the current folder and below, a folder is
//! searched, and `[lint]` `include`, `exclude`, `extend_exclude`, `respect_gitignore` and
//! `force_exclude` shape the search.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

/// One finding (NOPRT/NOPTS: no semicolon) per file.
const SOURCE: &str = "x = 1\n";

fn malt(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_malt"))
        .current_dir(dir)
        .args(args)
        .env_remove("MALT_CONFIG")
        .output()
        .expect("failed to run malt")
}

fn project(files: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    for file in files {
        let path = dir.path().join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, SOURCE).unwrap();
    }
    dir
}

/// The files that have findings, in the order of the output.
fn linted(output: &Output) -> Vec<String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files: Vec<String> = Vec::new();
    for line in stdout.lines() {
        if let Some((file, _)) = line.split_once(':') {
            if file.ends_with(".m") || file.ends_with(".txt") {
                files.push(file.replace('\\', "/"));
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

#[test]
fn no_path_lints_the_current_folder_and_below() {
    let dir = project(&["a.m", "sub/b.m", "sub/deep/c.m", "notes.txt"]);
    let out = malt(dir.path(), &[]);
    assert_eq!(linted(&out), ["a.m", "sub/b.m", "sub/deep/c.m"]);
}

#[test]
fn a_folder_argument_is_searched() {
    let dir = project(&["a.m", "sub/b.m"]);
    let out = malt(dir.path(), &["sub"]);
    assert_eq!(linted(&out), ["sub/b.m"]);
}

#[test]
fn a_file_argument_is_linted_as_before() {
    let dir = project(&["a.m", "b.m"]);
    let out = malt(dir.path(), &["a.m"]);
    assert_eq!(linted(&out), ["a.m"]);
}

#[test]
fn a_missing_file_is_still_reported() {
    let dir = project(&["a.m"]);
    let out = malt(dir.path(), &["nope.m"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("NOFIL"));
}

#[test]
fn an_empty_folder_warns_and_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let out = malt(dir.path(), &[]);
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no MATLAB files found"));
}

#[test]
fn version_control_and_dependency_folders_are_skipped_by_default() {
    let dir = project(&["a.m", ".git/x.m", "node_modules/p/y.m", "slprj/z.m"]);
    assert_eq!(linted(&malt(dir.path(), &[])), ["a.m"]);
}

#[test]
fn exclude_and_extend_exclude_come_from_the_config() {
    let dir = project(&["a.m", "vendor/v.m", "gen/g.m", ".git/x.m"]);
    fs::write(
        dir.path().join(".malt.toml"),
        "[lint]\nexclude = [\"vendor/\"]\n",
    )
    .unwrap();
    assert_eq!(
        linted(&malt(dir.path(), &[])),
        [".git/x.m", "a.m", "gen/g.m"]
    );
    fs::write(
        dir.path().join(".malt.toml"),
        "[lint]\nextend_exclude = [\"gen/\"]\n",
    )
    .unwrap();
    assert_eq!(linted(&malt(dir.path(), &[])), ["a.m", "vendor/v.m"]);
}

#[test]
fn include_adds_other_extensions() {
    let dir = project(&["a.m", "b.txt"]);
    fs::write(
        dir.path().join(".malt.toml"),
        "[lint]\ninclude = [\"*.m\", \"*.txt\"]\n",
    )
    .unwrap();
    assert_eq!(linted(&malt(dir.path(), &[])), ["a.m", "b.txt"]);
}

#[test]
fn gitignore_is_respected_unless_switched_off() {
    let dir = project(&["a.m", "build/b.m"]);
    fs::write(dir.path().join(".gitignore"), "build/\n").unwrap();
    assert_eq!(linted(&malt(dir.path(), &[])), ["a.m"]);
    fs::write(
        dir.path().join(".malt.toml"),
        "[lint]\nrespect_gitignore = false\n",
    )
    .unwrap();
    assert_eq!(linted(&malt(dir.path(), &[])), ["a.m", "build/b.m"]);
}

#[test]
fn force_exclude_applies_to_named_files() {
    let dir = project(&["a.m", "vendor/v.m"]);
    fs::write(
        dir.path().join(".malt.toml"),
        "[lint]\nexclude = [\"vendor/\"]\n",
    )
    .unwrap();
    assert_eq!(
        linted(&malt(dir.path(), &["a.m", "vendor/v.m"])),
        ["a.m", "vendor/v.m"]
    );
    fs::write(
        dir.path().join(".malt.toml"),
        "[lint]\nexclude = [\"vendor/\"]\nforce_exclude = true\n",
    )
    .unwrap();
    assert_eq!(linted(&malt(dir.path(), &["a.m", "vendor/v.m"])), ["a.m"]);
}

#[test]
fn fix_works_on_a_folder_run() {
    let dir = project(&["a.m", "sub/b.m"]);
    let out = malt(dir.path(), &["--fix"]);
    assert!(
        out.status.success() || out.status.code() == Some(1),
        "{out:?}"
    );
    for file in ["a.m", "sub/b.m"] {
        let text = fs::read_to_string(dir.path().join(file)).unwrap();
        assert_eq!(text, "x = 1;\n", "{file}");
    }
}

#[test]
fn patterns_are_relative_to_the_config_file() {
    let dir = project(&["proj/a.m", "proj/vendor/v.m"]);
    let config = dir.path().join("proj/.malt.toml");
    fs::write(&config, "[lint]\nexclude = [\"vendor/\"]\n").unwrap();
    let out = malt(dir.path(), &["--config", "proj/.malt.toml", "proj"]);
    assert_eq!(linted(&out), ["proj/a.m"]);
}
