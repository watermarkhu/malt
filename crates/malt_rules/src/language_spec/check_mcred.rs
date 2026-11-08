//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCRED: Property/event/enum name same as class name.
    pub(crate) fn check_mcred(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        // Check properties
        for pb in &class.properties_blocks {
            for prop in &pb.properties {
                if prop.name == class.name {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCRED",
                        message: format!("Property, event, or enumeration names must be different from the name of the class {}.", class.name),
                        severity: Severity::Error,
                        byte_range: prop.byte_range.clone(),
                        line: prop.line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }

        // Check events
        for eb in &class.events_blocks {
            for event in &eb.events {
                if *event == class.name {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCRED",
                        message: format!("Property, event, or enumeration names must be different from the name of the class {}.", class.name),
                        severity: Severity::Error,
                        byte_range: class.byte_range.clone(),
                        line: class.line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }

        // Check enumeration members
        for enb in &class.enumeration_blocks {
            for member in &enb.members {
                if *member == class.name {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCRED",
                        message: format!("Property, event, or enumeration names must be different from the name of the class {}.", class.name),
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
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mcred_property_same_as_class() {
        let source = "\
classdef Foo
    properties
        Foo
    end
end
";
        let diags = check_source(source, "Foo.m");
        let mcred = filter_by_id(&diags, "MCRED");
        assert!(
            !mcred.is_empty(),
            "MCRED should fire when property name matches class name"
        );
    }
}
