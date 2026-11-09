//! `check_error_nodes` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_error_nodes(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let last = Self::last_descendant(root);
        self.walk_error_nodes(root, source, last, &mut diagnostics);

        // ENDPAR: Check if the file ends with a missing closing bracket.
        if self.is_check_enabled("ENDPAR") {
            if let Some(last) = last {
                let start = last.start_byte();
                let end = last.end_byte().max(start + 1);
                let text = &source[start..end.min(source.len())];
                let is_bracket_error = last.is_error() && Self::looks_like_missing_bracket(text);
                let missing_kind = Self::missing_bracket_opener(last.kind());
                if is_bracket_error || (last.is_missing() && missing_kind.is_some()) {
                    let pos = last.start_position();
                    let bracket = Self::bracket_kinds(text)
                        .first()
                        .copied()
                        .or(missing_kind)
                        .unwrap_or("bracket");
                    diagnostics.push(Diagnostic {
                            rule_id: "ENDPAR",
                            message: format!(
                                "A {bracket} might be missing a closing {bracket}, causing invalid syntax at end of file."
                            ),
                            severity: Severity::Error,
                            byte_range: start..end,
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                }
            }
        }

        diagnostics
    }

    pub(crate) fn walk_error_nodes(
        &self,
        node: Node,
        source: &str,
        last: Option<Node>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.is_error() {
            let start = node.start_byte();
            let end = node.end_byte().max(start + 1);
            let pos = node.start_position();
            let text = &source[start..end.min(source.len())];

            // Classify the error node.
            let endct_variant = if self.is_check_enabled("ENDCT") {
                Self::classify_missing_end(&node, source)
            } else {
                None
            };

            // NOPAR2: Looks like a missing closing bracket.
            if Self::looks_like_missing_bracket(text) {
                let is_last = last.is_some_and(|l| l.id() == node.id());
                if self.is_check_enabled("NOPAR2") && !is_last {
                    let bracket = Self::bracket_kinds(text)
                        .first()
                        .copied()
                        .unwrap_or("bracket");
                    diagnostics.push(Diagnostic {
                            rule_id: "NOPAR2",
                            message: format!(
                                "A {bracket} might be missing a closing {bracket}, causing invalid syntax at {bracket} on line {}.",
                                pos.row + 1
                            ),
                            severity: Severity::Error,
                            byte_range: start..end,
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                }
            }
            // ENDCT: Looks like a missing END.
            else if let Some((variant_id, variant_msg)) = endct_variant {
                if self.is_check_enabled(variant_id) {
                    diagnostics.push(Diagnostic {
                        rule_id: variant_id,
                        message: variant_msg,
                        severity: Severity::Error,
                        byte_range: start..end,
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                } else {
                    diagnostics.push(Diagnostic {
                        rule_id: "ENDCT",
                        message: endct_message(text),
                        severity: Severity::Error,
                        byte_range: start..end,
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            } else if self.is_check_enabled("ENDCT") && Self::looks_like_missing_end(text, &node) {
                diagnostics.push(Diagnostic {
                    rule_id: "ENDCT",
                    message: endct_message(text),
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
            // FVSYN: Invalid function argument syntax. Fires for ERROR nodes
            // that appear inside a function call's argument list. Replaces
            // SYNER for these nodes (no double-fire).
            else if self.is_check_enabled("FVSYN")
                && Self::is_in_function_call_args(&node)
                && !text.contains('=')
            {
                diagnostics.push(Diagnostic {
                    rule_id: "FVSYN",
                    message: format!(
                        "Invalid function argument syntax at {}.",
                        text.chars().take(40).collect::<String>().trim()
                    ),
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
            // SYNER: Generic syntax error.
            else if self.is_check_enabled("SYNER")
                && !(Self::is_in_function_call_args(&node) && text.contains('='))
                && !Self::is_known_valid_error(&node, source)
            {
                let snippet: String = text.chars().take(40).collect();
                let msg = if snippet.is_empty() {
                    "Parse error: usage might be invalid MATLAB syntax.".to_string()
                } else {
                    format!("Parse error at {snippet}: usage might be invalid MATLAB syntax.")
                };
                diagnostics.push(Diagnostic {
                    rule_id: "SYNER",
                    message: msg,
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        } else if node.is_missing() {
            if let Some(bracket) = Self::missing_bracket_opener(node.kind()) {
                let is_last = last.is_some_and(|l| l.id() == node.id());
                if !is_last {
                    let start = node.start_byte();
                    let end = node.end_byte().max(start + 1);
                    let pos = node.start_position();
                    if Self::is_end_of_file(node, source) {
                        if self.is_check_enabled("ENDPAR") {
                            diagnostics.push(Diagnostic {
                                    rule_id: "ENDPAR",
                                    message: format!(
                                        "A '{bracket}' might be missing a closing '{closer}', causing invalid syntax at end of file.",
                                        closer = node.kind()
                                    ),
                                    severity: Severity::Error,
                                    byte_range: start..end,
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                        }
                    } else if Self::is_line_ending(node, source) {
                        if self.is_check_enabled("EOLPAR") {
                            diagnostics.push(Diagnostic {
                                    rule_id: "EOLPAR",
                                    message: format!(
                                        "A '{bracket}' might be missing a closing '{closer}', causing invalid syntax at end of line.",
                                        closer = node.kind()
                                    ),
                                    severity: Severity::Error,
                                    byte_range: start..end,
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                        }
                    } else if let Some((open, token)) = self.nopar_opener(node, source, bracket) {
                        diagnostics.push(Diagnostic {
                            rule_id: "NOPAR",
                            message: format!(
                                "A '{bracket}' might be missing a closing '{closer}', causing invalid syntax at '{token}'.",
                                closer = node.kind()
                            ),
                            severity: Severity::Error,
                            byte_range: open..open + 1,
                            line: pos.row + 1,
                            column: open + 1 - source[..open].rfind('\n').map_or(0, |i| i + 1),
                            fix: None,
                        });
                    } else if self.is_check_enabled("NOPAR2") {
                        diagnostics.push(Diagnostic {
                                rule_id: "NOPAR2",
                                message: format!(
                                    "A {bracket} might be missing a closing {bracket}, causing invalid syntax at {bracket} on line {}.",
                                    pos.row + 1
                                ),
                                severity: Severity::Error,
                                byte_range: start..end,
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                    }
                }
            } else if self.is_check_enabled("SYNER")
                // MATLAB does not report a missing `,` or `;` as a syntax error
                // (probe: `x = 1; 2` with no final newline gets no SYNER).
                && !matches!(node.kind(), "," | ";")
            {
                let start = node.start_byte();
                let end = node.end_byte().max(start + 1);
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "SYNER",
                    message: format!(
                        "Parse error at {}: usage might be invalid MATLAB syntax.",
                        node.kind()
                    ),
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_error_nodes(child, source, last, diagnostics);
        }
    }
}

impl SyntaxErrorsEngine {
    /// True for an ERROR node that MATLAB accepts. Tree-sitter does not parse
    /// these, so SYNER would be a false positive. Probed against R2026b:
    /// - validation syntax inside an `arguments` block (`x (1,2) double`,
    ///   `x {@(v) v > 0}`, `arguments (Input = true)`, `opts.?cls = 1`);
    /// - colon chains such as `1:2:3:4` (MATLAB reports M3COL instead);
    /// - `spmd (1, 2, 3)` (MATLAB reports SPDEC3 instead).
    ///
    /// NOPAR: an unclosed `(` or `{` whose closer is missing on the opener's
    /// line, so the offending token is on that line too. Returns the opener's
    /// byte offset and the offending token. Probed against R2026b: a later line
    /// gives NOPAR2, the end of the line EOLPAR, and `[` is not covered.
    fn nopar_opener(&self, node: Node, source: &str, bracket: &str) -> Option<(usize, String)> {
        if !self.is_check_enabled("NOPAR") || !matches!(bracket, "(" | "{") {
            return None;
        }
        let at = node.start_byte();
        let line_start = source[..at].rfind('\n').map_or(0, |i| i + 1);
        let open = line_start + source[line_start..at].find(bracket)?;
        let rest = &source[at..];
        let token: String = if rest.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
            rest.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect()
        } else {
            rest.chars().take(1).collect()
        };
        if token.is_empty() || token.starts_with(char::is_whitespace) {
            return None;
        }
        Some((open, token))
    }

    fn is_known_valid_error(node: &Node, source: &str) -> bool {
        let mut current = node.parent();
        while let Some(ancestor) = current {
            if ancestor.kind() == "arguments_statement" {
                return true;
            }
            current = ancestor.parent();
        }
        if Self::is_colon_chain(node, source) {
            return true;
        }
        let line_start = source[..node.start_byte()].rfind('\n').map_or(0, |i| i + 1);
        let line = source[line_start..]
            .lines()
            .next()
            .unwrap_or("")
            .trim_start();
        line.strip_prefix("spmd")
            .is_some_and(|rest| rest.starts_with([' ', '\t', '(']))
    }

    /// True when the error token sits in a colon chain of plain operands, such
    /// as `1:2:3:4` (the error token is `:4`), or `:3:4:5` at the start of a range.
    fn is_colon_chain(node: &Node, source: &str) -> bool {
        let is_operand_char = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | ':');
        let start = source[..node.start_byte()]
            .char_indices()
            .rev()
            .take_while(|(_, c)| is_operand_char(*c))
            .last()
            .map_or(node.start_byte(), |(i, _)| i);
        let end = source[node.start_byte()..]
            .find(|c: char| !is_operand_char(c))
            .map_or(source.len(), |i| node.start_byte() + i);
        let run = &source[start..end];
        run.matches(':').count() >= 2 && !run.contains("::") && !run.ends_with(':')
    }
}

/// The message for a missing END, naming the block keyword it probably closes.
fn endct_message(text: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "function", "if", "for", "parfor", "while", "switch", "try", "spmd",
    ];
    let keyword = text
        .split(|c: char| !c.is_alphanumeric())
        .find(|w| KEYWORDS.contains(w));
    match keyword {
        Some(k) => format!("An END might be missing, possibly matching {k}."),
        None => "An END might be missing.".to_string(),
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

    // -- SYNER: generic syntax error ----------------------------------------

    #[test]
    fn syner_fires_on_parse_error() {
        let diags = lint_file(&*engine(), "x = ;\n");
        assert!(has_id(&diags, "SYNER"), "got: {diags:?}");
    }

    #[test]
    fn syner_ok_on_valid_source() {
        let diags = lint_file(&*engine(), "x = 5;\ny = x + 1;\n");
        assert!(!has_id(&diags, "SYNER"), "got: {diags:?}");
    }

    #[test]
    fn syner_ok_on_forms_that_matlab_accepts() {
        // Probed against MATLAB R2026b: none of these gets SYNER. The colon
        // chain is M3COL, `spmd (...)` is SPDEC3, and a missing final newline
        // after `2` is not reported.
        for source in [
            "a = 1:2:3:4:5;\n",
            "b = a(1:2:3:4);\n",
            "spmd (1, 2, 3)\nend\n",
            "x = 1; 2",
            "function r = f(x)\narguments\n    x (-1,2)\n    y {@(v) v > 0}\nend\nr = x;\nend\n",
            "function r = f(x)\narguments (Input = true)\n    x (1,:) double\nend\nr = x;\nend\n",
        ] {
            let diags = lint_file(&*engine(), source);
            assert!(!has_id(&diags, "SYNER"), "{source:?}: got: {diags:?}");
        }
    }

    #[test]
    fn syner_silent_where_a_lexical_check_names_the_error() {
        // MATLAB reports BADNE and TWOCM here, not a parse error. (`0xZ1` is
        // BADHBH in MATLAB, but malt does not emit BADHBH for it yet, so it is
        // not covered by this test.)
        for source in ["x = 1 != 2;\n", "x = [1,,2];\n"] {
            let diags = lint_file(&*engine(), source);
            assert!(!has_id(&diags, "SYNER"), "{source:?}: got: {diags:?}");
        }
        let diags = lint_file(&*engine(), "x = 1 != 2;\n");
        assert!(has_id(&diags, "BADNE"), "got: {diags:?}");
    }

    #[test]
    fn syner_not_reported_after_unterminated_string() {
        // MATLAB reports only STRIN for this file (probe); the parse error on the
        // next line is a cascade from the unterminated quote.
        let diags = lint_file(&*engine(), "x = 'abc\nc = \"q\";\n");
        assert!(has_id(&diags, "STRIN"), "got: {diags:?}");
        assert!(!has_id(&diags, "SYNER"), "got: {diags:?}");
    }

    // -- NOPAR2: missing closing bracket (mid-file) --------------------------

    #[test]
    fn nopar2_fires_on_unclosed_bracket_mid_file() {
        let diags = lint_file(&*engine(), "y = [1 2;\n");
        assert!(has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn nopar_fires_on_missing_paren_before_semicolon() {
        // MATLAB reports the `;` on the opener's line as the offending token: NOPAR.
        let diags = lint_file(&*engine(), "x = f(1;\ny = 2;\n");
        assert!(has_id(&diags, "NOPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn nopar2_ok_on_balanced_brackets() {
        let diags = lint_file(&*engine(), "y = [1 2];\n");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    // -- EOLPAR: missing closing bracket at end of line ----------------------

    #[test]
    fn missing_paren_before_newline_is_not_nopar() {
        let diags = lint_file(&*engine(), "function foo()\n    x = f(1\n    y = 2;\nend\n");
        assert!(!has_id(&diags, "NOPAR"), "got: {diags:?}");
    }

    #[test]
    fn missing_paren_before_newline_at_top_level_is_not_nopar() {
        let diags = lint_file(&*engine(), "x = f(1\ny = 2;\n");
        assert!(!has_id(&diags, "NOPAR"), "got: {diags:?}");
    }

    #[test]
    fn eolpar_ok_on_balanced_parens() {
        let diags = lint_file(&*engine(), "x = f(1);\n");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
    }

    // -- ENDPAR: missing closing bracket at end of file ----------------------

    #[test]
    fn endpar_fires_on_missing_paren_at_eof() {
        let diags = lint_file(&*engine(), "x = f(1 + 2;\n");
        assert!(has_id(&diags, "ENDPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
    }

    #[test]
    fn endpar_fires_on_missing_bracket_at_eof_no_newline() {
        let diags = lint_file(&*engine(), "x = f(1 + 2;");
        assert!(has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn endpar_ok_on_complete_file() {
        let diags = lint_file(&*engine(), "x = [1, 2];\n");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    // -- Missing bracket variants: must fire / must not fire -----------------

    #[test]
    fn missing_bracket_variants_must_fire_example() {
        let diags = lint_file(
            &*engine(),
            "x = f(1;\nfunction foo()\n    x = f(1;\n    y = 2;\nend\nx = f(1 + 2;\n",
        );
        assert!(has_id(&diags, "NOPAR"), "got: {diags:?}");
        assert!(has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn missing_bracket_variants_must_not_fire_example() {
        let diags = lint_file(&*engine(), "x = f(1);\nx = [1, 2];\nx = f(g(1));\n");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn nopar_fires_on_unclosed_call() {
        let diags = lint_file(&*engine(), "x = f(1;\ny = 1; 2");
        assert!(has_id(&diags, "NOPAR"), "got: {diags:?}");
    }

    #[test]
    fn missing_bracket_variants_respect_disabled_checks() {
        let engine = engine_with_disabled(&["NOPAR2"]);
        let diags = lint_file(&*engine, "y = [1 2;\n");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");

        let engine = engine_with_disabled(&["EOLPAR"]);
        let diags = lint_file(&*engine, "x = f(1;\ny = 2;\n");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");

        let engine = engine_with_disabled(&["ENDPAR"]);
        let diags = lint_file(&*engine, "x = f(1 + 2;\n");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    // -- ENDCT family: possible missing `end` -------------------------------

    #[test]
    fn endct2_fires_on_unterminated_if() {
        let diags = lint_file(&*engine(), "if x > 0\n    y = 1;\n");
        assert!(has_id(&diags, "ENDCT2"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
    }

    #[test]
    fn endct2_fires_on_unterminated_loops() {
        let diags = lint_file(&*engine(), "for i = 1:10\n    disp(i);\n");
        assert!(has_id(&diags, "ENDCT2"), "got: {diags:?}");
        let diags = lint_file(&*engine(), "while x\n    y = 1;\n");
        assert!(has_id(&diags, "ENDCT2"), "got: {diags:?}");
        let diags = lint_file(&*engine(), "switch x\n    case 1\n        y = 1;\n");
        assert!(has_id(&diags, "ENDCT2"), "got: {diags:?}");
    }

    #[test]
    fn endct2_ok_on_terminated_if() {
        let diags = lint_file(&*engine(), "if x > 0\n    y = 1;\nend\n");
        assert!(!has_id(&diags, "ENDCT2"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
    }

    #[test]
    fn endct3_fires_on_stray_else_followed_by_opener() {
        let diags = lint_file(&*engine(), "else\nif x\n");
        assert!(has_id(&diags, "ENDCT3"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
    }

    #[test]
    fn endct3_ok_without_following_opener() {
        let diags = lint_file(&*engine(), "if x\n    y = 1;\nend\nelse\n");
        assert!(!has_id(&diags, "ENDCT3"), "got: {diags:?}");
    }

    #[test]
    fn endct4_fires_on_classdef_without_methods() {
        let diags = lint_file(&*engine(), "classdef Foo\n  function f()\n  end\nend\n");
        assert!(has_id(&diags, "ENDCT4"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
    }

    #[test]
    fn endct4_fires_on_missing_methods_block() {
        let diags = lint_file(
            &*engine(),
            "function f()\n    if x\n        y = 1;\nclassdef Foo\n    function f()\n    end\nend\n",
        );
        assert!(has_id(&diags, "ENDCT4"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT2"), "got: {diags:?}");
    }

    #[test]
    fn endct_family_ok_on_valid_code() {
        let diags = lint_file(
            &*engine(),
            "function f()\n    if x\n        y = 1;\n    end\nend\nclassdef Foo\n    methods\n        function f()\n            x = 1;\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT2"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT3"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT4"), "got: {diags:?}");
    }

    #[test]
    fn endct_variant_disabled_falls_back_to_generic() {
        let engine = engine_with_disabled(&["ENDCT2"]);
        let diags = lint_file(&*engine, "if x > 0\n    y = 1;\n");
        assert!(has_id(&diags, "ENDCT"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT2"), "got: {diags:?}");
    }

    #[test]
    fn endct3_disabled_falls_back_to_generic() {
        let engine = engine_with_disabled(&["ENDCT3"]);
        let diags = lint_file(&*engine, "else\nif x\n");
        assert!(has_id(&diags, "ENDCT"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT3"), "got: {diags:?}");
    }

    #[test]
    fn endct_disabled_suppresses_variants() {
        let engine = engine_with_disabled(&["ENDCT"]);
        let diags = lint_file(
            &*engine,
            "if x > 0\n    y = 1;\nelse\nif x\nclassdef Foo\n  function f()\n  end\nend\n",
        );
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT2"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT3"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT4"), "got: {diags:?}");
    }
}
