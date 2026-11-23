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
        // A Repeating block with the Output attribute declares repeating outputs; a required
        // output in a later block is out of order. MATLAB reports this only with `-allmsg`, so
        // the check is off under the mathworks preset.
        let mut saw_repeating_output = false;
        for block in blocks {
            if block.role.is_repeating() && block.attributes.iter().any(|a| a == "Output") {
                saw_repeating_output = true;
                continue;
            }
            if !block.role.is_output() {
                continue;
            }
            for prop in block_properties(block.node) {
                if prop_has_ignored(prop) || prop_has_name_value_src(prop, source) {
                    continue;
                }
                if prop_plain_name(prop, source).is_some() && saw_repeating_output {
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

    use crate::language_spec::tests::{check_source, check_source_all_presets, filter_by_id};

    #[test]
    fn test_fvordo_fires_required_output_after_repeating_block() {
        // Probed with `-allmsg` only (MATLAB's default analysis is silent), so the `all` preset:
        // a required output in a later block after a Repeating Output block (line 6).
        let source = "\
function [a, varargout] = f(x)
    arguments (Repeating, Output)
        varargout
    end
    arguments (Output)
        a (1,1) double
    end
end
";
        let diags = check_source_all_presets(source, "f.m");
        let fvordo = filter_by_id(&diags, "FVORDO");
        assert_eq!(fvordo.len(), 1, "got: {diags:?}");
        assert_eq!(fvordo[0].line, 6);
    }

    #[test]
    fn test_fvordo_no_fire_varargout_first_in_plain_output_block() {
        // Probed: varargout first in a plain Output block is FVOVREP only (fvordo01).
        let source = "\
function [a, varargout] = f(x)
arguments (Output)
    varargout
    a (1,1) double
end
end
";
        let diags = check_source_all_presets(source, "f.m");
        assert!(filter_by_id(&diags, "FVORDO").is_empty(), "got: {diags:?}");
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
