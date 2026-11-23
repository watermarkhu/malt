//! The CLI lints several files side by side; its output must not depend on
//! which thread finishes first: the same lines, in command-line order, as
//! linting each file on its own.

use std::process::Command;

fn malt_bin() -> &'static str {
    env!("CARGO_BIN_EXE_malt")
}

/// The diagnostic lines of a run, without the closing summary.
fn diagnostic_lines(stdout: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(stdout)
        .lines()
        .take_while(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

#[test]
fn files_are_reported_in_command_line_order() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let mut files = Vec::new();
    for i in 0..40 {
        let path = dir.path().join(format!("f{i:02}.m"));
        // Files of different sizes, so threads finish out of order.
        let body = "x = 1\ny = x + 2\n".repeat(1 + (i * 7) % 23);
        std::fs::write(
            &path,
            format!("function r = f{i:02}(a)\n{body}r = a;\nend\n"),
        )
        .expect("failed to write test file");
        files.push(path);
    }
    // A missing file in the middle is reported in its place.
    files.insert(20, dir.path().join("missing.m"));

    let together = Command::new(malt_bin())
        .args(&files)
        .output()
        .expect("failed to run malt");
    let mut one_by_one = Vec::new();
    for file in &files {
        let output = Command::new(malt_bin())
            .arg(file)
            .output()
            .expect("failed to run malt");
        one_by_one.extend(diagnostic_lines(&output.stdout));
    }

    let together = diagnostic_lines(&together.stdout);
    assert!(together.len() > 40, "got {} lines", together.len());
    assert_eq!(together, one_by_one);
}
