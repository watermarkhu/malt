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
//! A directive applies to the statement it ends: the messages from the first
//! line of that statement down to the directive's line (see
//! [`Suppressions::covers`]).

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use tree_sitter::{Node, Tree};

/// A check ID named by a directive, as MATLAB reports an unused one (MSNU).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedId {
    /// The ID as written. Wildcard patterns (`*`) are not listed.
    pub id: String,
    /// 1-indexed line of the directive.
    pub line: usize,
    /// 1-indexed column MATLAB reports: the `%` when the directive names one
    /// ID, the ID itself when it names several.
    pub column: usize,
    /// Byte range of the same position in the source.
    pub byte_range: Range<usize>,
}

/// Suppression directives parsed from a source buffer, keyed by 1-indexed line.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Suppressions {
    /// Lines on which every rule is suppressed (`%#ok<*all*>` / `%#ok<*>`).
    all_lines: HashSet<usize>,
    /// Lines that suppress specific rule IDs (or wildcard patterns).
    rule_lines: HashMap<usize, Vec<String>>,
    /// Exact IDs named by the directives, in source order.
    named: Vec<NamedId>,
    /// For each directive line, the first line of the statement it ends (see
    /// [`Suppressions::covers`]). A line missing from the map starts its own.
    group_start: HashMap<usize, usize>,
}

impl Suppressions {
    /// Whether no suppression directives were found.
    pub fn is_empty(&self) -> bool {
        self.all_lines.is_empty() && self.rule_lines.is_empty()
    }

    /// The exact check IDs named by the directives (wildcards excluded).
    pub fn named_ids(&self) -> &[NamedId] {
        &self.named
    }

    /// Whether a diagnostic at `line` with `rule_id` is suppressed by any directive.
    ///
    /// `line` is 1-indexed, matching [`crate::Diagnostic::line`].
    pub fn is_suppressed(&self, line: usize, rule_id: &str) -> bool {
        self.all_lines
            .iter()
            .chain(self.rule_lines.keys())
            .any(|&directive| self.suppresses_from(directive, line, rule_id))
    }

    /// Whether the directive on line `directive` suppresses a diagnostic at
    /// `line` with `rule_id`.
    pub fn suppresses_from(&self, directive: usize, line: usize, rule_id: &str) -> bool {
        if !self.covers(directive, line) {
            return false;
        }
        if self.all_lines.contains(&directive) {
            return true;
        }
        self.rule_lines
            .get(&directive)
            .is_some_and(|patterns| patterns.iter().any(|p| glob_match(rule_id, p)))
    }

    /// Whether the directive on line `directive` covers a message on line `line`.
    ///
    /// MATLAB associates a directive with the statement it ends: it covers the
    /// messages from the first line of that statement down to its own line. So
    /// a directive on the last line of `x = [a ...` / `b];` covers the message
    /// on the first line, but a directive on the first line does not cover one
    /// on a later line, and a directive after a compound statement's `end` does
    /// not cover its header.
    pub fn covers(&self, directive: usize, line: usize) -> bool {
        let start = self
            .group_start
            .get(&directive)
            .copied()
            .unwrap_or(directive);
        start <= line && line <= directive
    }
}

/// Node kinds that can span several lines as one statement, so that a directive
/// on a later line of them covers the earlier lines. Compound statements (`if`,
/// loops, functions, classes) are left out: their body would join the header.
const GROUPING_KINDS: &[&str] = &[
    "assignment",
    "command",
    "function_call",
    "matrix",
    "cell",
    "binary_operator",
    "comparison_operator",
    "boolean_operator",
    "boolean_operator_short",
    "parenthesis",
    "unary_operator",
    "postfix_operator",
    "range",
    "field_expression",
    "lambda",
];

/// The first line of the statement each directive line ends, for
/// [`Suppressions::covers`]: the earliest multi-line node of [`GROUPING_KINDS`]
/// that spans the directive line.
fn group_starts(
    tree: &Tree,
    directive_lines: impl Iterator<Item = usize>,
) -> HashMap<usize, usize> {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        if GROUPING_KINDS.contains(&node.kind())
            && node.start_position().row < node.end_position().row
        {
            spans.push((node.start_position().row + 1, node.end_position().row + 1));
        }
        let mut cursor = node.walk();
        stack.extend(node.children(&mut cursor));
    }
    directive_lines
        .map(|line| {
            let start = spans
                .iter()
                .filter(|&&(start, end)| start < line && line <= end)
                .map(|&(start, _)| start)
                .min()
                .unwrap_or(line);
            (line, start)
        })
        .collect()
}

