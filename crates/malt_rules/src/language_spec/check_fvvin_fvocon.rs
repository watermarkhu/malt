//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVVIN / FVOCON: output validation function rules.
    pub(crate) fn check_fvvin_fvocon(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let vin = self.is_check_enabled("FVVIN");
        let ocon = self.is_check_enabled("FVOCON");
        if !vin && !ocon {
            return;
        }
        for block in blocks {
            if !(block.role.is_output() || block.is_output_repeating()) {
                continue;
            }
            for prop in block_properties(block.node) {
                if prop_has_ignored(prop) || prop_has_name_value(prop) {
                    continue;
                }
                let Some(name) = prop_plain_name(prop, source) else {
                    continue;
                };
                // `y(1)` indexes the argument rather than naming it: MATLAB reports FVOCON.
                if ocon && !block.role.is_repeating() && indexes_argument(prop, source, &name) {
                    self.push_diag(
                        prop,
                        "FVOCON",
                        "For output arguments, validation functions must only use the argument being validated or literals.",
                        diagnostics,
                    );
                }
                for call in prop_validator_calls(prop, source) {
                    let uses_arg = call.references.iter().any(|r| match r {
                        CallReference::Identifier(n) => n == &name,
                        // A field reference `y.a` holds the whole text: its base is a use of `y`.
                        CallReference::Field(n) => n.split('.').next() == Some(name.as_str()),
                        CallReference::Other => false,
                    });
                    if vin && !uses_arg {
                        self.push_diag(
                            prop,
                            "FVVIN",
                            "Validation function must use the argument as an input.",
                            diagnostics,
                        );
                    }
                    for reference in &call.references {
                        let is_arg =
                            matches!(reference, CallReference::Identifier(n) if n == &name);
                        // Repeating outputs are silent for FVOCON (MATLAB reports FVVIN only).
                        if ocon && !block.role.is_repeating() && !is_arg {
                            self.push_diag(
                                prop,
                                "FVOCON",
                                "For output arguments, validation functions must only use the argument being validated or literals.",
                                diagnostics,
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Whether a validation function call in `prop` indexes the variable `name` (`name(1)`).
fn indexes_argument(prop: tree_sitter::Node, source: &str, name: &str) -> bool {
    let Some(validation) = find_child_kind(prop, "validation_functions") else {
        return false;
    };
    fn walk(node: tree_sitter::Node, source: &str, name: &str) -> bool {
        if node.kind() == "function_call"
            && find_child_kind(node, "identifier").is_some_and(|c| node_text(c, source) == name)
        {
            return true;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if walk(child, source, name) {
                return true;
            }
        }
        false
    }
    walk(validation, source, name)
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvocon_fires_indexed_output_argument() {
        // Probed: `y(1)` in a validator indexes the output (MATLAB: FVOCON, line 3).
        let source = "\
function [y] = f(x)
arguments (Output)
    y (1,1) double {mustBeFoo(y(1))}
end
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "FVOCON").len(), 1, "got: {diags:?}");
        assert!(filter_by_id(&diags, "FVVIN").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_fvvin_no_fire_field_of_argument_and_fvocon_does() {
        // Probed: `y.a` uses `y` (no FVVIN) but is not the bare argument, so FVOCON (line 3).
        let source = "\
function [y] = f(x)
arguments (Output)
    y (1,1) struct {mustBeFoo(y.a)}
end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVVIN").is_empty(), "got: {diags:?}");
        assert_eq!(filter_by_id(&diags, "FVOCON").len(), 1, "got: {diags:?}");
    }

    #[test]
    fn test_fvocon_no_fire_repeating_output_block() {
        // Probed: a Repeating output is silent for FVOCON; MATLAB reports FVVIN only.
        let source = "\
function varargout = f(x)
arguments (Repeating)
    varargout {mustBeFoo(x)}
end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVOCON").is_empty(), "got: {diags:?}");
        assert_eq!(filter_by_id(&diags, "FVVIN").len(), 1, "got: {diags:?}");
    }

    #[test]
    fn test_fvocon_fires_output_validator_using_input() {
        let source = "\
function y = f(x)
    arguments (Output)
        y (1,1) double {mustBeGreaterThan(y, x)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVOCON").is_empty(),
            "FVOCON should fire when an output validator references an input argument"
        );
    }

    #[test]
    fn test_fvocon_no_fire_output_validator_self_literal() {
        let source = "\
function y = f(x)
    arguments (Output)
        y (1,1) double {mustBeGreaterThan(y, 0)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVOCON").is_empty(),
            "FVOCON should NOT fire when an output validator only uses the argument or literals"
        );
    }
}
