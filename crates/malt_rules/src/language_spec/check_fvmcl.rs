//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVMCL: `.?ClassName` can only be used for one name-value structure.
    pub(crate) fn check_fvmcl(
        &self,
        blocks: &[ArgumentsBlockMeta],
        _source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVMCL") {
            return;
        }
        let mut seen_class_property = false;
        for block in blocks {
            for prop in block_properties(block.node) {
                if prop.kind() != "class_property" {
                    continue;
                }
                if seen_class_property {
                    self.push_diag(
                        prop,
                        "FVMCL",
                        "Specifying multiple name-value structures using .? syntax and a class name is not supported.",
                        diagnostics,
                    );
                }
                seen_class_property = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvmcl_fires_multiple_class_properties() {
        let source = "\
function f(pa, pb)
    arguments
        pa.?matlab.graphics.chart.primitive.Bar
        pb.?matlab.graphics.chart.primitive.Line
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVMCL").is_empty(),
            "FVMCL should fire when .? syntax is used for multiple name-value structures"
        );
    }

    #[test]
    fn test_fvmcl_no_fire_single_class_property() {
        let source = "\
function f(pa)
    arguments
        pa.?matlab.graphics.chart.primitive.Bar
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVMCL").is_empty(),
            "FVMCL should NOT fire for a single .? declaration"
        );
    }
}
