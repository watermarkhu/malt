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
//! `matlab_live.rs` checks that the recorded file still matches a live MATLAB.
//! `fixtures/matlab_files.txt` must list every fixture file (`fixtures/README.md`).

use std::collections::BTreeSet;
use std::fs;

mod common;

use common::expected::{only_in, parse_triples, Triple};
use common::fixtures::{fixture_files, fixtures_dir, lint_fixtures, listed_files, relative};

fn read_expected() -> BTreeSet<Triple> {
    let text = fs::read_to_string(fixtures_dir().join("matlab_expected.tsv")).unwrap_or_default();
    parse_triples(&text)
}

/// Everything malt reports on the fixtures, as (path relative to fixtures, line, ID).
fn malt_findings() -> BTreeSet<Triple> {
    lint_fixtures()
        .into_iter()
        .map(|(triple, _)| triple)
        .collect()
}

/// All differences with MATLAB: ("matlab-only" | "malt-only", triple).
fn differences() -> BTreeSet<(String, Triple)> {
    let matlab = read_expected();
    let malt = malt_findings();
    let mut diff = BTreeSet::new();
    for t in only_in(&matlab, &malt) {
        diff.insert(("matlab-only".to_string(), t));
    }
    for t in only_in(&malt, &matlab) {
        diff.insert(("malt-only".to_string(), t));
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

#[test]
fn fixture_list_names_every_fixture_file() {
    let on_disk: BTreeSet<String> = fixture_files().iter().map(|p| relative(p)).collect();
    let listed = listed_files();
    let unlisted: Vec<&String> = on_disk.difference(&listed).collect();
    let missing: Vec<&String> = listed.difference(&on_disk).collect();
    assert!(
        unlisted.is_empty() && missing.is_empty(),
        "tests/fixtures/matlab_files.txt does not match the .m files under tests/fixtures.\n\
         not listed ({}): {}\nlisted but no file ({}): {}\n\
         Regenerate the list with the command in tests/fixtures/README.md, and see that file for \
         how a new fixture is added.",
        unlisted.len(),
        unlisted
            .iter()
            .take(10)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        missing.len(),
        missing
            .iter()
            .take(10)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    );
}
