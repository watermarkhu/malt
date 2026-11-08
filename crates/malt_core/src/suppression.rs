//! Inline suppression directives (`%#ok<RULE_ID>`).
//!
//! MATLAB supports suppressing individual Code Analyzer messages by placing a
//! directive comment at the end of the offending line, e.g.:
//!
//! ```text
//! x = 1  %#ok<NOSEMI>
//! ```
//!
//! This module parses those directives and exposes a [`Suppressions`] map that
//! the [`crate::Linter`] consults to filter out suppressed diagnostics.
//!
//! Supported forms (mirroring MATLAB's `%#ok` syntax):
//!
//! - `%#ok<NOSEMI>` — suppress the single rule `NOSEMI` on that line.
//! - `%#ok<*NOSEMI*>` — same, with optional surrounding `*` wildcards.
//! - `%#ok<*all*>` or `%#ok<*>` — suppress *all* rules on that line.
//! - `%#ok<NOSEMI,AGROW>` — suppress multiple rules (comma-separated).
//! - A `*` inside a rule id is a wildcard: `*X*` matches ids containing `X`,
//!   `X*` matches ids starting with `X`, and `*X` matches ids ending with `X`.
//!
//! A directive applies only to the line it appears on.

use std::collections::{HashMap, HashSet};

/// Suppression directives parsed from a source buffer, keyed by 1-indexed line.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Suppressions {
    /// Lines on which every rule is suppressed (`%#ok<*all*>` / `%#ok<*>`).
    all_lines: HashSet<usize>,
    /// Lines that suppress specific rule IDs (or wildcard patterns).
    rule_lines: HashMap<usize, Vec<String>>,
}

impl Suppressions {
    /// Whether no suppression directives were found.
    pub fn is_empty(&self) -> bool {
        self.all_lines.is_empty() && self.rule_lines.is_empty()
    }

    /// Whether a diagnostic at `line` with `rule_id` is suppressed.
    ///
    /// `line` is 1-indexed, matching [`crate::Diagnostic::line`].
    pub fn is_suppressed(&self, line: usize, rule_id: &str) -> bool {
        if self.all_lines.contains(&line) {
            return true;
        }
        self.rule_lines
            .get(&line)
            .is_some_and(|patterns| patterns.iter().any(|p| glob_match(rule_id, p)))
    }
}

/// Scan `source` line-by-line and collect every `%#ok<...>` directive found.
///
/// A single line may contain multiple directives; each directive may list
/// several comma-separated rule ids. Directives never suppress other lines.
pub fn parse(source: &str) -> Suppressions {
    let mut all_lines = HashSet::new();
    let mut rule_lines: HashMap<usize, Vec<String>> = HashMap::new();

    for (idx, line) in source.lines().enumerate() {
        let line_number = idx + 1;
        let mut search_from = 0;
        while let Some(rel_start) = line[search_from..].find("%#ok<") {
            let content_start = search_from + rel_start + "%#ok<".len();
            let Some(rel_end) = line[content_start..].find('>') else {
                break;
            };
            let content = line[content_start..content_start + rel_end].trim();
            search_from = content_start + rel_end + 1;

            if is_all_directive(content) {
                all_lines.insert(line_number);
                continue;
            }
            for rule in content.split(',') {
                let rule = rule.trim();
                if !rule.is_empty() {
                    rule_lines
                        .entry(line_number)
                        .or_default()
                        .push(rule.to_string());
                }
            }
        }
    }

    Suppressions {
        all_lines,
        rule_lines,
    }
}

/// Whether a directive body means "suppress all rules".
///
/// `*all*`, `*`, and `all` (any `*` placement, case-insensitive) all count.
fn is_all_directive(content: &str) -> bool {
    let stripped: String = content.chars().filter(|c| *c != '*').collect();
    let stripped = stripped.trim();
    stripped.is_empty() || stripped.eq_ignore_ascii_case("all")
}

