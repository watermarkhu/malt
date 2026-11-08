//! VUNUS check: variable assigned in all branches but unused after.

use super::*;

impl UnusedEngine {
    /// Run VUNUS check: variable assigned in all branches but unused after.
    pub(crate) fn check_vunus(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("VUNUS") {
            return;
        }

        for scope in &table.scopes {
            let output_args: HashSet<&str> = scope
                .defs
                .iter()
                .filter(|d| d.kind == DefKind::OutputArg)
                .map(|d| d.name.as_str())
                .collect();

            // Find variables with multiple defs but no uses.
            let mut def_counts: HashMap<&str, usize> = HashMap::new();
            for def in &scope.defs {
                if def.kind == DefKind::Assignment {
                    *def_counts.entry(def.name.as_str()).or_insert(0) += 1;
                }
            }

            for (name, count) in &def_counts {
                if *count < 2 {
                    continue;
                }
                if output_args.contains(name) {
                    continue;
                }
                if self.should_ignore_name(name) {
                    continue;
                }
                if !scope.is_used(name) {
                    // Find the last definition for this variable.
                    if let Some(last_def) = scope
                        .defs
                        .iter()
                        .rev()
                        .find(|d| d.name == *name && d.kind == DefKind::Assignment)
                    {
                        diagnostics.push(Diagnostic {
                            rule_id: "VUNUS",
                            message: "VAR_OPERATOR produces a value that might be unused."
                                .to_string(),
                            severity: Severity::Warning,
                            byte_range: last_def.byte_range.clone(),
                            line: last_def.line,
                            column: last_def.column,
                            fix: None,
                        });
                    }
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

    // -- VUNUS: assigned in branches but unused after -----------------------

    #[test]
    fn vunus_fires_on_branch_assignment_unused() {
        let diags = lint_file(
            &*engine(),
            "function foo(flag)\n    if flag\n        x = 1;\n    else\n        x = 2;\n    end\nend\n",
        );
        assert!(has_id(&diags, "VUNUS"), "got: {diags:?}");
    }

    #[test]
    fn vunus_ok_when_used_after_branches() {
        let diags = lint_file(
            &*engine(),
            "function foo(flag)\n    if flag\n        x = 1;\n    else\n        x = 2;\n    end\n    disp(x);\nend\n",
        );
        assert!(!has_id(&diags, "VUNUS"), "got: {diags:?}");
    }
}
