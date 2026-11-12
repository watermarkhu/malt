use super::*;

use crate::analysis::control_flow::is_script_body;

impl GoodPracticesEngine {
    /// PFRIN / PFRUS: PARFOR reduction variable data-flow checks.
    ///
    /// A reduction variable is assigned in the loop body as `x = x op ...`. A
    /// variable qualifies only when every assignment to it in the body is such a
    /// reduction with the same operator (probed: a plain `s = 0` in the body, or
    /// `s = s + i` mixed with `s = s * 2`, is silent in MATLAB). MATLAB reports
    /// the variable once, at its first reduction assignment.
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
        let script = is_script_body(tree.root_node());

        for parfor in &parfors {
            let Some(body) = find_child_of_kind(*parfor, "block") else {
                continue;
            };
            if script {
                continue;
            }
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
                    && !has_dynamic_call(body, source)
                    && !used_after_parfor(&sym, &name, *parfor, tree, source)
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

/// Whether a parfor body calls `evalin` or `assignin`, which MATLAB reports as
/// PFEVC; it then gives no PFRUS (c3__p1fix:9-12). `eval`, `who` and `save` do
/// not suppress PFRUS (bugs__t2:3-7).
fn has_dynamic_call(body: Node, source: &str) -> bool {
    let mut stack = vec![body];
    while let Some(node) = stack.pop() {
        if node.kind() == "function_call"
            && node
                .child(0)
                .is_some_and(|name| matches!(node_text(name, source), "evalin" | "assignin"))
        {
            return true;
        }
        let mut cursor = node.walk();
        stack.extend(node.children(&mut cursor));
    }
    false
}

/// Whether the value that the loop leaves in `name` is read after the loop.
///
/// A plain overwrite in the loop's block before any read ends the value
/// (probed: `s = 1; disp(s)` and `s(2) = 1` after the loop are PFRUS). Reads
/// inside a later loop are ignored, as in `uses_after_parfor`, and a string
/// argument such as `save('x.mat', 's')` is not a read (probed: PFRUS).
fn used_after_parfor(
    sym: &SymbolTable,
    name: &str,
    parfor: Node,
    tree: &tree_sitter::Tree,
    source: &str,
) -> bool {
    if overwritten_before_read(parfor, name, source) {
        return false;
    }
    sym.scopes
        .iter()
        .flat_map(|scope| scope.uses.iter())
        .any(|u| {
            u.name == name
                && u.byte_range.start >= parfor.end_byte()
                && tree
                    .root_node()
                    .descendant_for_byte_range(u.byte_range.start, u.byte_range.end)
                    .is_none_or(|n| !is_inside_loop(n) && !inside_string(n))
        })
}

/// Whether the first statement after the loop that mentions `name` is a plain
/// assignment to it that does not read it.
fn overwritten_before_read(parfor: Node, name: &str, source: &str) -> bool {
    let Some(block) = parfor.parent() else {
        return false;
    };
    let mut cursor = block.walk();
    let later: Vec<Node> = block
        .named_children(&mut cursor)
        .filter(|s| s.start_byte() >= parfor.end_byte())
        .collect();
    for statement in later {
        if !mentions(statement, name, source) {
            continue;
        }
        return statement.kind() == "assignment"
            && statement
                .child_by_field_name("left")
                .is_some_and(|lhs| lhs_base_name(lhs, source).as_deref() == Some(name))
            && statement
                .child_by_field_name("right")
                .is_none_or(|rhs| !mentions(rhs, name, source));
    }
    false
}

/// Whether a node contains an identifier named `name`.
fn mentions(node: Node, name: &str, source: &str) -> bool {
    if node.kind() == "identifier" {
        return node_text(node, source) == name;
    }
    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();
    children
        .into_iter()
        .any(|child| mentions(child, name, source))
}

/// Whether a node is part of a string literal.
fn inside_string(node: Node) -> bool {
    std::iter::successors(Some(node), |n| n.parent()).any(|n| n.kind() == "string")
}

/// Collect the reduction variables of a parfor body.
///
/// Returns the variable name and the LHS identifier of its first assignment.
/// A name qualifies when every assignment to it is `name = name op ...` (or
/// `max`/`min`) with the same operator.
fn collect_reduction_assignments<'a>(body: Node<'a>, source: &str) -> Vec<(String, Node<'a>)> {
    let mut assignments = Vec::new();
    collect_nodes_of_kind(body, "assignment", &mut assignments);

    let mut names: Vec<String> = Vec::new();
    for assign in &assignments {
        let Some(name) = assign
            .child_by_field_name("left")
            .and_then(|lhs| lhs_base_name(lhs, source))
        else {
            continue;
        };
        if !names.contains(&name) {
            names.push(name);
        }
    }

    let mut out = Vec::new();
    for name in names {
        let mut operator: Option<String> = None;
        let mut first: Option<Node<'a>> = None;
        let mut reduction = true;
        for assign in &assignments {
            let Some(lhs) = assign.child_by_field_name("left") else {
                continue;
            };
            if lhs_base_name(lhs, source).as_deref() != Some(name.as_str()) {
                continue;
            }
            let op = (lhs.kind() == "identifier")
                .then(|| assign.child_by_field_name("right"))
                .flatten()
                .and_then(|rhs| reduction_operator(rhs, &name, source));
            match op {
                Some(op) if operator.as_ref().is_none_or(|o| *o == op) => {
                    operator = Some(op);
                    if first.is_none_or(|f| lhs.start_byte() < f.start_byte()) {
                        first = Some(lhs);
                    }
                }
                _ => {
                    reduction = false;
                    break;
                }
            }
        }
        if let (true, Some(lhs)) = (reduction, first) {
            out.push((name, lhs));
        }
    }
    out.sort_by_key(|(_, lhs)| lhs.start_byte());
    out
}

/// The operator of a reduction right-hand side `name op ...`, or `max` / `min`
/// for `max(name, ...)`. `None` when the right-hand side does not reduce `name`.
fn reduction_operator(rhs: Node, name: &str, source: &str) -> Option<String> {
    if !expression_uses_variable(rhs, name, source) {
        return None;
    }
    if let Some(f) = get_function_call_name(rhs, source).filter(|f| *f == "max" || *f == "min") {
        return Some(f.to_string());
    }
    let root = unwrap_parentheses(rhs);
    if root.kind() != "binary_operator" {
        return None;
    }
    (0..root.child_count())
        .filter_map(|i| root.child(i))
        .find(|c| !c.is_named())
        .map(|op| node_text(op, source).to_string())
}

/// The expression inside any parentheses around `node`.
fn unwrap_parentheses(mut node: Node) -> Node {
    while node.kind() == "parenthesis" {
        match first_named_child(node) {
            Some(inner) => node = inner,
            None => break,
        }
    }
    node
}

/// Check whether an expression reads the given variable, without descending
/// into nested functions or anonymous functions.
fn expression_uses_variable(node: Node, name: &str, source: &str) -> bool {
    let mut identifiers = Vec::new();
    collect_plain_identifiers(node, &mut identifiers);
    identifiers.iter().any(|id| {
        let indexed = id.parent().is_some_and(|p| {
            p.kind() == "function_call"
                && p.child_by_field_name("name")
                    .is_some_and(|n| n.id() == id.id())
        });
        !indexed && node_text(*id, source) == name
    })
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
        let source = "function f()\nparfor i = 1:10\n    total = total + i;\nend\nend\n";
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
        let source =
            "function f()\ntotal = 0;\nparfor i = 1:10\n    total = total + i;\nend\nend\n";
        let ids = pfr_ids_for(source, "PFRUS");
        assert_eq!(ids.len(), 1, "got: {:?}", pfr_ids(source));
    }

