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
//! Supported forms (mirroring MATLAB's `%#ok` syntax, probed against R2026b):
//!
//! - `%#ok` — suppress every message on the statement it ends (bare form).
//! - `%#ok<NOSEMI>` — suppress the rule `NOSEMI` on the statement it ends.
//! - `%#ok<NOSEMI,AGROW>` — suppress several rules (comma-separated).
//! - `%#ok<*NOSEMI>` — suppress `NOSEMI` in the whole file, wherever the
//!   directive is. Mixed lists such as `%#ok<AGROW,*NOSEMI>` are allowed.
//!
//! IDs are exact and case-sensitive. `%#ok<*NOSEMI*>`, `%#ok<NO*>`, `%#ok<*>`,
//! `%#ok<*all*>` and an ID with a space in it match nothing.
//!
//! A line directive applies to the statement it ends: the messages from the
//! first line of that statement down to the directive's line (see
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
    /// Byte range of the whole directive comment (`%#ok<...>` and what follows).
    pub comment: Range<usize>,
    /// Byte range of this ID inside the `<...>` list; empty for a bare `%#ok`.
    pub part: Range<usize>,
    /// Number of entries in the `<...>` list (wildcards included).
    pub parts: usize,
    /// Index of this ID in the list.
    pub index: usize,
    /// Byte offset just past the closing `>`, when the list is well formed
    /// (closed, no empty entry); only such a directive gets an MSNU fix.
    pub list_end: Option<usize>,
}

/// Suppression directives parsed from a source buffer, keyed by 1-indexed line.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Suppressions {
    /// Lines on which every rule is suppressed by a bare `%#ok`.
    all_lines: HashSet<usize>,
    /// Lines that suppress specific rule IDs (exact names).
    rule_lines: HashMap<usize, Vec<String>>,
    /// IDs named by a `%#ok<*ID>` wildcard, which suppress them in the whole file.
    file_wide: HashSet<String>,
    /// Exact IDs named by the directives, in source order.
    named: Vec<NamedId>,
    /// Bare `%#ok` directives, in source order. Their `id` is empty and their
    /// column is the `%`.
    bare: Vec<NamedId>,
    /// For each directive line, the first line of the statement it ends (see
    /// [`Suppressions::covers`]). A line missing from the map starts its own.
    group_start: HashMap<usize, usize>,
}

impl Suppressions {
    /// Whether no suppression directives were found.
    pub fn is_empty(&self) -> bool {
        self.all_lines.is_empty() && self.rule_lines.is_empty() && self.file_wide.is_empty()
    }

    /// The exact check IDs named by the directives (wildcards excluded).
    pub fn named_ids(&self) -> &[NamedId] {
        &self.named
    }

    /// The bare `%#ok` directives (no ID named), in source order of discovery.
    pub fn bare_directives(&self) -> &[NamedId] {
        &self.bare
    }

    /// Whether a diagnostic at `line` with `rule_id` is suppressed by any directive.
    ///
    /// `line` is 1-indexed, matching [`crate::Diagnostic::line`].
    pub fn is_suppressed(&self, line: usize, rule_id: &str) -> bool {
        self.is_file_wide(rule_id)
            || self
                .all_lines
                .iter()
                .chain(self.rule_lines.keys())
                .any(|&directive| self.suppresses_from(directive, line, rule_id))
    }

    /// Whether a `%#ok<*ID>` wildcard suppresses `rule_id` in the whole file.
    ///
    /// MATLAB applies these before the line directives, so a message removed
    /// this way leaves a named directive on its line unused (MSNU).
    pub fn is_file_wide(&self, rule_id: &str) -> bool {
        self.file_wide.contains(rule_id)
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
            .is_some_and(|ids| ids.iter().any(|id| id == rule_id))
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
            let start = comment.start_byte();
            out.bare.push(NamedId {
                id: String::new(),
                line,
                column: column_of(source, start),
                byte_range: start..start + 1,
                comment: comment.byte_range(),
                part: start..start,
                parts: 0,
                index: 0,
                list_end: None,
            });
        }
        return;
    };

    // The IDs run to the closing `>`, or to the end of the comment when unclosed.
    // An ID is taken as written: MATLAB does not trim white space around it.
    let content = &body[..body.find('>').unwrap_or(body.len())];

    // Each ID with its offset in the comment (for the column of the ID).
    let mut names: Vec<(usize, &str)> = Vec::new();
    let mut part_start = 0;
    for part in content.split(',') {
        if !part.is_empty() {
            names.push((part_start, part));
        }
        part_start += part.len() + 1;
    }

    let single = names.len() == 1;
    let closed = body.contains('>');
    let list_end = (closed && content.split(',').all(|p| !p.is_empty()))
        .then(|| comment.start_byte() + "%#ok<".len() + content.len() + 1);
    let parts = content.split(',').count();
    for (name_start, name) in names {
        if let Some(id) = name.strip_prefix('*') {
            out.file_wide.insert(id.to_string());
            continue;
        }
        if name.contains('*') {
            // `NO*`: not an ID, so it suppresses nothing.
            continue;
        }
        out.rule_lines
            .entry(line)
            .or_default()
            .push(name.to_string());
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
            comment: comment.byte_range(),
            part: comment.start_byte() + "%#ok<".len() + name_start
                ..comment.start_byte() + "%#ok<".len() + name_start + name.len(),
            parts,
            index: content[..name_start].matches(',').count(),
            list_end,
        });
    }
}