/// Match a rule id against a directive pattern.
///
/// `*` matches any sequence (including the empty sequence):
/// - `NOSEMI` matches only the exact id `NOSEMI`.
/// - `*NOSEMI*` matches any id containing `NOSEMI`.
/// - `NOSEMI*` matches ids starting with `NOSEMI`.
/// - `*NOSEMI` matches ids ending with `NOSEMI`.
fn glob_match(rule_id: &str, pattern: &str) -> bool {
    let segments: Vec<&str> = pattern.split('*').collect();
    if segments.len() == 1 {
        return rule_id == segments[0];
    }

    // The first segment must match at the start...
    let first = segments[0];
    if !first.is_empty() && !rule_id.starts_with(first) {
        return false;
    }
    // ...and the last segment must match at the end.
    let last = segments[segments.len() - 1];
    if !last.is_empty() && !rule_id.ends_with(last) {
        return false;
    }

    // Middle segments must appear in order between the prefix and suffix.
    let mut search_from = first.len();
    let end_bound = rule_id.len() - last.len();
    for seg in &segments[1..segments.len() - 1] {
        if seg.is_empty() {
            continue;
        }
        if search_from > end_bound {
            return false;
        }
        let Some(pos) = rule_id[search_from..end_bound].find(*seg) else {
            return false;
        };
        search_from += pos + seg.len();
    }
    // The prefix and suffix must not overlap (e.g. "A" does not match "A*A").
    search_from <= end_bound
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_rule_directive_suppresses_exact_match() {
        let s = parse("x = 1; %#ok<NOSEMI>\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(!s.is_suppressed(1, "AGROW"));
        assert!(!s.is_suppressed(2, "NOSEMI"));
    }

    #[test]
    fn surrounding_stars_around_rule_are_contains() {
        let s = parse("x = 1; %#ok<*NOSEMI*>\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(s.is_suppressed(1, "X_NOSEMI_Y"));
        assert!(!s.is_suppressed(1, "AGROW"));
    }

    #[test]
    fn star_all_suppresses_everything_on_line() {
        let s = parse("x = 1; %#ok<*all*>\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(s.is_suppressed(1, "AGROW"));
        assert!(!s.is_suppressed(2, "NOSEMI"));
    }

    #[test]
    fn bare_star_suppresses_everything_on_line() {
        let s = parse("x = 1; %#ok<*>\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(s.is_suppressed(1, "AGROW"));
    }

    #[test]
    fn comma_separated_directive_suppresses_multiple_rules() {
        let s = parse("x = 1; %#ok<NOSEMI,AGROW>\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(s.is_suppressed(1, "AGROW"));
        assert!(!s.is_suppressed(1, "OTHER"));
    }

    #[test]
    fn wildcard_prefix_suffix_contains() {
        let contains = parse("x = 1; %#ok<*SEMI*>\n");
        assert!(contains.is_suppressed(1, "NOSEMI"));
        assert!(!contains.is_suppressed(1, "AGROW"));

        let suffix = parse("x = 1; %#ok<*SEMI>\n");
        assert!(suffix.is_suppressed(1, "NOSEMI"));
        assert!(!suffix.is_suppressed(1, "NOSEMIA"));

        let prefix = parse("x = 1; %#ok<NO*>\n");
        assert!(prefix.is_suppressed(1, "NOSEMI"));
        assert!(!prefix.is_suppressed(1, "AGROW"));
    }

    #[test]
    fn directive_on_other_line_does_not_suppress() {
        let s = parse("x = 1;\n%#ok<NOSEMI>\ny = 2;\n");
        assert!(!s.is_suppressed(1, "NOSEMI"));
        assert!(s.is_suppressed(2, "NOSEMI"));
        assert!(!s.is_suppressed(3, "NOSEMI"));
    }

    #[test]
    fn no_directives_is_empty() {
        assert!(parse("x = 1;\ny = 2;\n").is_empty());
    }

    #[test]
    fn glob_match_edge_cases() {
        assert!(glob_match("NOSEMI", "NOSEMI"));
        assert!(glob_match("NOSEMI", "*"));
        assert!(glob_match("NOSEMI", "**"));
        assert!(glob_match("ABCDEF", "*BCD*"));
        assert!(glob_match("ABCDEF", "A*F"));
        assert!(!glob_match("ABCDEF", "A*E"));
        assert!(glob_match("ABCDEF", "*CD*E*"));
        assert!(!glob_match("AB", "*AB*C*"));
        assert!(glob_match("A", "*A*"));
        assert!(!glob_match("A", "A*A"));
    }
}
