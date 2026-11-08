//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVDAN: Same name as both a name-value structure and a positional argument.
    pub(crate) fn check_fvdan(
        &self,
        input_props: &[tree_sitter::Node],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVDAN") {
            return;
        }
        let positional_names: Vec<String> = input_props
            .iter()
            .filter_map(|p| {
                if prop_has_ignored(*p) || prop_has_name_value(*p) {
                    None
                } else {
                    prop_plain_name(*p, source)
                }
            })
            .collect();
        let mut reported: Vec<String> = Vec::new();
        for prop in input_props {
            if !prop_has_name_value(*prop) {
                continue;
            }
            let Some(struct_name) = prop_name_value_struct(*prop, source) else {
                continue;
            };
            if positional_names.contains(&struct_name) && !reported.contains(&struct_name) {
                reported.push(struct_name);
                self.push_diag(
                    *prop,
                    "FVDAN",
                    "Using the same name as both a name-value argument structure and as a positional argument is not supported.",
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
    fn test_fvdan_fires_shared_name() {
        let source = "\
function f(x, y)
    arguments
        y.Name = 'x'
        y (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVDAN").is_empty(),
            "FVDAN should fire when a name-value structure shares a name with a positional argument"
        );
    }

    #[test]
    fn test_fvdan_no_fire_distinct_names() {
        let source = "\
function f(opts, b)
    arguments
        opts.Name = 'x'
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVDAN").is_empty(),
            "FVDAN should NOT fire for distinct struct and positional names"
        );
    }
}
