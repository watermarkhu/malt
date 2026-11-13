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
                if prop_has_ignored(*p) || prop_has_name_value_src(*p, source) {
                    None
                } else {
                    prop_plain_name(*p, source)
                }
            })
            .collect();
        // Every name-value declaration of a positional name is reported (MATLAB points at each).
        for prop in input_props {
            if !prop_has_name_value_src(*prop, source) {
                continue;
            }
            let Some(struct_name) = prop_name_value_struct_src(*prop, source) else {
                continue;
            };
            if positional_names.contains(&struct_name) {
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
    fn test_fvdan_reports_each_name_value_declaration() {
        // Probed: MATLAB reports both `y.A` and `y.B` (lines 3 and 4) against the positional `y`.
        let source = "\
function f(y)
arguments
    y.A = 1
    y.B = 2
    y (1,1) double
end
end
";
        let diags = check_source(source, "f.m");
        let lines: Vec<usize> = filter_by_id(&diags, "FVDAN")
            .iter()
            .map(|d| d.line)
            .collect();
        assert_eq!(lines, vec![3, 4], "got: {diags:?}");
    }

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
