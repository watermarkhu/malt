//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCAPP: a private property cannot be Abstract.
    pub(crate) fn check_mcapp(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MCAPP") {
            return;
        }
        for pb in &class.properties_blocks {
            let is_abstract = pb
                .attributes
                .iter()
                .any(|a| a.name == "Abstract" && !a.negated);
            let is_private = pb
                .attributes
                .iter()
                .any(|a| a.name == "Access" && !a.negated && a.value.as_deref() == Some("private"));
            if is_abstract && is_private {
                for prop in &pb.properties {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCAPP",
                        message: "Private property cannot be Abstract.".to_string(),
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

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mcapp_fires_private_abstract_property() {
        let source = "\
classdef Foo
    properties (Access = private, Abstract)
        x
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCAPP");
        assert!(
            !hits.is_empty(),
            "MCAPP should fire for a private abstract property"
        );
    }

    #[test]
    fn test_mcapp_no_fire() {
        let source = "\
classdef Foo
    properties (Access = private)
        x
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCAPP");
        assert!(hits.is_empty(), "MCAPP should NOT fire for private only");
    }
}
