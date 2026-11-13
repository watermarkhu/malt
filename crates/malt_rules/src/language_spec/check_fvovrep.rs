//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVOVREP: varargout can only be used inside a repeating output arguments block.
    pub(crate) fn check_fvovrep(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVOVREP") {
            return;
        }
        for block in blocks {
            // Only a plain Output block: MATLAB is silent on varargout in an Input block.
            if !block.role.is_output() {
                continue;
            }
            // MATLAB points at the varargout declaration, not at the block.
            for prop in block_properties(block.node) {
                if prop_plain_name(prop, source).as_deref() == Some("varargout") {
                    self.push_diag(
                        prop,
                        "FVOVREP",
                        "Output argument varargout can only be used inside a Repeating output arguments block.",
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
    fn test_fvovrep_reported_at_the_varargout_declaration() {
        // Probed: MATLAB points at the varargout declaration (line 3, column 5), not the block.
        let source = "\
function [varargout] = f(x)
arguments (Output)
    varargout
end
end
";
        let diags = check_source(source, "f.m");
        let fvovrep = filter_by_id(&diags, "FVOVREP");
        assert_eq!(fvovrep.len(), 1, "got: {diags:?}");
        assert_eq!((fvovrep[0].line, fvovrep[0].column), (3, 5));
    }

    #[test]
    fn test_fvovrep_no_fire_varargout_in_input_block() {
        // Probed: MATLAB is silent on varargout in a plain (Input) arguments block.
        let source = "\
function varargout = f(x)
arguments
    varargout
end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVOVREP").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_fvovrep_fires_varargout_in_output_block() {
        let source = "\
function [varargout] = f(x)
    arguments (Output)
        varargout
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVOVREP").is_empty(),
            "FVOVREP should fire for varargout outside a Repeating block"
        );
    }

    #[test]
    fn test_fvovrep_no_fire_varargout_in_repeating() {
        let source = "\
function [varargout] = f(x)
    arguments (Repeating)
        varargout
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVOVREP").is_empty(),
            "FVOVREP should NOT fire for varargout in a Repeating block"
        );
    }
}
