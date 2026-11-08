//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVDNF: Name-value argument can only be declared once.
    pub(crate) fn check_fvdnf(
        &self,
        input_props: &[tree_sitter::Node],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVDNF") {
            return;
        }
        let mut seen: Vec<String> = Vec::new();
        for prop in input_props {
            if !prop_has_name_value(*prop) {
                continue;
            }
            let name = prop_name_value_text(*prop, source);
            if seen.contains(&name) {
                self.push_diag(
                    *prop,
                    "FVDNF",
                    "Name-value argument can only be declared once.",
                    diagnostics,
                );
            } else {
                seen.push(name);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvdnf_fires_duplicate_name_value() {
        let source = "\
function f(opts)
    arguments
        opts.Name = 'a'
        opts.Name = 'b'
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVDNF").is_empty(),
            "FVDNF should fire when a name-value argument is declared twice"
        );
    }

    #[test]
    fn test_fvdnf_no_fire_unique_name_values() {
        let source = "\
function f(opts)
    arguments
        opts.Name1 = 'a'
        opts.Name2 = 'b'
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVDNF").is_empty(),
            "FVDNF should NOT fire for unique name-value arguments"
        );
    }
}
