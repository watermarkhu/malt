//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVAPN: name-value arguments using the name=value syntax must come at the end.
    pub(crate) fn check_fvapn(
        &self,
        input_props: &[tree_sitter::Node],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVAPN") {
            return;
        }
        for (index, prop) in input_props.iter().enumerate() {
            if !prop_has_name_value(*prop) || !prop_has_default(*prop) {
                continue;
            }
            let later_positional = input_props[index + 1..]
                .iter()
                .any(|p| self.fv_is_positional_prop(*p, source));
            if later_positional {
                self.push_diag(
                    *prop,
                    "FVAPN",
                    "Move name-value arguments that use the name=value syntax to the end of the argument list.",
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
    fn test_fvapn_fires_name_value_before_positional() {
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
            !filter_by_id(&diags, "FVAPN").is_empty(),
            "FVAPN should fire when name=value name-value argument precedes a positional"
        );
    }

    #[test]
    fn test_fvapn_no_fire_name_value_last() {
        let source = "\
function f(opts)
    arguments
        opts.Name = 'x'
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVAPN").is_empty(),
            "FVAPN should NOT fire when name-value arguments are last"
        );
    }
}
