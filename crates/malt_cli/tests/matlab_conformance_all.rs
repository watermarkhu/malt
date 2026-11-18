//! Conformance with MATLAB's Code Analyzer with every check turned on, on the fixture files.
//!
//! `fixtures/matlab_expected_all.tsv` holds what MATLAB R2026b's `checkcode` reports for every
//! file under `tests/fixtures` when its configuration enables every check
//! (`tests/matlab/generate_checkcode_all.m`: path, line, check ID, no message text). This test
//! lints the same files with `--preset all` and compares the (path, line, ID) sets. The checks
//! that are off by default (the experimental ones, `docs/VERIFY.md`, 'Rule maturity') are
//! compared here, which the default conformance test (`matlab_conformance.rs`) cannot do.
//!
//! The differences that are not fixed are in `tests/matlab/all_preset_allowlist.tsv`: path, line,
//! ID, direction and a reason, each (`*` for the path and the line accepts a whole check). An
//! entry that matches no difference fails the test, so the list only shrinks. The default-preset comparison has no allowlist and stays exact.
//!
//! `matlab_live_all.rs` checks the recorded file against live MATLAB. See `docs/VERIFY.md`,
//! 'Preset all against MATLAB'.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;
use std::path::Path;

mod common;

use common::expected::{only_in, parse_allowlist, parse_triples, AllowEntry, Triple};
use common::fixtures::{fixtures_dir, lint_fixtures_with};
use malt_core::Config;

/// Differences to print in full on a failure; the rest are counted per ID.
const SHOWN: usize = 40;

fn all_config() -> Config {
    Config::from_toml("[lint]\npreset = \"all\"\n").expect("the all preset")
}

fn read_expected() -> BTreeSet<Triple> {
    let text =
        fs::read_to_string(fixtures_dir().join("matlab_expected_all.tsv")).unwrap_or_default();
    parse_triples(&text)
}

fn allowlist() -> Vec<AllowEntry> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/matlab/all_preset_allowlist.tsv");
    let text = fs::read_to_string(&path).unwrap_or_default();
    parse_allowlist(&text, &path.display().to_string())
}

/// All differences with MATLAB: (direction, triple), direction `"matlab-only"` or `"malt-only"`.
fn differences() -> BTreeSet<(String, Triple)> {
    let matlab = read_expected();
    let malt: BTreeSet<Triple> = lint_fixtures_with(all_config)
        .into_iter()
        .map(|(triple, _)| triple)
        .collect();
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
fn all_preset_differences_with_matlab_are_exactly_the_allowlist() {
    let diff = differences();
    let allowed = allowlist();
    if let Some(out) = std::env::var_os("MALT_ALL_DIFF_OUT") {
        let text: String = diff
            .iter()
            .map(|(dir, (p, l, i))| format!("{p}\t{l}\t{i}\t{dir}\n"))
            .collect();
        fs::write(out, text).expect("writing MALT_ALL_DIFF_OUT");
    }
    let unlisted: Vec<&(String, Triple)> = diff
        .iter()
        .filter(|(dir, t)| !allowed.iter().any(|e| e.matches(dir, t)))
        .collect();
    let stale: Vec<&AllowEntry> = allowed
        .iter()
        .filter(|e| !diff.iter().any(|(dir, t)| e.matches(dir, t)))
        .collect();
    eprintln!(
        "{} difference(s) with MATLAB under --preset all, {} allowlisted, {} not, {} stale entries",
        diff.len(),
        diff.len() - unlisted.len(),
        unlisted.len(),
        stale.len()
    );
    if unlisted.is_empty() && stale.is_empty() {
        return;
    }
    let mut text = String::new();
    let _ = writeln!(
        text,
        "{} difference(s) with MATLAB (every check enabled) that are not allowlisted:",
        unlisted.len()
    );
    let mut per_id: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for (dir, (_, _, id)) in &unlisted {
        *per_id.entry((id.as_str(), dir.as_str())).or_default() += 1;
    }
    for ((id, dir), count) in &per_id {
        let _ = writeln!(text, "    {id:<16} {dir:<12} {count}");
    }
    let _ = writeln!(text, "  first differences:");
    for (dir, (path, line, id)) in unlisted.iter().take(SHOWN) {
        let _ = writeln!(text, "    {dir}\t{path}:{line}\t{id}");
    }
    if unlisted.len() > SHOWN {
        let _ = writeln!(text, "    ... and {} more", unlisted.len() - SHOWN);
    }
    for e in stale.iter().take(SHOWN) {
        let _ = writeln!(
            text,
            "  stale allowlist entry (no such difference now, remove it): {}:{}\t{}\t{}",
            e.path.as_deref().unwrap_or("*"),
            e.line.map_or("*".to_string(), |l| l.to_string()),
            e.id,
            e.direction
        );
    }
    if stale.len() > SHOWN {
        let _ = writeln!(text, "  ... and {} more stale entries", stale.len() - SHOWN);
    }
    panic!("{text}\nfix the rule, or accept the difference in tests/matlab/all_preset_allowlist.tsv with a reason");
}
