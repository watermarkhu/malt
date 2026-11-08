//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCMTP: TestParameterDefinition methods must be Static.
    pub(crate) fn check_mcmtp(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MCMTP") {
            return;
        }
        for mb in &class.methods_blocks {
            let is_static = mb
                .attributes
                .iter()
                .any(|a| a.name == "Static" && !a.negated);
            if is_static {
                continue;
            }
            for method in &mb.methods {
                if method.name == "TestParameterDefinition" {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCMTP",
                        message: "TestParameterDefinition methods must be Static, so that they can be called at test suite creation time to set test parameter values.".to_string(),
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
    fn test_mcmtp_fires_non_static_test_parameter_definition() {
        let source = "\
classdef Foo
    methods
        function params = TestParameterDefinition()
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCMTP");
        assert!(
            !hits.is_empty(),
            "MCMTP should fire for a non-Static TestParameterDefinition method"
        );
    }

    #[test]
    fn test_mcmtp_no_fire_static() {
        let source = "\
classdef Foo
    methods (Static)
        function params = TestParameterDefinition()
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCMTP");
        assert!(hits.is_empty(), "MCMTP should NOT fire for a Static method");
    }
}
