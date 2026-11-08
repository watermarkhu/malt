//! MANU check: method input argument that is never used.

use super::*;

impl UnusedEngine {
    /// Run MANU check: an input argument of a method that is never used in the
    /// method body. MATLAB suggests replacing it with `~` or making the method
    /// `Static`.
    ///
    /// # Limitations
    ///
    /// MANU overlaps with the INUSD/INUSA checks (`check_unused_inputs`), which
    /// also report unused method inputs. A method with an unused input argument
    /// may therefore emit MANU alongside INUSD/INUSA; this overlap follows the
    /// distinct MathWorks messages verbatim.
    pub(crate) fn check_manu(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("MANU") {
            return;
        }

        for scope in &table.scopes {
            if scope.kind != ScopeKind::Method {
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
    fn manu_fires_on_unused_method_input() {
        let diags = lint_file(
            &*engine(),
            "classdef Foo\n    methods\n        function y = bar(self, x)\n            y = self.v;\n        end\n    end\nend\n",
        );
        assert!(has_id(&diags, "MANU"), "got: {diags:?}");
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
