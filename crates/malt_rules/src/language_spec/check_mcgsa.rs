//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCGSA: a set/get method cannot target an abstract property.
    pub(crate) fn check_mcgsa(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MCGSA") {
            return;
        }
        let abstract_props: Vec<&str> = class
            .properties_blocks
            .iter()
            .filter(|pb| {
                pb.attributes
                    .iter()
                    .any(|a| a.name == "Abstract" && !a.negated)
            })
            .flat_map(|pb| pb.properties.iter().map(|p| p.name.as_str()))
            .collect();
        if abstract_props.is_empty() {
            return;
        }
        for mb in &class.methods_blocks {
            for method in &mb.methods {
                if method.is_setter || method.is_getter {
                    let prop_name = method
                        .name
                        .strip_prefix("set.")
                        .or_else(|| method.name.strip_prefix("get."));
                    if let Some(pn) = prop_name {
                        if abstract_props.contains(&pn) {
                            diagnostics.push(Diagnostic {
                                rule_id: "MCGSA",
                                message: format!(
                                    "Method {} tries to set or get an abstract property.",
                                    method.name
                                ),
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
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mcgsa_fires_setter_for_abstract_property() {
        let source = "\
classdef Foo
    properties (Abstract)
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
        let hits = filter_by_id(&diags, "MCGSA");
        assert!(
            !hits.is_empty(),
            "MCGSA should fire when a setter targets an abstract property"
        );
    }

    #[test]
    fn test_mcgsa_no_fire() {
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
        let hits = filter_by_id(&diags, "MCGSA");
        assert!(
            hits.is_empty(),
            "MCGSA should NOT fire for a normal property"
        );
    }
}
