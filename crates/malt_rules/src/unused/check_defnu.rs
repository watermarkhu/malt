//! DEFNU check: local function defined but never called.

use super::*;

impl UnusedEngine {
    /// Run DEFNU check: local function defined but never called.
    pub(crate) fn check_defnu(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("DEFNU") {
            return;
        }

        // Collect all uses across all scopes to check if a local function is called.
        let all_uses: HashSet<&str> = table
            .scopes
            .iter()
            .flat_map(|s| s.uses.iter().map(|u| u.name.as_str()))
            .collect();

        for scope in &table.scopes {
            if scope.kind != ScopeKind::LocalFunction {
                continue;
            }
            let name = &scope.name;
            if name.is_empty() {
                continue;
            }
            if self.should_ignore_name(name) {
                continue;
            }
            if !all_uses.contains(name.as_str()) {
                diagnostics.push(Diagnostic {
                    rule_id: "DEFNU",
                    message: "Function might be unused.".to_string(),
                    severity: Severity::Warning,
                    byte_range: scope.byte_range.clone(),
                    line: scope.line,
                    column: 1,
                    fix: None,
                });
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

    // -- DEFNU: local function never called ---------------------------------

    #[test]
    fn defnu_fires_on_uncalled_local_function() {
        let diags = lint_file(
            &*engine(),
            "function main()\n    disp(1);\nend\n\nfunction helper()\n    disp(2);\nend\n",
        );
        assert!(has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_ok_when_local_function_called() {
        let diags = lint_file(
            &*engine(),
            "function main()\n    helper();\nend\n\nfunction helper()\n    disp(2);\nend\n",
        );
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }
}
