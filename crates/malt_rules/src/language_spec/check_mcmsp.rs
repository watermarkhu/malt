//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCMSP: a private method cannot be Abstract.
    pub(crate) fn check_mcmsp(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MCMSP") {
            return;
        }
        for mb in &class.methods_blocks {
            let is_abstract = mb
                .attributes
                .iter()
                .any(|a| a.name == "Abstract" && !a.negated);
            let is_private = mb
                .attributes
                .iter()
                .any(|a| a.name == "Access" && !a.negated && a.value.as_deref() == Some("private"));
            if is_abstract && is_private {
                for method in &mb.methods {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCMSP",
                        message: "Private method cannot be Abstract.".to_string(),
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
    fn test_mcmsp_fires_private_abstract_method() {
        let source = "\
classdef Foo
    methods (Access = private, Abstract)
        function y = f(obj)
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCMSP");
        assert!(
            !hits.is_empty(),
            "MCMSP should fire for a private abstract method"
        );
    }

    #[test]
    fn test_mcmsp_no_fire() {
        let source = "\
classdef Foo
    methods (Access = private)
        function y = f(obj)
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCMSP");
        assert!(hits.is_empty(), "MCMSP should NOT fire for private only");
    }
}
