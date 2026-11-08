use super::*;

impl UnsetVariablesEngine {
    /// Check an if_statement for partial assignment (PSET).
    pub(crate) fn check_if_pset(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut branch_assignments: Vec<HashSet<String>> = Vec::new();
        let mut has_else = false;

        let count = node.child_count();
        for i in 0..count {
            let Some(child) = node.child(i) else {
                continue;
            };
            match child.kind() {
                "block" => {
                    // The main if-body block.
                    branch_assignments.push(collect_block_assignments(child, source));
                }
                "elseif_clause" => {
                    let inner_count = child.child_count();
                    for j in 0..inner_count {
                        if let Some(inner) = child.child(j) {
                            if inner.kind() == "block" {
                                branch_assignments.push(collect_block_assignments(inner, source));
                                break;
                            }
                        }
                    }
                }
                "else_clause" => {
                    has_else = true;
                    let inner_count = child.child_count();
                    for j in 0..inner_count {
                        if let Some(inner) = child.child(j) {
                            if inner.kind() == "block" {
                                branch_assignments.push(collect_block_assignments(inner, source));
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Only emit PSET if there's an else branch (complete coverage).
        if !has_else || branch_assignments.len() < 2 {
            return;
        }

        // Find variables assigned in some branches but not all.
        let all_vars: HashSet<String> = branch_assignments
            .iter()
            .flat_map(|s| s.iter().cloned())
            .collect();

        for var_name in &all_vars {
            if self.is_ignored(var_name) {
                continue;
            }
            let assigned_in_all = branch_assignments.iter().all(|s| s.contains(var_name));
            if !assigned_in_all {
                // Find the first branch that assigns this variable for location info.
                if let Some(loc) = self.find_assignment_location(node, source, var_name) {
                    diagnostics.push(Diagnostic {
                        rule_id: "PSET",
                        message: "Persistent variable is used, but might be unset.".to_string(),
                        severity: Severity::Warning,
                        byte_range: loc.0,
                        line: loc.1,
                        column: loc.2,
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
    use crate::test_util::lint_file;
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnsetVariablesEngine::from_config(&Config::default())
    }

    // -- PSET tests ---------------------------------------------------------

    #[test]
    fn pset_variable_set_in_some_branches() {
        let source = "\
function foo(x)
    if x > 0
        z = 1;
    else
        w = 2;
    end
end
";
        let diags = lint_file(&*engine(), source);
        assert!(
            diags.iter().any(|d| d.rule_id == "PSET"),
            "expected PSET for 'z' (set in if but not else), got: {diags:?}"
        );
    }

    #[test]
    fn pset_not_fired_when_set_in_all_branches() {
        let source = "\
function foo(x)
    if x > 0
        z = 1;
    else
        z = 2;
    end
end
";
        let diags = lint_file(&*engine(), source);
        assert!(
            !diags.iter().any(|d| d.rule_id == "PSET"),
            "should not fire PSET when 'z' is set in all branches"
        );
    }
}
