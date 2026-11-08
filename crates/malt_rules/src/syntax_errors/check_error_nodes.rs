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

        // EOFMI: Check if file ends with ERROR node.
        if self.is_check_enabled("EOFMI") {
            if let Some(last) = Self::last_descendant(root) {
                if last.is_error() || last.is_missing() {
                    let start = last.start_byte();
                    let end = last.end_byte().max(start + 1);
                    let pos = last.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "EOFMI",
                        message: "Invalid syntax at end of file. File is incomplete.".to_string(),
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
                                        "A {bracket} might be missing a closing {bracket}, causing invalid syntax at end of file."
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
                                        "A {bracket} might be missing a closing {bracket}, causing invalid syntax at end of line."
                                    ),
                                    severity: Severity::Error,
                                    byte_range: start..end,
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                        }
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
            } else if self.is_check_enabled("SYNER") {
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

    // -- NOPAR2: missing closing bracket (mid-file) --------------------------

    #[test]
    fn nopar2_fires_on_unclosed_bracket_mid_file() {
        let diags = lint_file(&*engine(), "y = [1 2;\n");
        assert!(has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn nopar2_fires_on_missing_bracket_not_at_line_end() {
        let diags = lint_file(&*engine(), "x = f(1;\ny = 2;\n");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(has_id(&diags, "EOLPAR"), "got: {diags:?}");
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
    fn eolpar_fires_on_missing_paren_at_line_end() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    x = f(1;\n    y = 2;\nend\n",
        );
        assert!(has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn eolpar_fires_on_top_level_missing_paren_at_line_end() {
        let diags = lint_file(&*engine(), "x = f(1;\ny = 2;\n");
        assert!(has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
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
        assert!(has_id(&diags, "EOLPAR"), "got: {diags:?}");
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
    fn eolpar_cooccurs_with_eofmi() {
        let diags = lint_file(&*engine(), "x = f(1;\ny = 1; 2");
        assert!(has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(has_id(&diags, "EOFMI"), "got: {diags:?}");
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

    // -- EOFMI: file ends with ERROR node -----------------------------------

    #[test]
    fn eofmi_fires_on_incomplete_tail() {
        let diags = lint_file(&*engine(), "x = 1; 2");
        assert!(has_id(&diags, "EOFMI"), "got: {diags:?}");
    }

    #[test]
    fn eofmi_ok_on_complete_file() {
        let diags = lint_file(&*engine(), "x = 5;\n");
        assert!(!has_id(&diags, "EOFMI"), "got: {diags:?}");
    }
}
