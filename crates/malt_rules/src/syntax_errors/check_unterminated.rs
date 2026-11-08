//! `check_unterminated` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_unterminated(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut reported: HashSet<usize> = HashSet::new();

        if self.is_check_enabled("STRIN") || self.is_check_enabled("DOUQT") {
            Self::walk_unterminated_strings(root, source, &mut diagnostics, &mut reported);
        }

        if self.is_check_enabled("INBLK") {
            Self::walk_unterminated_comments(root, source, &mut diagnostics);
        }

        diagnostics
    }

    pub(crate) fn walk_unterminated_strings(
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
        reported: &mut HashSet<usize>,
    ) {
        if node.is_error() {
            // STRIN: ERROR node whose text starts with an unmatched quote.
            let start = node.start_byte();
            let end = node.end_byte().min(source.len());
            let text = &source[start..end];
            if text.starts_with('\'')
                && text.matches('\'').count() % 2 == 1
                && !reported.contains(&start)
                && !Self::is_transpose_position(source, start)
            {
                let pos = node.start_position();
                reported.insert(start);
                diagnostics.push(Diagnostic {
                    rule_id: "STRIN",
                    message: "A quoted character vector is unterminated.".to_string(),
                    severity: Severity::Error,
                    byte_range: start..start + 1,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }

            // STRIN/DOUQT: direct quote tokens inside the ERROR node.
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "'" {
                    let pos = child.start_byte();
                    if !reported.contains(&pos) && !Self::is_transpose_position(source, pos) {
                        let cpos = child.start_position();
                        reported.insert(pos);
                        diagnostics.push(Diagnostic {
                            rule_id: "STRIN",
                            message: "A quoted character vector is unterminated.".to_string(),
                            severity: Severity::Error,
                            byte_range: pos..pos + 1,
                            line: cpos.row + 1,
                            column: cpos.column + 1,
                            fix: None,
                        });
                    }
                } else if child.kind() == "\"" {
                    let pos = child.start_byte();
                    if !reported.contains(&pos) {
                        let cpos = child.start_position();
                        reported.insert(pos);
                        diagnostics.push(Diagnostic {
                            rule_id: "DOUQT",
                            message: "A double quoted string is unterminated.".to_string(),
                            severity: Severity::Error,
                            byte_range: pos..pos + 1,
                            line: cpos.row + 1,
                            column: cpos.column + 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_unterminated_strings(child, source, diagnostics, reported);
        }
    }

    pub(crate) fn walk_unterminated_comments(
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "comment" {
            let text = &source[node.start_byte()..node.end_byte()];
            if text.starts_with("%{") && !text.contains("%}") {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "INBLK",
                    message: "A block comment is unterminated at the end of the file.".to_string(),
                    severity: Severity::Error,
                    byte_range: node.byte_range(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_unterminated_comments(child, source, diagnostics);
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

    // -- STRIN: unterminated single-quoted character vector -----------------

    #[test]
    fn strin_fires_on_unterminated_char_vector() {
        let diags = lint_file(&*engine(), "x = 'abc;\n");
        assert!(has_id(&diags, "STRIN"), "got: {diags:?}");
    }

    #[test]
    fn strin_fires_on_unterminated_at_eof() {
        let diags = lint_file(&*engine(), "x = 'abc");
        assert!(has_id(&diags, "STRIN"), "got: {diags:?}");
    }

    #[test]
    fn strin_fires_on_unterminated_with_escaped_quote() {
        let diags = lint_file(&*engine(), "x = 'don''t stop\n");
        assert!(has_id(&diags, "STRIN"), "got: {diags:?}");
    }

    #[test]
    fn strin_ok_on_closed_char_vector() {
        let diags = lint_file(&*engine(), "x = 'abc';\n");
        assert!(!has_id(&diags, "STRIN"), "got: {diags:?}");
    }

    #[test]
    fn strin_ok_on_escaped_quote() {
        let diags = lint_file(&*engine(), "x = 'it''s ok';\n");
        assert!(!has_id(&diags, "STRIN"), "got: {diags:?}");
    }

    #[test]
    fn strin_ok_on_transpose() {
        let diags = lint_file(&*engine(), "x = A';\n");
        assert!(!has_id(&diags, "STRIN"), "got: {diags:?}");
    }

    #[test]
    fn strin_ok_on_elementwise_transpose() {
        let diags = lint_file(&*engine(), "x = A.';\n");
        assert!(!has_id(&diags, "STRIN"), "got: {diags:?}");
    }

    #[test]
    fn strin_ok_on_transpose_of_expression() {
        let diags = lint_file(&*engine(), "x = (1:5)';\n");
        assert!(!has_id(&diags, "STRIN"), "got: {diags:?}");
    }

    #[test]
    fn strin_ok_on_matrix_transpose() {
        let diags = lint_file(&*engine(), "x = [1 2; 3 4]';\n");
        assert!(!has_id(&diags, "STRIN"), "got: {diags:?}");
    }

    #[test]
    fn strin_ok_on_transpose_without_semicolon() {
        let diags = lint_file(&*engine(), "x = A'\n");
        assert!(!has_id(&diags, "STRIN"), "got: {diags:?}");
    }

    #[test]
    fn strin_ok_on_adjacent_strings() {
        let diags = lint_file(&*engine(), "x = 'abc' 'def';\n");
        assert!(!has_id(&diags, "STRIN"), "got: {diags:?}");
    }

    // -- DOUQT: unterminated double-quoted string ---------------------------

    #[test]
    fn douqt_fires_on_unterminated_string() {
        let diags = lint_file(&*engine(), "x = \"abc;\n");
        assert!(has_id(&diags, "DOUQT"), "got: {diags:?}");
    }

    #[test]
    fn douqt_fires_on_unterminated_at_eof() {
        let diags = lint_file(&*engine(), "x = \"abc");
        assert!(has_id(&diags, "DOUQT"), "got: {diags:?}");
    }

    #[test]
    fn douqt_fires_on_empty_unterminated() {
        let diags = lint_file(&*engine(), "s = \"\n");
        assert!(has_id(&diags, "DOUQT"), "got: {diags:?}");
    }

    #[test]
    fn douqt_ok_on_closed_string() {
        let diags = lint_file(&*engine(), "x = \"abc\";\n");
        assert!(!has_id(&diags, "DOUQT"), "got: {diags:?}");
    }

    #[test]
    fn douqt_ok_on_multiline_string() {
        let diags = lint_file(&*engine(), "x = \"multi\nline\";\n");
        assert!(!has_id(&diags, "DOUQT"), "got: {diags:?}");
    }

    #[test]
    fn douqt_ok_on_concatenated_strings() {
        let diags = lint_file(&*engine(), "x = \"a\" + \"b\";\n");
        assert!(!has_id(&diags, "DOUQT"), "got: {diags:?}");
    }

    // -- INBLK: unterminated block comment ----------------------------------

    #[test]
    fn inblk_fires_on_unterminated_block_comment() {
        let diags = lint_file(&*engine(), "%{\nunterminated\n");
        assert!(has_id(&diags, "INBLK"), "got: {diags:?}");
    }

    #[test]
    fn inblk_fires_on_unterminated_block_comment_at_eof() {
        let diags = lint_file(&*engine(), "%{ unterminated");
        assert!(has_id(&diags, "INBLK"), "got: {diags:?}");
    }

    #[test]
    fn inblk_ok_on_closed_block_comment() {
        let diags = lint_file(&*engine(), "%{\nclosed\n%}\n");
        assert!(!has_id(&diags, "INBLK"), "got: {diags:?}");
    }

    #[test]
    fn inblk_ok_on_single_line_block_comment() {
        let diags = lint_file(&*engine(), "%{ block %}\n");
        assert!(!has_id(&diags, "INBLK"), "got: {diags:?}");
    }

    #[test]
    fn inblk_ok_on_regular_comment() {
        let diags = lint_file(&*engine(), "x = 1; % regular comment\n");
        assert!(!has_id(&diags, "INBLK"), "got: {diags:?}");
    }

    // -- STRIN/DOUQT/INBLK disabled via config ------------------------------

    #[test]
    fn disabled_checks_turn_off_unterminated_checks() {
        let engine = engine_with_disabled(&["STRIN", "DOUQT", "INBLK"]);
        let diags = lint_file(&*engine, "x = 'abc;\nx = \"abc;\n%{\nunterminated\n");
        assert!(!has_id(&diags, "STRIN"), "got: {diags:?}");
        assert!(!has_id(&diags, "DOUQT"), "got: {diags:?}");
        assert!(!has_id(&diags, "INBLK"), "got: {diags:?}");
    }
}
