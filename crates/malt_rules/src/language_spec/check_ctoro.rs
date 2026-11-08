//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// CTORO: class constructors must declare at least one output argument.
    pub(crate) fn check_ctoro(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        for mb in &class.methods_blocks {
            for func in &mb.methods {
                if func.is_constructor && func.outputs.is_empty() {
                    diagnostics.push(Diagnostic {
                        rule_id: "CTORO",
                        message:
                            "Class constructors must be declared with at least one output argument."
                                .to_string(),
                        severity: Severity::Error,
                        byte_range: func.byte_range.clone(),
                        line: func.line,
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
    fn test_ctoro_fires_constructor_without_output() {
        let source = "\
classdef Bar
    methods
        function Bar(x)
        end
    end
end
";
        let diags = check_source(source, "Bar.m");
        let ctoro = filter_by_id(&diags, "CTORO");
        assert!(
            !ctoro.is_empty(),
            "CTORO should fire for constructor without output"
        );
    }

    #[test]
    fn test_ctoro_no_fire_constructor_with_output() {
        let source = "\
classdef Bar
    methods
        function obj = Bar(x)
        end
    end
end
";
        let diags = check_source(source, "Bar.m");
        let ctoro = filter_by_id(&diags, "CTORO");
        assert!(
            ctoro.is_empty(),
            "CTORO should NOT fire for constructor with output"
        );
    }
}
