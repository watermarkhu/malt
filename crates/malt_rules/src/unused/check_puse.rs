//! PUSE check: persistent variable declared but never used.

use super::*;

impl UnusedEngine {
    /// Run PUSE check: persistent variables declared but not used.
    ///
    /// # Limitations
    ///
    /// PUSE is the narrower sibling of NUSED (which also covers globals). An
    /// unused `persistent` declaration fires both PUSE and NUSED; this overlap
    /// follows the two MathWorks messages verbatim.
    pub(crate) fn check_puse(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("PUSE") {
            return;
        }

        for scope in &table.scopes {
            for def in &scope.defs {
                if def.kind != DefKind::Persistent {
                    continue;
                }
                let name = &def.name;

                if self.should_ignore_name(name) {
                    continue;
                }

                if !scope.is_used(name) {
                    diagnostics.push(Diagnostic {
                        rule_id: "PUSE",
                        message: "Persistent variable might be unused.".to_string(),
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

    // -- PUSE: persistent declared but not used ------------------------------

    #[test]
    fn puse_fires_on_unused_persistent() {
        let diags = lint_file(&*engine(), "function foo()\n    persistent p;\nend\n");
        assert!(has_id(&diags, "PUSE"), "got: {diags:?}");
    }

    #[test]
    fn puse_ok_when_persistent_used() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    persistent p;\n    disp(p);\nend\n",
        );
        assert!(!has_id(&diags, "PUSE"), "got: {diags:?}");
    }

    #[test]
    fn puse_not_fire_on_unused_global() {
        let diags = lint_file(&*engine(), "function foo()\n    global g;\nend\n");
        assert!(!has_id(&diags, "PUSE"), "got: {diags:?}");
    }
}
