//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCSGP: Set/get method must refer to a valid property.
    pub(crate) fn check_mcsgp(
        &self,
        _class: &ClassMeta,
        meta: &FileMeta,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for func in &meta.functions {
            if func.is_setter || func.is_getter {
                // Extract property name from "set.PropName" or "get.PropName"
                let prop_name = if func.is_setter {
                    func.name.strip_prefix("set.")
                } else {
                    func.name.strip_prefix("get.")
                };

                if let Some(prop_name) = prop_name {
                    if !meta.has_property(prop_name) {
                        diagnostics.push(Diagnostic {
                            rule_id: "MCSGP",
                            message: format!(
                                "The method {} does not refer to a valid property name.",
                                func.name
                            ),
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
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mcsgp_setter_invalid_property() {
        let source = "\
classdef MyClass
    properties
        Value
    end

    methods
        function set.NonExistent(obj, val)
            obj.NonExistent = val;
        end
    end
end
";
        let diags = check_source(source, "MyClass.m");
        let mcsgp = filter_by_id(&diags, "MCSGP");
        assert!(
            !mcsgp.is_empty(),
            "MCSGP should fire when setter refers to non-existent property"
        );
    }
}
