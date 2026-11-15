//! # VTP* / VTFIN validator checks
//!
//! These checks read the validators of a property (`{validator(...)}` after its
//! name, in a `properties` block of a class or in an `arguments` block). Probed
//! against `checkcode` in R2026b:
//!
//! - **VTPEAL** — a validator called with no arguments, `{mustBeNumeric()}`, in a
//!   class property or an argument. A bare name, `{mustBeNumeric}`, is not one.
//! - **VTPIN** — (class properties) the validator does not use the property.
//! - **VTPCON** — (class properties) the validator uses a name other than the
//!   property, or a variable, in place of a literal.
//! - **VTFIN** — the property is an argument of the validator but not the first.
//!
//! Only the validator calls are checked; the value of the default (`= expr`) is
//! FVNIV's business.

use super::*;
use crate::readability::find_arguments;

/// Identifiers treated as literal constants inside a validator (VTPCON).
const VALIDATOR_CONSTANTS: &[&str] = &["true", "false", "inf", "nan", "pi", "eps"];

impl LanguageSpecEngine {
    /// VTPEAL / VTPIN / VTPCON / VTFIN for every property validator in the file.
    pub(crate) fn check_property_validation_functions(
        &self,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut stack = vec![ctx.tree.root_node()];
        while let Some(node) = stack.pop() {
            if node.kind() == "property" {
                self.check_property_validators(node, ctx.source, diagnostics);
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }
    }

    fn check_property_validators(
        &self,
        property: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(validation) = find_child_kind(property, "validation_functions") else {
            return;
        };
        let Some(name_node) = property.child_by_field_name("name") else {
            return;
        };
        let name = node_text(name_node, source);
        let in_class = property
            .parent()
            .is_some_and(|parent| parent.kind() == "properties");
        let mut cursor = validation.walk();
        for validator in validation.named_children(&mut cursor) {
            if validator.kind() == "function_call" {
                self.check_validator(validator, name, in_class, source, diagnostics);
            }
        }
    }

    fn check_validator(
        &self,
        validator: tree_sitter::Node,
        name: &str,
        in_class: bool,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let callee = validator
            .child_by_field_name("name")
            .map(|node| node_text(node, source))
            .unwrap_or_default();
        let args: Vec<tree_sitter::Node> = match find_arguments(validator) {
            Some(arguments) => {
                let mut cursor = arguments.walk();
                arguments.named_children(&mut cursor).collect()
            }
            None => Vec::new(),
        };
        if args.is_empty() {
            self.push_diag(
                validator,
                "VTPEAL",
                "Specify at least one input argument for validator.",
                diagnostics,
            );
            return;
        }

        let uses_property = args
            .iter()
            .any(|arg| value_identifiers(*arg, source).iter().any(|id| id == name));
        let first_is_property =
            args[0].kind() == "identifier" && node_text(args[0], source) == name;

        if in_class && !uses_property {
            self.push_diag(
                validator,
                "VTPIN",
                "Validation function must use the property as an input.",
                diagnostics,
            );
        }
        if uses_property && !first_is_property {
            let message =
                format!("'{name}' should be the first input argument to the '{callee}' function.");
            self.push_diag(validator, "VTFIN", &message, diagnostics);
        }
        if in_class
            && args.iter().any(|arg| {
                value_identifiers(*arg, source)
                    .iter()
                    .any(|id| id != name && !VALIDATOR_CONSTANTS.contains(&id.as_str()))
            })
        {
            self.push_diag(
                validator,
                "VTPCON",
                "For properties, validation functions must only use the property being validated or literals.",
                diagnostics,
            );
        }
    }
}

/// Identifiers in a validator argument: variable references, not call names, field
/// names or the constants of `VALIDATOR_CONSTANTS` (which VTPCON still needs to skip).
fn value_identifiers(node: tree_sitter::Node, source: &str) -> Vec<String> {
    let mut names = Vec::new();
    collect_value_identifiers(node, source, &mut names);
    names
}

fn collect_value_identifiers(node: tree_sitter::Node, source: &str, names: &mut Vec<String>) {
    if node.kind() == "identifier" {
        let parent = node.parent();
        let is_callee = parent.is_some_and(|p| {
            p.kind() == "function_call"
                && p.child_by_field_name("name")
                    .is_some_and(|n| n.id() == node.id())
        });
        let is_field = parent.is_some_and(|p| {
            p.kind() == "field_expression"
                && p.child_by_field_name("field")
                    .is_some_and(|f| f.id() == node.id())
        });
        if !is_callee && !is_field {
            names.push(node_text(node, source).to_string());
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_value_identifiers(child, source, names);
    }
}

#[cfg(test)]
mod tests {
    use crate::language_spec::tests::{check_source, filter_by_id};

    fn class_with_property(property: &str) -> String {
        format!("classdef vtp\n    properties\n        {property}\n    end\nend\n")
    }

    #[test]
    fn test_vtp_checks_fire_on_class_property_validators() {
        for (property, id) in [
            ("x double {mustBeInRange(1, 2, 3)} = 1", "VTPIN"),
            ("x double {mustBeLessThan(y, 3)} = 1", "VTPCON"),
            ("x double {mustBeGreaterThan(5, x)} = 6", "VTFIN"),
            ("x double {mustBeNumeric()} = 1", "VTPEAL"),
        ] {
            let diags = check_source(&class_with_property(property), "vtp.m");
            assert!(
                !filter_by_id(&diags, id).is_empty(),
                "{property}: {diags:?}"
            );
        }
    }

    #[test]
    fn test_vtp_checks_silent_on_valid_validators() {
        for property in [
            "x double {mustBeGreaterThan(x, 5)} = 6",
            "x double {mustBeNumeric(x, 'extra')} = 1",
            "x double {mustBeNumeric} = 1",
        ] {
            let diags = check_source(&class_with_property(property), "vtp.m");
            for id in ["VTPEAL", "VTPIN", "VTPCON", "VTFIN"] {
                assert!(filter_by_id(&diags, id).is_empty(), "{property}: {diags:?}");
            }
        }
    }

    #[test]
    fn test_vtpeal_in_arguments_block() {
        let source = "function f(a)\narguments\n    a {mustBeText()}\nend\nend\n";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "VTPEAL").len(), 1, "{diags:?}");
    }

    #[test]
    fn test_vtfin_in_arguments_block_and_no_vtpin() {
        let source = "function f(a, b)\narguments\n    a double {mustBeGreaterThan(5, a)}\n    b double {mustBeLessThan(3, a)}\nend\nend\n";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "VTFIN").len(), 1, "{diags:?}");
        assert!(filter_by_id(&diags, "VTPIN").is_empty(), "{diags:?}");
        assert!(filter_by_id(&diags, "VTPCON").is_empty(), "{diags:?}");
    }
}
