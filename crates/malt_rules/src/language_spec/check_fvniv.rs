//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVNIV: A declared argument must be an input to the function.
    pub(crate) fn check_fvniv(
        &self,
        func_meta: &FunctionMeta,
        input_props: &[tree_sitter::Node],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVNIV") {
            return;
        }
        for prop in input_props {
            if prop_has_ignored(*prop) {
                continue;
            }
            let declared = if prop_has_name_value(*prop) {
                prop_name_value_struct(*prop, source)
            } else {
                prop_plain_name(*prop, source)
            };
            let Some(name) = declared else {
                continue;
            };
            if name == "varargin" || name == "varargout" {
                continue;
            }
            if !func_meta.inputs.iter().any(|i| i == &name) {
                self.push_diag(
                    *prop,
                    "FVNIV",
                    "This variable is not an input to the function and cannot be used in an arguments block.",
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
    fn test_fvniv_fires_not_an_input() {
        let source = "\
function f(a)
    arguments
        z (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVNIV").is_empty(),
            "FVNIV should fire for an argument that is not a function input"
        );
    }

    #[test]
    fn test_fvniv_no_fire_declared_input() {
        let source = "\
function f(a)
    arguments
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVNIV").is_empty(),
            "FVNIV should NOT fire for a declared input"
        );
    }
}
