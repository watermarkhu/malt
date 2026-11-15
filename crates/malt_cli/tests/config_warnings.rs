//! A config file with an unknown rule ID or parameter gives a warning on stderr,
//! and `--strict-config` turns it into an error (exit code 2). The diagnostics on
//! stdout and the exit code of a clean run do not change.

use std::process::{Command, Output};

/// Run malt in `dir` on `a.m` with the given extra arguments.
fn malt(dir: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_malt"))
        .current_dir(dir)
        .args(args)
        .arg("a.m")
        .output()
        .expect("failed to run malt")
}

fn project(config: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    std::fs::write(dir.path().join("a.m"), "function r = a(x)\nr = x;\nend\n")
        .expect("failed to write a.m");
    if let Some(config) = config {
        std::fs::write(dir.path().join(".malt.toml"), config).expect("failed to write config");
    }
    dir
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

const TYPO_CONFIG: &str = "[lint.rules]\nNOSMI = \"off\"\n";

#[test]
fn unknown_rule_key_is_a_warning_and_the_run_succeeds() {
    let dir = project(Some(TYPO_CONFIG));
    let out = malt(dir.path(), &[]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "stderr: {stderr}");
    assert!(stderr.starts_with("warning: .malt.toml: "), "{stderr}");
    assert!(stderr.contains("unknown rule ID 'NOSMI'"), "{stderr}");
    assert!(stderr.contains("did you mean 'NOSEMI'?"), "{stderr}");
}

#[test]
fn stdout_is_the_same_with_and_without_the_warning() {
    let typo = project(Some(TYPO_CONFIG));
    let none = project(None);
    assert_eq!(malt(typo.path(), &[]).stdout, malt(none.path(), &[]).stdout);
}

#[test]
fn unknown_parameter_is_a_warning_with_a_suggestion() {
    let dir = project(Some(
        "[lint.rules.GOOD_PRACTICES_ENGINE]\nmax_variable_nam_length = 40\n",
    ));
    let stderr = text(&malt(dir.path(), &[]).stderr);
    assert!(
        stderr.contains("unknown parameter 'max_variable_nam_length' for rule 'GOOD_PRACTICES_ENGINE' (did you mean 'max_variable_name_length'?)"),
        "{stderr}"
    );
}

#[test]
fn clean_config_prints_nothing_on_stderr() {
    let dir = project(Some(
        "[lint]\npreset = \"all\"\n[lint.rules]\nNOSEMI = \"warn\"\n\"naming.class.casing\" = \"off\"\n",
    ));
    let out = malt(dir.path(), &[]);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty(), "stderr: {}", text(&out.stderr));
}

#[test]
fn strict_config_turns_the_warning_into_exit_code_2() {
    let dir = project(Some(TYPO_CONFIG));
    let out = malt(dir.path(), &["--strict-config"]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "stderr: {stderr}");
    assert!(stderr.contains("unknown rule ID 'NOSMI'"), "{stderr}");
    assert!(
        stderr.contains("error: 1 problem(s) in .malt.toml (--strict-config)"),
        "{stderr}"
    );
}

#[test]
fn strict_config_passes_a_clean_config() {
    let dir = project(Some("[lint.rules]\nNOSEMI = \"warn\"\n"));
    let out = malt(dir.path(), &["--strict-config"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
}

#[test]
fn strict_config_without_a_config_file_is_clean() {
    let dir = project(None);
    let out = malt(dir.path(), &["--strict-config"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
}
