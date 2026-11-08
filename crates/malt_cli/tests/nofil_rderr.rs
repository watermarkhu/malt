//! Integration tests for the NOFIL and RDERR CLI-level guards.
//!
//! NOFIL fires when the input file does not exist; RDERR fires when the input
//! path exists but cannot be read (directory, permission denied, etc.). Both
//! are emitted by `malt_cli` before linting and are not registered rules.

use std::process::Command;

fn malt_bin() -> &'static str {
    env!("CARGO_BIN_EXE_malt")
}

#[test]
fn nofil_fires_on_missing_file() {
    let missing = "/tmp/definitely_missing_malt_test.m";

    let output = Command::new(malt_bin())
        .arg(missing)
        .output()
        .expect("failed to run malt");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("NOFIL"),
        "expected NOFIL in output:\n{stdout}"
    );
    assert!(
        stdout.contains("File is not found"),
        "expected 'File is not found' in output:\n{stdout}"
    );
    assert!(!output.status.success(), "expected non-zero exit status");
}

#[test]
fn nofil_does_not_fire_on_existing_file() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let file_path = dir.path().join("clean.m");
    std::fs::write(&file_path, "x = 1;\n").expect("failed to write test file");

    let output = Command::new(malt_bin())
        .arg(&file_path)
        .output()
        .expect("failed to run malt");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("NOFIL"),
        "expected no NOFIL in output:\n{stdout}"
    );
}

#[test]
fn rderr_fires_on_directory() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");

    let output = Command::new(malt_bin())
        .arg(dir.path())
        .output()
        .expect("failed to run malt");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("RDERR"),
        "expected RDERR in output:\n{stdout}"
    );
    assert!(!output.status.success(), "expected non-zero exit status");
}
