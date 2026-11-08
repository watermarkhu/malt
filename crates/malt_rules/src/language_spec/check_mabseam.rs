//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MABSEAM: a method cannot be both Abstract and Sealed.
    pub(crate) fn check_mabseam(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MABSEAM") {
            return;
        }
        for mb in &class.methods_blocks {
            let is_abstract = mb
                .attributes
                .iter()
                .any(|a| a.name == "Abstract" && !a.negated);
            let is_sealed = mb
                .attributes
                .iter()
                .any(|a| a.name == "Sealed" && !a.negated);
            if is_abstract && is_sealed {
                for method in &mb.methods {
                    diagnostics.push(Diagnostic {
                        rule_id: "MABSEAM",
                        message: "A method cannot be both Abstract and Sealed.".to_string(),
                        severity: Severity::Error,
                        byte_range: method.byte_range.clone(),
                        line: method.line,
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
    fn test_mabseam_fires_abstract_sealed_method() {
        let source = "\
classdef Foo
    methods (Abstract, Sealed)
        function y = f(obj)
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MABSEAM");
        assert!(
            !hits.is_empty(),
            "MABSEAM should fire for a method that is both Abstract and Sealed"
        );
    }

    #[test]
    fn test_mabseam_no_fire() {
        let source = "\
classdef Foo
    methods (Abstract)
        function y = f(obj)
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MABSEAM");
        assert!(hits.is_empty(), "MABSEAM should NOT fire for Abstract only");
    }
}
