//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVORDN: Positional arguments must be defined before name-value arguments.
    pub(crate) fn check_fvordn(
        &self,
        input_props: &[tree_sitter::Node],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVORDN") {
            return;
        }
        for (index, prop) in input_props.iter().enumerate() {
            if !self.fv_is_positional_prop(*prop, source) {
                continue;
            }
            let has_earlier_name_value =
                input_props[..index].iter().any(|p| prop_has_name_value(*p));
            if has_earlier_name_value {
                self.push_diag(
                    *prop,
                    "FVORDN",
                    "Positional arguments must be defined before name-value arguments.",
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
    fn test_fvordn_fires_positional_after_name_value() {
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
            !filter_by_id(&diags, "FVORDN").is_empty(),
            "FVORDN should fire when a positional argument follows name-value arguments"
        );
    }

    #[test]
    fn test_fvordn_no_fire_positional_before_name_value() {
        let source = "\
function f(b, opts)
    arguments
        b (1,1) double
        opts.Name = 'x'
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVORDN").is_empty(),
            "FVORDN should NOT fire when positional arguments precede name-value arguments"
        );
    }
}
