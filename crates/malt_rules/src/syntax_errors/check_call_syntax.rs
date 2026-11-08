//! `check_call_syntax` — one check of the Syntax Errors engine.

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
                if name.kind() == "function_call" {
                    let pos = node.start_position();
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
                    let pos = node.start_position();
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
    fn sbtmp_ok_on_method_call() {
        let diags = lint_file(&*engine(), "obj.method();\n");
        assert!(!has_id(&diags, "SBTMP"), "got: {diags:?}");
    }

    // -- FVSYN: invalid function argument syntax ----------------------------

    #[test]
    fn fvsyn_fires_on_missing_comma() {
        let diags = lint_file(&*engine(), "f(1 2);\n");
        assert!(has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_replaces_syner_in_args() {
        let diags = lint_file(&*engine(), "f(1 2);\n");
        assert!(has_id(&diags, "FVSYN"), "got: {diags:?}");
        assert!(!has_id(&diags, "SYNER"), "got: {diags:?}");
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
    fn fvacs_fires_on_quoted_name() {
        let diags = lint_file(&*engine(), "f('Bad Name' = 1);\n");
        assert!(has_id(&diags, "FVACS"), "got: {diags:?}");
    }

    #[test]
    fn fvacs_fires_on_quoted_name_no_space() {
        let diags = lint_file(&*engine(), "f('Bad Name'=1);\n");
        assert!(has_id(&diags, "FVACS"), "got: {diags:?}");
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

    // -- FVAMI: name not a valid identifier in name=value -------------------

    #[test]
    fn fvami_fires_on_number_name() {
        let diags = lint_file(&*engine(), "f(123 = 1);\n");
        assert!(has_id(&diags, "FVAMI"), "got: {diags:?}");
    }

    #[test]
    fn fvami_fires_on_number_name_no_space() {
        let diags = lint_file(&*engine(), "f(123=1);\n");
        assert!(has_id(&diags, "FVAMI"), "got: {diags:?}");
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
