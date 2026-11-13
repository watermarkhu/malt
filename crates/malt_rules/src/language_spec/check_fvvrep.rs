//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVVREP: varargin can only be used inside a repeating input arguments block.
    pub(crate) fn check_fvvrep(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVVREP") {
            return;
        }
        for block in blocks {
            // Input blocks only: MATLAB is silent on varargin in an Output block.
            if !block.role.is_input() {
                continue;
            }
            // MATLAB points at the `varargin` declaration, not at the block.
            for prop in block_properties(block.node) {
                if prop_plain_name(prop, source).as_deref() == Some("varargin") {
                    self.push_diag(
                        prop,
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
    fn test_fvvrep_no_fire_varargin_in_output_block() {
        // Probed: MATLAB is silent on FVVREP for a varargin in an Output block (fvvrep04).
        let source = "\
function [a] = f(x)
arguments (Output)
    varargin
end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVVREP").is_empty(), "got: {diags:?}");
    }

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

    #[test]
    fn test_fvvrep_points_at_varargin_declaration() {
        // Probed: MATLAB reports at the `varargin` declaration (line 4, column 5).
        let source = "\
function f(a, varargin)
arguments
    a double
    varargin
end
";
        let diags = check_source(source, "f.m");
        let fvvrep = filter_by_id(&diags, "FVVREP");
        assert_eq!(fvvrep.len(), 1, "got: {diags:?}");
        assert_eq!((fvvrep[0].line, fvvrep[0].column), (4, 5));
    }
}
