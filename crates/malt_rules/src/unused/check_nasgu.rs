//! NASGU check: variable is assigned but never used.

use super::*;

impl UnusedEngine {
    /// Run NASGU check: variable assigned but never used.
    pub(crate) fn check_nasgu(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("NASGU") {
            return;
        }

        for scope in &table.scopes {
            // Collect output argument names for this scope (they are "used" externally).
            let output_args: HashSet<&str> = scope
                .defs
                .iter()
                .filter(|d| d.kind == DefKind::OutputArg)
                .map(|d| d.name.as_str())
                .collect();

            for def in &scope.defs {
                // Only check assignment defs.
                // Partial assignments (A(i) = v) are SETNU's business.
                if def.kind != DefKind::Assignment || def.partial || def.multi_output {
                    continue;
                }
                let name = &def.name;

                // Skip if this variable is an output arg (returned externally).
                if output_args.contains(name.as_str()) {
                    continue;
                }
                // Skip ignored names.
                if self.should_ignore_name(name) {
                    continue;
                }
                // Check if the variable has any uses in scope.
                if !scope.is_used(name) {
                    diagnostics.push(Diagnostic {
                        rule_id: "NASGU",
                        message: "Value assigned to variable might be unused.".to_string(),
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

    // -- NASGU: assigned but never used -------------------------------------

    #[test]
    fn nasgu_fires_on_unused_assignment() {
        let diags = lint_file(&*engine(), "function foo()\n    x = 1;\nend\n");
        assert!(has_id(&diags, "NASGU"), "got: {diags:?}");
    }

    #[test]
    fn nasgu_ok_when_used() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    x = 1;\n    disp(x);\nend\n",
        );
        assert!(!has_id(&diags, "NASGU"), "got: {diags:?}");
    }
}
