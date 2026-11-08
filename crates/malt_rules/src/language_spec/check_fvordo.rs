//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVORDO: Repeating output arguments must be defined after required output arguments.
    pub(crate) fn check_fvordo(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVORDO") {
            return;
        }
        let mut saw_repeating_output = false;
        for block in blocks {
            if !(block.role.is_output() || block.is_output_repeating()) {
                continue;
            }
            let block_repeating_output = block.is_output_repeating();
            for prop in block_properties(block.node) {
                if prop_has_ignored(prop) || prop_has_name_value(prop) {
                    continue;
                }
                let Some(name) = prop_plain_name(prop, source) else {
                    continue;
                };
                if name == "varargout" || block_repeating_output {
                    saw_repeating_output = true;
                    continue;
                }
                if saw_repeating_output {
                    self.push_diag(
                        prop,
                        "FVORDO",
                        "Repeating output arguments must be defined after required output arguments.",
                        diagnostics,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvordo_fires_repeating_output_before_required() {
        let source = "\
function [a, varargout] = f(x)
    arguments (Output)
        varargout
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVORDO").is_empty(),
            "FVORDO should fire when a repeating output precedes a required output"
        );
    }

    #[test]
    fn test_fvordo_no_fire_required_before_repeating() {
        let source = "\
function [a, varargout] = f(x)
    arguments (Output)
        a (1,1) double
    end
    arguments (Repeating)
        varargout
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVORDO").is_empty(),
            "FVORDO should NOT fire when required outputs precede repeating outputs"
        );
    }
}