/// Parse the directives of `source`, building the syntax tree first.
///
/// The linter calls [`from_tree`] with the tree it already has.
pub fn parse(source: &str) -> Suppressions {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_matlab::LANGUAGE.into())
        .expect("failed to load tree-sitter-matlab grammar");
    match parser.parse(source, None) {
        Some(tree) => from_tree(source, &tree),
        None => Suppressions::default(),
    }
}

/// Collect the directives of `source` from the comments of its syntax tree.
///
/// MATLAB reads a directive only where a comment starts with `%#ok`: `%%#ok`,
/// `% note %#ok<X>`, a `%#ok` inside a string and one after a `...`
/// continuation are not directives. A comment holds one directive, so only
/// the first of several on a line counts. A directive covers only the lines of
/// the statement it ends.
pub fn from_tree(source: &str, tree: &Tree) -> Suppressions {
    let mut suppressions = Suppressions::default();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        if node.kind() == "comment" {
            read_directive(source, node, &mut suppressions);
            continue;
        }
        let mut cursor = node.walk();
        stack.extend(node.children(&mut cursor));
    }
    suppressions.named.sort_by_key(|n| n.byte_range.start);
    let directive_lines: HashSet<usize> = suppressions
        .all_lines
        .iter()
        .chain(suppressions.rule_lines.keys())
        .copied()
        .collect();
    suppressions.group_start = group_starts(tree, directive_lines.into_iter());
    suppressions
}

/// Record the directive at the start of one comment, if it has one.
fn read_directive(source: &str, comment: Node, out: &mut Suppressions) {
    let text = &source[comment.byte_range()];
    let Some(rest) = text.strip_prefix("%#ok") else {
        return;
    };
    let line = comment.start_position().row + 1;
    let Some(body) = rest.strip_prefix('<') else {
        // A bare `%#ok` is followed by nothing, or by white space.
        if rest.is_empty() || rest.starts_with(char::is_whitespace) {
            out.all_lines.insert(line);
        }
        return;
    };

    // The IDs run to the closing `>`, or to the end of the comment when unclosed.
    let content = &body[..body.find('>').unwrap_or(body.len())];
    if is_all_directive(content) {
        out.all_lines.insert(line);
        return;
    }

    // Each ID with its offset in the comment (for the column of the ID).
    let mut names: Vec<(usize, &str)> = Vec::new();
    let mut part_start = 0;
    for part in content.split(',') {
        let name = part.trim();
        if !name.is_empty() {
            let name_start = part_start + (part.len() - part.trim_start().len());
            names.push((name_start, name));
        }
        part_start += part.len() + 1;
    }

    let single = names.len() == 1;
    for (name_start, name) in names {
        out.rule_lines
            .entry(line)
            .or_default()
            .push(name.to_string());
        if name.contains('*') {
            continue;
        }
        let byte = if single {
            comment.start_byte()
        } else {
            comment.start_byte() + "%#ok<".len() + name_start
        };
        let width = if single { 1 } else { name.len() };
        out.named.push(NamedId {
            id: name.to_string(),
            line,
            column: column_of(source, byte),
            byte_range: byte..byte + width,
        });
    }
}

