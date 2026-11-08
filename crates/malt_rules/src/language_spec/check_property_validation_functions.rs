//! # VTP* property validation function checks
//!
//! These checks validate property-validation functions: `function validate*`
//! definitions inside a class `methods` block.
//!
//! - **VTPEAL** — the validator declares no input arguments.
//! - **VTPIN** — the validator accepts inputs but never uses any of them.
//! - **VTPCON** — the validator body references a variable other than the
//!   property being validated or a literal (function-call callees and known
//!   constants are allowed).
//!
//! Detection is deliberately heuristic and conservative: only functions whose
//! name starts with `validate` inside a `methods` block are considered.
//!
//! See the parent module `super` for the shared engine and helpers.

use super::*;

/// Identifiers treated as literal constants inside a validator body (VTPCON).
const VALIDATOR_CONSTANTS: &[&str] = &["true", "false", "inf", "nan", "pi", "eps"];

impl LanguageSpecEngine {
    /// VTP* checks for property-validation functions in a class file.
    pub(crate) fn check_property_validation_functions(
        &self,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut stack = vec![ctx.tree.root_node()];
        while let Some(node) = stack.pop() {
            if node.kind() == "methods" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "function_definition" {
                        self.check_validator_function(child, ctx.source, diagnostics);
                    }
                }
                continue;
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }
    }

    /// Validate a single candidate validator function (VTPEAL / VTPIN / VTPCON).
    fn check_validator_function(
        &self,
        func_node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(name_node) = func_node.child_by_field_name("name") else {
            return;
        };
        let name = node_text(name_node, source);
        if !name.starts_with("validate") {
            return;
        }

        // Collect input argument names from the function signature.
        let mut inputs = Vec::new();
        let mut cursor = func_node.walk();
        for child in func_node.children(&mut cursor) {
            if child.kind() == "function_arguments" {
                let mut inner = child.walk();
                for arg in child.children(&mut inner) {
                    if arg.kind() == "identifier" {
                        inputs.push(node_text(arg, source).to_string());
                    }
                }
            }
        }

        // Find the function body block.
        let mut body = None;
        let mut c2 = func_node.walk();
        for child in func_node.children(&mut c2) {
            if child.kind() == "block" {
                body = Some(child);
            }
        }
        let Some(body_node) = body else {
            // Abstract function signatures have no body; not a validator.
            return;
        };

        // VTPEAL: the validator must declare at least one input argument.
        if inputs.is_empty() {
            if self.is_check_enabled("VTPEAL") {
                self.push_diag(
                    func_node,
                    "VTPEAL",
                    "Specify at least one input argument for validator.",
                    diagnostics,
                );
            }
            return;
        }

        // VTPIN: the validator body must use at least one of its inputs.
        let uses_input = inputs
            .iter()
            .any(|input| Self::body_references_identifier(body_node, source, input));
        if !uses_input {
            if self.is_check_enabled("VTPIN") {
                self.push_diag(
                    func_node,
                    "VTPIN",
                    "Validation function must use the property as an input.",
                    diagnostics,
                );
            }
            return;
        }

        // VTPCON: the body must only reference the property (its inputs) or
        // literals — no other variables.
        if self.is_check_enabled("VTPCON") {
            let mut cursor = body_node.walk();
            for child in body_node.children(&mut cursor) {
                Self::check_validator_body_identifier(child, source, &inputs, diagnostics);
            }
        }
    }

    /// Whether the body of a function references the given identifier name.
    fn body_references_identifier(body: tree_sitter::Node, source: &str, name: &str) -> bool {
        if body.kind() == "identifier" {
            return node_text(body, source) == name;
        }
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if Self::body_references_identifier(child, source, name) {
                return true;
            }
        }
        false
    }

    /// Walk one node of a validator body and fire VTPCON for any variable
    /// reference that is not an input, a function-call callee, a field name,
    /// or a known constant.
    fn check_validator_body_identifier(
        node: tree_sitter::Node,
        source: &str,
        inputs: &[String],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "identifier" {
            let name = node_text(node, source);
            let is_input = inputs.iter().any(|i| i == name);
            let is_callee = node
                .parent()
                .map(|p| {
                    p.kind() == "function_call"
                        && p.child_by_field_name("name")
                            .map(|n| n.id() == node.id())
                            .unwrap_or(false)
                })
                .unwrap_or(false);
            let is_field = node
                .parent()
                .map(|p| {
                    p.kind() == "field_expression"
                        && p.child_by_field_name("field")
                            .map(|f| f.id() == node.id())
                            .unwrap_or(false)
                })
                .unwrap_or(false);
            let is_constant = VALIDATOR_CONSTANTS.contains(&name);
            if !is_input && !is_callee && !is_field && !is_constant {
                diagnostics.push(Diagnostic {
                    rule_id: "VTPCON",
                    message: "For properties, validation functions must only use the property being validated or literals."
                        .to_string(),
                    severity: Severity::Error,
                    byte_range: node.start_byte()..node.end_byte(),
                    line: node.start_position().row + 1,
                    column: node.start_position().column + 1,
                    fix: None,
                });
            }
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::check_validator_body_identifier(child, source, inputs, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};
    use malt_core::Rule;

    // -- VTPEAL --------------------------------------------------------------

    #[test]
    fn test_vtpeal_fires_validator_without_inputs() {
        let source = "\
classdef Foo
    methods (Static)
        function validate()
            error('bad');
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "VTPEAL").is_empty(),
            "VTPEAL should fire for a validator with no inputs"
        );
    }

    #[test]
    fn test_vtpeal_no_fire_validator_with_input() {
        let source = "\
classdef Foo
    methods (Static)
        function validate(value)
            if value < 0
                error('bad');
            end
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "VTPEAL").is_empty(),
            "VTPEAL should NOT fire for a validator with an input"
        );
    }

    // -- VTPIN ---------------------------------------------------------------

    #[test]
    fn test_vtpin_fires_unused_input() {
        let source = "\
classdef Foo
    methods (Static)
        function validate(value)
            y = 1;
            z = y + 1;
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "VTPIN").is_empty(),
            "VTPIN should fire when the validator never uses its input"
        );
    }

    #[test]
    fn test_vtpin_no_fire_used_input() {
        let source = "\
classdef Foo
    methods (Static)
        function validate(value)
            if ~isa(value, 'double')
                error('bad');
            end
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "VTPIN").is_empty(),
            "VTPIN should NOT fire when the validator uses its input"
        );
    }

    // -- VTPCON --------------------------------------------------------------

    #[test]
    fn test_vtpcon_fires_other_variable_reference() {
        let source = "\
classdef Foo
    methods (Static)
        function validate(value)
            y = value * 2;
            if y < 0
                error('bad');
            end
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "VTPCON").is_empty(),
            "VTPCON should fire when the validator references a variable other than the property"
        );
    }

    #[test]
    fn test_vtpcon_no_fire_input_and_callees_only() {
        let source = "\
classdef Foo
    methods (Static)
        function validate(value)
            if ~isa(value, 'double')
                error('bad');
            end
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "VTPCON").is_empty(),
            "VTPCON should NOT fire when only the input and function calls are used"
        );
    }

    #[test]
    fn test_vtp_checks_no_fire_non_validator_function() {
        let source = "\
classdef Foo
    methods
        function y = compute(self, x)
            y = x + 1;
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "VTPCON").is_empty(),
            "VTPCON should NOT fire for a non-validator method"
        );
        assert!(
            filter_by_id(&diags, "VTPEAL").is_empty(),
            "VTPEAL should NOT fire for a non-validator method"
        );
        assert!(
            filter_by_id(&diags, "VTPIN").is_empty(),
            "VTPIN should NOT fire for a non-validator method"
        );
    }

    // -- Config respect ------------------------------------------------------

    #[test]
    fn test_vtp_checks_respect_disabled_config() {
        let mut rule_config = crate::language_spec::LanguageSpecConfig::default();
        rule_config.disabled_checks.push("VTPEAL".to_string());
        let engine = crate::language_spec::LanguageSpecEngine {
            config: rule_config,
        };
        let source = "\
classdef Foo
    methods (Static)
        function validate()
            error('bad');
        end
    end
end
";
        let tree = crate::language_spec::tests::parse_matlab(source);
        let path = std::path::Path::new("Foo.m");
        let ctx = malt_core::FileContext {
            tree: &tree,
            source,
            file_path: path,
        };
        let diags = engine.check_file(&ctx);
        assert!(
            filter_by_id(&diags, "VTPEAL").is_empty(),
            "VTPEAL should be disabled via config disabled_checks"
        );
    }
}
