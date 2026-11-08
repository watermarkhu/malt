//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVOVREP: varargout can only be used inside a repeating output arguments block.
    pub(crate) fn check_fvovrep(
        &self,
        blocks: &[ArgumentsBlockMeta],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVOVREP") {
            return;
        }
        for block in blocks {
            if block.role.is_repeating() {
                continue;
            }
            for arg in &block.args {
                if arg.name == "varargout" {
                    self.push_block_diag(
                        block,
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
