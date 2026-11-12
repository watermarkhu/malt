use super::*;

impl GoodPracticesEngine {
    /// PFGP: assignment to a GLOBAL or PERSISTENT variable inside a PARFOR loop.
    pub(crate) fn check_pfgp(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("PFGP") || !is_parfor_node(node, source) {
            return Vec::new();
        }

        // Declarations of the enclosing function only (probed: another function's
        // `global G` does not make `G = 1` here a GLOBAL write).
        let scope = enclosing_function(node).unwrap_or_else(|| root_node_of(node));
        let globals = collect_global_persistent_vars(scope, source);
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
            let Some(lhs) = assign.child_by_field_name("left") else {
                continue;
            };
            // `[G, H] = ...` assigns each name in the list (probed: PFGP on G and H).
            let targets: Vec<Node> = if lhs.kind() == "multioutput_variable" {
                let mut cursor = lhs.walk();
                lhs.named_children(&mut cursor)
                    .filter(|t| t.kind() == "identifier")
                    .collect()
            } else {
                vec![lhs]
            };
            for target in targets {
                let Some(name) = lhs_base_name(target, source) else {
                    continue;
                };
                if !globals.contains(&name) || !seen.insert(name.clone()) {
                    continue;
                }
                let pos = target.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "PFGP",
                    message: format!(
                        "Avoid assigning to GLOBAL or PERSISTENT variable {name} inside a PARFOR loop."
                    ),
                    severity: Severity::Warning,
                    byte_range: target.start_byte()..target.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pfgp_ignores_another_functions_global() {
        // Probed: `G = 1` in a function without `global G` is not PFGP, even when
        // another function in the file declares `G` global.
        let engine =
            crate::good_practices::GoodPracticesEngine::from_config(&malt_core::Config::default());
        let src = "function f()\nparfor i = 1:2\n  G = 1;\nend\nend\nfunction g()\nglobal G\nend\n";
        let diags = crate::test_util::lint_nodes(&*engine, src);
        assert!(diags.iter().all(|d| d.rule_id != "PFGP"), "{diags:?}");
    }

    #[test]
    fn test_pfgp_fires_on_each_name_of_a_multi_assignment() {
        // Probed: `[G, H] = deal(i, i)` gives PFGP for G and for H (pg10).
        let engine =
            crate::good_practices::GoodPracticesEngine::from_config(&malt_core::Config::default());
        let src = "function f()\nglobal G H\nparfor i = 1:4\n    [G, H] = deal(i, i);\nend\nend\n";
        let diags = crate::test_util::lint_nodes(&*engine, src);
        let names: Vec<&str> = diags
            .iter()
            .filter(|d| d.rule_id == "PFGP")
            .map(|d| d.message.as_str())
            .collect();
        assert_eq!(names.len(), 2, "{diags:?}");
        assert!(names[1].contains("variable H"), "{names:?}");
    }

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
