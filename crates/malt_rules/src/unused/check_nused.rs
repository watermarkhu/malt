//! NUSED check: global or persistent variable declared but never used.

use super::*;

impl UnusedEngine {
    /// Run NUSED check: global or persistent variables declared but never used.
    ///
    /// MATLAB reports a declaration only when the name is never referenced in
    /// its scope. An assignment counts as a reference: `global g; g = 1;` and
    /// `persistent p; p = 1;` are silent (probes q11, nu1). PUSE may report the
    /// same persistent declaration.
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

                let assigned = scope.defs.iter().any(|d| {
                    d.name == *name && !matches!(d.kind, DefKind::Global | DefKind::Persistent)
                });
                if !scope.is_used(name) && !assigned {
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
    fn nused_silent_when_global_is_assigned() {
        // An assignment counts as a reference (probe q11).
        let diags = lint_file(
            &*engine(),
            "function foo()\n    global g;\n    g = 1;\nend\n",
        );
        assert!(!has_id(&diags, "NUSED"), "got: {diags:?}");
    }

    #[test]
    fn nused_fires_on_unused_global() {
        let diags = lint_file(&*engine(), "function foo()\n    global g;\nend\n");
        assert!(has_id(&diags, "NUSED"), "got: {diags:?}");
    }

    #[test]
    fn nused_fires_on_unused_persistent() {
        let diags = lint_file(&*engine(), "function foo()\n    persistent p;\nend\n");
        // MATLAB reports an unreferenced persistent under NUSED (probe nu5).
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
