use super::*;

impl GoodPracticesEngine {
    /// SUBSINDEX / VTFIN / CTOINW / FXUP: miscellaneous general practice checks.
    pub(crate) fn check_misc_general(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        let any_enabled = self.is_check_enabled("SUBSINDEX")
            || self.is_check_enabled("VTFIN")
            || self.is_check_enabled("CTOINW")
            || self.is_check_enabled("FXUP");
        if !any_enabled {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();

        if self.is_check_enabled("SUBSINDEX") {
            collect_subsindex(tree, source, &mut diagnostics);
        }
        if self.is_check_enabled("VTFIN") {
            collect_vtfin(tree, source, &mut diagnostics);
        }
        if self.is_check_enabled("CTOINW") {
            collect_ctoinw(tree, source, &mut diagnostics);
        }
        if self.is_check_enabled("FXUP") {
            collect_fxup(tree, source, &mut diagnostics);
        }

        diagnostics
    }
}

/// SUBSINDEX: a class method named `subsindex` overloads the fundamental data
/// type indexing behavior.
fn collect_subsindex(tree: &tree_sitter::Tree, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut methods_blocks = Vec::new();
    collect_nodes_of_kind(tree.root_node(), "methods", &mut methods_blocks);
    for mb in methods_blocks {
        let mut cursor = mb.walk();
        for child in mb.children(&mut cursor) {
            if child.kind() != "function_definition" {
                continue;
            }
            let name = child
                .child_by_field_name("name")
                .map(|n| node_text(n, source))
                .unwrap_or("");
            if name != "subsindex" {
                continue;
            }
            let pos = child.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "SUBSINDEX",
                message: "Do not overload 'subsindex' for fundamental data types.".to_string(),
                severity: Severity::Warning,
                byte_range: child.start_byte()..child.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
    }
}

/// VTFIN: a `validate*` function whose body validates a value that is not its
/// first input argument.
fn collect_vtfin(tree: &tree_sitter::Tree, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut functions = Vec::new();
    collect_nodes_of_kind(tree.root_node(), "function_definition", &mut functions);
    for f in functions {
        let name = f
            .child_by_field_name("name")
            .map(|n| node_text(n, source))
            .unwrap_or("");
        if !name.starts_with("validate") || name == "validateattributes" {
            continue;
        }
        let Some(first_input) = first_input_arg(f, source) else {
            continue;
        };
        let Some(value) = validated_value(f, source) else {
            continue;
        };
        if value == first_input {
            continue;
        }
        let pos = f.start_position();
        diagnostics.push(Diagnostic {
            rule_id: "VTFIN",
            message: format!("{value} should be the first input argument to the {name} function."),
            severity: Severity::Warning,
            byte_range: f.start_byte()..f.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        });
    }
}

/// CTOINW: `ClassName(ClassName(...))` — passing a constructed object to its
/// own constructor is unnecessary.
fn collect_ctoinw(tree: &tree_sitter::Tree, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut calls = Vec::new();
    collect_nodes_of_kind(tree.root_node(), "function_call", &mut calls);
    for call in calls {
        let Some(name_node) = call.child_by_field_name("name") else {
            continue;
        };
        if name_node.kind() != "identifier" {
            continue;
        }
        let name = node_text(name_node, source);
        let Some(args) = find_child_of_kind(call, "arguments") else {
            continue;
        };
        let Some(first) = first_named_child(args) else {
            continue;
        };
        if first.kind() != "function_call" {
            continue;
        }
        if get_function_call_name(first, source) != Some(name) {
            continue;
        }
        let pos = call.start_position();
        diagnostics.push(Diagnostic {
            rule_id: "CTOINW",
            message: "Use of constructed object as input to constructor is not necessary."
                .to_string(),
            severity: Severity::Warning,
            byte_range: call.start_byte()..call.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        });
    }
}

/// FXUP: a nested function assigns to the index variable of a `for` loop in
/// its enclosing function.
fn collect_fxup(tree: &tree_sitter::Tree, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut functions = Vec::new();
    collect_nodes_of_kind(tree.root_node(), "function_definition", &mut functions);
    let mut seen = std::collections::HashSet::new();
    for f in functions {
        // Only nested functions (enclosed by another function definition).
        let Some(outer) = enclosing_function(f) else {
            continue;
        };
        let loop_indices = collect_outer_loop_indices(outer, source);
        if loop_indices.is_empty() {
            continue;
        }
        let mut assignments = Vec::new();
        collect_fxup_assignments(f, source, &loop_indices, &mut assignments);
        for (name, lhs) in assignments {
            if !seen.insert(lhs.start_byte()) {
                continue;
            }
            let pos = lhs.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "FXUP",
                message: format!("Outer loop variable {name} is set inside a nested function."),
                severity: Severity::Warning,
                byte_range: lhs.start_byte()..lhs.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
    }
}

/// Get the first non-ignored input argument name of a function.
fn first_input_arg(func: Node, source: &str) -> Option<String> {
    let args = find_child_of_kind(func, "function_arguments")?;
    let mut cursor = args.walk();
    for child in args.children(&mut cursor) {
        match child.kind() {
            "identifier" => return Some(node_text(child, source).to_string()),
            "ignored_argument" => continue,
            _ => {}
        }
    }
    None
}

/// Find the value that a `validate*` function validates: the first argument of
/// a `validateattributes(...)` or `mustBe*(...)` call in its body. Returns the
/// value's name only when it is a plain identifier.
fn validated_value(func: Node, source: &str) -> Option<String> {
    let block = find_child_of_kind(func, "block")?;
    let mut calls = Vec::new();
    collect_nodes_of_kind(block, "function_call", &mut calls);
    for call in calls {
        let Some(call_name) = get_function_call_name(call, source) else {
            continue;
        };
        if call_name != "validateattributes" && !call_name.starts_with("mustBe") {
            continue;
        }
        let Some(args) = find_child_of_kind(call, "arguments") else {
            continue;
        };
        let Some(first) = first_named_child(args) else {
            continue;
        };
        if first.kind() != "identifier" {
            return None;
        }
        return Some(node_text(first, source).to_string());
    }
    None
}

/// Collect the index variables of `for` loops in a function's own body,
/// excluding loops inside any nested function.
fn collect_outer_loop_indices(outer: Node, source: &str) -> std::collections::HashSet<String> {
    let mut result = std::collections::HashSet::new();
    collect_loops_pruning_functions(outer, outer, source, &mut result);
    result
}

fn collect_loops_pruning_functions(
    node: Node,
    outer: Node,
    source: &str,
    out: &mut std::collections::HashSet<String>,
) {
    if node.kind() == "function_definition" && node.id() != outer.id() {
        return;
    }
    if node.kind() == "for_statement" {
        if let Some(idx) = find_parfor_index_identifier(node) {
            out.insert(node_text(idx, source).to_string());
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            collect_loops_pruning_functions(child, outer, source, out);
        }
    }
}

/// Collect assignments to any of the given loop index variables within a
/// nested function, skipping assignments shadowed by a loop inside the nested
/// function itself.
fn collect_fxup_assignments<'a>(
    nested: Node<'a>,
    source: &str,
    loop_indices: &std::collections::HashSet<String>,
    out: &mut Vec<(String, Node<'a>)>,
) {
    let mut assignments = Vec::new();
    collect_nodes_of_kind(nested, "assignment", &mut assignments);
    for assign in assignments {
        let Some(lhs) = assign.child_by_field_name("left") else {
            continue;
        };
        if lhs.kind() != "identifier" {
            continue;
        }
        let name = node_text(lhs, source).to_string();
        if !loop_indices.contains(&name) {
            continue;
        }
        if is_shadowed_by_own_loop(assign, &name, source, nested) {
            continue;
        }
        out.push((name, lhs));
    }
}

/// Check whether an assignment is inside a `for` loop (within the nested
/// function) whose index variable shadows the assigned name.
fn is_shadowed_by_own_loop(assign: Node, name: &str, source: &str, boundary: Node) -> bool {
    let mut current = assign.parent();
    while let Some(p) = current {
        if p.id() == boundary.id() {
            break;
        }
        if p.kind() == "for_statement" {
            if let Some(idx) = find_parfor_index_identifier(p) {
                if node_text(idx, source) == name {
                    return true;
                }
            }
        }
        current = p.parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn misc_ids(source: &str) -> Vec<&'static str> {
        let tree = parse(source);
        let eng = engine();
        eng.check_misc_general(&tree, source)
            .iter()
            .map(|d| d.rule_id)
            .collect()
    }

    fn misc_ids_for(source: &str, check: &str) -> Vec<&'static str> {
        misc_ids(source)
            .iter()
            .copied()
            .filter(|id| *id == check)
            .collect()
    }

    #[test]
    fn test_subsindex_fires_on_subsindex_method() {
        let source = "classdef Foo\n    methods\n        function s = subsindex(obj)\n            s = 1;\n        end\n    end\nend\n";
        let ids = misc_ids_for(source, "SUBSINDEX");
        assert_eq!(ids.len(), 1, "got: {:?}", misc_ids(source));
    }

    #[test]
    fn test_subsindex_silent_without_subsindex_method() {
        let source = "classdef Foo\n    methods\n        function s = index(obj)\n            s = 1;\n        end\n    end\nend\n";
        assert!(misc_ids_for(source, "SUBSINDEX").is_empty());
    }

    #[test]
    fn test_vtfin_fires_when_validated_value_not_first_input() {
        let source = "function validateFoo(name, value)\n    validateattributes(value, {'numeric'}, {'scalar'});\nend\n";
        let ids = misc_ids_for(source, "VTFIN");
        assert_eq!(ids.len(), 1, "got: {:?}", misc_ids(source));
        let tree = parse(source);
        let eng = engine();
        let diag = eng
            .check_misc_general(&tree, source)
            .into_iter()
            .find(|d| d.rule_id == "VTFIN")
            .unwrap();
        assert!(diag.message.contains("value"), "got: {}", diag.message);
        assert!(
            diag.message.contains("validateFoo"),
            "got: {}",
            diag.message
        );
    }

    #[test]
    fn test_vtfin_silent_when_validated_value_is_first_input() {
        let source = "function validateFoo(value, name)\n    validateattributes(value, {'numeric'}, {'scalar'});\nend\n";
        assert!(misc_ids_for(source, "VTFIN").is_empty());
    }

    #[test]
    fn test_vtfin_silent_for_non_validator_function() {
        let source = "function helper(name, value)\n    validateattributes(value, {'numeric'}, {'scalar'});\nend\n";
        assert!(misc_ids_for(source, "VTFIN").is_empty());
    }

    #[test]
    fn test_ctoinw_fires_on_constructor_wrapping() {
        let source = "classdef Foo\n    methods\n        function y = ctor(x)\n            y = Foo(Foo(x));\n        end\n    end\nend\n";
        let ids = misc_ids_for(source, "CTOINW");
        assert_eq!(ids.len(), 1, "got: {:?}", misc_ids(source));
    }

    #[test]
    fn test_ctoinw_silent_on_plain_constructor_call() {
        let source = "classdef Foo\n    methods\n        function y = ctor(x)\n            y = Foo(x);\n        end\n    end\nend\n";
        assert!(misc_ids_for(source, "CTOINW").is_empty());
    }

    #[test]
    fn test_fxup_fires_on_outer_loop_index_set_in_nested_function() {
        let source = "function outer()\n    for i = 1:10\n        inner();\n    end\n    function inner()\n        i = 5;\n    end\nend\n";
        let ids = misc_ids_for(source, "FXUP");
        assert_eq!(ids.len(), 1, "got: {:?}", misc_ids(source));
    }

    #[test]
    fn test_fxup_silent_when_nested_function_does_not_touch_loop_index() {
        let source = "function outer()\n    for i = 1:10\n        inner();\n    end\n    function inner()\n        j = 5;\n    end\nend\n";
        assert!(misc_ids_for(source, "FXUP").is_empty());
    }

    #[test]
    fn test_fxup_silent_for_local_function_assigning_own_variable() {
        let source = "function outer()\n    for i = 1:10\n        inner();\n    end\nend\nfunction inner()\n    i = 5;\nend\n";
        assert!(misc_ids_for(source, "FXUP").is_empty());
    }

    #[test]
    fn test_misc_checks_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec![
                    "SUBSINDEX".to_string(),
                    "VTFIN".to_string(),
                    "CTOINW".to_string(),
                    "FXUP".to_string(),
                ],
            },
        };
        let source = "function outer()\n    for i = 1:10\n        inner();\n    end\n    function inner()\n        i = 5;\n    end\nend\nclassdef Foo\n    methods\n        function y = ctor(x)\n            y = Foo(Foo(x));\n        end\n    end\nend\n";
        let tree = parse(source);
        assert!(eng.check_misc_general(&tree, source).is_empty());
    }
}
