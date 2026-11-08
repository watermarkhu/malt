//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCMIO: a method has too many inputs or outputs (limit 64 each).
    pub(crate) fn check_mcmio(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MCMIO") {
            return;
        }
        for mb in &class.methods_blocks {
            for method in &mb.methods {
                let n_inputs = method.inputs.iter().filter(|s| s.as_str() != "~").count();
                let n_outputs = method.outputs.iter().filter(|s| s.as_str() != "~").count();
                if n_inputs > 64 || n_outputs > 64 {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCMIO",
                        message: "Method has too many inputs or outputs.".to_string(),
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
    fn test_mcmio_fires_65_outputs() {
        let outputs: Vec<String> = (1..=65).map(|i| format!("a{i}")).collect();
        let source = format!(
            "classdef Foo\n    methods\n        function [{}] = f()\n        end\n    end\nend\n",
            outputs.join(",")
        );
        let diags = check_source(&source, "Foo.m");
        let hits = filter_by_id(&diags, "MCMIO");
        assert!(!hits.is_empty(), "MCMIO should fire for 65 outputs");
    }

    #[test]
    fn test_mcmio_no_fire_64_outputs() {
        let outputs: Vec<String> = (1..=64).map(|i| format!("a{i}")).collect();
        let source = format!(
            "classdef Foo\n    methods\n        function [{}] = f()\n        end\n    end\nend\n",
            outputs.join(",")
        );
        let diags = check_source(&source, "Foo.m");
        let hits = filter_by_id(&diags, "MCMIO");
        assert!(hits.is_empty(), "MCMIO should NOT fire for 64 outputs");
    }
}
