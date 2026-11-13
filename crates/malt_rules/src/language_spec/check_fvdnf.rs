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
            // MATLAB compares the part after the first dot, so `opts.Name` and
            // `other.Name` are the same name-value argument (probed).
            let text = prop_name_value_text(*prop, source);
            let name = text
                .split_once('.')
                .map_or(text.as_str(), |(_, field)| field)
                .to_string();
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
    fn test_fvdnf_fires_same_field_in_other_struct() {
        // MATLAB compares the field after the first dot: `other.Name` repeats `opts.Name` (probed).
        let source = "\
function f(opts, other)
    arguments
        opts.Name = 1
        other.Name = 2
    end
end
";
        let diags = check_source(source, "f.m");
        let lines: Vec<usize> = filter_by_id(&diags, "FVDNF")
            .iter()
            .map(|d| d.line)
            .collect();
        assert_eq!(lines, vec![4]);
    }

    #[test]
    fn test_fvdnf_no_fire_different_nested_fields() {
        let source = "\
function f(opts)
    arguments
        opts.A.Name = 1
        opts.B.Name = 2
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVDNF").is_empty(), "got: {diags:?}");
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
