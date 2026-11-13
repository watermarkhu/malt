//! `check_reserved_and_not` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_reserved_and_not(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut skip = Vec::new();
        Self::collect_skip_ranges(root, &mut skip);
        if self.is_check_enabled("BADNOT") {
            self.badnot_diagnostics(source, &skip, &mut diagnostics);
        }
        self.walk_reserved_and_not(root, source, &mut diagnostics);
        diagnostics
    }

    /// True when `pos` is inside a `[` or `{` literal (a matrix or a cell). An index
    /// (`x(end)`, `c{end}`) is not a literal: MATLAB accepts `end` there.
    pub(crate) fn in_literal_bracket(source: &str, pos: usize) -> bool {
        let line_start = source[..pos].rfind('\n').map_or(0, |i| i + 1);
        match last_unmatched_opener(&source[line_start..pos]).map(|i| line_start + i) {
            Some(at) if source.as_bytes()[at] != b'(' => {
                let before = source[..at]
                    .trim_end_matches([' ', '\t'])
                    .chars()
                    .next_back();
                !before.is_some_and(|c| {
                    c.is_ascii_alphanumeric() || matches!(c, '_' | ')' | ']' | '}' | '\'')
                })
            }
            _ => false,
        }
    }

    /// The byte offsets of the whole-word `end` in `text` (an ERROR node that starts
    /// at byte `start`) that are used as a value inside a literal.
    pub(crate) fn literal_end_offsets(text: &str, start: usize, source: &str) -> Vec<usize> {
        let bytes = source.as_bytes();
        text.match_indices("end")
            .map(|(i, _)| start + i)
            .filter(|&at| {
                let before_ok = at == 0
                    || !(bytes[at - 1].is_ascii_alphanumeric()
                        || matches!(bytes[at - 1], b'_' | b'.'));
                let after_ok = bytes
                    .get(at + 3)
                    .is_none_or(|b| !(b.is_ascii_alphanumeric() || *b == b'_'));
                before_ok && after_ok && Self::in_literal_bracket(source, at)
            })
            .collect()
    }

    fn push_reswd(at: usize, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let (line, column) = line_col(source, at);
        diagnostics.push(Diagnostic {
            rule_id: "RESWD",
            message: "Invalid use of a reserved word.".to_string(),
            severity: Severity::Error,
            byte_range: at..at + 3,
            line,
            column,
            fix: None,
        });
    }

    /// The first `~` of a `[~ ~ ...]` row that holds only `~` and blanks (at least two
    /// `~` with the one at `pos`), or `None`. `pos` is the last `~` of the row.
    fn ignored_row_start(source: &str, pos: usize) -> Option<usize> {
        let line_start = source[..pos].rfind('\n').map_or(0, |p| p + 1);
        let open = source[line_start..pos].rfind('[')? + line_start;
        let inner = &source[open + 1..pos];
        let only_ignores = inner.chars().all(|c| matches!(c, '~' | ' ' | '\t'));
        let first = inner.find('~')? + open + 1;
        (only_ignores && inner.contains('~')).then_some(first)
    }

    /// True when the `~` at `pos` sits in a `[...]` that is followed by `=`: an output list.
    fn bracket_is_output_list(source: &str, pos: usize) -> bool {
        let line_start = source[..pos].rfind('\n').map_or(0, |p| p + 1);
        let Some(open) = last_unmatched_opener(&source[line_start..pos]).map(|p| line_start + p)
        else {
            return false;
        };
        if source.as_bytes()[open] != b'[' {
            return false;
        }
        let mut depth = 0usize;
        let mut close = None;
        for (j, c) in source[open + 1..].char_indices() {
            match c {
                '[' => depth += 1,
                ']' if depth == 0 => {
                    close = Some(open + 1 + j);
                    break;
                }
                ']' => depth -= 1,
                _ => {}
            }
        }
        let Some(close) = close else {
            return false;
        };
        let after = source[close + 1..].trim_start_matches([' ', '\t']);
        after.starts_with('=') && !after.starts_with("==")
    }

    /// BADNOT, read from the text: a `~` followed by a spaced `=` (`x = ~ = 5`), or a `~`
    /// with no operand (`x = ~;`, `f(~, 1)`, `x(~) = 1`, `~` alone). `~=` is an operator,
    /// `[~, idx] = f()` is an output list, and `@(~)` and function headers are valid.
    fn badnot_diagnostics(
        &self,
        source: &str,
        skip: &[Range<usize>],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let bytes = source.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if b != b'~' || bytes.get(i + 1) == Some(&b'=') || Self::in_skip_range(i, skip) {
                continue;
            }
            let line_start = source[..i].rfind('\n').map_or(0, |p| p + 1);
            let prefix = &source[line_start..i];
            if prefix.trim_start().starts_with("function")
                || prefix
                    .rfind("@(")
                    .is_some_and(|p| !prefix[p..].contains(')'))
            {
                continue;
            }
            let rest = &source[i + 1..];
            let gap = rest.len() - rest.trim_start_matches([' ', '\t']).len();
            // A `~` alone on a line after other code is a continuation line of a multi-line
            // output list or call (not checked further here).
            if prefix.trim().is_empty()
                && matches!(rest[gap..].chars().next(), None | Some('\n' | '\r'))
                && source[..line_start].trim_end().contains('\n')
            {
                continue;
            }
            let after = &rest[gap..];
            let next = after.chars().next();
            let spaced_eq = gap > 0 && after.starts_with('=') && !after.starts_with("==");
            let no_operand = matches!(
                next,
                None | Some('\n' | '\r' | ';' | ',' | ')' | ']' | '}' | '%')
            );
            let in_output_list =
                matches!(next, Some(',' | ']')) && Self::bracket_is_output_list(source, i);
            // `[~ ~] = f()`: a row of only ignored outputs, separated by blanks, is BADNOT
            // at its last `~` (probed; `[~]`, `[~,~]` and `[~ a]` are not).
            let ignored_row = if next == Some(']') && Self::bracket_is_output_list(source, i) {
                Self::ignored_row_start(source, i)
            } else {
                None
            };
            if !(spaced_eq || (no_operand && !in_output_list) || ignored_row.is_some()) {
                continue;
            }
            // MATLAB also reports BADNOTLHS at the first `~` of such a row.
            if let Some(first) = ignored_row {
                if self.is_check_enabled("BADNOTLHS") {
                    let (line, column) = line_col(source, first);
                    diagnostics.push(Diagnostic {
                        rule_id: "BADNOTLHS",
                        message: "Invalid use of logical not operator (~) on left side of an assignment. To use ~ to ignore function outputs, separate output variables with commas.".to_string(),
                        severity: Severity::Error,
                        byte_range: first..first + 1,
                        line,
                        column,
                        fix: None,
                    });
                }
            }
            let end = i + 1 + gap + next.map_or(0, char::len_utf8);
            let (line, column) = line_col(source, i);
            diagnostics.push(Diagnostic {
                rule_id: "BADNOT",
                message: "Using ~ to ignore a value is not permitted in this context.".to_string(),
                severity: Severity::Error,
                byte_range: i..end,
                line,
                column,
                fix: None,
            });
        }
    }

    pub(crate) fn walk_reserved_and_not(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let kind = node.kind();

        // RESWD: `end` as a value inside a matrix or cell (`[end]`, `{end}`). Other
        // keywords used as values are SYNER, and `end` outside a literal is IDXCOLND.
        if kind == "end_keyword"
            && self.is_check_enabled("RESWD")
            && Self::in_literal_bracket(source, node.start_byte())
        {
            Self::push_reswd(node.start_byte(), source, diagnostics);
        }

        if node.is_error() {
            let start = node.start_byte();
            let end = node.end_byte().max(start + 1).min(source.len());
            let text = &source[start..end];

            if self.is_check_enabled("RESWD") && !Self::has_descendant_kind(node, "end_keyword") {
                for at in Self::literal_end_offsets(text, start, source) {
                    Self::push_reswd(at, source, diagnostics);
                }
            }

            // MCPLD: ERROR inside a property declaration in a properties block. The
            // validation-order check reports out-of-order tokens and literal indexes.
            let property = Self::ancestor_of_kind(node, "property");
            if self.is_check_enabled("MCPLD")
                && Self::inside_property(node)
                && !property.is_some_and(|p| Self::property_order_reported(p, source))
            {
                // MATLAB names the second `=` of `p = 1 = 2`, and the property otherwise.
                let second_equals = property.and_then(|p| {
                    let text = &source[p.start_byte()..p.end_byte().min(source.len())];
                    text.match_indices('=')
                        .nth(1)
                        .map(|(i, _)| p.start_byte() + i)
                });
                let (at, what) = match second_equals {
                    Some(i) => (i, "'='".to_string()),
                    None => (
                        start,
                        property
                            .and_then(|prop| Self::first_child_text(prop, "identifier", source))
                            .unwrap_or("(unknown)")
                            .to_string(),
                    ),
                };
                let (line, column) = line_col(source, at);
                diagnostics.push(Diagnostic {
                    rule_id: "MCPLD",
                    message: format!("Invalid property syntax at {what}."),
                    severity: Severity::Error,
                    byte_range: at..at + 1,
                    line,
                    column,
                    fix: None,
                });
            }
        }

        // BADNOTLHS: `~` followed by a name in an output list, without a comma (`[~ y]`).
        // MATLAB is silent for `[a ~]`, so only a following name counts.
        if kind == "multioutput_variable" && self.is_check_enabled("BADNOTLHS") {
            let mut cursor = node.walk();
            let children: Vec<Node> = node.children(&mut cursor).collect();
            for (i, child) in children.iter().enumerate() {
                if child.kind() != "ignored_argument" {
                    continue;
                }
                let next_is_id = i + 1 < children.len() && children[i + 1].kind() == "identifier";
                if next_is_id {
                    let pos = child.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "BADNOTLHS",
                        message: "Invalid use of logical not operator (~) on left side of an assignment. To use ~ to ignore function outputs, separate output variables with commas.".to_string(),
                        severity: Severity::Error,
                        byte_range: child.byte_range(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_reserved_and_not(child, source, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        SyntaxErrorsEngine::from_config(&Config::default())
    }

    fn engine_with_disabled(checks: &[&str]) -> Box<dyn Rule> {
        let disabled = checks
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let config = Config::from_toml(&format!(
            "[lint.rules.SYNTAX_ERRORS_ENGINE]\ndisabled_checks = [{disabled}]\n"
        ))
        .unwrap();
        SyntaxErrorsEngine::from_config(&config)
    }

    fn at(diags: &[Diagnostic], id: &str) -> (usize, usize) {
        let d = diags
            .iter()
            .find(|d| d.rule_id == id)
            .unwrap_or_else(|| panic!("no {id} in {diags:?}"));
        (d.line, d.column)
    }

    // -- RESWD: `end` as a value inside a literal ----------------------------

    // Probe b4-b6, t8, t9: MATLAB reports RESWD at `end` in a matrix or cell.
    #[test]
    fn badnot_at_the_last_tilde_of_an_all_ignored_row() {
        // Probed: `[~ ~] = size(x);` gives BADNOTLHS at the first `~` and BADNOT at the
        // second. `[~]`, `[~,~]`, `[a, ~, ~]` and `[~ a]` give no BADNOT.
        let diags = lint_file(&*engine(), "[~ ~] = size(x);\n");
        assert_eq!(at(&diags, "BADNOT"), (1, 4));
        assert_eq!(at(&diags, "BADNOTLHS"), (1, 2));
        assert!(!has_id(
            &lint_file(&*engine(), "[~] = size(x);\n"),
            "BADNOT"
        ));
        assert!(!has_id(
            &lint_file(&*engine(), "[~,~] = size(x);\n"),
            "BADNOT"
        ));
        assert!(!has_id(
            &lint_file(&*engine(), "[a, ~, ~] = size(x);\n"),
            "BADNOT"
        ));
        assert!(!has_id(
            &lint_file(&*engine(), "[~ a] = size(x);\n"),
            "BADNOT"
        ));
    }

    #[test]
    fn reswd_fires_on_end_in_matrix_and_cell() {
        assert_eq!(at(&lint_file(&*engine(), "x = [end];\n"), "RESWD"), (1, 6));
        assert_eq!(at(&lint_file(&*engine(), "y = {end};\n"), "RESWD"), (1, 6));
        assert_eq!(
            at(&lint_file(&*engine(), "y = [end 1];\n"), "RESWD"),
            (1, 6)
        );
        assert_eq!(
            at(&lint_file(&*engine(), "y = [1 end];\n"), "RESWD"),
            (1, 8)
        );
        assert_eq!(
            at(&lint_file(&*engine(), "y = {1, end};\n"), "RESWD"),
            (1, 9)
        );
    }

    // Probe b90-b98: a keyword used as a value is SYNER at the keyword, not RESWD.
    #[test]
    fn keyword_value_is_syner_at_keyword() {
        for (src, col) in [("x = for;\n", 5), ("x = case;\n", 5), ("x = try;\n", 5)] {
            let diags = lint_file(&*engine(), src);
            assert!(!has_id(&diags, "RESWD"), "{src:?} got: {diags:?}");
            assert_eq!(at(&diags, "SYNER"), (1, col), "{src:?} got: {diags:?}");
        }
    }

    #[test]
    fn reswd_ok_on_end_index() {
        for src in ["x = y(end);\n", "x = c{end};\n", "x = [y(end) 1];\n"] {
            let diags = lint_file(&*engine(), src);
            assert!(!has_id(&diags, "RESWD"), "{src:?} got: {diags:?}");
        }
    }

    #[test]
    fn reswd_ok_on_valid_identifiers_and_blocks() {
        let diags = lint_file(
            &*engine(),
            "foo_else = 5;\nfor x = 1:10\nend\nswitch x\ncase 1\ny = 1;\nend\n",
        );
        assert!(!has_id(&diags, "RESWD"), "got: {diags:?}");
    }

    #[test]
    fn reswd_not_reported_on_multi_line_error() {
        // A function body tree-sitter could not parse is not a misused keyword.
        let diags = lint_file(
            &*engine(),
            "function t1(x, y)\nwhile true\n    x = x + 1;\nend\nelseif x, a = 2; end\n",
        );
        assert!(!has_id(&diags, "RESWD"), "got: {diags:?}");
    }

    // -- SYNEND: invalid use of END -----------------------------------------

    // MATLAB never reports SYNEND in the probes: `y = end;` is IDXCOLND or SYNER.

    #[test]
    fn synend_not_reported_on_end_assignment_lhs() {
        let diags = lint_file(&*engine(), "end = 5;\n");
        assert!(!has_id(&diags, "SYNEND"), "got: {diags:?}");
    }

    #[test]
    fn synend_not_reported_on_end_as_value() {
        let diags = lint_file(&*engine(), "y = end;\n");
        assert!(!has_id(&diags, "SYNEND"), "got: {diags:?}");
        assert!(!has_id(&diags, "RESWD"), "got: {diags:?}");
    }

    #[test]
    fn synend_ok_on_end_indexing() {
        let diags = lint_file(&*engine(), "x(end) = 5;\n");
        assert!(!has_id(&diags, "SYNEND"), "got: {diags:?}");
    }

    #[test]
    fn synend_ok_on_block_end() {
        let diags = lint_file(&*engine(), "for x = 1:10\nend\n");
        assert!(!has_id(&diags, "SYNEND"), "got: {diags:?}");
    }

    // -- MCPLD: invalid property syntax -------------------------------------

    // Probe b152: MATLAB names the second `=` (column 9), not the property.
    #[test]
    fn mcpld_fires_on_double_equals_property_at_second_equals() {
        let diags = lint_file(
            &*engine(),
            "classdef Foo\nproperties\n  x = 1 = 2\nend\nend\n",
        );
        assert_eq!(at(&diags, "MCPLD"), (3, 9), "got: {diags:?}");
    }

    #[test]
    fn mcpld_ok_on_valid_properties() {
        let diags = lint_file(&*engine(), "classdef Foo\nproperties\nx = 1;\nend\nend\n");
        assert!(!has_id(&diags, "MCPLD"), "got: {diags:?}");
    }

    #[test]
    fn mcpld_ok_outside_properties() {
        let diags = lint_file(&*engine(), "x = 1 = 2;\n");
        assert!(!has_id(&diags, "MCPLD"), "got: {diags:?}");
    }

    #[test]
    fn mcpld_silent_in_arguments_block() {
        // Probed: MATLAB reports no MCPLD for entries of an `arguments` block.
        let src = "function f(x)\narguments\n    x (1,:) double\nend\nend\n";
        assert!(!has_id(&lint_file(&*engine(), src), "MCPLD"));
    }

    // -- BADNOT: ~ misuse ----------------------------------------------------

    // Probe b102-b111, t43-t46: a `~` with no operand, or a spaced `~ =`.
    #[test]
    fn badnot_fires_on_operandless_tilde() {
        assert_eq!(at(&lint_file(&*engine(), "x = ~ = 5;\n"), "BADNOT"), (1, 5));
        assert_eq!(at(&lint_file(&*engine(), "~\n"), "BADNOT"), (1, 1));
        assert_eq!(at(&lint_file(&*engine(), "~ = 5;\n"), "BADNOT"), (1, 1));
        assert_eq!(at(&lint_file(&*engine(), "x = ~;\n"), "BADNOT"), (1, 5));
        assert_eq!(at(&lint_file(&*engine(), "y = ~ ;\n"), "BADNOT"), (1, 5));
        assert_eq!(at(&lint_file(&*engine(), "f(~);\n"), "BADNOT"), (1, 3));
        assert_eq!(at(&lint_file(&*engine(), "f(~, 1);\n"), "BADNOT"), (1, 3));
        assert_eq!(at(&lint_file(&*engine(), "x(~) = 1;\n"), "BADNOT"), (1, 3));
        assert_eq!(
            at(&lint_file(&*engine(), "x = [~, 1];\n"), "BADNOT"),
            (1, 6)
        );
    }

    // Probe b114: `~=` without a space is the operator, so no BADNOT (MATLAB: SYNER).
    #[test]
    fn badnot_ok_on_tilde_equals_operator() {
        let diags = lint_file(&*engine(), "x = ~= 5;\n");
        assert!(!has_id(&diags, "BADNOT"), "got: {diags:?}");
    }

    #[test]
    fn badnot_ok_on_logical_not_and_output_lists() {
        for src in [
            "x = ~y;\n",
            "if ~x\n  y = 1;\nend\n",
            "[~, y] = f();\n",
            "[a, ~] = f();\n",
            "[a,~,~] = size(x);\n",
            "[~] = f();\n",
            "g = @(~) 1;\n",
            "function f(~)\nend\n",
        ] {
            let diags = lint_file(&*engine(), src);
            assert!(!has_id(&diags, "BADNOT"), "{src:?} got: {diags:?}");
        }
    }

    // -- BADNOTLHS: ~ adjacent to output without comma ----------------------

    #[test]
    fn badnotlhs_fires_on_adjacent_tilde() {
        let diags = lint_file(&*engine(), "[x ~ y] = f();\n");
        assert!(has_id(&diags, "BADNOTLHS"), "got: {diags:?}");
    }

    // Probe b107: `[a ~]` is silent in MATLAB.
    #[test]
    fn badnotlhs_ok_on_trailing_tilde() {
        let diags = lint_file(&*engine(), "[a ~] = f();\n");
        assert!(!has_id(&diags, "BADNOTLHS"), "got: {diags:?}");
    }

    #[test]
    fn badnotlhs_ok_on_comma_separated() {
        let diags = lint_file(&*engine(), "[~, y] = f();\n");
        assert!(!has_id(&diags, "BADNOTLHS"), "got: {diags:?}");
    }

    #[test]
    fn badnotlhs_ok_on_solo_tilde() {
        let diags = lint_file(&*engine(), "[~] = f();\n");
        assert!(!has_id(&diags, "BADNOTLHS"), "got: {diags:?}");
    }

    #[test]
    fn disabled_checks_turn_off_reserved_and_not_checks() {
        let engine = engine_with_disabled(&["RESWD", "SYNEND", "MCPLD", "BADNOT", "BADNOTLHS"]);
        let diags = lint_file(
            &*engine,
            "x = [end];\nend = 5;\nclassdef Foo\nproperties\nx = 1 = 2\nend\nend\nx = ~ = 5;\n[x ~ y] = f();\n",
        );
        assert!(!has_id(&diags, "RESWD"), "got: {diags:?}");
        assert!(!has_id(&diags, "SYNEND"), "got: {diags:?}");
        assert!(!has_id(&diags, "MCPLD"), "got: {diags:?}");
        assert!(!has_id(&diags, "BADNOT"), "got: {diags:?}");
        assert!(!has_id(&diags, "BADNOTLHS"), "got: {diags:?}");
    }
}
