//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVVIN / FVVCON / FVUBD: input validation function rules.
    pub(crate) fn check_fvvin_fvvcon_fvubd(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let vin = self.is_check_enabled("FVVIN");
        let vcon = self.is_check_enabled("FVVCON");
        let ubd = self.is_check_enabled("FVUBD");
        if !vin && !vcon && !ubd {
            return;
        }

        let mut input_props: Vec<(tree_sitter::Node, String)> = Vec::new();
        for block in blocks {
            if !(block.role.is_input() || block.is_input_repeating()) {
                continue;
            }
            for prop in block_properties(block.node) {
                if prop_has_ignored(prop) || prop_has_name_value(prop) {
                    continue;
                }
                if let Some(name) = prop_plain_name(prop, source) {
                    if name != "varargin" {
                        input_props.push((prop, name));
                    }
                }
            }
        }

        for (index, (prop, name)) in input_props.iter().enumerate() {
            let calls = prop_validator_calls(*prop, source);
            if calls.is_empty() {
                continue;
            }
            let previously_declared: Vec<&String> =
                input_props[..index].iter().map(|(_, n)| n).collect();
            let declared_later: Vec<&String> =
                input_props[index + 1..].iter().map(|(_, n)| n).collect();
            for call in &calls {
                let uses_arg = call
                    .references
                    .iter()
                    .any(|r| matches!(r, CallReference::Identifier(n) if n == name));
                if vin && !uses_arg {
                    self.push_diag(
                        *prop,
                        "FVVIN",
                        "Validation function must use the argument as an input.",
                        diagnostics,
                    );
                }
                for reference in &call.references {
                    match reference {
                        CallReference::Identifier(ref_name) => {
                            if ref_name == name {
                                continue;
                            }
                            if ubd && declared_later.contains(&ref_name) {
                                self.push_diag(
                                    *prop,
                                    "FVUBD",
                                    "Argument is referenced before it is declared in the arguments block.",
                                    diagnostics,
                                );
                            }
                            if vcon && !previously_declared.contains(&ref_name) {
                                self.push_diag(
                                    *prop,
                                    "FVVCON",
                                    "For input arguments, validation functions must only use previously declared positional arguments, the argument being validated, or literals.",
                                    diagnostics,
                                );
                            }
                        }
                        CallReference::Field => {
                            if vcon {
                                self.push_diag(
                                    *prop,
                                    "FVVCON",
                                    "For input arguments, validation functions must only use previously declared positional arguments, the argument being validated, or literals.",
                                    diagnostics,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvubd_fires_reference_before_declaration() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double {mustBeLessThan(a, b)}
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVUBD").is_empty(),
            "FVUBD should fire when a validator references a later argument"
        );
    }

    #[test]
    fn test_fvubd_no_fire_reference_after_declaration() {
        let source = "\
function f(a, b)
    arguments
        b (1,1) double
        a (1,1) double {mustBeLessThan(a, b)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVUBD").is_empty(),
            "FVUBD should NOT fire when a validator references an earlier argument"
        );
    }

    #[test]
    fn test_fvvcon_fires_undeclared_reference() {
        let source = "\
function f(a)
    arguments
        a (1,1) double {mustBeLessThan(a, z)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVVCON").is_empty(),
            "FVVCON should fire when a validator references a non-positional value"
        );
    }

    #[test]
    fn test_fvvcon_no_fire_self_and_prior() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double {mustBeGreaterThan(a, 0)}
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVVCON").is_empty(),
            "FVVCON should NOT fire when validators use the argument or literals"
        );
    }

    #[test]
    fn test_fvvin_fires_argument_not_used() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double {mustBeGreaterThan(b, 0)}
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVVIN").is_empty(),
            "FVVIN should fire when the validated argument is not used by the validator"
        );
    }

    #[test]
    fn test_fvvin_no_fire_argument_used() {
        let source = "\
function f(a)
    arguments
        a (1,1) double {mustBeReal}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVVIN").is_empty(),
            "FVVIN should NOT fire when the validator uses the argument (or is bare)"
        );
    }
}
