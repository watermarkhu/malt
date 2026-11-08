//! ASGLU check: value assigned to a variable that is immediately overwritten.

use super::*;

impl UnusedEngine {
    /// Run ASGLU check: consecutive assignments to the same variable with no
    /// use between them.
    pub(crate) fn check_asglu(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("ASGLU") {
            return;
        }

        for scope in &table.scopes {
            // Group assignment defs by variable name, sorted by byte offset.
            let mut assignments_by_name: HashMap<&str, Vec<usize>> = HashMap::new();
            for (idx, def) in scope.defs.iter().enumerate() {
                if def.kind == DefKind::Assignment {
                    assignments_by_name
                        .entry(def.name.as_str())
                        .or_default()
                        .push(idx);
                }
            }

            for (name, indices) in &assignments_by_name {
                if self.should_ignore_name(name) {
                    continue;
                }
                if indices.len() < 2 {
                    continue;
                }

                // Check consecutive pairs: if no use exists between them,
                // the first assignment is overwritten.
                for pair in indices.windows(2) {
                    let first_def = &scope.defs[pair[0]];
                    let second_def = &scope.defs[pair[1]];

                    let has_use_between = scope.uses.iter().any(|u| {
                        u.name == *name
                            && u.byte_range.start > first_def.byte_range.end
                            && u.byte_range.start < second_def.byte_range.start
                    });

                    if !has_use_between {
                        // Also skip if the first def is an OutputArg (initial
                        // declaration before assignment).
                        let output_args: HashSet<&str> = scope
                            .defs
                            .iter()
                            .filter(|d| d.kind == DefKind::OutputArg)
                            .map(|d| d.name.as_str())
                            .collect();
                        if output_args.contains(name) {
                            continue;
                        }

                        diagnostics.push(Diagnostic {
                            rule_id: "ASGLU",
                            message: "Value assigned to variable might be unused. Consider replacing the variable with ~ instead.".to_string(),
                            severity: Severity::Warning,
                            byte_range: first_def.byte_range.clone(),
                            line: first_def.line,
                            column: first_def.column,
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

    // -- ASGLU: assignment immediately overwritten --------------------------

    #[test]
    fn asglu_fires_on_overwritten_assignment() {
        let diags = lint_file(&*engine(), "function foo()\n    x = 1;\n    x = 2;\nend\n");
        assert!(has_id(&diags, "ASGLU"), "got: {diags:?}");
    }

    #[test]
    fn asglu_ok_when_used_between() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    x = 1;\n    disp(x);\n    x = 2;\nend\n",
        );
        assert!(!has_id(&diags, "ASGLU"), "got: {diags:?}");
    }
}
