//! Conformance with MATLAB's Code Analyzer on the fixture files.
//!
//! `fixtures/matlab_expected.tsv` holds what MATLAB R2026b's `checkcode` reports
//! for every file under `tests/fixtures` (path, line, check ID: facts only, no
//! message text). This test lints the same files with the default configuration
//! and requires the (path, line, ID) sets to be identical. Any difference, a missed
//! finding or an extra one, fails the test.
//!
//! A new fixture needs its MATLAB expectations: run `docs/scripts/matlab_probe.m`
//! over it and append the result to `matlab_expected.tsv` (see `docs/VERIFY.md`).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use malt_core::{Config, Linter, RuleRegistry};

type Triple = (String, usize, String);

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_expected() -> BTreeSet<Triple> {
    let text = fs::read_to_string(fixtures().join("matlab_expected.tsv")).unwrap_or_default();
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let cols: Vec<&str> = l.split('\t').collect();
            (
                cols[0].to_string(),
                cols[1].parse().unwrap(),
                cols[2].to_string(),
            )
        })
        .collect()
}

fn mfiles(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            mfiles(&path, out);
        } else if path.extension().is_some_and(|e| e == "m") {
            out.push(path);
        }
    }
}

/// Everything malt reports on the fixtures, as (path relative to fixtures, line, ID).
fn malt_findings() -> BTreeSet<Triple> {
    let root = fixtures();
    let mut files = Vec::new();
    mfiles(&root, &mut files);
    files.sort();
    let config = Config::default();
    let registry = RuleRegistry::new(malt_rules::active_rules(&config), &config);
    let mut linter = Linter::new(registry);
    let mut found = BTreeSet::new();
    for path in files {
        let source = fs::read_to_string(&path).unwrap();
        let rel = path
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        for d in linter.lint(&source, &path).unwrap() {
            found.insert((rel.clone(), d.line, d.rule_id.to_string()));
        }
    }
    found
}

/// All differences with MATLAB: ("matlab-only" | "malt-only", triple).
fn differences() -> BTreeSet<(String, Triple)> {
    let matlab = read_expected();
    let malt = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(malt_findings)
        .unwrap()
        .join()
        .unwrap();
    let mut diff = BTreeSet::new();
    for t in matlab.difference(&malt) {
        diff.insert(("matlab-only".to_string(), t.clone()));
    }
    for t in malt.difference(&matlab) {
        diff.insert(("malt-only".to_string(), t.clone()));
    }
    diff
}

#[test]
fn no_differences_with_matlab() {
    let diff = differences();
    assert!(
        diff.is_empty(),
        "{} difference(s) with MATLAB:\n{}",
        diff.len(),
        diff.iter()
            .take(40)
            .map(|(k, (p, l, i))| format!("  {k}\t{p}:{l}\t{i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
