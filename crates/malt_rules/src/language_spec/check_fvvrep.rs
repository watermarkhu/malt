//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVVREP: varargin can only be used inside a repeating input arguments block.
    pub(crate) fn check_fvvrep(
        &self,
        blocks: &[ArgumentsBlockMeta],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVVREP") {
            return;
        }
        for block in blocks {
            if block.role.is_repeating() {
                continue;
            }
            for arg in &block.args {
                if arg.name == "varargin" {
                    self.push_block_diag(
                        block,
                        "FVVREP",
                        "varargin can only be used inside repeating input arguments block.",
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
    fn test_fvvrep_fires_varargin_outside_repeating() {
        let source = "\
function f(varargin)
    arguments
        varargin
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVVREP").is_empty(),
            "FVVREP should fire for varargin outside a Repeating block"
        );
    }

    #[test]
    fn test_fvvrep_no_fire_varargin_in_repeating() {
        let source = "\
function f(varargin)
    arguments (Repeating)
        varargin
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVVREP").is_empty(),
            "FVVREP should NOT fire for varargin in a Repeating block"
        );
    }
}
