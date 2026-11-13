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
            // A varargin is positional here too: MATLAB reports `opts.Name` then `varargin`.
            if prop_has_ignored(*prop) || prop_has_name_value_src(*prop, source) {
                continue;
            }
            let has_earlier_name_value = input_props[..index]
                .iter()
                .any(|p| prop_has_name_value_src(*p, source));
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
    fn test_fvordn_fires_varargin_after_name_value() {
        // Probed: a varargin declared after a name-value argument is positional here (line 4).
        let source = "\
function fvordn05(opts, varargin)
arguments
    opts.Name = 1
    varargin
end
end
";
        let diags = check_source(source, "fvordn05.m");
        let fvordn = filter_by_id(&diags, "FVORDN");
        assert_eq!(fvordn.len(), 1, "got: {diags:?}");
        assert_eq!((fvordn[0].line, fvordn[0].column), (4, 5));
    }

    #[test]
    fn test_fvordn_reports_only_the_first_positional() {
        // Probed: MATLAB reports the first positional after a name-value argument only.
        let source = "\
function f(opts, a, b)
arguments
    opts.Name = 1
    a (1,1) double
    b (1,1) double
end
end
";
        let diags = check_source(source, "f.m");
        let fvordn = filter_by_id(&diags, "FVORDN");
        assert_eq!(fvordn.len(), 1, "got: {diags:?}");
        assert_eq!(fvordn[0].line, 4);
    }

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

    #[test]
    fn test_fvordn_fires_class_typed_after_name_value() {
        // Probed: `opts.Name = 1` then `b pkg.Class` reports FVORDN at the class-typed line.
        let source = "\
function f(opts, b)
arguments
    opts.Name = 1
    b pkg.Class
end
";
        let diags = check_source(source, "f.m");
        let fvordn = filter_by_id(&diags, "FVORDN");
        assert_eq!(fvordn.len(), 1, "got: {diags:?}");
        assert_eq!((fvordn[0].line, fvordn[0].column), (4, 5));
    }
}
