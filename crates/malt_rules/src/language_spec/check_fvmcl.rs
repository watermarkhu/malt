//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVMCL: `.?ClassName` can only be used for one name-value structure.
    pub(crate) fn check_fvmcl(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVMCL") {
            return;
        }
        let mut seen_class_property = false;
        for block in blocks {
            // Output blocks give FVOON in MATLAB, not FVMCL (probed).
            if block.attributes.iter().any(|a| a == "Output") {
                continue;
            }
            // A declaration with a default or validator may parse as an error node, so
            // the `.?` form is read from the text of every property and error node.
            let mut props = block_properties(block.node);
            let mut cursor = block.node.walk();
            props.extend(block.node.children(&mut cursor).filter(|c| c.is_error()));
            props.sort_by_key(|p| p.start_byte());
            // The grammar can give one declaration two overlapping nodes; count it once.
            let mut covered_until = 0;
            for prop in props {
                if prop.start_byte() < covered_until || !declares_class_name(prop, source) {
                    continue;
                }
                covered_until = prop.end_byte();
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

/// Whether a property declares a class name with `struct.?Class`.
fn declares_class_name(prop: tree_sitter::Node, source: &str) -> bool {
    node_text(prop, source)
        .split_once('.')
        .is_some_and(|(head, rest)| {
            let head = head.trim();
            !head.is_empty()
                && head.chars().all(|c| c.is_alphanumeric() || c == '_')
                && rest.trim_start().starts_with('?')
        })
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvmcl_fires_second_class_property_after_default() {
        // MATLAB reports the second `.?` declaration when the first has a default (probed).
        let source = "\
function f(opts, o2)
    arguments
        opts.?double {mustBeReal} = 3
        o2.?single
    end
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "FVMCL").len(), 1, "got: {diags:?}");
    }

    #[test]
    fn test_fvmcl_silent_output_block() {
        // MATLAB reports FVOON for class names in an output block, not FVMCL (probed).
        let source = "\
function y = f(x)
    arguments (Output)
        y.?double
        x.?single
    end
    y = x;
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVMCL").is_empty(), "got: {diags:?}");
    }

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
