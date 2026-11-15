//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MABSEAM: a method cannot be both Abstract and Sealed.
    pub(crate) fn check_mabseam(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MABSEAM") {
            return;
        }
        for mb in &class.methods_blocks {
            let is_abstract = Attributes(&mb.attributes).is_present("Abstract");
            let is_sealed = Attributes(&mb.attributes).is_present("Sealed");
            if is_abstract && is_sealed {
                for method in &mb.methods {
                    diagnostics.push(Diagnostic {
                        rule_id: "MABSEAM",
                        message: "A method cannot be both Abstract and Sealed.".to_string(),
                        severity: Severity::Error,
                        byte_range: method.byte_range.clone(),
                        line: mb.line,
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

    #[test]
    fn test_mabseam_reports_at_methods_line() {
        let source = "classdef Foo < handle\n    methods (Abstract, Sealed)\n        r = m(obj)\n    end\nend\n";
        let diags = crate::language_spec::tests::check_source(source, "Foo.m");
        let hits = crate::language_spec::tests::filter_by_id(&diags, "MABSEAM");
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|d| d.line == 2));
    }
}
