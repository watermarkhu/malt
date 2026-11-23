//! PUSE check: persistent variable declared but never used.

use super::*;

impl UnusedEngine {
    /// Run PUSE check: persistent variables declared but not used.
    ///
    /// # Limitations
    ///
    /// PUSE is for a persistent that is written but never read. A declaration
    /// that nothing else touches is NUSED's alone (NUSED also covers globals).
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

                // A declaration that nothing else touches is NUSED's alone (probed).
                let touched = scope
                    .defs
                    .iter()
                    .any(|d| d.kind != DefKind::Persistent && d.name == *name)
                    || scope.uses.iter().any(|u| u.name == *name);

                if touched && !scope.is_used(name) {
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

    fn engine() -> Box<dyn Rule> {
        Box::new(UnusedEngine {
            config: UnusedConfig::default(),
        })
    }

    // -- PUSE: persistent declared but not used ------------------------------

    #[test]
    fn puse_fires_on_persistent_only_written() {
        // Probed: a persistent that is written and never read.
        let diags = lint_file(
            &*engine(),
            "function foo()\n    persistent p;\n    p = 1;\nend\n",
        );
        assert!(has_id(&diags, "PUSE"), "got: {diags:?}");
    }

    #[test]
    fn puse_silent_on_bare_declaration_nused_only() {
        // Probed: MATLAB reports a declaration that nothing else touches as NUSED.
        let diags = lint_file(&*engine(), "function foo()\n    persistent p;\nend\n");
        assert!(!has_id(&diags, "PUSE"), "got: {diags:?}");
        assert!(has_id(&diags, "NUSED"), "got: {diags:?}");
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
