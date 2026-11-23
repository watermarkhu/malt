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

/// One line of an allowlist of differences with MATLAB: where, which way, and why. A path of `*`
/// matches every file and a line of `*` every line: `*\t*\tID\tmalt-only` accepts every
/// malt-only finding of that check (a whole check that is known to differ).
#[derive(Debug, Clone)]
pub struct AllowEntry {
    /// The path relative to the fixtures folder, or `None` for `*`.
    pub path: Option<String>,
    /// The line, or `None` for `*`.
    pub line: Option<usize>,
    /// The check ID.
    pub id: String,
    /// `"malt-only"` (malt reports, MATLAB does not) or `"matlab-only"`.
    pub direction: String,
    /// The reason the difference is accepted (never empty).
    pub reason: String,
}

impl AllowEntry {
    /// Whether this entry accepts the difference `direction` of `triple`.
    pub fn matches(&self, direction: &str, triple: &Triple) -> bool {
        self.direction == direction
            && self.id == triple.2
            && self.path.as_ref().is_none_or(|p| *p == triple.0)
            && self.line.is_none_or(|l| l == triple.1)
    }
}

/// The allowlist of `tests/matlab/all_preset_allowlist.tsv`. Columns, tab-separated: path, line,
/// ID, direction, reason. Panics on a malformed line.
pub fn parse_allowlist(text: &str, what: &str) -> Vec<AllowEntry> {
    let mut out = Vec::new();
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let cols: Vec<&str> = line.splitn(5, '\t').collect();
        let reason = cols.get(4).map_or("", |r| r.trim());
        assert!(
            cols.len() == 5
                && !reason.is_empty()
                && (cols[3] == "malt-only" || cols[3] == "matlab-only"),
            "{what}: each line is path, line, ID, malt-only|matlab-only and a reason: {line}"
        );
        out.push(AllowEntry {
            path: (cols[0] != "*").then(|| cols[0].to_string()),
            line: (cols[1] != "*").then(|| cols[1].parse().expect("a line number or *")),
            id: cols[2].to_string(),
            direction: cols[3].to_string(),
            reason: reason.to_string(),
        });
    }
    out
}
