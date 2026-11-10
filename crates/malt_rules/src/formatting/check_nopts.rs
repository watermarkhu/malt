//! NOPTS check: statement in a script that produces output without a semicolon.

use super::*;

impl FormattingEngine {
    /// Flag an `assignment` statement at script level that is not terminated by
    /// a semicolon, so it echoes its result to the console.
    pub(crate) fn check_nopts(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let is_statement_level = node
            .parent()
            .map(|p| STATEMENT_PARENTS.contains(&p.kind()))
            .unwrap_or(false);
        if !is_statement_level {
            return;
        }
        // Only scripts (not function bodies).
        if is_in_function(node) {
            return;
        }
        if has_trailing_semicolon(node, source) {
            return;
        }

        let start = node.start_position();
        let end_byte = node.end_byte();
        diagnostics.push(Diagnostic {
            rule_id: "NOPTS",
            message: "Add a semicolon after the statement to hide the output (in a script)."
                .to_string(),
            severity: Severity::Info,
            byte_range: node.start_byte()..node.end_byte(),
            line: start.row + 1,
            column: start.column + 1,
            fix: Some(Fix::insert(end_byte, ";")),
        });
    }
}

/// Whether `node` is an expression statement that MATLAB echoes when it has no
/// semicolon (probed): an operator expression, a single-quoted literal, or a
/// variable or indexing of one (`x`, `x(2)`, `a{1}`). Plain calls such as
/// `disp(x)`, field reads such as `y.a`, and a non-variable name are silent.
pub(crate) fn is_expression_statement(node: Node, source: &str) -> bool {
    let at_statement_level = node
        .parent()
        .is_some_and(|p| STATEMENT_PARENTS.contains(&p.kind()));
    if !at_statement_level {
        return false;
    }
    match node.kind() {
        "comparison_operator" | "binary_operator" | "unary_operator" | "boolean_operator" => true,
        "string" => text_of(node, source).starts_with('\''),
        "identifier" => is_variable_name(node, text_of(node, source), source),
        // A bare word such as `x` parses as a command; it is a variable statement
        // when the word names one (probed: `x` in a function, with `x` an input).
        "command" => {
            let word = text_of(node, source).trim();
            word.chars().all(|c| c.is_alphanumeric() || c == '_')
                && is_variable_name(node, word, source)
        }
        "function_call" => node
            .child_by_field_name("name")
            .filter(|name| name.kind() == "identifier")
            .is_some_and(|name| is_variable_name(node, text_of(name, source), source)),
        _ => false,
    }
}

/// Source text of `node`, empty when it is not valid UTF-8.
fn text_of<'a>(node: Node, source: &'a str) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}

/// Whether `name` is a variable of the scope enclosing `node` (or `ans`).
fn is_variable_name(node: Node, name: &str, source: &str) -> bool {
    if name == "ans" {
        return true;
    }
    let mut scope = node;
    let mut current = node.parent();
    while let Some(parent) = current {
        scope = parent;
        if parent.kind() == "function_definition" {
            break;
        }
        current = parent.parent();
    }
    scope_declares_parameter(scope, name, source)
        || crate::good_practices::is_assigned_in_body(scope, name, source)
}

/// Whether `name` is an input argument of the function `scope`. Probed: a bare
/// `x` statement is an expression statement when `x` is an input argument.
fn scope_declares_parameter(scope: Node, name: &str, source: &str) -> bool {
    if scope.kind() != "function_definition" {
        return false;
    }
    let blocks: Vec<Node> = scope.children(&mut scope.walk()).collect();
    blocks
        .into_iter()
        .filter(|c| c.kind() == "function_arguments")
        .any(|args| {
            let params: Vec<Node> = args.children(&mut args.walk()).collect();
            params
                .into_iter()
                .any(|c| c.kind() == "identifier" && &source[c.start_byte()..c.end_byte()] == name)
        })
}

#[cfg(test)]
mod tests {

    use crate::formatting::tests::{has_id, lint};

    #[test]
    fn noprt_fires_on_bare_input_argument_statement() {
        // Probed: `x` alone in a function, with `x` an input argument, gets NOPRT.
        let diags = lint("function f(x)\n    x\nend\n");
        assert!(has_id(&diags, "NOPRT"), "got: {diags:?}");
    }

    // -- NOPTS ---------------------------------------------------------------

    #[test]
    fn nopts_assignment_in_script_without_semicolon() {
        let source = "x = 5\n";
        let diags = lint(source);
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_ok_with_semicolon() {
        let source = "x = 5;\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_not_fire_in_function() {
        let source = "function f()\n    x = 5\nend\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "NOPTS"), "got: {diags:?}");
    }
}
