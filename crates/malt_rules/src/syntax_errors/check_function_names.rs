//! `check_function_names` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_function_names(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        self.walk_function_names(root, source, &mut diagnostics, false);
        diagnostics
    }

    pub(crate) fn walk_function_names(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
        in_methods_block: bool,
    ) {
        let kind = node.kind();
        let is_methods = kind == "methods";

        if kind == "function_definition" {
            // Find the function name node.
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = &source[name_node.start_byte()..name_node.end_byte()];
                let pos = name_node.start_position();

                // FNSWA: Function name doesn't start with alphabetic character.
                if self.is_check_enabled("FNSWA")
                    && !name.starts_with(|c: char| c.is_ascii_alphabetic())
                {
                    diagnostics.push(Diagnostic {
                        rule_id: "FNSWA",
                        message: "Function name must start with alphabetic character.".to_string(),
                        severity: Severity::Error,
                        byte_range: name_node.byte_range(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }

                // FNDOT: Function name contains dots but is not in a class methods block.
                if self.is_check_enabled("FNDOT") && name.contains('.') && !in_methods_block {
                    diagnostics.push(Diagnostic {
                        rule_id: "FNDOT",
                        message: "Function name can only contain dots if it is a class method."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: name_node.byte_range(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_function_names(child, source, diagnostics, in_methods_block || is_methods);
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

    // -- FNDOT: dotted function name outside methods ------------------------

    #[test]
    fn fndot_fires_on_dotted_function_name() {
        let diags = lint_file(&*engine(), "function y = foo.bar()\n    y = 1;\nend\n");
        assert!(has_id(&diags, "FNDOT"), "got: {diags:?}");
    }

    #[test]
    fn fndot_ok_on_simple_function_name() {
        let diags = lint_file(&*engine(), "function y = foo()\n    y = 1;\nend\n");
        assert!(!has_id(&diags, "FNDOT"), "got: {diags:?}");
    }

    // -- FNSWA: function name starts with non-alphabetic --------------------

    #[test]
    fn fnswa_fires_on_underscore_name() {
        let diags = lint_file(&*engine(), "function y = _foo()\n    y = 1;\nend\n");
        assert!(has_id(&diags, "FNSWA"), "got: {diags:?}");
    }

    #[test]
    fn fnswa_ok_on_alphabetic_name() {
        let diags = lint_file(&*engine(), "function y = foo()\n    y = 1;\nend\n");
        assert!(!has_id(&diags, "FNSWA"), "got: {diags:?}");
    }
}
