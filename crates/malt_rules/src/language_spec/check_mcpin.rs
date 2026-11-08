//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCPIN: a property cannot be initialized to an instance of the class itself.
    pub(crate) fn check_mcpin(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MCPIN") {
            return;
        }
        for pb in &class.properties_blocks {
            for prop in &pb.properties {
                if let Some(ref default_value) = prop.default_value {
                    if contains_self_constructor_call(default_value, &class.name) {
                        diagnostics.push(Diagnostic {
                            rule_id: "MCPIN",
                            message: "Unable to initialize class property to an instance of the class itself.".to_string(),
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
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mcpin_fires_self_instance_default() {
        let source = "\
classdef Foo
    properties
        x Foo = Foo()
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCPIN");
        assert!(
            !hits.is_empty(),
            "MCPIN should fire when a property is initialized to an instance of the class itself"
        );
    }

    #[test]
    fn test_mcpin_no_fire() {
        let source = "\
classdef Foo
    properties
        x double = 5
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCPIN");
        assert!(
            hits.is_empty(),
            "MCPIN should NOT fire for a plain default value"
        );
    }
}
