//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCANI: Abstract property must not have a default value.
    pub(crate) fn check_mcani(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        for pb in &class.properties_blocks {
            let is_abstract = pb
                .attributes
                .iter()
                .any(|a| a.name == "Abstract" && !a.negated);
            if is_abstract {
                for prop in &pb.properties {
                    if prop.has_default {
                        diagnostics.push(Diagnostic {
                            rule_id: "MCANI",
                            message: format!(
                                "Abstract property {} cannot be initialized.",
                                prop.name
                            ),
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
    fn test_mcani_abstract_property_with_default() {
        let source = "\
classdef MyClass
    properties (Abstract)
        Value = 42
    end
end
";
        let diags = check_source(source, "MyClass.m");
        let mcani = filter_by_id(&diags, "MCANI");
        assert!(
            !mcani.is_empty(),
            "MCANI should fire when abstract property has default"
        );
    }
}
