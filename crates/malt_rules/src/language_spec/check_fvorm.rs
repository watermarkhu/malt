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
        let mut seen_varargout = false;
        for block in blocks {
            for prop in block_properties(block.node) {
                let is_varargout = prop_plain_name(prop, source)
                    .as_deref()
                    .map(|n| n == "varargout")
                    .unwrap_or(false);
                if !is_varargout {
                    continue;
                }
                if seen_varargout {
                    self.push_diag(
                        prop,
                        "FVORM",
                        "Declaring multiple repeating output arguments is not supported.",
                        diagnostics,
                    );
                }
                seen_varargout = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvorm_fires_multiple_varargout() {
        let source = "\
function [a, varargout] = f(x)
    arguments (Output)
        a (1,1) double
        varargout
    end
    arguments (Repeating)
        varargout
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVORM").is_empty(),
            "FVORM should fire when varargout is declared more than once"
        );
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
