//! Integration tests for the NOFIL and RDERR CLI-level guards.
//!
//! NOFIL fires when the input file does not exist; RDERR fires when the input
//! path exists but cannot be read (permission denied, etc.). A folder is not an
//! unreadable file: it is searched for MATLAB files. Both are emitted by
//! `malt_cli` before linting and are not registered rules.

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

#[cfg(unix)]
#[test]
fn rderr_fires_on_an_unreadable_file() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = dir.path().join("locked.m");
    std::fs::write(&path, "x = 1;\n").expect("failed to write test file");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000))
        .expect("failed to set permissions");
    // Root reads anything: the case cannot be set up.
    if std::fs::read(&path).is_ok() {
        return;
    }

    let output = Command::new(malt_bin())
        .arg(&path)
        .output()
        .expect("failed to run malt");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("RDERR"),
        "expected RDERR in output:\n{stdout}"
    );
    assert!(!output.status.success(), "expected non-zero exit status");
}

#[test]
fn a_directory_is_searched_not_reported() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");

    let output = Command::new(malt_bin())
        .arg(dir.path())
        .output()
        .expect("failed to run malt");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("RDERR"), "{stdout}");
    assert!(
        output.status.success(),
        "an empty folder has nothing to report"
    );
}

/// A file that is not valid UTF-8 is read as Latin-1, as MATLAB reads it, and analyzed: no
/// RDERR. `--fix` leaves it untouched (it would have to be written in another encoding).
#[test]
fn a_latin1_file_is_analyzed_and_not_rewritten_by_fix() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = dir.path().join("latin1.m");
    let source = b"function latin1(x)\n% caf\xe9 \xe9\xe8\ny = 1\nend\n";
    std::fs::write(&path, source).expect("failed to write test file");

    let output = Command::new(malt_bin())
        .arg(&path)
        .output()
        .expect("failed to run malt");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("RDERR"), "{stdout}");
    assert!(
        stdout.contains("INUSD") && stdout.contains("NASGU"),
        "{stdout}"
    );
    assert!(!stdout.contains("--fix"), "no fix is advertised: {stdout}");

    let output = Command::new(malt_bin())
        .arg("--fix")
        .arg(&path)
        .output()
        .expect("failed to run malt");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Fixed"));
    assert_eq!(
        std::fs::read(&path).unwrap(),
        source,
        "the file is not rewritten"
    );
}
