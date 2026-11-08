//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MWKCT: WeakHandle and Constant attributes cannot be combined.
    pub(crate) fn check_mwkct(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MWKCT") {
            return;
        }
        for pb in &class.properties_blocks {
            let is_weak = pb
                .attributes
                .iter()
                .any(|a| a.name == "WeakHandle" && !a.negated);
            let is_constant = pb
                .attributes
                .iter()
                .any(|a| a.name == "Constant" && !a.negated);
            if is_weak && is_constant {
                diagnostics.push(Diagnostic {
                    rule_id: "MWKCT",
                    message:
                        "Specifying both WeakHandle and Constant attributes on the same property is not supported."
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
    fn test_mwkct_fires_weakhandle_constant() {
        let source = "\
classdef Foo
    properties (WeakHandle, Constant)
        x = 5
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MWKCT");
        assert!(
            !hits.is_empty(),
            "MWKCT should fire for WeakHandle combined with Constant"
        );
    }

    #[test]
    fn test_mwkct_no_fire() {
        let source = "\
classdef Foo
    properties (WeakHandle)
        x SomeClass
    end
    properties (Constant)
        y = 5
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MWKCT");
        assert!(hits.is_empty(), "MWKCT should NOT fire for separate blocks");
    }
}
