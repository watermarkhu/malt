//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVDAP: Positional argument can only be declared once.
    pub(crate) fn check_fvdap(
        &self,
        input_props: &[tree_sitter::Node],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVDAP") {
            return;
        }
        let mut seen: Vec<String> = Vec::new();
        for prop in input_props {
            if prop_has_ignored(*prop) || prop_has_name_value(*prop) {
                continue;
            }
            let Some(name) = prop_plain_name(*prop, source) else {
                continue;
            };
            if seen.contains(&name) {
                self.push_diag(
                    *prop,
                    "FVDAP",
                    "Positional argument can only be declared once.",
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
    fn test_fvdap_fires_duplicate_positional() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double
        a (1,1) double = 2
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVDAP").is_empty(),
            "FVDAP should fire when a positional argument is declared twice"
        );
    }

    #[test]
    fn test_fvdap_no_fire_unique_positionals() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVDAP").is_empty(),
            "FVDAP should NOT fire for unique positional arguments"
        );
    }
}
