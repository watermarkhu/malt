//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVORDP: Positional arguments must be ordered required, optional, repeating.
    pub(crate) fn check_fvordp(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVORDP") {
            return;
        }
        let mut saw_optional = false;
        let mut saw_repeating = false;
        for block in blocks {
            if !(block.role.is_input() || block.is_input_repeating()) {
                continue;
            }
            let block_repeating = block.is_input_repeating();
            for prop in block_properties(block.node) {
                if prop_has_ignored(prop) || prop_has_name_value(prop) {
                    continue;
                }
                let Some(name) = prop_plain_name(prop, source) else {
                    continue;
                };
                if name == "varargin" || block_repeating {
                    saw_repeating = true;
                    continue;
                }
                let is_optional = prop_has_default(prop);
                if saw_repeating || (!is_optional && saw_optional) {
                    self.push_diag(
                        prop,
                        "FVORDP",
                        "Positional arguments must be defined in the following order: required, optional, and repeating.",
                        diagnostics,
                    );
                }
                if is_optional {
                    saw_optional = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvordp_fires_optional_before_required() {
        let source = "\
function f(a, b)
    arguments
        b = 3
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVORDP").is_empty(),
            "FVORDP should fire when an optional positional precedes a required one"
        );
    }

    #[test]
    fn test_fvordp_no_fire_required_before_optional() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double
        b = 3
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVORDP").is_empty(),
            "FVORDP should NOT fire when required positionals precede optional ones"
        );
    }
}
