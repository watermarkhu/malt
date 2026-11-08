use super::*;

use crate::analysis::symbols::ScopeKind;

impl UnsetVariablesEngine {
    /// Check script scopes for variables used before being set.
    pub(crate) fn check_script_scopes(&self, symbol_table: &SymbolTable) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for scope in &symbol_table.scopes {
            if scope.kind != ScopeKind::Script {
                continue;
            }

            // In scripts, variables must be defined before use within the file.
            // We check for uses that appear before any definition.
            let mut reported: HashSet<String> = HashSet::new();

            for var_use in &scope.uses {
                if self.is_ignored(&var_use.name) {
                    continue;
                }
                if reported.contains(&var_use.name) {
                    continue;
                }

                // Find the earliest definition of this variable in the scope.
                let earliest_def = scope
                    .defs
                    .iter()
                    .filter(|d| d.name == var_use.name)
                    .min_by_key(|d| d.byte_range.start);

                match earliest_def {
                    None => {
                        // Variable is used but never defined in the script → SVNODEF.
                        reported.insert(var_use.name.clone());
                        diagnostics.push(Diagnostic {
                            rule_id: "SVNODEF",
                            message: "Variable might not have a value, because variable definition may not be executed.".to_string(),
                            severity: Severity::Warning,
                            byte_range: var_use.byte_range.clone(),
                            line: var_use.line,
                            column: var_use.column,
                            fix: None,
                        });
                    }
                    Some(def) if var_use.byte_range.start < def.byte_range.start => {
                        // Variable is used before its first definition → SUSENS.
                        reported.insert(var_use.name.clone());
                        diagnostics.push(Diagnostic {
                            rule_id: "SUSENS",
                            message: "Variable is used, but might be unset (within a script)."
                                .to_string(),
                            severity: Severity::Warning,
                            byte_range: var_use.byte_range.clone(),
                            line: var_use.line,
                            column: var_use.column,
                            fix: None,
                        });
                    }
                    _ => {
                        // Variable is used after being defined — OK.
                    }
                }
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::lint_file;
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnsetVariablesEngine::from_config(&Config::default())
    }

    // -- SVNODEF / SUSENS tests (script scope) ------------------------------

    #[test]
    fn svnodef_script_var_never_defined() {
        let source = "y = x + 1;\n";
        let diags = lint_file(&*engine(), source);
        assert!(
            diags.iter().any(|d| d.rule_id == "SVNODEF"),
            "expected SVNODEF for 'x' in script, got: {diags:?}"
        );
    }

    #[test]
    fn susens_script_var_used_before_set() {
        let source = "y = x + 1;\nx = 5;\n";
        let diags = lint_file(&*engine(), source);
        assert!(
            diags.iter().any(|d| d.rule_id == "SUSENS"),
            "expected SUSENS for 'x' used before set, got: {diags:?}"
        );
    }

    #[test]
    fn script_no_warning_when_defined_first() {
        let source = "x = 5;\ny = x + 1;\n";
        let diags = lint_file(&*engine(), source);
        assert!(
            !diags
                .iter()
                .any(|d| d.rule_id == "SVNODEF" || d.rule_id == "SUSENS"),
            "should not warn for 'x' in script when defined first"
        );
    }
}
