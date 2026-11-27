//! `check_assignment_lhs`: one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_assignment_lhs(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        self.walk_assignment_lhs(root, source, &mut diagnostics);
        diagnostics
    }

    /// LHROW for a statement that starts with a bracket holding a row separator and is then
    /// assigned to (`[a;b;c] = size(x)`). The grammar makes it an error node, so the line is
    /// found in the text. Returns the byte span of each such line with its diagnostic: the
    /// caller drops the parse errors on that span. MATLAB reports the first token after the
    /// `;` (`[a;b] = f()` is column 4).
    pub(crate) fn check_row_separated_lhs(&self, source: &str) -> Vec<(Range<usize>, Diagnostic)> {
        let mut found = Vec::new();
        if !self.is_check_enabled("LHROW") {
            return found;
        }
        let mut line_start = 0;
        for line in source.split_inclusive('\n') {
            let at = line_start;
            line_start += line.len();
            let code = line.trim_end_matches(['\n', '\r']);
            let indent = code.len() - code.trim_start().len();
            let code = code.trim_start();
            if !code.starts_with('[') {
                continue;
            }
            // The `;` directly inside the opening bracket, and the bracket's close on this line.
            let mut depth = 0;
            let mut semi = None;
            let mut close = None;
            for (i, b) in code.bytes().enumerate() {
                match b {
                    b'[' | b'(' | b'{' => depth += 1,
                    b']' | b')' | b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            close = Some(i);
                            break;
                        }
                    }
                    b';' if depth == 1 && semi.is_none() => semi = Some(i),
                    // Strings and comments are not scanned: such a line is left alone.
                    b'\'' | b'"' | b'%' => break,
                    _ => {}
                }
            }
            let (Some(semi), Some(close)) = (semi, close) else {
                continue;
            };
            let after = code[close + 1..].trim_start();
            if !after.starts_with('=') || after.starts_with("==") {
                continue;
            }
            let rest = &code[semi + 1..];
            let token = semi + 1 + (rest.len() - rest.trim_start_matches([' ', '\t']).len());
            let token_at = at + indent + token;
            let (line_no, column) = line_col(source, token_at);
            let diagnostic = Diagnostic {
                rule_id: "LHROW",
                message: "The left side of an assignment cannot have multiple rows (';')."
                    .to_string(),
                severity: Severity::Error,
                byte_range: token_at..token_at + 1,
                line: line_no,
                column,
                fix: None,
            };
            found.push((at..at + line.len(), diagnostic));
        }
        found
    }

    pub(crate) fn walk_assignment_lhs(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        const OPERATOR_LHS_KINDS: [&str; 5] = [
            "comparison_operator",
            "binary_operator",
            "boolean_operator",
            "unary_operator",
            "postfix_operator",
        ];

        let kind = node.kind();

        if kind == "assignment" {
            let lhs = node.child_by_field_name("left");
            if let Some(lhs) = lhs {
                let pos = lhs.start_position();

                // UNSET: operator expression on the left side of `=`.
                if self.is_check_enabled("UNSET") && OPERATOR_LHS_KINDS.contains(&lhs.kind()) {
                    diagnostics.push(Diagnostic {
                        rule_id: "UNSET",
                        message: format!(
                            "Invalid use of {} on the left side of an assignment.",
                            operator_text(lhs, source)
                        ),
                        severity: Severity::Error,
                        byte_range: lhs.byte_range(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }

                // LHROW: multi-output LHS containing a row separator (`;`).
                if self.is_check_enabled("LHROW") && lhs.kind() == "multioutput_variable" {
                    let lhs_text = &source[lhs.start_byte()..lhs.end_byte()];
                    if let Some(semi) = lhs_text.find(';') {
                        // MATLAB reports the first token of the next row (`[a;b] = f()` is column 4).
                        let after = lhs.start_byte() + semi + 1;
                        let at = after
                            + (source[after..].len()
                                - source[after..].trim_start_matches([' ', '\t']).len());
                        let (line, column) = line_col(source, at);
                        diagnostics.push(Diagnostic {
                            rule_id: "LHROW",
                            message:
                                "The left side of an assignment cannot have multiple rows (';')."
                                    .to_string(),
                            severity: Severity::Error,
                            byte_range: at..at + 1,
                            line,
                            column,
                            fix: None,
                        });
                    }
                }
            }
        }

        // UNSET fallback: the parser may emit an ERROR node (text starting
        // with `=`) as the sibling of a statement-level operator expression
        // instead of an assignment node (e.g., `x == 5 = 3;`).
        if self.is_check_enabled("UNSET") && node.is_error() {
            if let Some(prev) = node.prev_sibling() {
                if OPERATOR_LHS_KINDS.contains(&prev.kind()) {
                    let err_text = &source[node.start_byte()..node.end_byte()];
                    if err_text.trim_start().starts_with('=') {
                        let pos = prev.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "UNSET",
                            message: format!(
                                "Invalid use of {} on the left side of an assignment.",
                                operator_text(prev, source)
                            ),
                            severity: Severity::Error,
                            byte_range: prev.byte_range(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_assignment_lhs(child, source, diagnostics);
        }
    }
}

/// The operator of an operator expression (`+` in `a + b`), or the node's
/// own text when it has none.
fn operator_text(node: tree_sitter::Node, source: &str) -> String {
    let mut cursor = node.walk();
    let op = node
        .children(&mut cursor)
        .find(|c| !c.is_named())
        .map(|c| source[c.byte_range()].to_string());
    op.unwrap_or_else(|| source[node.byte_range()].chars().take(40).collect())
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

    // -- UNSET: operator on the left side of an assignment ------------------

    #[test]
    fn unset_fires_on_comparison_lhs() {
        let diags = lint_file(&*engine(), "x == 5 = 3;\n");
        assert!(has_id(&diags, "UNSET"), "got: {diags:?}");
    }

    #[test]
    fn unset_fires_on_binary_lhs() {
        let diags = lint_file(&*engine(), "x + 1 = 2;\n");
        assert!(has_id(&diags, "UNSET"), "got: {diags:?}");
    }

    #[test]
    fn unset_fires_on_unary_lhs() {
        let diags = lint_file(&*engine(), "-x = 3;\n");
        assert!(has_id(&diags, "UNSET"), "got: {diags:?}");
    }

    #[test]
    fn unset_ok_on_valid_assignment() {
        let diags = lint_file(&*engine(), "x = 5;\n");
        assert!(!has_id(&diags, "UNSET"), "got: {diags:?}");
    }

    #[test]
    fn unset_ok_on_operator_rhs() {
        let diags = lint_file(&*engine(), "x = x + 1;\n");
        assert!(!has_id(&diags, "UNSET"), "got: {diags:?}");
    }

    #[test]
    fn unset_ok_on_indexed_lhs() {
        let diags = lint_file(&*engine(), "x(1) = 5;\n");
        assert!(!has_id(&diags, "UNSET"), "got: {diags:?}");
    }

    #[test]
    fn unset_ok_on_field_lhs() {
        let diags = lint_file(&*engine(), "obj.field = 5;\n");
        assert!(!has_id(&diags, "UNSET"), "got: {diags:?}");
    }

    // -- LHROW: assignment LHS with multiple rows ---------------------------

    #[test]
    fn lhrow_fires_on_semicolon_rows() {
        let diags = lint_file(&*engine(), "[a;b] = f();\n");
        assert!(has_id(&diags, "LHROW"), "got: {diags:?}");
    }

    #[test]
    fn lhrow_fires_on_semicolon_rows_with_spaces() {
        let diags = lint_file(&*engine(), "[a; b] = f();\n");
        assert!(has_id(&diags, "LHROW"), "got: {diags:?}");
    }

    #[test]
    fn lhrow_ok_on_comma_list() {
        let diags = lint_file(&*engine(), "[a, b] = f();\n");
        assert!(!has_id(&diags, "LHROW"), "got: {diags:?}");
    }

    #[test]
    fn lhrow_ok_on_single_output() {
        let diags = lint_file(&*engine(), "x = f();\n");
        assert!(!has_id(&diags, "LHROW"), "got: {diags:?}");
    }

    // -- UNSET/LHROW disabled via config ------------------------------------

    #[test]
    fn disabled_checks_turn_off_unset_and_lhrow() {
        let engine = engine_with_disabled(&["UNSET", "LHROW"]);
        let diags = lint_file(&*engine, "x == 5 = 3;\n[a;b] = f();\n");
        assert!(!has_id(&diags, "UNSET"), "got: {diags:?}");
        assert!(!has_id(&diags, "LHROW"), "got: {diags:?}");
    }
}
