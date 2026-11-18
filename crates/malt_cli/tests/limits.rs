//! `[lint.limits]`: a file over `max_file_size` is not read and gives MALT_TOO_BIG;
//! raising the limit in the config analyzes it; invalid values are config errors;
//! nesting and depth limits report MALT_TOO_DEEP; `max_threads = 1` changes no output.

use std::path::Path;
use std::process::{Command, Output};

fn malt(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_malt"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("failed to run malt")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn project(config: Option<&str>, files: &[(&str, &[u8])]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    for (name, content) in files {
        std::fs::write(dir.path().join(name), content).expect("failed to write file");
    }
    if let Some(config) = config {
        std::fs::write(dir.path().join(".malt.toml"), config).expect("failed to write config");
    }
    dir
}

/// A file of 4 KiB that has a finding.
fn statements() -> Vec<u8> {
    "x = 1\n".repeat(700).into_bytes()
}

#[test]
fn a_file_over_max_file_size_is_not_read() {
    // Eight KiB of 0xff bytes: analyzed (as Latin-1) it would give many findings, so the single
    // MALT_TOO_BIG finding below proves the file was refused before it was read.
    let mut bytes = vec![0xff; 8 * 1024];
    bytes.push(b'\n');
    let dir = project(
        Some("[lint.limits]\nmax_file_size = \"1 KiB\"\n"),
        &[("big.m", &bytes)],
    );
    let out = malt(dir.path(), &["big.m"]);
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{stdout}");
    assert!(stdout.contains("big.m:1:1 [E] MALT_TOO_BIG: "), "{stdout}");
    assert!(
        stdout.contains("8193 bytes") && stdout.contains("1 KiB"),
        "{stdout}"
    );
    assert!(stdout.contains("max_file_size"), "{stdout}");
    assert!(!stdout.contains("RDERR"), "{stdout}");
    assert!(stdout.contains("Found 1 issue in 1 file."), "{stdout}");
}

#[test]
fn raising_the_limit_analyzes_the_file() {
    let source = statements();
    let small = project(
        Some("[lint.limits]\nmax_file_size = 1024\n"),
        &[("a.m", &source)],
    );
    let out = text(&malt(small.path(), &["a.m"]).stdout);
    assert!(
        out.contains("MALT_TOO_BIG") && !out.contains("NOPTS"),
        "{out}"
    );

    let raised = project(
        Some("[lint.limits]\nmax_file_size = \"1 MiB\"\n"),
        &[("a.m", &source)],
    );
    let out = text(&malt(raised.path(), &["a.m"]).stdout);
    assert!(
        out.contains("NOPTS") && !out.contains("MALT_TOO_BIG"),
        "{out}"
    );
}

#[test]
fn the_default_limit_is_5_mib() {
    // A sparse file just over 5 MiB: never read, never allocated.
    let dir = project(None, &[]);
    let file = std::fs::File::create(dir.path().join("huge.m")).unwrap();
    file.set_len(5 * 1024 * 1024 + 1).unwrap();
    drop(file);
    let out = malt(dir.path(), &["huge.m"]);
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{stdout}");
    assert!(
        stdout.contains("MALT_TOO_BIG") && stdout.contains("5 MiB"),
        "{stdout}"
    );
}

#[test]
fn other_files_of_the_run_are_still_analyzed() {
    let dir = project(
        Some("[lint.limits]\nmax_file_size = 2000\n"),
        &[("a.m", &statements()), ("b.m", b"x = 1\n")],
    );
    let out = text(&malt(dir.path(), &["a.m", "b.m"]).stdout);
    assert!(out.contains("a.m:1:1 [E] MALT_TOO_BIG"), "{out}");
    assert!(out.contains("b.m:1:3"), "{out}");
    assert!(out.contains("NOPTS"), "{out}");
}

#[test]
fn the_size_limit_applies_with_fix_and_leaves_the_file_alone() {
    let source = statements();
    let dir = project(
        Some("[lint.limits]\nmax_file_size = 1024\n"),
        &[("a.m", &source)],
    );
    let out = malt(dir.path(), &["--fix", "a.m"]);
    assert!(text(&out.stdout).contains("MALT_TOO_BIG"));
    assert_eq!(std::fs::read(dir.path().join("a.m")).unwrap(), source);
}

#[test]
fn nesting_over_max_nesting_is_refused_before_the_parse() {
    let source = format!("x = {}1{};\n", "(".repeat(40), ")".repeat(40));
    let dir = project(
        Some("[lint.limits]\nmax_nesting = 30\n"),
        &[("a.m", source.as_bytes())],
    );
    let stdout = text(&malt(dir.path(), &["a.m"]).stdout);
    assert!(stdout.contains("[E] MALT_TOO_DEEP: "), "{stdout}");
    assert!(stdout.contains("max_nesting"), "{stdout}");
    assert!(!stdout.contains("MDEEP"), "{stdout}");
    // Brackets inside strings and comments are not counted.
    let quiet =
        "x = '((((((((((((((((((((((((((((((((((((((((';% (((((((((((((((((((((((((((((((\n";
    std::fs::write(dir.path().join("b.m"), quiet).unwrap();
    assert!(!text(&malt(dir.path(), &["b.m"]).stdout).contains("MALT_TOO_DEEP"));
}

