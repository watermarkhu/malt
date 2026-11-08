use super::*;

impl UnsetVariablesEngine {
    /// Check a switch_statement for partial assignment (PSET).
    pub(crate) fn check_switch_pset(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut branch_assignments: Vec<HashSet<String>> = Vec::new();
        let mut has_otherwise = false;

        let count = node.child_count();
        for i in 0..count {
            let Some(child) = node.child(i) else {
                continue;
            };
            match child.kind() {
                "case_clause" => {
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
                "otherwise_clause" => {
                    has_otherwise = true;
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

        // Only emit PSET if there's an otherwise clause.
        if !has_otherwise || branch_assignments.len() < 2 {
            return;
        }

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
