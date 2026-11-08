//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVNDE / FVNVL: name-value arguments using a class name cannot have
    /// default values or validation functions.
    pub(crate) fn check_fvnde_fvnvl(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let nde = self.is_check_enabled("FVNDE");
        let nvl = self.is_check_enabled("FVNVL");
        if !nde && !nvl {
            return;
        }
        for block in blocks {
            for prop in block_properties(block.node) {
                if !prop_has_name_value(prop) {
                    continue;
                }
                let uses_class_name = prop_name_value_field(prop, source)
                    .as_deref()
                    .map(is_known_class_name)
                    .unwrap_or(false);
                if !uses_class_name {
                    continue;
                }
                if nde && prop_has_default(prop) {
                    self.push_diag(
                        prop,
                        "FVNDE",
                        "When specifying name-value arguments using a class name, it is illegal to specify default values for the arguments.",
                        diagnostics,
                    );
                }
                if nvl && prop_has_validation(prop) {
                    self.push_diag(
                        prop,
                        "FVNVL",
                        "When specifying name-value arguments using a class name, it is illegal to specify validation for the arguments.",
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
    fn test_fvnde_fires_class_name_default() {
        let source = "\
function f(opts)
    arguments
        opts.double = 3
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVNDE").is_empty(),
            "FVNDE should fire for a default value on a class-name name-value argument"
        );
    }

    #[test]
    fn test_fvnde_no_fire_plain_name_value() {
        let source = "\
function f(opts)
    arguments
        opts.Name = 3
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVNDE").is_empty(),
            "FVNDE should NOT fire for a regular name-value argument"
        );
    }

    #[test]
    fn test_fvnvl_fires_class_name_validation() {
        let source = "\
function f(opts)
    arguments
        opts.double {mustBeReal}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVNVL").is_empty(),
            "FVNVL should fire for validation on a class-name name-value argument"
        );
    }

    #[test]
    fn test_fvnvl_no_fire_plain_name_value_validation() {
        let source = "\
function f(opts)
    arguments
        opts.Name {mustBeReal}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVNVL").is_empty(),
            "FVNVL should NOT fire for a regular name-value argument with validation"
        );
    }
}
