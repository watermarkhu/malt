use super::*;
use std::collections::HashSet;

impl GoodPracticesEngine {
    /// FNCOLND: `end` used as column index without explicit dimension specification.
    pub(crate) fn check_fncolnd(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("FNCOLND") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        find_end_as_index(tree.root_node(), &mut diagnostics);
        let mut variables = HashSet::new();
        collect_variable_names(tree.root_node(), source, &mut variables);
        find_end_as_function_argument(
            tree.root_node(),
            source,
            &variables,
            false,
            &mut diagnostics,
        );
        diagnostics
    }
}

/// Names the file uses as variables: assignment targets and function inputs and
/// outputs. `x(end)` on one of these is an index, not a function call.
fn collect_variable_names(node: tree_sitter::Node, source: &str, names: &mut HashSet<String>) {
    match node.kind() {
        "assignment" => {
            if let Some(left) = node.child_by_field_name("left") {
                collect_identifiers(left, source, names);
            }
        }
        "function_arguments" | "function_output" => collect_identifiers(node, source, names),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_variable_names(child, source, names);
    }
}

fn collect_identifiers(node: tree_sitter::Node, source: &str, names: &mut HashSet<String>) {
    if node.kind() == "identifier" {
        names.insert(source[node.start_byte()..node.end_byte()].to_string());
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_identifiers(child, source, names);
    }
}

/// FNCOLND inside a function body: `end` as an argument of a call to a name the
/// file never assigns (`disp(end)`, `numel(end)`). MATLAB reports nothing at
/// script level (probed), so the check is limited to function bodies.
fn find_end_as_function_argument(
    node: tree_sitter::Node,
    source: &str,
    variables: &HashSet<String>,
    in_function: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let in_function = in_function || node.kind() == "function_definition";
    if in_function && node.kind() == "end_keyword" {
        let call = node
            .parent()
            .filter(|p| p.kind() == "arguments")
            .and_then(|p| p.parent())
            .filter(|c| c.kind() == "function_call");
        if call.is_some_and(|c| plain_call(c, source, variables))
            && !inside_index(node, source, variables)
        {
            let pos = node.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "FNCOLND",
                message: "Consider explicitly defining the array, and then using the END operator to index into it.".to_string(),
                severity: Severity::Warning,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_end_as_function_argument(child, source, variables, in_function, diagnostics);
    }
}

/// A call of a bare name that the file does not assign: `numel(...)`, not `data.values(...)`
/// and not `x(...)` for a variable `x`.
fn plain_call(call: tree_sitter::Node, source: &str, variables: &HashSet<String>) -> bool {
    if call.kind() != "function_call" {
        return false;
    }
    let Some(name) = call.child_by_field_name("name") else {
        return false;
    };
    let callee = &source[name.start_byte()..name.end_byte()];
    // `data.values(end)` indexes a field; only a bare name is a function call.
    let is_field = source[..name.start_byte()].trim_end().ends_with('.');
    name.kind() == "identifier" && !is_field && !variables.contains(callee)
}

/// Whether an argument list around `node`, up to its function body, is not a plain call:
/// `y(min(2, end))` and `c{min(2, end)}` index a variable, so MATLAB resolves `end` to that
/// index and reports nothing.
fn inside_index(node: tree_sitter::Node, source: &str, variables: &HashSet<String>) -> bool {
    let mut current = node.parent();
    while let Some(p) = current {
        if p.kind() == "function_definition" {
            break;
        }
        if p.kind() == "arguments" && !p.parent().is_some_and(|c| plain_call(c, source, variables))
        {
            return true;
        }
        current = p.parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::super::GoodPracticesEngine;
    use crate::test_util::{has_id, lint_file};
    use malt_core::{Config, Rule};

    fn engine() -> Box<dyn Rule> {
        GoodPracticesEngine::from_config(&Config::default())
    }

    #[test]
    fn end_as_argument_of_function_in_body_is_fncolnd() {
        // Probed: `r = disp(end);`, `r = numel(end);` and `r = max(end, 1);` (fn01-fn04).
        for source in [
            "function r = f(x)\nr = disp(end);\nend\n",
            "function r = f(x)\nr = numel(end);\nend\n",
            "function r = f(x)\nr = max(end, 1);\nend\n",
            // Probed (fn04, fn07): no enclosing index of a variable.
            "function r = f(y)\nr = foo(min(2, end));\nend\n",
            "function r = f(y)\nr = min(end, 3);\nend\n",
        ] {
            let diags = lint_file(&*engine(), source);
            assert!(has_id(&diags, "FNCOLND"), "{source:?} got: {diags:?}");
        }
    }

    #[test]
    fn end_index_of_variable_is_not_fncolnd() {
        // Probed: `x(end)` on an input or an assigned variable is an index (fn05, fn07).
        for source in [
            "function r = f(x)\nr = x(end);\nend\n",
            "function r = f(x)\nr = foo(x(end));\nend\n",
            "function r = f(x)\nr = num2str(s.values(end));\nend\n",
            "function r = f(x)\ny = zeros(3);\nr = y(end, 1);\nend\n",
            // Probed (fn01, fn02, fn05, fn08, fn09, fn10, fn11, fn14): `end` inside a call
            // nested in a variable index, a field index or a brace index is that index.
            "function r = f(y)\nr = y(min(2, end));\nend\n",
            "function r = f(y)\nz = y(3:end);\nr = z(min(2, end));\nend\n",
            "function r = f(y)\nr = y(foo(end));\nend\n",
            "function r = f(s)\nr = s.data(min(2, end));\nend\n",
            "function r = f(c)\nr = c{min(2, end)};\nend\n",
        ] {
            let diags = lint_file(&*engine(), source);
            assert!(!has_id(&diags, "FNCOLND"), "{source:?} got: {diags:?}");
        }
    }

    #[test]
    fn end_argument_at_script_level_is_not_fncolnd() {
        // MATLAB reports nothing for `disp(end)` in a script (probed).
        let diags = lint_file(&*engine(), "disp(end)\n");
        assert!(!has_id(&diags, "FNCOLND"), "got: {diags:?}");
    }
}