/// 1-indexed column of byte `byte` within its line.
fn column_of(source: &str, byte: usize) -> usize {
    let line_start = source[..byte].rfind('\n').map_or(0, |p| p + 1);
    source[line_start..byte].chars().count() + 1
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
    fn comma_separated_directive_suppresses_multiple_rules() {
        let s = parse("x = 1; %#ok<NOSEMI,AGROW>\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(s.is_suppressed(1, "AGROW"));
        assert!(!s.is_suppressed(1, "OTHER"));
    }

    // MATLAB probes (R2026b checkcode): `%#ok<*ID>` is exact and file-wide;
    // `%#ok<*>`, `%#ok<*all*>`, `%#ok<*SEMI*>`, `%#ok<*SEMI>` (partial), `%#ok<NO*>`
    // and `%#ok<NOSEMI*>` suppress nothing.

    #[test]
    fn wildcard_is_file_wide_before_and_after_the_directive() {
        // Probe: `%#ok<*NOPTS>` on line 3 silences NOPTS on lines 1 and 2.
        let before = parse("x = 1 %#ok<*NOSEMI>\ny = 2\n");
        assert!(before.is_suppressed(1, "NOSEMI"));
        assert!(before.is_suppressed(2, "NOSEMI"));
        let after = parse("x = 1\ny = 2\nz = 3 %#ok<*NOSEMI>\n");
        assert!(after.is_suppressed(1, "NOSEMI"));
        assert!(after.is_suppressed(2, "NOSEMI"));
        assert!(!after.is_suppressed(1, "AGROW"));
    }

    #[test]
    fn wildcard_ids_are_exact() {
        let s = parse("x = 1 %#ok<*SEMI>\ny = 2 %#ok<*NOSEMIX>\n");
        assert!(!s.is_suppressed(1, "NOSEMI"));
        assert!(!s.is_suppressed(2, "NOSEMI"));
        assert!(!parse("x = 1 %#ok<*NOSEMI*>\n").is_suppressed(1, "NOSEMI"));
        assert!(!parse("x = 1 %#ok<*>\n").is_suppressed(1, "NOSEMI"));
        assert!(!parse("x = 1 %#ok<*all*>\n").is_suppressed(1, "NOSEMI"));
        assert!(!parse("x = 1 %#ok<NO*>\n").is_suppressed(1, "NOSEMI"));
        assert!(!parse("x = 1 %#ok<NOSEMI*>\n").is_suppressed(1, "NOSEMI"));
    }

    #[test]
    fn mixed_list_has_a_file_wide_and_a_line_id() {
        // Probe: `%#ok<AGROW,*NOPTS>` on line 2 silences NOPTS on line 1 too.
        let s = parse("x = 1\ny = 2 %#ok<AGROW,*NOSEMI>\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(!s.is_suppressed(1, "AGROW"));
        assert!(s.is_suppressed(2, "AGROW"));
        let t = parse("x = 1 %#ok<NOSEMI>\ny = 2\n");
        assert!(!t.is_suppressed(2, "NOSEMI"));
    }

    #[test]
    fn names_are_exact_case_sensitive_and_untrimmed() {
        // Probes: `%#ok<nopts>`, `%#ok< NOPTS>` and `%#ok<NOPTS >` do not suppress NOPTS.
        assert!(!parse("x = 1 %#ok<nosemi>\n").is_suppressed(1, "NOSEMI"));
        assert!(!parse("x = 1 %#ok< NOSEMI>\n").is_suppressed(1, "NOSEMI"));
        assert!(!parse("x = 1 %#ok<NOSEMI >\n").is_suppressed(1, "NOSEMI"));
    }

    #[test]
    fn space_before_the_angle_bracket_is_a_bare_directive() {
        // Probe: `%#ok <NOPTS>` suppressed every message on its line.
        let s = parse("x = 1 %#ok <NOSEMI>\ny = 2\n");
        assert!(s.is_suppressed(1, "NOSEMI"));
        assert!(s.is_suppressed(1, "AGROW"));
        assert!(!s.is_suppressed(2, "NOSEMI"));
    }

    #[test]
    fn wildcard_in_a_block_comment_is_not_a_directive() {
        // Probe: `%#ok<*NOPTS>` inside `%{ ... %}` silenced nothing.
        assert!(parse("x = 1\n%{\n%#ok<*NOSEMI>\n%}\n").is_empty());
        assert!(!parse("x = 1\n%{\n%#ok<*NOSEMI>\n%}\n").is_suppressed(1, "NOSEMI"));
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
        assert!(parse("x = 1; %#ok<*NOSEMI>\n").named_ids().is_empty());
        assert!(parse("x = 1; %#ok<*>\n").named_ids().is_empty());
        assert!(parse("x = 1; %#ok<NO*>\n").named_ids().is_empty());
    }
}
