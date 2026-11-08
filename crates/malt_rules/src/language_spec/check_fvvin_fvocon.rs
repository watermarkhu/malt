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
                for call in prop_validator_calls(prop, source) {
                    let uses_arg = call
                        .references
                        .iter()
                        .any(|r| matches!(r, CallReference::Identifier(n) if n == &name));
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
                        if ocon && !is_arg {
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

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

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
