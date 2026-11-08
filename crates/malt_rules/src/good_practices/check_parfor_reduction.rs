use super::*;

impl GoodPracticesEngine {
    /// PFRIN / PFRUS: PARFOR reduction variable data-flow checks.
    ///
    /// A reduction variable is assigned in the loop body as `x = x op ...`.
    /// - PFRIN: the reduction variable is not defined before the loop.
    /// - PFRUS: the reduction variable is never used after the loop.
    pub(crate) fn check_parfor_reduction(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        let any_enabled = self.is_check_enabled("PFRIN") || self.is_check_enabled("PFRUS");
        if !any_enabled {
            return Vec::new();
        }

        let sym = SymbolTable::build(tree, source);
        let mut diagnostics = Vec::new();
        let parfors = collect_parfor_nodes(tree.root_node(), source);

        for parfor in &parfors {
            let Some(body) = find_child_of_kind(*parfor, "block") else {
                continue;
            };
            for (name, lhs_node) in collect_reduction_assignments(body, source) {
                if self.is_check_enabled("PFRIN")
                    && !is_defined_before_parfor(&sym, &name, parfor.start_byte())
                {
                    let pos = lhs_node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "PFRIN",
                        message: format!(
                            "The reduction variable {name} might not be set before the PARFOR loop."
                        ),
                        severity: Severity::Warning,
                        byte_range: lhs_node.start_byte()..lhs_node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
                if self.is_check_enabled("PFRUS")
                    && uses_after_parfor(&sym, &name, parfor.end_byte(), tree, true).is_empty()
                {
                    let pos = lhs_node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "PFRUS",
                        message: format!(
                            "The reduction variable {name} might not be used after the PARFOR loop."
                        ),
                        severity: Severity::Warning,
                        byte_range: lhs_node.start_byte()..lhs_node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

/// Collect reduction assignments (`x = x op ...`) inside a parfor body.
///
/// Returns the variable name and the LHS identifier node of each assignment.
fn collect_reduction_assignments<'a>(node: Node<'a>, source: &str) -> Vec<(String, Node<'a>)> {
    let mut out = Vec::new();
    let mut assignments = Vec::new();
    collect_nodes_of_kind(node, "assignment", &mut assignments);
    for assign in assignments {
        let Some(lhs) = assign.child_by_field_name("left") else {
            continue;
        };
        if lhs.kind() != "identifier" {
            continue;
        }
        let name = node_text(lhs, source);
        let Some(rhs) = assign.child_by_field_name("right") else {
            continue;
        };
        // The classic reduction shape: `x = x op ...` with a binary operator
        // at the top of the right-hand side.
        if rhs_root_is_binary(rhs) && expression_uses_variable(rhs, name, source) {
            out.push((name.to_string(), lhs));
        }
    }
    out
}

/// Check whether a node is a binary operator expression, unwrapping any
/// parentheses around it.
fn rhs_root_is_binary(mut node: Node) -> bool {
    loop {
        match node.kind() {
            "binary_operator" => return true,
            "parenthesized_expression" => {
                let Some(inner) = first_named_child(node) else {
                    return false;
                };
                node = inner;
            }
            _ => return false,
        }
    }
}

/// Check whether an expression reads the given variable, without descending
/// into nested functions or anonymous functions.
fn expression_uses_variable(node: Node, name: &str, source: &str) -> bool {
    let mut identifiers = Vec::new();
    collect_plain_identifiers(node, &mut identifiers);
    identifiers.iter().any(|id| node_text(*id, source) == name)
}

/// Collect identifier descendants, pruning nested functions and lambdas.
fn collect_plain_identifiers<'a>(node: Node<'a>, out: &mut Vec<Node<'a>>) {
    if node.kind() == "function_definition" || node.kind() == "lambda" {
        return;
    }
    if node.kind() == "identifier" {
        out.push(node);
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            collect_plain_identifiers(child, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pfr_ids(source: &str) -> Vec<&'static str> {
        let tree = parse(source);
        let eng = engine();
        eng.check_parfor_reduction(&tree, source)
            .iter()
            .map(|d| d.rule_id)
            .collect()
    }

    fn pfr_ids_for(source: &str, check: &str) -> Vec<&'static str> {
        pfr_ids(source)
            .iter()
            .copied()
            .filter(|id| *id == check)
            .collect()
    }

    #[test]
    fn test_pfrin_fires_on_uninitialized_reduction_variable() {
        let source = "parfor i = 1:10\n    total = total + i;\nend\n";
        let ids = pfr_ids_for(source, "PFRIN");
        assert_eq!(ids.len(), 1, "got: {:?}", pfr_ids(source));
    }

    #[test]
    fn test_pfrin_silent_when_initialized_before() {
        let source = "total = 0;\nparfor i = 1:10\n    total = total + i;\nend\n";
        assert!(pfr_ids_for(source, "PFRIN").is_empty());
    }

    #[test]
    fn test_pfrus_fires_when_reduction_result_unused() {
        let source = "total = 0;\nparfor i = 1:10\n    total = total + i;\nend\n";
        let ids = pfr_ids_for(source, "PFRUS");
        assert_eq!(ids.len(), 1, "got: {:?}", pfr_ids(source));
    }

    #[test]
    fn test_pfrus_silent_when_reduction_result_used_after() {
        let source = "total = 0;\nparfor i = 1:10\n    total = total + i;\nend\ndisp(total);\n";
        assert!(pfr_ids_for(source, "PFRUS").is_empty());
    }

    #[test]
    fn test_pfr_silent_on_non_reduction_assignment() {
        // `q = i` is not a reduction (no `q = q op ...` form).
        let source = "parfor i = 1:10\n    q = i;\nend\n";
        assert!(pfr_ids(source).is_empty());
    }

    #[test]
    fn test_pfr_respects_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec!["PFRIN".to_string(), "PFRUS".to_string()],
            },
        };
        let source = "parfor i = 1:10\n    total = total + i;\nend\n";
        let tree = parse(source);
        assert!(eng.check_parfor_reduction(&tree, source).is_empty());
    }
}
