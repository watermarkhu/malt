//! `--level` shows findings of that severity and more severe ones; hidden findings are not
//! counted and do not change the exit code, and `--fix` still fixes them.

use std::path::Path;
use std::process::{Command, Output};

/// A syntax error (E), a name warning (W: FNDEF, W: NASGU) and an info finding (I: NOPRT).
const MIXED: &str = "function f(x)\ny = 1\nif x = 1\nend\n";
/// Only warnings and info: a clean function with a missing semicolon and a wrong name.
const NO_ERRORS: &str = "function r = b(x)\nr = x\nend\n";

fn malt(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_malt"))
        .current_dir(dir)
        .args(args)
        .env_remove("MALT_CONFIG")
        .output()
        .expect("failed to run malt")
}

fn project(source: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    std::fs::write(dir.path().join("a.m"), source).expect("failed to write test file");
    dir
}

fn tags(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|l| l.split_whitespace().nth(1))
        .filter(|t| t.starts_with('['))
        .map(str::to_string)
        .collect()
}

#[test]
fn error_level_shows_only_errors() {
    let dir = project(MIXED);
    let out = malt(dir.path(), &["--level", "error", "a.m"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(!tags(&out).is_empty());
    assert!(tags(&out).iter().all(|t| t == "[E]"), "{:?}", tags(&out));
}

#[test]
fn warning_level_shows_errors_and_warnings() {
    let dir = project(MIXED);
    let out = malt(dir.path(), &["--level", "warning", "a.m"]);
    let tags = tags(&out);
    assert!(tags.iter().any(|t| t == "[E]") && tags.iter().any(|t| t == "[W]"));
    assert!(!tags.iter().any(|t| t == "[I]"), "{tags:?}");
}

#[test]
fn info_level_is_the_default() {
    let dir = project(MIXED);
    let all = malt(dir.path(), &["a.m"]);
    let info = malt(dir.path(), &["--level", "info", "a.m"]);
    assert_eq!(all.stdout, info.stdout);
    assert!(tags(&all).iter().any(|t| t == "[I]"));
}

#[test]
fn warn_is_an_alias_of_warning() {
    let dir = project(MIXED);
    let a = malt(dir.path(), &["--level", "warn", "a.m"]);
    let b = malt(dir.path(), &["--level", "warning", "a.m"]);
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn hidden_findings_are_not_counted_and_do_not_set_the_exit_code() {
    let dir = project(NO_ERRORS);
    let all = malt(dir.path(), &["a.m"]);
    assert_eq!(all.status.code(), Some(1));
    let out = malt(dir.path(), &["--level", "error", "a.m"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(
        out.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn the_summary_counts_only_what_is_shown() {
    let dir = project(MIXED);
    let out = malt(dir.path(), &["--level", "error", "a.m"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Found 1 issue in 1 file."), "{text}");
    assert!(!text.contains("fixable"), "{text}");
}

#[test]
fn an_unreadable_file_is_an_error_and_always_shown() {
    let dir = project(MIXED);
    let out = malt(dir.path(), &["--level", "error", "missing.m"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("NOFIL"));
}

#[test]
fn fix_still_fixes_findings_the_level_hides() {
    let dir = project(NO_ERRORS);
    let out = malt(dir.path(), &["--level", "error", "--fix", "a.m"]);
    assert_eq!(out.status.code(), Some(0));
    let fixed = std::fs::read_to_string(dir.path().join("a.m")).unwrap();
    assert!(fixed.contains("r = x;"), "{fixed}");
}

#[test]
fn an_unknown_level_is_a_usage_error() {
    let dir = project(MIXED);
    let out = malt(dir.path(), &["--level", "loud", "a.m"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("possible values"));
}
