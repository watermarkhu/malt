//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCPSG: set/get methods must be fully defined in the class definition file.
    pub(crate) fn check_mcpsg(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MCPSG") {
            return;
        }
        for mb in &class.methods_blocks {
            let block_abstract = mb
                .attributes
                .iter()
                .any(|a| a.name == "Abstract" && !a.negated);
            for method in &mb.methods {
                if (method.is_setter || method.is_getter) && (method.is_abstract || block_abstract)
                {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCPSG",
                        message:
                            "Set or get method must be fully defined in the class definition file."
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
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mcpsg_fires_abstract_setter() {
        let source = "\
classdef Foo
    properties
        x
    end
    methods (Abstract)
        function set.x(obj, val)
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCPSG");
        assert!(
            !hits.is_empty(),
            "MCPSG should fire when a setter is declared abstract"
        );
    }

    #[test]
    fn test_mcpsg_no_fire() {
        let source = "\
classdef Foo
    properties
        x
    end
    methods
        function set.x(obj, val)
            obj.x = val;
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCPSG");
        assert!(
            hits.is_empty(),
            "MCPSG should NOT fire for a defined setter"
        );
    }
}
