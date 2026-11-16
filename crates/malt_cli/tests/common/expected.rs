//! The (path, line, check ID) facts of a MATLAB expectations file, shared by the tests that
//! compare with MATLAB: `matlab_conformance.rs` (malt against the recorded facts) and
//! `matlab_live.rs` (the recorded facts against live MATLAB). Both read the same format.

use std::collections::BTreeSet;

/// One finding: (path relative to the fixtures folder, line, check ID).
pub type Triple = (String, usize, String);

/// The findings of a TSV in the `matlab_expected.tsv` format. Comment lines (`#`) and blank
/// lines are skipped; the first three columns are path, line and check ID.
pub fn parse_triples(text: &str) -> BTreeSet<Triple> {
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

/// The findings of `left` that `right` does not have.
pub fn only_in(left: &BTreeSet<Triple>, right: &BTreeSet<Triple>) -> BTreeSet<Triple> {
    left.difference(right).cloned().collect()
}
