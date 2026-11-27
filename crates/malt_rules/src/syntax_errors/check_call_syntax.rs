//! `check_call_syntax`: one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_call_syntax(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        self.walk_call_syntax(root, source, &mut diagnostics);
        diagnostics
    }

    pub(crate) fn walk_call_syntax(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "function_call" {
            self.check_function_call(node, source, diagnostics);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_call_syntax(child, source, diagnostics);
        }
    }

    pub(crate) fn check_function_call(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // SBTMP: chaining outputs after parenthesis is not supported.
        if self.is_check_enabled("SBTMP") {
            if let Some(name) = node.child_by_field_name("name") {
                // `c{1}(end)` and `x{1}{2}` are cell indexing, which MATLAB accepts.
                let name_is_brace =
                    (0..name.child_count()).any(|i| name.child(i).is_some_and(|c| c.kind() == "{"));
                // A chained call `f(1)(2)` is reported at the `(` of the outer call's
                // arguments, and so is `s.f(1)(2)`, whose name is a field expression
                // (probed: `x = f(1)(2);` is column 9; `z = s.f(1)(2);` is column 11).
                // `s.f{1}(v)` indexes a cell after the field (probed: silent in MATLAB).
                let chained_field = name.kind() == "field_expression"
                    && name.child_by_field_name("field").is_some_and(|f| {
                        f.kind() == "function_call"
                            && !(0..f.child_count())
                                .any(|i| f.child(i).is_some_and(|c| c.kind() == "{"))
                    });
                if (name.kind() == "function_call" && !name_is_brace) || chained_field {
                    let pos = Self::arguments_child(node).unwrap_or(node).start_position();
                    diagnostics.push(Diagnostic {
                            rule_id: "SBTMP",
                            message: "Invalid array indexing or function call. Chaining outputs after parenthesis is not supported."
                                .to_string(),
                            severity: Severity::Error,
                            byte_range: node.byte_range(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                }
            }
        }

        let is_brace =
            (0..node.child_count()).any(|i| node.child(i).is_some_and(|c| c.kind() == "{"));
        let args = Self::arguments_child(node);

        // FVACI: name=value syntax in cell indexing (`{}` calls).
        if self.is_check_enabled("FVACI") && is_brace {
            if let Some(a) = args {
                if Self::has_eq_token(a) {
                    // MATLAB reports the `=` of the name=value (`x = c{a=1};` is column 8).
                    let pos = (0..a.child_count())
                        .find_map(|i| a.child(i).filter(|c| c.kind() == "="))
                        .unwrap_or(node)
                        .start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "FVACI",
                        message: "Use of name-value arguments in cell indexing is not supported."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: node.byte_range(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        // FVACS/FVAMI: name=value with an invalid name in a call.
        if self.is_check_enabled("FVACS") || self.is_check_enabled("FVAMI") {
            self.check_name_value(node, args, source, diagnostics);
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

    // -- SBTMP: chained call result indexing --------------------------------

    #[test]
    fn sbtmp_no_fire_on_cell_brace_chain() {
        let diags = lint_file(&*engine(), "v = c{1}(end);\ny = [x{1}{2}];\n");
        assert!(!has_id(&diags, "SBTMP"), "got: {diags:?}");
    }

    #[test]
    fn sbtmp_fires_on_chained_call() {
        let diags = lint_file(&*engine(), "x = f()(1);\n");
        assert!(has_id(&diags, "SBTMP"), "got: {diags:?}");
    }

    #[test]
    fn sbtmp_fires_on_chained_call_with_args() {
        let diags = lint_file(&*engine(), "x = f(1)(2);\n");
        assert!(has_id(&diags, "SBTMP"), "got: {diags:?}");
    }

    #[test]
    fn sbtmp_ok_on_plain_call() {
        let diags = lint_file(&*engine(), "x = f(1);\n");
        assert!(!has_id(&diags, "SBTMP"), "got: {diags:?}");
    }

    #[test]
    fn sbtmp_ok_on_cell_index() {
        let diags = lint_file(&*engine(), "x = y{1};\n");
        assert!(!has_id(&diags, "SBTMP"), "got: {diags:?}");
    }

    #[test]
    fn sbtmp_ok_on_cell_brace_after_field() {
        // Probed (b1, b4, b5, b9): `obj.Subscribers{k}(value);` and `s.f{1}(v);` are silent.
        for src in [
            "function f(obj, value)\nobj.Subscribers{1}(value);\nend\n",
            "function f(s, v)\ns.f{1}(v);\nend\n",
            "function f(obj, v)\nobj.cb{2}(v, 1);\nend\n",
        ] {
            let diags = lint_file(&*engine(), src);
            assert!(!has_id(&diags, "SBTMP"), "{src:?} got: {diags:?}");
        }
    }

    #[test]
    fn sbtmp_ok_on_method_call() {
        let diags = lint_file(&*engine(), "obj.method();\n");
        assert!(!has_id(&diags, "SBTMP"), "got: {diags:?}");
    }

    // -- FVSYN: invalid function argument syntax ----------------------------

    #[test]
    fn two_values_in_call_are_syner_at_second_value() {
        // Probed: `f(1 2);` is SYNER at the `2` (column 5), not FVSYN.
        let diags = lint_file(&*engine(), "f(1 2);\n");
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
        let syner = diags.iter().find(|d| d.rule_id == "SYNER").expect("SYNER");
        assert_eq!((syner.line, syner.column), (1, 5), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_ok_on_comma_separated_args() {
        let diags = lint_file(&*engine(), "f(1, 2);\n");
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_ok_on_name_value_args() {
        let diags = lint_file(&*engine(), "f(Name = 1);\n");
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    // -- FVACI: name=value in cell indexing ---------------------------------

    #[test]
    fn fvaci_fires_on_name_value_cell_index() {
        let diags = lint_file(&*engine(), "c{key = 'Name'} = 5;\n");
        assert!(has_id(&diags, "FVACI"), "got: {diags:?}");
    }

    #[test]
    fn fvaci_ok_on_comma_cell_index() {
        let diags = lint_file(&*engine(), "c{key, 'Name'} = 5;\n");
        assert!(!has_id(&diags, "FVACI"), "got: {diags:?}");
    }

    #[test]
    fn fvaci_ok_on_scalar_cell_index() {
        let diags = lint_file(&*engine(), "x = y{1};\n");
        assert!(!has_id(&diags, "FVACI"), "got: {diags:?}");
    }

    // -- FVACS: quoted string as name in name=value -------------------------

    #[test]
    fn fvacs_fires_on_identifier_quoted_name() {
        // Probed: `'a'` and `'Good'` are identifiers in quotes (FVACS).
        let diags = lint_file(&*engine(), "x = f('Good' = 1);\n");
        assert!(has_id(&diags, "FVACS"), "got: {diags:?}");
        assert!(!has_id(&diags, "FVAMI"), "got: {diags:?}");
    }

    #[test]
    fn fvami_fires_on_quoted_name_that_is_not_an_identifier() {
        // Probed: `f('Bad Name' = 1);` gives FVAMI at the quote (column 3), not FVACS.
        let diags = lint_file(&*engine(), "f('Bad Name' = 1);\n");
        assert!(has_id(&diags, "FVAMI"), "got: {diags:?}");
        assert!(!has_id(&diags, "FVACS"), "got: {diags:?}");
    }

    #[test]
    fn fvacs_ok_on_valid_identifier_name() {
        let diags = lint_file(&*engine(), "f(Name = 1);\n");
        assert!(!has_id(&diags, "FVACS"), "got: {diags:?}");
    }

    #[test]
    fn fvacs_ok_on_no_equals() {
        let diags = lint_file(&*engine(), "f(Name, 1);\n");
        assert!(!has_id(&diags, "FVACS"), "got: {diags:?}");
    }

    #[test]
    fn fvami_alone_on_a_statement_with_a_bad_name() {
        // Probed: `f('Bad Name' = 1)` without a semicolon gives FVAMI only, no SYNER or NOPAR2.
        let diags = lint_file(&*engine(), "f('Bad Name' = 1)\n");
        assert!(has_id(&diags, "FVAMI"), "got: {diags:?}");
        assert!(
            !has_id(&diags, "SYNER") && !has_id(&diags, "NOPAR2"),
            "got: {diags:?}"
        );
    }

    // -- FVAMI: name not a valid identifier in name=value -------------------

    #[test]
    fn fvami_ok_on_number_name() {
        // Probed: MATLAB reports NOPAR and SYNER for `f(123 = 1)` and `f(123=1)`, not FVAMI.
        for src in ["f(123 = 1);\n", "f(123=1);\n"] {
            let diags = lint_file(&*engine(), src);
            assert!(!has_id(&diags, "FVAMI"), "{src:?} got: {diags:?}");
        }
    }

    #[test]
    fn fvami_ok_on_field_assignment_in_call() {
        // MATLAB reports SYNER/NOPAR here, not FVAMI.
        let diags = lint_file(&*engine(), "f(a.b=1);\n");
        assert!(!has_id(&diags, "FVAMI"), "got: {diags:?}");
    }

    #[test]
    fn fvami_ok_on_valid_identifier_name() {
        let diags = lint_file(&*engine(), "f(Name = 1);\n");
        assert!(!has_id(&diags, "FVAMI"), "got: {diags:?}");
    }

    #[test]
    fn fvami_ok_on_plain_args() {
        let diags = lint_file(&*engine(), "f(1, 2);\n");
        assert!(!has_id(&diags, "FVAMI"), "got: {diags:?}");
    }

    // -- call syntax checks disabled via config -----------------------------

    #[test]
    fn disabled_checks_turn_off_call_syntax_checks() {
        let engine = engine_with_disabled(&["SBTMP", "FVSYN", "FVACI", "FVACS", "FVAMI"]);
        let src = "x = f()(1);\nf(1 2);\nc{key = 'Name'} = 5;\nf('Bad Name' = 1);\nf(123 = 1);\n";
        let diags = lint_file(&*engine, src);
        assert!(!has_id(&diags, "SBTMP"), "got: {diags:?}");
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
        assert!(!has_id(&diags, "FVACI"), "got: {diags:?}");
        assert!(!has_id(&diags, "FVACS"), "got: {diags:?}");
        assert!(!has_id(&diags, "FVAMI"), "got: {diags:?}");
    }
}
