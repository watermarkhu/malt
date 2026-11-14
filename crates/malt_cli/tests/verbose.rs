//! `-v` and `-vv` report on stderr which files are linted and, with `-vv`,
//! the time and the findings of each file. They never change stdout.

use std::process::{Command, Output};

fn malt(args: &[&str], files: &[&std::path::Path]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_malt"))
        .args(args)
        .args(files)
        .output()
        .expect("failed to run malt")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn project() -> (tempfile::TempDir, Vec<std::path::PathBuf>) {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let mut files = Vec::new();
    for (name, body) in [
        ("a.m", "function r = a(x)\nr = x\nend\n"),
        ("b.m", "function r = b(x)\nr = x;\nend\n"),
    ] {
        let path = dir.path().join(name);
        std::fs::write(&path, body).expect("failed to write test file");
        files.push(path);
    }
    files.push(dir.path().join("missing.m"));
    (dir, files)
}

#[test]
fn quiet_by_default() {
    let (_dir, files) = project();
    let refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();
    let output = malt(&[], &refs);
    assert!(text(&output.stderr).is_empty());
}

#[test]
fn v_lists_each_file_in_order_and_the_total() {
    let (_dir, files) = project();
    let refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();
    let stderr = text(&malt(&["-v"], &refs).stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 4, "got: {stderr}");
    assert_eq!(lines[0], format!("malt: linted {}", files[0].display()));
    assert_eq!(lines[1], format!("malt: linted {}", files[1].display()));
    assert_eq!(
        lines[2],
        format!("malt: skipped {} (NOFIL)", files[2].display())
    );
    assert!(lines[3].starts_with("malt: 3 files in "), "got: {stderr}");
}

#[test]
fn vv_adds_time_and_findings_per_file() {
    let (_dir, files) = project();
    let refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();
    let stderr = text(&malt(&["-vv"], &refs).stderr);
    let first = stderr.lines().next().expect("no output");
    let prefix = format!("malt: linted {} in ", files[0].display());
    assert!(first.starts_with(&prefix), "got: {first}");
    // "<time> (<n> finding(s))" with a µs, ms or s unit.
    let rest = &first[prefix.len()..];
    assert!(
        rest.contains(" µs (") || rest.contains(" ms (") || rest.contains(" s ("),
        "got: {first}"
    );
    assert!(
        rest.ends_with("finding)") || rest.ends_with("findings)"),
        "got: {first}"
    );
    // The clean file has no findings.
    let second = stderr.lines().nth(1).expect("no second line");
    assert!(second.ends_with("(0 findings)"), "got: {second}");
}

#[test]
fn verbosity_does_not_change_stdout() {
    let (_dir, files) = project();
    let refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();
    let plain = malt(&[], &refs);
    for flags in [&["-v"][..], &["-vv"][..], &["--verbose", "--verbose"][..]] {
        let verbose = malt(flags, &refs);
        assert_eq!(plain.stdout, verbose.stdout, "flags: {flags:?}");
        assert_eq!(plain.status.code(), verbose.status.code());
    }
}

#[test]
fn verbose_lists_files_when_fixing_too() {
    let (_dir, files) = project();
    let refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();
    let stderr = text(&malt(&["--fix", "-v"], &refs).stderr);
    assert!(stderr.contains(&format!("malt: linted {}", files[0].display())));
}
