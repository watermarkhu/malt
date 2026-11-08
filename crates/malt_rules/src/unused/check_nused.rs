//! NUSED check: global or persistent variable declared but never used.

use super::*;

impl UnusedEngine {
    /// Run NUSED check: global or persistent variables declared but never used.
    ///
    /// # Limitations
    ///
    /// NUSED covers both global AND persistent variables, per the MathWorks
    /// message. An unused `persistent` variable therefore also matches the
    /// narrower PUSE check ("Persistent variable might be unused") and will be
    /// reported by both. This overlap is intentional — it follows the two
    /// messages verbatim — but means a single unused persistent declaration can
    /// emit two diagnostics.
    pub(crate) fn check_nused(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("NUSED") {
            return;
        }

        for scope in &table.scopes {
            for def in &scope.defs {
                if !matches!(def.kind, DefKind::Global | DefKind::Persistent) {
                    continue;
                }
                let name = &def.name;

                if self.should_ignore_name(name) {
                    continue;
                }

                if !scope.is_used(name) {
                    diagnostics.push(Diagnostic {
                        rule_id: "NUSED",
                        message: "Global or persistent variable might be unused or unset in this function or script.".to_string(),
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

    // -- NUSED: global/persistent declared but never used --------------------

    #[test]
    fn nused_fires_on_unused_global() {
        let diags = lint_file(&*engine(), "function foo()\n    global g;\nend\n");
        assert!(has_id(&diags, "NUSED"), "got: {diags:?}");
    }

    #[test]
    fn nused_fires_on_unused_persistent() {
        let diags = lint_file(&*engine(), "function foo()\n    persistent p;\nend\n");
        assert!(has_id(&diags, "NUSED"), "got: {diags:?}");
    }

    #[test]
    fn nused_ok_when_global_used() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    global g;\n    disp(g);\nend\n",
        );
        assert!(!has_id(&diags, "NUSED"), "got: {diags:?}");
    }

    #[test]
    fn nused_ok_when_persistent_used() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    persistent p;\n    disp(p);\nend\n",
        );
        assert!(!has_id(&diags, "NUSED"), "got: {diags:?}");
    }
}