    #[test]
    fn test_pfrus_fires_on_min_max_reduction() {
        let source = "function f()\nu = 0;\nparfor i = 1:10\n    u = max(u, i);\nend\nend\n";
        assert_eq!(pfr_ids_for(source, "PFRUS").len(), 1);
    }

    #[test]
    fn test_pfrus_silent_on_indexed_self_reference() {
        let source = "function f()\nparfor i = 1:10\n    k = k(i) + 1;\nend\nend\n";
        assert!(pfr_ids_for(source, "PFRUS").is_empty());
    }

    #[test]
    fn test_pfrus_fires_on_explicit_step() {
        // Probed: MATLAB reports PFRUS for `1:2:10`, `10:-2:1`, `1:+2:10`, `1:(2):10`
        // and a range continued over lines (q01, q03, q05, q06, q07).
        for range in ["1:2:10", "10:-2:1", "1:+2:10", "1:(2):10"] {
            let source =
                format!("function f()\ns = 0;\nparfor i = {range}\n    s = s + i;\nend\nend\n");
            assert_eq!(pfr_ids_for(&source, "PFRUS").len(), 1, "{range}");
        }
        let source =
            "function f()\ns = 0;\nparfor i = 1:...\n        2:10\n    s = s + i;\nend\nend\n";
        assert_eq!(pfr_ids_for(source, "PFRUS").len(), 1);
    }

