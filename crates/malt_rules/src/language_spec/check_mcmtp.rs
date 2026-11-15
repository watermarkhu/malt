//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCMTP: a `TestParameterDefinition` methods block of a
    /// `matlab.unittest.TestCase` class must be `Static`, so that MATLAB can call
    /// it when the suite is created.
    ///
    /// MATLAB checks the block attribute. A method merely named
    /// `TestParameterDefinition` gets nothing, in a test class or not (probed).
    pub(crate) fn check_mcmtp(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MCMTP")
            || !class
                .superclasses
                .iter()
                .any(|s| s == "matlab.unittest.TestCase")
        {
            return;
        }
        for mb in &class.methods_blocks {
            let is_test_parameter =
                Attributes(&mb.attributes).is_present("TestParameterDefinition");
            // Static is set by a bare name, `true` or `1`; MATLAB reads other values
            // (`TRUE`, `0`) as not static.
            let is_static = Attributes(&mb.attributes).is_true("Static");
            if is_test_parameter && !is_static {
                diagnostics.push(Diagnostic {
                    rule_id: "MCMTP",
                    message: "TestParameterDefinition methods must be Static, so that they can be called at test suite creation time to set test parameter values.".to_string(),
                    severity: Severity::Error,
                    byte_range: class.byte_range.clone(),
                    line: mb.line,
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
    fn test_mcmtp_fires_non_static_test_parameter_definition_in_test_case() {
        // Probed: MCMTP on the methods block line (`methods (TestParameterDefinition)`).
        let source = "\
classdef Foo < matlab.unittest.TestCase
    methods (TestParameterDefinition)
        function p = pdef(~)
            p = {1, 2};
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCMTP");
        assert_eq!(hits.len(), 1, "got: {diags:?}");
        assert_eq!(hits[0].line, 2);
    }

    #[test]
    fn test_mcmtp_fires_when_static_is_true_uppercase() {
        // Probed: `Static = TRUE` is not evaluated, so the block is not static.
        let source = "\
classdef Foo < matlab.unittest.TestCase
    methods (TestParameterDefinition, Static = TRUE)
        function p = pdef()
            p = {1, 2};
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert_eq!(filter_by_id(&diags, "MCMTP").len(), 1, "got: {diags:?}");
    }

    #[test]
    fn test_mcmtp_no_fire_static() {
        // Probed: the static form is silent.
        let source = "\
classdef Foo < matlab.unittest.TestCase
    methods (TestParameterDefinition, Static)
        function p = pdef()
            p = {1, 2};
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(filter_by_id(&diags, "MCMTP").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_mcmtp_no_fire_method_named_test_parameter_definition() {
        // Probed: a method with that name is silent in a test class and in a plain one.
        let source = "\
classdef Foo < matlab.unittest.TestCase
    methods
        function params = TestParameterDefinition()
            params = {1, 2};
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(filter_by_id(&diags, "MCMTP").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_mcmtp_no_fire_outside_test_case() {
        // Probed: outside a TestCase the attribute is unknown (ATUNK), not MCMTP.
        let source = "\
classdef Foo
    methods (TestParameterDefinition)
        function p = pdef(~)
            p = {1, 2};
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(filter_by_id(&diags, "MCMTP").is_empty(), "got: {diags:?}");
        assert_eq!(filter_by_id(&diags, "ATUNK").len(), 1, "got: {diags:?}");
    }
}
