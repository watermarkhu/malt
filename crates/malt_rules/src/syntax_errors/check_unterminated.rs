//! `check_unterminated`: one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_unterminated(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut reported: HashSet<usize> = HashSet::new();

        if self.is_check_enabled("STRIN") || self.is_check_enabled("DOUQT") {
            Self::walk_unterminated_strings(root, source, &mut diagnostics, &mut reported);
        }

        if self.is_check_enabled("INBLK") {
            diagnostics.extend(Self::unterminated_block_comment(source));
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
            let start = node.start_byte();
            let end = node.end_byte().min(source.len());
            let text = &source[start..end];

            // The single quotes that are not transposes, in order.
            let mut cursor = node.walk();
            let quotes: Vec<usize> = node
                .children(&mut cursor)
                .filter(|c| c.kind() == "'" && !Self::is_transpose_position(source, c.start_byte()))
                .map(|c| c.start_byte())
                .collect();
            if quotes.is_empty() {
                // No quote token: the error text itself starts with an unmatched quote.
                if text.starts_with('\'')
                    && text.matches('\'').count() % 2 == 1
                    && !Self::is_transpose_position(source, start)
                {
                    Self::push_strin(start, source, diagnostics, reported);
                }
            } else {
                for opener in Self::unterminated_quotes(&quotes, source) {
                    Self::push_strin(opener, source, diagnostics, reported);
                }
            }

            // DOUQT: a `"` token inside the ERROR node.
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "\"" {
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

        // DOUQT: a double-quoted string cannot cross a line (probed: `q = "a` newline `b";`
        // gives DOUQT at both quotes). The tree holds the closing quote in an ERROR.
        if node.kind() == "string" && source[node.start_byte()..].starts_with('"') {
            let mut cursor = node.walk();
            let closer = node
                .children(&mut cursor)
                .find(|c| c.is_error() && source[c.start_byte()..].starts_with('"'))
                .map(|c| c.start_byte())
                .or_else(|| Self::quote_after_line_break(node, source));
            if let Some(closer) = closer {
                for at in [node.start_byte(), closer] {
                    if reported.insert(at) {
                        let (line, column) = line_col(source, at);
                        diagnostics.push(Diagnostic {
                            rule_id: "DOUQT",
                            message: "A double quoted string is unterminated.".to_string(),
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

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_unterminated_strings(child, source, diagnostics, reported);
        }
    }

    /// The `"` that closes a double-quoted string which a line break splits (`"abc` newline
    /// `def";`): the first `"` after the break, in the string's text. Probed: MATLAB reports
    /// DOUQT at both quotes (1:5 and 2:4).
    fn quote_after_line_break(node: Node, source: &str) -> Option<usize> {
        let text = &source[node.start_byte()..node.end_byte().min(source.len())];
        let after = text.find('\n')? + 1;
        let quote = text[after..].find('"')?;
        Some(node.start_byte() + after + quote)
    }

    /// The openers of the character vectors left unterminated, from the single quotes of one
    /// ERROR node in order. Inside a string `''` is an escaped quote; a quote closes the string,
    /// and a line break ends it unterminated (`'abc' 'def` leaves the second quote open).
    fn unterminated_quotes(quotes: &[usize], source: &str) -> Vec<usize> {
        let mut unterminated = Vec::new();
        let mut open: Option<usize> = None;
        let mut i = 0;
        while i < quotes.len() {
            let q = quotes[i];
            match open {
                Some(o) if source[o..q].contains('\n') => {
                    unterminated.push(o);
                    open = None;
                    continue;
                }
                Some(_) if quotes.get(i + 1) == Some(&(q + 1)) => {
                    i += 2;
                    continue;
                }
                Some(_) => open = None,
                None => open = Some(q),
            }
            i += 1;
        }
        unterminated.extend(open);
        unterminated
    }

    fn push_strin(
        at: usize,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
        reported: &mut HashSet<usize>,
    ) {
        if !reported.insert(at) {
            return;
        }
        let (line, column) = line_col(source, at);
        diagnostics.push(Diagnostic {
            rule_id: "STRIN",
            message: "A quoted character vector is unterminated.".to_string(),
            severity: Severity::Error,
            byte_range: at..at + 1,
            line,
            column,
            fix: None,
        });
    }

    /// INBLK, read as MATLAB reads block comments: a block opens on a line that
    /// is only `%{` (surrounding whitespace allowed) and closes on a line that is
    /// only `%}`. Openers nest. `%{ text`, `x = 1; %{`, `%} text` and `%}x` are
    /// plain comments (probed: the code after `%{ text` runs). A block still open
    /// at the end of the file is reported at the file's last character (probed).
    pub(crate) fn unterminated_block_comment(source: &str) -> Option<Diagnostic> {
        let mut depth: usize = 0;
        for line in source.lines() {
            match line.trim() {
                "%{" => depth += 1,
                "%}" => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
        if depth == 0 {
            return None;
        }
        let text = source.trim_end_matches(['\r', '\n']);
        let last_line = text.rsplit('\n').next().unwrap_or("");
        let line = text.matches('\n').count() + 1;
        let end = text.len();
        Some(Diagnostic {
            rule_id: "INBLK",
            message: "A block comment is unterminated at the end of the file.".to_string(),
            severity: Severity::Error,
            byte_range: end - last_line.len()..end,
            line,
            column: last_line.chars().count().max(1),
            fix: None,
        })
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
    fn douqt_on_multiline_string_at_both_quotes() {
        // Probed: `x = "abc` newline `def";` gives DOUQT at 1:5 and 2:4, and no SYNER.
        let diags = lint_file(&*engine(), "x = \"multi\nline\";\n");
        let douqt: Vec<(usize, usize)> = diags
            .iter()
            .filter(|d| d.rule_id == "DOUQT")
            .map(|d| (d.line, d.column))
            .collect();
        assert_eq!(douqt, vec![(1, 5), (2, 5)], "got: {diags:?}");
        assert!(!has_id(&diags, "SYNER"), "got: {diags:?}");
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

    // Probe c4_B v7, s8, s9, pf1: `%{ text` and `%{text` are plain comments in
    // MATLAB (the code after them runs), so no block is open at the end.
    #[test]
    fn inblk_ok_when_opener_has_text_on_its_line() {
        for src in [
            "%{ unterminated",
            "%{ text\nx = 1;\n",
            "%{text\nx = 1;\n",
            "x = 1; %{\nx\n",
        ] {
            let diags = lint_file(&*engine(), src);
            assert!(!has_id(&diags, "INBLK"), "{src:?} got: {diags:?}");
        }
    }

    // Probe c4_B v5: `%} trailing` does not close, so the block runs to the end
    // and INBLK is at the last character of the file (line 3, column 11).
    #[test]
    fn inblk_at_eof_when_closer_has_text_on_its_line() {
        let diags = lint_file(&*engine(), "%{\nzq5 = 1;\n%} trailing\n");
        let inblk: Vec<_> = diags.iter().filter(|d| d.rule_id == "INBLK").collect();
        assert_eq!(inblk.len(), 1, "got: {diags:?}");
        assert_eq!((inblk[0].line, inblk[0].column), (3, 11));
    }

    // Probe c4_B v16: the nested opener is still open at the end (line 3, column 2).
    #[test]
    fn inblk_fires_for_unterminated_nested_block() {
        let diags = lint_file(&*engine(), "%{\nzq16 = 1;\n%{\n");
        let inblk: Vec<_> = diags.iter().filter(|d| d.rule_id == "INBLK").collect();
        assert_eq!(inblk.len(), 1, "got: {diags:?}");
        assert_eq!((inblk[0].line, inblk[0].column), (3, 2));
    }

    // Probe c4_B v8 and s11: nested blocks that both close are fine.
    #[test]
    fn inblk_ok_on_nested_closed_block_comment() {
        let diags = lint_file(&*engine(), "%{\n%{\nx\n%}\ny\n%}\n");
        assert!(!has_id(&diags, "INBLK"), "got: {diags:?}");
    }

    // Probe c4_B s2, s3, s5, s6: indentation and trailing whitespace are allowed.
    #[test]
    fn inblk_fires_on_indented_unterminated_block_comment() {
        let diags = lint_file(&*engine(), "  %{  \nx = 1;\n");
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
