//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCSWA: a Sealed class cannot specify allowed subclasses.
    pub(crate) fn check_mcswa(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MCSWA") || !class.is_sealed() {
            return;
        }
        if class
            .attributes
            .iter()
            .any(|a| a.name == "AllowedSubclasses" && !a.negated)
        {
            diagnostics.push(Diagnostic {
                rule_id: "MCSWA",
                message: "A sealed class cannot specify allowed subclasses.".to_string(),
                severity: Severity::Error,
                byte_range: class.byte_range.clone(),
                line: class.line,
                column: 1,
                fix: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mcswa_fires_sealed_allowed_subclasses() {
        let source = "\
classdef (Sealed, AllowedSubclasses = ?Bar) Foo
    methods
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSWA");
        assert!(
            !hits.is_empty(),
            "MCSWA should fire for Sealed class with AllowedSubclasses"
        );
    }

    #[test]
    fn test_mcswa_no_fire() {
        let source = "\
classdef (Sealed) Foo
    methods
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSWA");
        assert!(
            hits.is_empty(),
            "MCSWA should NOT fire for a plain Sealed class"
        );
    }
}
