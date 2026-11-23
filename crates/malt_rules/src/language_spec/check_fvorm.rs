//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVORM: Declaring multiple repeating output arguments is not supported.
    pub(crate) fn check_fvorm(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVORM") {
            return;
        }
        for block in blocks {
            // In a Repeating block with the Output attribute every output after the first is a
            // repeating output too. Duplicate varargout across blocks is FVDAP, not this.
            if !(block.role.is_repeating() && block.attributes.iter().any(|a| a == "Output")) {
                continue;
            }
            let declarations: Vec<_> = block_properties(block.node)
                .into_iter()
                .filter(|prop| !prop_has_ignored(*prop) && !prop_has_name_value_src(*prop, source))
                .collect();
            for prop in declarations.into_iter().skip(1) {
                self.push_diag(
                    prop,
                    "FVORM",
                    "Declaring multiple repeating output arguments is not supported.",
                    diagnostics,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvorm_fires_second_declaration_of_repeating_output_block() {
        // Probed: in a `(Repeating, Output)` block every declaration after the first is a
        // repeating output (MATLAB: line 4, column 5).
        let source = "\
function [a, varargout] = f(x)
arguments (Repeating, Output)
    varargout
    a (1,1) double
end
end
";
        let diags = check_source(source, "f.m");
        let fvorm = filter_by_id(&diags, "FVORM");
        assert_eq!(fvorm.len(), 1, "got: {diags:?}");
        assert_eq!((fvorm[0].line, fvorm[0].column), (4, 5));
    }

    #[test]
    fn test_fvorm_no_fire_duplicate_varargout_across_blocks() {
        // Probed: a duplicate varargout in two Repeating blocks is FVDAP, not FVORM (fvorm02).
        let source = "\
function [a, varargout] = f(x)
arguments (Repeating)
    varargout
end
arguments (Repeating)
    varargout
end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVORM").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_fvorm_no_fire_single_varargout() {
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
            filter_by_id(&diags, "FVORM").is_empty(),
            "FVORM should NOT fire for a single varargout declaration"
        );
    }
}
