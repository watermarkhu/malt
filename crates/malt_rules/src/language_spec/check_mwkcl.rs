//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MWKCL: a WeakHandle property must have a class validation (type constraint).
    pub(crate) fn check_mwkcl(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MWKCL") {
            return;
        }
        for pb in &class.properties_blocks {
            let is_weak = pb
                .attributes
                .iter()
                .any(|a| a.name == "WeakHandle" && !a.negated);
            if !is_weak {
                continue;
            }
            for prop in &pb.properties {
                if prop.type_constraint.is_none() {
                    diagnostics.push(Diagnostic {
                        rule_id: "MWKCL",
                        message:
                            "A WeakHandle property must restrict its type using a class validation."
                                .to_string(),
                        severity: Severity::Error,
                        byte_range: prop.byte_range.clone(),
                        line: prop.line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mwkcl_fires_untyped_weakhandle() {
        let source = "\
classdef Foo
    properties (WeakHandle)
        x
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MWKCL");
        assert!(
            !hits.is_empty(),
            "MWKCL should fire for a WeakHandle property without a class validation"
        );
    }

    #[test]
    fn test_mwkcl_no_fire_typed() {
        let source = "\
classdef Foo
    properties (WeakHandle)
        x SomeClass
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MWKCL");
        assert!(
            hits.is_empty(),
            "MWKCL should NOT fire for a typed property"
        );
    }
}
