//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MWKREF: WeakHandle and Dependent attributes cannot be combined.
    pub(crate) fn check_mwkref(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MWKREF") {
            return;
        }
        for pb in &class.properties_blocks {
            let is_weak = pb
                .attributes
                .iter()
                .any(|a| a.name == "WeakHandle" && !a.negated);
            let is_dependent = pb
                .attributes
                .iter()
                .any(|a| a.name == "Dependent" && !a.negated);
            if is_weak && is_dependent {
                diagnostics.push(Diagnostic {
                    rule_id: "MWKREF",
                    message: "Specifying both WeakHandle and Dependent attributes is invalid. A dependent property does not store a value."
                        .to_string(),
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
    fn test_mwkref_fires_weakhandle_dependent() {
        let source = "\
classdef Foo
    properties (WeakHandle, Dependent)
        x
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MWKREF");
        assert!(
            !hits.is_empty(),
            "MWKREF should fire for WeakHandle combined with Dependent"
        );
    }

    #[test]
    fn test_mwkref_no_fire() {
        let source = "\
classdef Foo
    properties (WeakHandle)
        x SomeClass
    end
    properties (Dependent)
        y
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MWKREF");
        assert!(
            hits.is_empty(),
            "MWKREF should NOT fire for separate blocks"
        );
    }
}
