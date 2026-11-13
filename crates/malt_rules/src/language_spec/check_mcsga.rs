//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCSCN and MCGCP: a set method for a Constant property tries to set it, and a
    /// get method for a Constant property is not supported.
    fn check_constant_accessors(
        &self,
        class: &ClassMeta,
        meta: &FileMeta,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for func in &meta.functions {
            let (is_set, prop) = if func.is_setter {
                (true, func.name.trim_start_matches("set."))
            } else if func.is_getter {
                (false, func.name.trim_start_matches("get."))
            } else {
                continue;
            };
            let is_constant = class.properties_blocks.iter().any(|pb| {
                pb.attributes
                    .iter()
                    .any(|a| a.name == "Constant" && !a.negated)
                    && pb.properties.iter().any(|p| p.name == prop)
            });
            if !is_constant {
                continue;
            }
            if is_set && self.is_check_enabled("MCSCN") {
                diagnostics.push(Diagnostic {
                    rule_id: "MCSCN",
                    message: format!("Method set.{prop} tries to set a constant property."),
                    severity: Severity::Error,
                    byte_range: func.byte_range.clone(),
                    line: func.line,
                    column: 1,
                    fix: None,
                });
            } else if !is_set && self.is_check_enabled("MCGCP") {
                diagnostics.push(Diagnostic {
                    rule_id: "MCGCP",
                    message: "Defining a get method for a constant property is not supported."
                        .to_string(),
                    severity: Severity::Error,
                    byte_range: func.byte_range.clone(),
                    line: func.line,
                    column: 1,
                    fix: None,
                });
            }
        }
    }

    /// MCSGA: Set/get methods should not be in methods blocks with attributes.
    pub(crate) fn check_mcsga(
        &self,
        class: &ClassMeta,
        meta: &FileMeta,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.check_constant_accessors(class, meta, diagnostics);
        for func in &meta.functions {
            if (func.is_setter || func.is_getter) && !func.method_attributes.is_empty() {
                // Check if the methods block has attributes other than Access
                let has_non_trivial_attrs = func.method_attributes.iter().any(|a| {
                    a.name != "Access"
                        || a.value
                            .as_deref()
                            .is_some_and(|v| v != "public" && v != "?")
                });
                if has_non_trivial_attrs {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCSGA",
                        message: "Set or get method must be defined in a METHODS block with no attributes.".to_string(),
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
mod constant_accessor_tests {
    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn mcscn_and_mcgcp_fire_for_constant_property_accessors() {
        let source = "classdef V4\n    properties (Constant = true)\n        k = 1\n    end\n    methods (Abstract)\n        set.k(obj, v)\n        v = get.k(obj)\n    end\nend\n";
        let diags = check_source(source, "V4.m");
        assert!(!filter_by_id(&diags, "MCSCN").is_empty());
        assert!(!filter_by_id(&diags, "MCGCP").is_empty());
    }

    #[test]
    fn mcscn_and_mcgcp_do_not_fire_for_plain_property_accessors() {
        let source = "classdef V4\n    properties\n        k = 1\n    end\n    methods (Abstract)\n        set.k(obj, v)\n        v = get.k(obj)\n    end\nend\n";
        let diags = check_source(source, "V4.m");
        assert!(filter_by_id(&diags, "MCSCN").is_empty());
        assert!(filter_by_id(&diags, "MCGCP").is_empty());
    }

    #[test]
    fn test_mcgcp_reports_at_getter_line() {
        let source = "classdef Foo < handle\n    properties (Constant)\n        K = 1\n    end\n    methods\n        function r = get.K(obj)\n            r = 2;\n        end\n    end\nend\n";
        let diags = crate::language_spec::tests::check_source(source, "Foo.m");
        let hits = crate::language_spec::tests::filter_by_id(&diags, "MCGCP");
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|d| d.line == 6));
    }
}
