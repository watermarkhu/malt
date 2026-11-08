use super::*;

impl GoodPracticesEngine {
    /// PFGP: assignment to a GLOBAL or PERSISTENT variable inside a PARFOR loop.
    pub(crate) fn check_pfgp(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("PFGP") || !is_parfor_node(node, source) {
            return Vec::new();
        }

        let globals = collect_global_persistent_vars(root_node_of(node), source);
        if globals.is_empty() {
            return Vec::new();
        }
        let body = match find_child_of_kind(node, "block") {
            Some(b) => b,
            None => return Vec::new(),
        };

        let mut assignments = Vec::new();
        collect_nodes_of_kind(body, "assignment", &mut assignments);
        let mut seen = std::collections::HashSet::new();
        let mut diagnostics = Vec::new();
        for assign in assignments {
            let lhs = match assign.child_by_field_name("left") {
                Some(l) => l,
                None => continue,
            };
            let name = match lhs_base_name(lhs, source) {
                Some(n) => n,
                None => continue,
            };
            if !globals.contains(&name) || !seen.insert(name.clone()) {
                continue;
            }
            let pos = lhs.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "PFGP",
                message: format!(
                    "Avoid assigning to GLOBAL or PERSISTENT variable {name} inside a PARFOR loop."
                ),
                severity: Severity::Warning,
                byte_range: lhs.start_byte()..lhs.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pfgp_fires_on_global_assignment() {
        let source = "global gVar;\nparfor i = 1:10\n    gVar = i;\n    x(i) = gVar;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let parfor = find_child_of_kind(root, "for_statement").unwrap();
        let diags = eng.check_pfgp(parfor, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "PFGP");
        assert!(diags[0].message.contains("gVar"));
    }

    #[test]
    fn test_pfgp_silent_without_globals() {
        let source = "parfor i = 1:10\n    q = i;\n    x(i) = q;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let parfor = find_child_of_kind(root, "for_statement").unwrap();
        assert!(eng.check_pfgp(parfor, source).is_empty());
    }
}
