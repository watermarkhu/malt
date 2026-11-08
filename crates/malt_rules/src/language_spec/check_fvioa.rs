//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVIOA: Both 'Input' and 'Output' attributes on one block are not supported.
    pub(crate) fn check_fvioa(
        &self,
        blocks: &[ArgumentsBlockMeta],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVIOA") {
            return;
        }
        for block in blocks {
            let has_input = block.attributes.iter().any(|a| a == "Input");
            let has_output = block.attributes.iter().any(|a| a == "Output");
            if has_input && has_output {
                self.push_block_diag(
                    block,
                    "FVIOA",
                    "Specifying both 'Input' and 'Output' attributes on the same arguments block is not supported.",
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
    fn test_fvioa_fires_both_attributes() {
        let source = "\
function [a] = f(a)
    arguments (Input, Output)
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVIOA").is_empty(),
            "FVIOA should fire when a block has both Input and Output attributes"
        );
    }

    #[test]
    fn test_fvioa_no_fire_single_attribute() {
        let source = "\
function f(a)
    arguments (Input)
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVIOA").is_empty(),
            "FVIOA should NOT fire for a single attribute"
        );
    }
}
