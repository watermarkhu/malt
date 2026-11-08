//! PREALL check: loop variable preallocated but unused.

use super::*;

impl UnusedEngine {
    /// Run PREALL check: variable preallocated but unused.
    /// Similar to NASGU but targets for-iterator variables specifically.
    pub(crate) fn check_preall(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("PREALL") {
            return;
        }

        for scope in &table.scopes {
            for def in &scope.defs {
                if def.kind != DefKind::ForIterator {
                    continue;
                }
                let name = &def.name;

                if self.should_ignore_name(name) {
                    continue;
                }

                if !scope.is_used(name) {
                    diagnostics.push(Diagnostic {
                        rule_id: "PREALL",
                        message: "The preallocated value assigned to variable might be unused."
                            .to_string(),
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

    // -- PREALL: loop variable preallocated but unused ----------------------

    #[test]
    fn preall_fires_on_unused_loop_var() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    for i = 1:10\n        disp('hi');\n    end\nend\n",
        );
        assert!(has_id(&diags, "PREALL"), "got: {diags:?}");
    }

    #[test]
    fn preall_ok_when_loop_var_used() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    for i = 1:10\n        disp(i);\n    end\nend\n",
        );
        assert!(!has_id(&diags, "PREALL"), "got: {diags:?}");
    }
}
