//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVOND: Use of name-value arguments in default values is not supported.
    pub(crate) fn check_fvond(
        &self,
        blocks: &[ArgumentsBlockMeta],
        nv_structs: &[String],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVOND") {
            return;
        }
        for block in blocks {
            for prop in block_properties(block.node) {
                let Some(default_node) = find_child_kind(prop, "default_value") else {
                    continue;
                };
                if default_refs_name_value(default_node, source, nv_structs) {
                    self.push_diag(
                        prop,
                        "FVOND",
                        "Use of name-value arguments in default values is not supported.",
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
    fn test_fvond_fires_name_value_in_default() {
        let source = "\
function f(opts, y)
    arguments
        opts.Name = 3
        y (1,1) double = opts.Name
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVOND").is_empty(),
            "FVOND should fire when a default value uses a name-value argument"
        );
    }

    #[test]
    fn test_fvond_no_fire_default_uses_positional() {
        let source = "\
function f(a, y)
    arguments
        a (1,1) double
        y (1,1) double = a + 1
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVOND").is_empty(),
            "FVOND should NOT fire when a default value uses a positional argument"
        );
    }
}
