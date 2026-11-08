//! INUSA/INUSD checks: input arguments that are never used.

use super::*;
use malt_core::Fix;

impl UnusedEngine {
    /// Run INUSA/INUSD checks: input arguments that are never used.
    pub(crate) fn check_unused_inputs(
        &self,
        table: &SymbolTable,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // INUSA is about inputs declared in an `arguments` block, which the
        // symbol table does not track; signature inputs are INUSD's.
        if self.is_check_disabled("INUSD") {
            return;
        }
        let manu_enabled = !self.is_check_disabled("MANU");

        for scope in &table.scopes {
            if !matches!(
                scope.kind,
                ScopeKind::Function
                    | ScopeKind::LocalFunction
                    | ScopeKind::NestedFunction
                    | ScopeKind::Method
            ) {
                continue;
            }
            // MANU reports unused method inputs, with advice that fits methods.
            if scope.kind == ScopeKind::Method && manu_enabled {
                continue;
            }

            for def in &scope.defs {
                if def.kind != DefKind::InputArg {
                    continue;
                }
                let name = &def.name;
                // VANUS owns varargin.
                if name == "varargin" || self.should_ignore_name(name) {
                    continue;
                }

                if !scope.is_used(name) {
                    diagnostics.push(Diagnostic {
                        rule_id: "INUSD",
                        message: format!(
                            "Input argument '{name}' might be unused. Consider replacing the argument with ~ instead."
                        ),
                        severity: Severity::Warning,
                        byte_range: def.byte_range.clone(),
                        line: def.line,
                        column: def.column,
                        // Unsafe: eval/evalin/inputname can still see the name.
                        fix: Some(Fix::new(def.byte_range.clone(), "~").unsafe_fix()),
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

    // -- INUSA / INUSD: unused input arguments ----------------------

    #[test]
    fn signature_inputs_are_inusd_not_inusa() {
        let diags = lint_file(&*engine(), "function foo(x)\nend\n");
        assert!(!has_id(&diags, "INUSA"), "got: {diags:?}");
        assert!(has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn inusa_ok_when_input_used() {
        let diags = lint_file(&*engine(), "function foo(x)\n    disp(x);\nend\n");
        assert!(!has_id(&diags, "INUSA"), "got: {diags:?}");
    }

    #[test]
    fn inusd_fires_on_unused_input() {
        let diags = lint_file(&*engine(), "function foo(x)\nend\n");
        assert!(has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn inusd_ok_when_input_used() {
        let diags = lint_file(&*engine(), "function foo(x)\n    disp(x);\nend\n");
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
    }
}