/// 1-indexed column of byte `byte` within its line.
fn column_of(source: &str, byte: usize) -> usize {
    let line_start = source[..byte].rfind('\n').map_or(0, |p| p + 1);
    source[line_start..byte].chars().count() + 1
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

    // MATLAB probes (R2026b checkcode): a bare `%#ok` suppresses every message on
    // its line; a directive counts only at the start of a comment, not in a string
    // or after `...`; the first directive of a comment is the one read.

    #[test]
    fn bare_directive_suppresses_every_rule_on_line() {
        let s = parse("x = 1; %#ok\ny = 2;\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(!s.is_suppressed(2, "NOSEMI"));
    }

    #[test]
    fn directive_on_last_line_covers_the_first_line_of_its_statement() {
        // MATLAB probe: `lines{end+1} = sprintf(..., ...` / `k); %#ok<AGROW>`
        // reports no AGROW on the first line.
        let s = parse("x = [1 ...\n 2]; %#ok<NOSEMI>\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(!s.is_suppressed(3, "NOSEMI"));
    }

    #[test]
    fn directive_covers_the_lines_of_a_matrix_above_it() {
        // MATLAB probe: `x = [iscellstr(v)` / `2 %#ok<ISCLSTR>` / `3];` reports nothing.
        let s = parse("x = [1\n 2 %#ok<NOSEMI>\n 3];\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(s.is_suppressed(2, "NOSEMI"));
        assert!(!s.is_suppressed(3, "NOSEMI"));
    }

    #[test]
    fn directive_on_first_line_does_not_cover_a_later_line() {
        // MATLAB probe: `z = [1, 2, %#ok` / `iscellstr(v)];` still reports ISCLSTR
        // on the second line.
        let s = parse("x = [1 %#ok<NOSEMI>\n 2];\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(!s.is_suppressed(2, "NOSEMI"));
    }

    #[test]
    fn directive_covers_an_earlier_statement_on_the_same_physical_line() {
        // MATLAB probe: `x3 = iscellstr(v); y3 = [1 ...` / `2]; %#ok<ISCLSTR>`
        // reports nothing on the first line.
        let s = parse("x = 1; y = [1 ...\n 2]; %#ok<NOSEMI>\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
    }

    #[test]
    fn end_of_a_compound_statement_does_not_cover_its_header() {
        // MATLAB probe: `if iscellstr(v) && ...` / `n > 0` / `end %#ok<ISCLSTR>`
        // still reports ISCLSTR on the header line.
        let s = parse("if a && ...\n n > 0\n disp(1)\nend %#ok<NOSEMI>\n");
        assert!(!s.is_suppressed(1, "NOSEMI"));
        assert!(s.is_suppressed(4, "NOSEMI"));
    }

    #[test]
    fn directive_on_a_continued_condition_covers_the_condition() {
        // MATLAB probe: `if iscellstr(v) || ...` / `n > 0 %#ok<ISCLSTR>` reports nothing.
        let s = parse("if a || ...\n b %#ok<NOSEMI>\n disp(1)\nend\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
    }

    #[test]
    fn directive_must_start_the_comment() {
        assert!(parse("x = 1; % note %#ok<NOSEMI>\n").is_empty());
        assert!(parse("x = 1; %%#ok<NOSEMI>\n").is_empty());
    }

    #[test]
    fn directive_inside_a_string_is_not_a_directive() {
        assert!(parse("disp('%#ok<NOSEMI>'); x = 1;\n").is_empty());
    }

    #[test]
    fn directive_after_a_continuation_is_not_a_directive() {
        assert!(parse("x = [1, ... %#ok<NOSEMI>\n 2];\n").is_empty());
    }

    #[test]
    fn only_the_first_directive_of_a_comment_counts() {
        let s = parse("x = 1; %#ok<AGROW> %#ok<NOSEMI>\n");
        assert!(s.is_suppressed(1, "AGROW"));
        assert!(!s.is_suppressed(1, "NOSEMI"));
        assert_eq!(s.named_ids().len(), 1);
    }

    #[test]
    fn unclosed_directive_runs_to_the_end_of_the_comment() {
        assert!(parse("x = 1; %#ok<NOSEMI\n").is_suppressed(1, "NOSEMI"));
    }

    #[test]
    fn named_ids_carry_the_column_matlab_reports() {
        // One ID: the `%` (column 8). Several IDs: each ID's own column.
        let one = parse("x = 1; %#ok<AGROW>\n");
        assert_eq!(one.named_ids()[0].column, 8);
        let several = parse("x = 1; %#ok<NOSEMI,AGROW>\n");
        let columns: Vec<usize> = several.named_ids().iter().map(|n| n.column).collect();
        assert_eq!(columns, vec![13, 20]);
    }

    #[test]
    fn wildcards_are_not_named_ids() {
        assert!(parse("x = 1; %#ok<*NOSEMI*>\n").named_ids().is_empty());
        assert!(parse("x = 1; %#ok<*>\n").named_ids().is_empty());
    }
}
