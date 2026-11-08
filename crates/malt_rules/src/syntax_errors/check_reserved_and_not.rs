//! `check_reserved_and_not` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_reserved_and_not(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        self.walk_reserved_and_not(root, source, &mut diagnostics);
        diagnostics
    }

    pub(crate) fn walk_reserved_and_not(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let kind = node.kind();

        if node.is_error() {
            let start = node.start_byte();
            let end = node.end_byte().max(start + 1);
            let pos = node.start_position();
            let text = &source[start..end.min(source.len())];

            // MCPLD: ERROR inside a property declaration in a properties block.
            if self.is_check_enabled("MCPLD") && Self::inside_property(node) {
                let name = Self::ancestor_of_kind(node, "property")
                    .and_then(|prop| Self::first_child_text(prop, "identifier", source))
                    .unwrap_or("(unknown)");
                diagnostics.push(Diagnostic {
                    rule_id: "MCPLD",
                    message: format!("Invalid property syntax at {name}."),
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }

            // SYNEND takes priority over RESWD for `end`.
            if self.is_check_enabled("SYNEND") && Self::has_descendant_kind(node, "end_keyword") {
                diagnostics.push(Diagnostic {
                    rule_id: "SYNEND",
                    message: "Invalid use for END operator.".to_string(),
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            } else if self.is_check_enabled("RESWD") {
                if let Some(kw) = Self::contains_reserved_word(text) {
                    if kw != "end" {
                        diagnostics.push(Diagnostic {
                            rule_id: "RESWD",
                            message: "Invalid use of a reserved word.".to_string(),
                            severity: Severity::Error,
                            byte_range: start..end,
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }

            // BADNOT: bare `~` statement.
            if self.is_check_enabled("BADNOT") && text.trim() == "~" {
                diagnostics.push(Diagnostic {
                    rule_id: "BADNOT",
                    message: "Using ~ to ignore a value is not permitted in this context."
                        .to_string(),
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }

        // BADNOT: `~` used as a value operator with a malformed operand.
        if kind == "not_operator"
            && self.is_check_enabled("BADNOT")
            && Self::has_error_or_missing_descendant(node)
        {
            let start = node.start_byte();
            let end = node.end_byte();
            let pos = node.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "BADNOT",
                message: "Using ~ to ignore a value is not permitted in this context.".to_string(),
                severity: Severity::Error,
                byte_range: start..end,
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }

        // BADNOTLHS: `~` adjacent to an output variable without a comma.
        if kind == "multioutput_variable" && self.is_check_enabled("BADNOTLHS") {
            let mut cursor = node.walk();
            let children: Vec<Node> = node.children(&mut cursor).collect();
            for (i, child) in children.iter().enumerate() {
                if child.kind() != "ignored_argument" {
                    continue;
                }
                let prev_is_id = i > 0 && children[i - 1].kind() == "identifier";
                let next_is_id = i + 1 < children.len() && children[i + 1].kind() == "identifier";
                if prev_is_id || next_is_id {
                    let pos = child.start_position();
                    diagnostics.push(Diagnostic {
                            rule_id: "BADNOTLHS",
                            message: "Invalid use of logical not operator (~) on left side of an assignment. To use ~ to ignore function outputs, separate output variables with commas."
                                .to_string(),
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

    // -- RESWD: reserved word used as identifier ----------------------------

    #[test]
    fn reswd_fires_on_else_statement() {
        let diags = lint_file(&*engine(), "else\n");
        assert!(has_id(&diags, "RESWD"), "got: {diags:?}");
    }

    #[test]
    fn reswd_fires_on_reserved_word_in_assignment() {
        let diags = lint_file(&*engine(), "y = for;\n");
        assert!(has_id(&diags, "RESWD"), "got: {diags:?}");
    }

    #[test]
    fn reswd_fires_on_case_statement() {
        let diags = lint_file(&*engine(), "case = 5;\n");
        assert!(has_id(&diags, "RESWD"), "got: {diags:?}");
    }

    #[test]
    fn reswd_ok_on_valid_identifiers_and_blocks() {
        let diags = lint_file(
            &*engine(),
            "foo_else = 5;\nfor x = 1:10\nend\nswitch x\ncase 1\ny = 1;\nend\n",
        );
        assert!(!has_id(&diags, "RESWD"), "got: {diags:?}");
    }

    // -- SYNEND: invalid use of END -----------------------------------------

    #[test]
    fn synend_fires_on_end_assignment_lhs() {
        let diags = lint_file(&*engine(), "end = 5;\n");
        assert!(has_id(&diags, "SYNEND"), "got: {diags:?}");
    }

    #[test]
    fn synend_fires_on_end_as_value() {
        let diags = lint_file(&*engine(), "y = end;\n");
        assert!(has_id(&diags, "SYNEND"), "got: {diags:?}");
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

    #[test]
    fn synend_takes_priority_over_reswd() {
        let diags = lint_file(&*engine(), "y = end;\n");
        assert!(has_id(&diags, "SYNEND"), "got: {diags:?}");
        assert!(!has_id(&diags, "RESWD"), "got: {diags:?}");
    }

    // -- MCPLD: invalid property syntax -------------------------------------

    #[test]
    fn mcpld_fires_on_double_equals_property() {
        let diags = lint_file(
            &*engine(),
            "classdef Foo\nproperties\nx = 1 = 2\nend\nend\n",
        );
        assert!(has_id(&diags, "MCPLD"), "got: {diags:?}");
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

    // -- BADNOT: ~ misuse ----------------------------------------------------

    #[test]
    fn badnot_fires_on_tilde_equals_expression() {
        let diags = lint_file(&*engine(), "x = ~ = 5;\n");
        assert!(has_id(&diags, "BADNOT"), "got: {diags:?}");
    }

    #[test]
    fn badnot_fires_on_bare_tilde_statement() {
        let diags = lint_file(&*engine(), "~\n");
        assert!(has_id(&diags, "BADNOT"), "got: {diags:?}");
    }

    #[test]
    fn badnot_ok_on_logical_not() {
        let diags = lint_file(&*engine(), "x = ~y;\n");
        assert!(!has_id(&diags, "BADNOT"), "got: {diags:?}");
    }

    #[test]
    fn badnot_ok_on_ignored_assignment_lhs() {
        let diags = lint_file(&*engine(), "~ = 5;\n");
        assert!(!has_id(&diags, "BADNOT"), "got: {diags:?}");
    }

    // -- BADNOTLHS: ~ adjacent to output without comma ----------------------

    #[test]
    fn badnotlhs_fires_on_adjacent_tilde() {
        let diags = lint_file(&*engine(), "[x ~ y] = f();\n");
        assert!(has_id(&diags, "BADNOTLHS"), "got: {diags:?}");
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
    fn badnotlhs_ok_on_fully_comma_separated() {
        let diags = lint_file(&*engine(), "[x, ~, y] = f();\n");
        assert!(!has_id(&diags, "BADNOTLHS"), "got: {diags:?}");
    }

    // -- G3 checks disabled via config --------------------------------------

    #[test]
    fn disabled_checks_turn_off_g3_checks() {
        let engine = engine_with_disabled(&["RESWD", "SYNEND", "MCPLD", "BADNOT", "BADNOTLHS"]);
        let diags = lint_file(
            &*engine,
            "else\nend = 5;\nclassdef Foo\nproperties\nx = 1 = 2\nend\nend\nx = ~ = 5;\n[x ~ y] = f();\n",
        );
        assert!(!has_id(&diags, "RESWD"), "got: {diags:?}");
        assert!(!has_id(&diags, "SYNEND"), "got: {diags:?}");
        assert!(!has_id(&diags, "MCPLD"), "got: {diags:?}");
        assert!(!has_id(&diags, "BADNOT"), "got: {diags:?}");
        assert!(!has_id(&diags, "BADNOTLHS"), "got: {diags:?}");
    }
}