#[test]
fn max_tree_depth_is_configurable() {
    let source = "if a\nif b\nif c\nx = 1;\nend\nend\nend\n";
    let low = project(
        Some("[lint.limits]\nmax_tree_depth = 5\n"),
        &[("a.m", source.as_bytes())],
    );
    let stdout = text(&malt(low.path(), &["a.m"]).stdout);
    assert!(
        stdout.contains("MALT_TOO_DEEP") && stdout.contains("max_tree_depth"),
        "{stdout}"
    );
    let default = project(None, &[("a.m", source.as_bytes())]);
    assert!(!text(&malt(default.path(), &["a.m"]).stdout).contains("MALT_TOO_DEEP"));
}

#[test]
fn invalid_limits_are_config_errors() {
    for (config, needle) in [
        ("[lint.limits]\nmax_file_size = 0\n", "max_file_size"),
        (
            "[lint.limits]\nmax_file_size = \"lots\"\n",
            "cannot read 'lots'",
        ),
        (
            "[lint.limits]\nmax_file_size = \"5 XB\"\n",
            "unknown size unit",
        ),
        ("[lint.limits]\nmax_tree_depth = -3\n", "max_tree_depth"),
        ("[lint.limits]\nmax_nesting = 0\n", "max_nesting"),
        ("[lint.limits]\nmax_threads = 0\n", "max_threads"),
    ] {
        let dir = project(Some(config), &[("a.m", b"x = 1;\n")]);
        let out = malt(dir.path(), &["a.m"]);
        let stderr = text(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{config}: {stderr}");
        assert!(
            stderr.contains("[lint.limits]") && stderr.contains(needle),
            "{config}: {stderr}"
        );
    }
}

#[test]
fn unknown_limit_keys_are_warnings_and_strict_config_makes_them_errors() {
    let dir = project(
        Some("[lint.limits]\nmax_file_sise = \"1 MiB\"\n"),
        &[("a.m", b"x = 1;\n")],
    );
    let out = malt(dir.path(), &["a.m"]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert!(
        stderr.contains(
            "unknown key 'max_file_sise' in [lint.limits] (did you mean 'max_file_size'?)"
        ),
        "{stderr}"
    );
    let out = malt(dir.path(), &["--strict-config", "a.m"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn max_threads_one_gives_the_same_output() {
    let files: Vec<(String, Vec<u8>)> = (0..6)
        .map(|i| {
            (
                format!("f{i}.m"),
                format!("x{i} = {i}\ny = 2\n").into_bytes(),
            )
        })
        .collect();
    let named: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(n, c)| (n.as_str(), c.as_slice()))
        .collect();
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    let single = project(Some("[lint.limits]\nmax_threads = 1\n"), &named);
    let default = project(None, &named);
    let one = malt(single.path(), &names);
    let many = malt(default.path(), &names);
    assert_eq!(one.status.code(), Some(1));
    assert_eq!(one.stdout, many.stdout);
    assert!(text(&one.stdout).contains("f5.m:"));
}

#[test]
fn normal_files_are_unaffected_by_default_limits() {
    let dir = project(None, &[("a.m", b"function r = a(x)\nr = x\nend\n")]);
    let out = text(&malt(dir.path(), &["a.m"]).stdout);
    assert_eq!(
        out.lines().next().unwrap(),
        "a.m:2:3 [I] NOPRT: Add a semicolon after the statement to hide the output (in a function)."
    );
}

/// Every `.m` file under `dir`, recursively.
fn matlab_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            matlab_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "m") {
            out.push(path);
        }
    }
}

#[test]
fn the_nesting_scan_refuses_no_fixture() {
    let mut files = Vec::new();
    matlab_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
        &mut files,
    );
    assert!(files.len() > 100, "{} fixtures", files.len());
    for file in files {
        let source = String::from_utf8_lossy(&std::fs::read(&file).unwrap()).into_owned();
        // The fixtures that probe MATLAB's own limits nest up to a few hundred
        // brackets; the default of 10,000 is far above, and the scan must not
        // invent depth that is not there.
        assert!(
            !malt_core::limits::exceeds_nesting(&source, malt_core::limits::DEFAULT_MAX_NESTING),
            "{}",
            file.display()
        );
        assert!(
            !malt_core::limits::exceeds_nesting(&source, malt_core::limits::WASM_MAX_NESTING),
            "{} (wasm)",
            file.display()
        );
        assert!(source.len() as u64 <= malt_core::limits::WASM_MAX_FILE_SIZE);
    }
}
