//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCASC: Abstract properties cannot be defined in Sealed classes.
    pub(crate) fn check_mcasc(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if class.is_sealed() {
            for pb in &class.properties_blocks {
                let is_abstract = pb
                    .attributes
                    .iter()
                    .any(|a| a.name == "Abstract" && !a.negated);
                if is_abstract {
                    for prop in &pb.properties {
                        diagnostics.push(Diagnostic {
                            rule_id: "MCASC",
                            message: format!(
                                "Abstract property {} cannot be used in a Sealed class.",
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
    fn test_mcasc_abstract_in_sealed() {
        let source = "\
classdef (Sealed) MyClass
    properties (Abstract)
        Value
    end
end
";
        let diags = check_source(source, "MyClass.m");
        let mcasc = filter_by_id(&diags, "MCASC");
        assert!(
            !mcasc.is_empty(),
            "MCASC should fire when abstract property in sealed class"
        );
    }
}
