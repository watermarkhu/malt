//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVATF: Attribute values in arguments blocks must be logical constants.
    pub(crate) fn check_fvatf(
        &self,
        blocks: &[ArgumentsBlockMeta],
        _source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVATF") {
            return;
        }
        for block in blocks {
            if attributes_have_value(block.node) {
                self.push_block_diag(
                    block,
                    "FVATF",
                    "Attribute values in arguments blocks must be logical constants.",
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
    fn test_fvatf_fires_attribute_value() {
        let source = "\
function f(a)
    arguments (foo = 1)
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVATF").is_empty(),
            "FVATF should fire for attribute values in arguments blocks"
        );
    }

    #[test]
    fn test_fvatf_no_fire_bare_attributes() {
        let source = "\
function f(a)
    arguments (Input)
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVATF").is_empty(),
            "FVATF should NOT fire for bare attributes"
        );
    }
}
