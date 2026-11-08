//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MHERIT: deriving from certain built-in classes is not supported.
    pub(crate) fn check_mherit(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        for sc in &class.superclasses {
            if NON_SUBCLASSABLE_BUILTINS.contains(&sc.as_str()) {
                diagnostics.push(Diagnostic {
                    rule_id: "MHERIT",
                    message: format!(
                        "Deriving from the built-in MATLAB {sc} class is not supported."
                    ),
                    severity: Severity::Error,
                    byte_range: class.byte_range.clone(),
                    line: class.line,
                    column: 1,
                    fix: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mherit_fires_builtin_superclass() {
        let source = "\
classdef Foo < double
end
";
        let diags = check_source(source, "Foo.m");
        let mherit = filter_by_id(&diags, "MHERIT");
        assert!(
            !mherit.is_empty(),
            "MHERIT should fire for 'classdef Foo < double'"
        );
    }

    #[test]
    fn test_mherit_no_fire_supported_superclass() {
        let source = "\
classdef Foo < handle
end

classdef Bar < MyBase
end
";
        let diags = check_source(source, "Foo.m");
        let mherit = filter_by_id(&diags, "MHERIT");
        assert!(
            mherit.is_empty(),
            "MHERIT should NOT fire for handle or user superclass"
        );
    }
}
