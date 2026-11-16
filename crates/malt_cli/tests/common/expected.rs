//! The (path, line, check ID) facts of a MATLAB expectations file, shared by the tests that
//! compare with MATLAB: `matlab_conformance.rs` (malt against the recorded facts) and
//! `matlab_live.rs` (the recorded facts against live MATLAB). Both read the same format.

use std::collections::{BTreeMap, BTreeSet};

/// One finding: (path relative to the fixtures folder, line, check ID).
pub type Triple = (String, usize, String);

/// The message text of a finding, as the two sides compare it: tabs and line breaks become
/// spaces (the generator writes its messages that way), and the ends are trimmed. Nothing else
/// changes, so a difference in a word, a quote or a space between words is a difference.
pub fn normalise_message(text: &str) -> String {
    text.replace(['\t', '\r', '\n'], " ").trim().to_string()
}

/// The messages of a `checkcode_messages.tsv`: path, line, check ID, then the message (the
/// rest of the line). Several messages can belong to one finding triple.
pub fn parse_messages(text: &str) -> BTreeMap<Triple, Vec<String>> {
    let mut out: BTreeMap<Triple, Vec<String>> = BTreeMap::new();
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let cols: Vec<&str> = line.splitn(4, '\t').collect();
        let triple = (
            cols[0].to_string(),
            cols[1].parse().unwrap(),
            cols[2].to_string(),
        );
        out.entry(triple)
            .or_default()
            .push(normalise_message(cols.get(3).copied().unwrap_or_default()));
    }
    out
}

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
