//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCS2I, MCS1O, MCG1I, MCG1O: Setter/getter signature validation.
    pub(crate) fn check_setter_getter_signatures(
        &self,
        _class: &ClassMeta,
        meta: &FileMeta,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for func in &meta.functions {
            if func.is_setter {
                // MCS2I: Setter must have exactly 2 inputs (obj, value)
                if func.inputs.len() != 2 {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCS2I",
                        message: "Set Methods must have exactly two inputs.".to_string(),
                        severity: Severity::Error,
                        byte_range: func.byte_range.clone(),
                        line: func.line,
                        column: 1,
                        fix: None,
                    });
                }

                // MCS1O: Setter must have at most 1 output (obj)
                if func.outputs.len() > 1 {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCS1O",
                        message: "Set Methods must have at most one output.".to_string(),
                        severity: Severity::Error,
                        byte_range: func.byte_range.clone(),
                        line: func.line,
                        column: 1,
                        fix: None,
                    });
                }
            }

            if func.is_getter {
                // MCG1I: Getter must have exactly 1 input (obj)
                if func.inputs.len() != 1 {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCG1I",
                        message: "Get methods must have exactly one input.".to_string(),
                        severity: Severity::Error,
                        byte_range: func.byte_range.clone(),
                        line: func.line,
                        column: 1,
                        fix: None,
                    });
                }

                // MCG1O: Getter must have exactly 1 output
                if func.outputs.len() != 1 {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCG1O",
                        message: "Get methods must have exactly one output.".to_string(),
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
    fn test_mcs2i_setter_wrong_inputs() {
        let source = "\
classdef MyClass
    properties
        Value
    end

    methods
        function set.Value(obj)
            obj.Value = 0;
        end
    end
end
";
        let diags = check_source(source, "MyClass.m");
        let mcs2i = filter_by_id(&diags, "MCS2I");
        assert!(
            !mcs2i.is_empty(),
            "MCS2I should fire when setter has wrong number of inputs"
        );
    }

    #[test]
    fn test_mcg1i_getter_wrong_inputs() {
        let source = "\
classdef MyClass
    properties
        Value
    end

    methods
        function val = get.Value(obj, extra)
            val = obj.Value;
        end
    end
end
";
        let diags = check_source(source, "MyClass.m");
        let mcg1i = filter_by_id(&diags, "MCG1I");
        assert!(
            !mcg1i.is_empty(),
            "MCG1I should fire when getter has wrong number of inputs"
        );
    }
}