    #[test]
    fn test_pfrus_silent_when_body_calls_assignin() {
        // Probed: MATLAB reports PFEVC here and no PFRUS (c3__p1fix:11-12).
        let source = "function f(n)\ns = 0;\nparfor i = 1:n\n    assignin('caller', 'x', 1);\n    s = s + i;\nend\nend\n";
        assert!(pfr_ids_for(source, "PFRUS").is_empty());
    }

    #[test]
    fn test_pfrus_silent_in_script() {
        let source = "total = 0;\nparfor i = 1:10\n    total = total + i;\nend\n";
        assert!(pfr_ids_for(source, "PFRUS").is_empty());
    }

    #[test]
    fn test_pfrus_silent_when_reduction_result_used_after() {
        let source = "total = 0;\nparfor i = 1:10\n    total = total + i;\nend\ndisp(total);\n";
        assert!(pfr_ids_for(source, "PFRUS").is_empty());
    }

    #[test]
    fn test_pfrus_fires_when_overwritten_before_read_after_loop() {
        // Probed: `s = 1;` or `s(2) = 1;` after the loop ends the value (q16, q19).
        for after in ["s = 1;\ndisp(s);\n", "s(2) = 1;\n"] {
            let source =
                format!("function f()\ns = 0;\nparfor i = 1:10\n    s = s + i;\nend\n{after}end\n");
            assert_eq!(pfr_ids_for(&source, "PFRUS").len(), 1, "{after}");
        }
    }

    #[test]
    fn test_pfrus_fires_when_name_is_only_a_string_after_loop() {
        // Probed: `save('x.mat', 's')` is not a read of s (pr28); nor is `save x.mat s` (q09).
        let source = "function f()\ns = 0;\nparfor i = 1:10\n    s = s + i;\nend\nsave('x.mat', 's');\nend\n";
        assert_eq!(pfr_ids_for(source, "PFRUS").len(), 1);
    }

    #[test]
    fn test_pfrus_fires_on_parenthesised_reduction() {
        // Probed: `s = (s + i);` is a reduction (pr06).
        let source = "function f()\ns = 0;\nparfor i = 1:10\n    s = (s + i);\nend\nend\n";
        assert_eq!(pfr_ids_for(source, "PFRUS").len(), 1);
    }

    #[test]
    fn test_pfr_reports_first_reduction_assignment_only() {
        // Probed: MATLAB reports PFRIN and PFRUS once, at the first `s = s + i` (pr10).
        let source = "function f()\nparfor i = 1:10\n    s = s + i;\n    s = s + 1;\nend\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_parfor_reduction(&tree, source);
        assert_eq!(diags.len(), 2, "{diags:?}");
        assert!(diags.iter().all(|d| d.line == 3), "{diags:?}");
    }

    #[test]
    fn test_pfr_silent_when_body_assigns_a_plain_value_first() {
        // Probed: MATLAB reports neither PFRIN nor PFRUS for `s = 0; s = s + i;` (pr19).
        let source = "function f()\nparfor i = 1:10\n    s = 0;\n    s = s + i;\nend\nend\n";
        assert!(pfr_ids(source).is_empty(), "{:?}", pfr_ids(source));
        let source =
            "function f()\ns = 0;\nparfor i = 1:10\n    s = i;\n    s = s + 1;\nend\nend\n";
        assert!(pfr_ids(source).is_empty(), "{:?}", pfr_ids(source));
    }

    #[test]
    fn test_pfr_silent_when_operators_differ() {
        // Probed: `s = s + i; s = s * 2;` is PFINCR in MATLAB, never PFRUS (q15).
        let source =
            "function f()\ns = 0;\nparfor i = 1:10\n    s = s + i;\n    s = s * 2;\nend\nend\n";
        assert!(pfr_ids(source).is_empty(), "{:?}", pfr_ids(source));
    }

    #[test]
    fn test_pfrus_silent_on_plain_read_after_when_not_overwritten() {
        // A later statement that reads `s` is a use, whatever comes after it.
        let source =
            "function f()\ns = 0;\nparfor i = 1:10\n    s = s + i;\nend\nq = s + 1;\nend\n";
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
