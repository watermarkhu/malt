//! `[lint] experimental` (plan 6.3): experimental checks are off by default,
//! on with the flag, with `--preset all`, or with an explicit rule setting.

use std::path::Path;
use std::process::{Command, Output};

/// An empty `catch` block: CTCH is on MATLAB's off-list, so it is experimental.
const SOURCE: &str = "try\n    x = 1;\ncatch\nend\n";

fn malt(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_malt"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

fn project(config: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("s.m"), SOURCE).unwrap();
    std::fs::write(dir.path().join(".malt.toml"), config).unwrap();
    dir
}

fn reports_ctch(dir: &Path, args: &[&str]) -> bool {
    let mut all = vec!["s.m"];
    all.extend_from_slice(args);
    String::from_utf8_lossy(&malt(dir, &all).stdout).contains("CTCH")
}

#[test]
fn off_by_default_and_with_the_flag_set_to_false() {
    assert!(!reports_ctch(project("").path(), &[]));
    assert!(!reports_ctch(
        project("[lint]\nexperimental = false\n").path(),
        &[]
    ));
}

#[test]
fn on_with_the_flag_the_all_preset_or_an_explicit_rule() {
    assert!(reports_ctch(
        project("[lint]\nexperimental = true\n").path(),
        &[]
    ));
    assert!(reports_ctch(project("").path(), &["--preset", "all"]));
    assert!(reports_ctch(
        project("[lint]\npreset = \"all\"\n").path(),
        &[]
    ));
}

#[test]
fn the_flag_survives_a_preset_override() {
    let dir = project("[lint]\nexperimental = true\n");
    assert!(reports_ctch(dir.path(), &["--preset", "mathworks"]));
}

#[test]
fn a_non_boolean_value_is_an_error() {
    let dir = project("[lint]\nexperimental = \"yes\"\n");
    let out = malt(dir.path(), &["s.m"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("expected a boolean"));
}

#[test]
fn an_explicit_off_wins_over_the_flag() {
    let dir =
        project("[lint]\nexperimental = true\n\n[lint.rules]\nGOOD_PRACTICES_ENGINE = \"off\"\n");
    assert!(!reports_ctch(dir.path(), &[]));
}
