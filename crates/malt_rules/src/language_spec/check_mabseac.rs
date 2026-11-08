//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MABSEAC: instance properties/methods are illegal in Sealed+Abstract classes.
    pub(crate) fn check_mabseac(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !(self.is_check_enabled("MABSEAC") && class.is_sealed() && class.is_abstract()) {
            return;
        }
        for pb in &class.properties_blocks {
            let is_constant = pb
                .attributes
                .iter()
                .any(|a| a.name == "Constant" && !a.negated);
            if is_constant {
                continue;
            }
            for prop in &pb.properties {
                diagnostics.push(Diagnostic {
                    rule_id: "MABSEAC",
                    message:
                        "Instance properties and methods are illegal in classes that are both Sealed and Abstract."
                            .to_string(),
                    severity: Severity::Error,
                    byte_range: prop.byte_range.clone(),
                    line: prop.line,
                    column: 1,
                    fix: None,
                });
            }
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
                diagnostics.push(Diagnostic {
                    rule_id: "MABSEAC",
                    message:
                        "Instance properties and methods are illegal in classes that are both Sealed and Abstract."
                            .to_string(),
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

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mabseac_fires_sealed_abstract_instance_members() {
        let source = "\
classdef (Sealed, Abstract) Foo
    properties
        x
    end
    methods
        function y = f(obj)
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MABSEAC");
        assert!(
            !hits.is_empty(),
            "MABSEAC should fire for instance members in a Sealed+Abstract class"
        );
    }

    #[test]
    fn test_mabseac_no_fire_constant_static() {
        let source = "\
classdef (Sealed, Abstract) Foo
    properties (Constant)
        x = 5
    end
    methods (Static)
        function y = f()
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MABSEAC");
        assert!(
            hits.is_empty(),
            "MABSEAC should NOT fire for Constant properties and Static methods"
        );
    }
}
