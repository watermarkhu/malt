//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVIDV: Validation or default values on ignored arguments are not supported.
    pub(crate) fn check_fvidv(
        &self,
        blocks: &[ArgumentsBlockMeta],
        _source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVIDV") {
            return;
        }
        for block in blocks {
            for prop in block_properties(block.node) {
                if !prop_has_ignored(prop) {
                    continue;
                }
                let has_spec = prop_has_dimensions(prop)
                    || prop_has_validation(prop)
                    || prop_has_default(prop)
                    || find_child_kind(prop, "identifier").is_some();
                if has_spec {
                    self.push_diag(
                        prop,
                        "FVIDV",
                        "Specifying validation or default value for ignored arguments is not supported.",
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
    fn test_fvidv_fires_validation_on_ignored() {
        let source = "\
function f(a, ~)
    arguments
        a (1,1) double
        ~ (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVIDV").is_empty(),
            "FVIDV should fire for validation on an ignored argument"
        );
    }

    #[test]
    fn test_fvidv_no_fire_bare_ignored() {
        let source = "\
function f(a, ~)
    arguments
        a (1,1) double
        ~
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVIDV").is_empty(),
            "FVIDV should NOT fire for a bare ignored argument"
        );
    }
}
