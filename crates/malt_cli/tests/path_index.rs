//! The opt-in path index (`--index`, `[index] enabled`). With it off, malt's output
//! is the one it was before the index existed (the golden file, recorded before the
//! change). With it on, only one check reports more: MCEB on
//! classes whose superclasses are value classes on the search path (see
//! `docs/VERIFY.md`, 'Path index, stage 3.4').

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The sample project: classes with value and handle bases, and plain code.
const PROJECT: &str = "tests/index_project";

/// The files of the sample project, in the order the runs lint them.
const FILES: [&str; 9] = [
    "base_v.m",
    "base_h.m",
    "sub_ev.m",
    "sub_evh.m",
    "sub_nobase.m",
    "sub_missing.m",
    "obs_none.m",
    "obs_h.m",
    "plain.m",
];

/// The stdout of a default run on the sample project, recorded before the index existed.
const GOLDEN: &str = include_str!("index_golden/default_stdout.txt");

/// Run malt in `dir` with `args` followed by `files`.
fn malt(dir: &Path, args: &[&str], files: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_malt"))
        .current_dir(dir)
        .args(args)
        .args(files)
        .output()
        .expect("malt runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The sample project's folder, absolute.
fn project_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PROJECT);
    fs::canonicalize(dir).unwrap()
}

/// A temporary copy of the sample project's classes (no `.malt.toml`), with `config`
/// written as its `.malt.toml` when given.
fn copy_project(config: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for name in FILES {
        fs::copy(project_dir().join(name), dir.path().join(name)).unwrap();
    }
    if let Some(config) = config {
        fs::write(dir.path().join(".malt.toml"), config).unwrap();
    }
    dir
}

/// The stdout of an enabled run on the sample project: the finding the index
/// adds (MCEB on `sub_ev.m`), in its file's place.
const ENABLED: &str = "\
sub_ev.m:2:1 [E] MCEB: Events can be defined only in a handle class.
sub_nobase.m:2:1 [E] MCEB: Events can be defined only in a handle class.
obs_none.m:2:5 [W] MCPO: SetObservable property has no effect in a value class.
plain.m:2:3 [I] NOPRT: Add a semicolon after the statement to hide the output (in a function).

Found 4 issues in 4 files.
1 fixable with the `--fix` option.
";

#[test]
fn default_run_is_byte_identical_to_the_recorded_output() {
    let out = malt(&project_dir(), &[], &FILES);
    assert_eq!(text(&out.stdout), GOLDEN);
    assert_eq!(text(&out.stderr), "");
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn no_index_flag_is_the_same_as_the_default() {
    let out = malt(&project_dir(), &["--no-index"], &FILES);
    assert_eq!(text(&out.stdout), GOLDEN);
    assert_eq!(text(&out.stderr), "");
}

#[test]
fn index_flag_adds_only_the_two_intended_findings() {
    let out = malt(&project_dir(), &["--index"], &FILES);
    assert_eq!(text(&out.stdout), ENABLED);
    assert_eq!(text(&out.stderr), "");
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn enabled_config_turns_the_index_on() {
    let dir = copy_project(Some("[index]\nenabled = true\n"));
    let out = malt(dir.path(), &[], &FILES);
    assert_eq!(text(&out.stdout), ENABLED);
}

#[test]
fn no_index_flag_overrides_an_enabled_config() {
    let dir = copy_project(Some("[index]\nenabled = true\n"));
    let out = malt(dir.path(), &["--no-index"], &FILES);
    assert_eq!(text(&out.stdout), GOLDEN);
}

#[test]
fn disabled_config_is_the_default() {
    let dir = copy_project(Some("[index]\nenabled = false\n"));
    let out = malt(dir.path(), &[], &FILES);
    assert_eq!(text(&out.stdout), GOLDEN);
}

#[test]
fn index_flag_overrides_a_disabled_config() {
    let dir = copy_project(Some("[index]\nenabled = false\n"));
    let out = malt(dir.path(), &["--index"], &FILES);
    assert_eq!(text(&out.stdout), ENABLED);
}

#[test]
fn index_and_no_index_conflict() {
    let out = malt(&project_dir(), &["--index", "--no-index"], &["plain.m"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("cannot be used with"),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn a_base_on_the_configured_search_path_resolves_from_another_folder() {
    // lib/base_v.m is a value class on the path (paths = ["lib"]); src/sub.m is not in
    // lib and derives from it. The index finds the base through the configured path.
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("lib")).unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("lib/base_v.m"), "classdef base_v\nend\n").unwrap();
    fs::write(
        dir.path().join("src/sub.m"),
        "classdef sub < base_v\n    events\n        Changed\n    end\nend\n",
    )
    .unwrap();
    fs::write(
        dir.path().join(".malt.toml"),
        "[index]\npaths = [\"lib\"]\n",
    )
    .unwrap();

    let off = malt(dir.path(), &[], &["src/sub.m"]);
    assert_eq!(text(&off.stdout), "", "without the index MATLAB is silent");

    let on = malt(dir.path(), &["--index"], &["src/sub.m"]);
    assert_eq!(
        text(&on.stdout),
        "src/sub.m:2:1 [E] MCEB: Events can be defined only in a handle class.\n\
         \n\
         Found 1 issue in 1 file.\n"
    );
    assert_eq!(text(&on.stderr), "");
}

#[test]
fn a_file_outside_the_search_path_finds_its_own_folder_bases() {
    // The linted file is in src, which is not on the path; its base is beside it.
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/base_v.m"), "classdef base_v\nend\n").unwrap();
    fs::write(
        dir.path().join("src/sub.m"),
        "classdef sub < base_v\n    events\n        Changed\n    end\nend\n",
    )
    .unwrap();

    let on = malt(dir.path(), &["--index"], &["src/sub.m"]);
    assert!(text(&on.stdout).contains("MCEB"), "{}", text(&on.stdout));
}

#[test]
fn a_missing_configured_folder_is_a_warning_on_stderr_only() {
    let dir = copy_project(Some("[index]\npaths = [\"nowhere\"]\n"));
    let out = malt(dir.path(), &["--index"], &FILES);
    assert_eq!(text(&out.stdout), ENABLED);
    assert!(
        text(&out.stderr).contains("warning: index:"),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn an_invalid_index_table_is_an_error_even_when_the_index_is_off() {
    let dir = copy_project(Some("[index]\nenable = true\n"));
    let out = malt(dir.path(), &[], &["plain.m"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("enable"),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn config_option_reads_the_index_table_of_that_file() {
    // With --config, the [index] table comes from that file, not from .malt.toml.
    let dir = copy_project(Some("[index]\nenabled = true\n"));
    fs::write(
        dir.path().join("other.toml"),
        "[lint]\npreset = \"mathworks\"\n",
    )
    .unwrap();
    let out = malt(dir.path(), &["--config", "other.toml"], &FILES);
    assert_eq!(text(&out.stdout), GOLDEN);
}
