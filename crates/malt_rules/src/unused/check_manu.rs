//! MANU check: method input argument that is never used.

use super::check_unused_inputs::collect_arguments_blocks;
use super::*;
use crate::analysis::symbols::Scope;
use tree_sitter::Node;

/// Whether a method has exactly one input argument, the case MANU reports.
pub(crate) fn sole_input(scope: &Scope) -> bool {
    scope
        .defs
        .iter()
        .filter(|d| d.kind == DefKind::InputArg)
        .count()
        == 1
}

impl UnusedEngine {
    /// Run MANU check: an input argument of a method that is never used in the
    /// method body. MATLAB suggests replacing it with `~` or making the method
    /// `Static`.
    ///
    /// # Limitations
    ///
    /// MANU is reported only for a method whose sole input is unused (probes M1,
    /// M5 and ls-c__V4). A method with several inputs reports each unused one
    /// as INUSD instead (probes M2, M3, M4 and M7).
    pub(crate) fn check_manu(
        &self,
        table: &SymbolTable,
        root: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if self.is_check_disabled("MANU") {
            return;
        }
        let blocks = collect_arguments_blocks(root, source);

        for scope in &table.scopes {
            if scope.kind != ScopeKind::Method || !sole_input(scope) {
                continue;
            }

            for def in &scope.defs {
                if def.kind != DefKind::InputArg {
                    continue;
                }
                let name = &def.name;

                if self.should_ignore_name(name) {
                    continue;
                }

                // An input declared in the arguments block is INUSA's.
                let declared = blocks
                    .by_function
                    .get(&(scope.byte_range.start, scope.byte_range.end))
                    .is_some_and(|d| d.iter().any(|d| d.name == *name));
                if declared {
                    continue;
                }

                if !scope.is_used(name) {
                    diagnostics.push(Diagnostic {
                        rule_id: "MANU",
                        message: "Input argument might be unused. Consider replacing the argument with ~, or make this method Static instead.".to_string(),
                        severity: Severity::Warning,
                        byte_range: def.byte_range.clone(),
                        line: def.line,
                        column: def.column,
                        fix: None,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    // -- MANU: unused method input argument ----------------------------------

    #[test]
    fn manu_fires_on_unused_sole_method_input() {
        // Probes M1 and M5: the only input of a method, unused.
        let diags = lint_file(
            &*engine(),
            "classdef M1\n    methods\n        function r = m(obj)\n            r = 1;\n        end\n    end\nend\n",
        );
        assert!(has_id(&diags, "MANU"), "got: {diags:?}");
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn manu_silent_with_several_method_inputs_reports_inusd() {
        // Probes M2, M3 and M7: with more than one input, MATLAB reports
        // each unused input as INUSD and no MANU.
        let diags = lint_file(
            &*engine(),
            "classdef M3\n    methods\n        function r = m(obj, a)\n            r = 1;\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "MANU"), "got: {diags:?}");
        let inusd = diags.iter().filter(|d| d.rule_id == "INUSD").count();
        assert_eq!(inusd, 2, "got: {diags:?}");
    }

    #[test]
    fn manu_silent_on_sole_input_declared_in_arguments_block() {
        // Probe t__Cls constructor: a block-declared input is INUSA's, not MANU's.
        let diags = lint_file(
            &*engine(),
            "classdef Cls\n    methods\n        function r = Cls(x)\n            arguments\n                x\n            end\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "MANU"), "got: {diags:?}");
    }

    #[test]
    fn manu_ok_when_method_input_used() {
        let diags = lint_file(
            &*engine(),
            "classdef Foo\n    methods\n        function y = bar(self, x)\n            y = self.v + x;\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "MANU"), "got: {diags:?}");
    }
}
