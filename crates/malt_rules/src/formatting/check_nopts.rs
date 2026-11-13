//! NOPTS check: statement in a script that produces output without a semicolon.

use super::*;

impl FormattingEngine {
    /// Flag an assignment or expression statement at script level that is not
    /// terminated by a semicolon, so it echoes its result to the console.
    pub(crate) fn check_nopts(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        // An assignment that a line break splits before `-x`/`+x` is reported by NOPRC
        // instead (probed: MATLAB gives NOPRC on `x = 1` and NOPTS on `-x` only).
        if node.kind() == "assignment" && crate::bugs::noprc_follows(node, source) {
            return;
        }
        let is_statement_level = node
            .parent()
            .map(|p| STATEMENT_PARENTS.contains(&p.kind()))
            .unwrap_or(false);
        if !is_statement_level {
            return;
        }
        // Only scripts (not function bodies), and not parfor bodies (probed: silent).
        if is_in_function(node) || is_in_parfor(node) {
            return;
        }
        if has_trailing_semicolon(node, source) {
            return;
        }

        let start = report_position(node, source);
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
/// semicolon (probed): an operator expression, a range, a literal (number,
/// string, matrix, cell), a handle or lambda, or a variable or indexing of one.
/// Plain calls such as `disp(x)`, field reads such as `y.a`, `pi` and `true`
/// are silent.
pub(crate) fn is_expression_statement(node: Node, source: &str) -> bool {
    let at_statement_level = node
        .parent()
        .is_some_and(|p| STATEMENT_PARENTS.contains(&p.kind()));
    if !at_statement_level || is_in_parfor(node) {
        return false;
    }
    match node.kind() {
        // Probed: `~x` is a `not_operator` in this grammar; `x:2` is a range.
        "comparison_operator"
        | "binary_operator"
        | "unary_operator"
        | "boolean_operator"
        | "not_operator"
        | "range" => true,
        // Probed: a number literal (`2`), a transposed value (`x'`), a string
        // (either quote), a matrix, a cell, a handle and a lambda echo too.
        "number" | "postfix_operator" | "string" | "matrix" | "cell" | "lambda"
        | "handle_operator" => true,
        // Probed: a bare name is a variable when it is assigned anywhere in the
        // scope (`y` before `y = 1;` echoes); `ans` only when it is assigned.
        "identifier" => is_variable_name(node, text_of(node, source), source, true),
        // A bare word such as `x` parses as a command; it is a variable statement
        // when the word names one (probed: `x` in a function, with `x` an input).
        "command" => {
            let word = text_of(node, source).trim();
            word.chars().all(|c| c.is_alphanumeric() || c == '_')
                && is_variable_name(node, word, source, true)
        }
        // Probed: `f(1)` after `f = @(v) v;` or `f = @sin;` is silent (a call of a
        // function handle); after `f = 3;` it echoes like indexing. Indexing is
        // only a variable once assigned (`f(1)` before `f = 2;` is silent).
        "function_call" => node
            .child_by_field_name("name")
            .filter(|name| name.kind() == "identifier")
            .is_some_and(|name| {
                let name_text = text_of(name, source);
                // Probed: a parenthesised index of an input argument, or of a local
                // copied from one, is silent; a brace index still echoes.
                let brace = source[name.end_byte()..].starts_with('{');
                if !brace && is_parameter_alias(node, name_text, source) {
                    return false;
                }
                is_variable_name(node, name_text, source, false)
                    && !is_handle_variable(node, name_text, source)
            }),
        _ => false,
    }
}

/// Where MATLAB points at an expression statement (probed): an assignment at
/// its `=`, a binary, comparison, boolean or range expression at its operator,
/// a postfix operator at its `'`, an indexing at its bracket, anything else at
/// its start.
pub(crate) fn report_position(node: Node, source: &str) -> tree_sitter::Point {
    let byte = match node.kind() {
        "assignment" => (0..node.child_count())
            .filter_map(|i| node.child(i))
            .find(|c| c.kind() == "=")
            .map(|op| op.start_byte()),
        "binary_operator" | "comparison_operator" | "boolean_operator" | "range" => node
            .child(0)
            .and_then(|left| next_token_byte(source, left.end_byte())),
        "postfix_operator" => node
            .child(0)
            .and_then(|operand| next_token_byte(source, operand.end_byte())),
        "function_call" => node.child_by_field_name("name").map(|n| n.end_byte()),
        _ => None,
    };
    point_at(source, byte.unwrap_or(node.start_byte()))
}

/// Byte offset of the first non-blank character at or after `from`.
fn next_token_byte(source: &str, from: usize) -> Option<usize> {
    source[from..]
        .find(|c: char| !matches!(c, ' ' | '\t'))
        .map(|offset| from + offset)
}

/// Row and column (zero-based, byte columns) of a byte offset.
fn point_at(source: &str, byte: usize) -> tree_sitter::Point {
    let line_start = line_start_byte(source, byte);
    tree_sitter::Point {
        row: source[..line_start].matches('\n').count(),
        column: byte - line_start,
    }
}

/// Whether `name` holds a function handle or an anonymous function at `node`:
/// its last whole assignment before `node` has a handle or lambda right-hand side.
fn is_handle_variable(node: Node, name: &str, source: &str) -> bool {
    let scope = scope_of(node);
    crate::good_practices::collect_simple_lhs_assignments(scope, source)
        .iter()
        .filter(|(n, lhs)| n == name && lhs.end_byte() <= node.start_byte())
        .map(|(_, lhs)| *lhs)
        .max_by_key(|lhs| lhs.end_byte())
        .and_then(|lhs| lhs.parent())
        .and_then(|assignment| assignment.child_by_field_name("right"))
        .is_some_and(|rhs| matches!(rhs.kind(), "lambda" | "handle_operator" | "function_handle"))
}

/// The function or script scope that encloses `node`.
fn scope_of(node: Node) -> Node {
    let mut scope = node;
    let mut current = node.parent();
    while let Some(parent) = current {
        scope = parent;
        if parent.kind() == "function_definition" {
            break;
        }
        current = parent.parent();
    }
    scope
}

/// Source text of `node`, empty when it is not valid UTF-8.
fn text_of<'a>(node: Node, source: &'a str) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}

/// Whether `name` is a variable of the scope enclosing `node`: an input argument,
/// or assigned before `node`. With `anywhere`, an assignment after `node` counts
/// too (bare names are flow-insensitive in MATLAB: probed `y` before `y = 1;`
/// echoes, and `ans` echoes only when it is assigned somewhere).
fn is_variable_name(node: Node, name: &str, source: &str, anywhere: bool) -> bool {
    let scope = scope_of(node);
    let assigned_by = |lhs: &Node| anywhere || lhs.end_byte() <= node.start_byte();
    // Probed: `r(i,j) = v` in a loop makes `r` a variable, so `r` on its own
    // echoes (NOPTS).
    scope_declares_parameter(scope, name, source)
        || crate::good_practices::collect_simple_lhs_assignments(scope, source)
            .iter()
            .any(|(n, lhs)| n == name && assigned_by(lhs))
        || crate::good_practices::collect_indexed_lhs_assignments(scope, source)
            .iter()
            .any(|(n, lhs)| n == name && assigned_by(lhs))
        || crate::good_practices::collect_field_lhs_assignments(scope, source)
            .iter()
            .any(|(n, lhs)| n == name && assigned_by(lhs))
}

/// Whether a parenthesised index of `name` at `node` reads as a call because the
/// value is unknown: an input argument, a local copied from one (`c = v; c(2)`),
/// or a local started as an empty matrix (`y = []; y(1) = 3; y(1)`). Probed:
/// MATLAB is silent on each of these; `x = 2; x(1)` and `c{1}` still echo.
fn is_parameter_alias(node: Node, name: &str, source: &str) -> bool {
    let scope = scope_of(node);
    if scope_declares_parameter(scope, name, source) {
        return true;
    }
    crate::good_practices::collect_simple_lhs_assignments(scope, source)
        .iter()
        .filter(|(n, lhs)| n == name && lhs.end_byte() <= node.start_byte())
        .map(|(_, lhs)| *lhs)
        .max_by_key(|lhs| lhs.end_byte())
        .and_then(|lhs| lhs.parent())
        .and_then(|assignment| assignment.child_by_field_name("right"))
        .is_some_and(|rhs| match rhs.kind() {
            "identifier" => scope_declares_parameter(scope, text_of(rhs, source), source),
            "matrix" => rhs.named_child_count() == 0,
            _ => false,
        })
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

    // Probe c4_B n1 (k_oneliners.m:35): `st` is defined by the field store, so the
    // bare `st` after `;` echoes (NOPTS). Probe n4: a bare field read is silent.
    #[test]
    fn nopts_bare_variable_defined_by_field_store() {
        let diags = lint("st.name='demo'; st.value=42; st");
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_silent_on_bare_field_read() {
        let diags = lint("st.name='demo'; st.value=42; st.name");
        assert!(!has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_assignment_in_script_without_semicolon() {
        let source = "x = 5\n";
        let diags = lint(source);
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_fires_on_number_and_transpose_statements() {
        // Probed: MATLAB reports NOPTS for `2` (incomplete__e2:1) and for `x'` (np_tr).
        let diags = lint("x = 1;\n2\n");
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
        let diags = lint("x = 1;\nx'\n");
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

    #[test]
    fn nopts_fires_on_indexing_of_variable_assigned_before() {
        // Probed in MATLAB: `f = 2;` then `f(1)` in a script is reported (np n3).
        let diags = lint("f = 2;\nf(1)\n");
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_silent_on_indexing_of_variable_assigned_later() {
        // Probed in MATLAB: `f(1)` before `f = 2;` is silent (ls-c__scr1:8).
        for source in ["f(1)\nf = 2;\n", "if true\n  f(1)\nend\nf = 2;\n"] {
            let diags = lint(source);
            assert!(!has_id(&diags, "NOPTS"), "{source}: got: {diags:?}");
        }
    }

    // Probed: `r(i,j) = i` in a loop makes `r` a variable, so `r` on its own
    // echoes in a script (NOPTS on line 6).
    #[test]
    fn nopts_fires_on_bare_name_created_by_indexed_assignment() {
        let source = "for i=1:2\n for j=1:2\n  r(i,j)=i;\n end\nend\nr\n";
        let diags = lint(source);
        assert!(
            diags.iter().any(|d| d.rule_id == "NOPTS" && d.line == 6),
            "got: {diags:?}"
        );
    }

    #[test]
    fn nopts_silent_on_call_of_function_handle_variable() {
        // Probed in MATLAB: `f(1)` after `f = @(v) v;` or `f = @sin;` is silent.
        for source in [
            "f = @(v) v;\nf(1)\n",
            "g = @sin;\ng(1)\n",
            "f = @(v) v;\nf(1, 2)\n",
        ] {
            let diags = lint(source);
            assert!(!has_id(&diags, "NOPTS"), "{source}: got: {diags:?}");
        }
    }

    #[test]
    fn nopts_fires_on_call_of_number_variable() {
        // Probed in MATLAB: `f = 3; f(1)` echoes in a function (NOPRT) as in a script.
        let diags = lint("f = 3;\nf(1)\n");
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
        let diags = lint("function g()\nf = 3;\nf(1)\nend\n");
        assert!(has_id(&diags, "NOPRT"), "got: {diags:?}");
    }

    #[test]
    fn nopts_and_noprt_point_at_the_assignment_operator() {
        // Probed in MATLAB: the column is the `=` (`x(2) = 3` at 6, `s.a = 1` at 5,
        // `[a, b] = size(1)` at 8, `x=1` at 2).
        let column = |source: &str, id: &str| {
            lint(source)
                .iter()
                .find(|d| d.rule_id == id)
                .map(|d| d.column)
        };
        assert_eq!(column("x(2) = 3\n", "NOPTS"), Some(6));
        assert_eq!(column("s.a = 1\n", "NOPTS"), Some(5));
        assert_eq!(column("[a, b] = size(1)\n", "NOPTS"), Some(8));
        assert_eq!(column("x=1\n", "NOPTS"), Some(2));
        assert_eq!(column("function f()\nx(2) = 3\nend\n", "NOPRT"), Some(6));
    }

    #[test]
    fn nopts_silent_on_plain_call_of_indexed_variable() {
        let source = "for i=1:2\n r(i)=i;\nend\ndisp(r)\n";
        let diags = lint(source);
        assert!(
            !diags.iter().any(|d| d.rule_id == "NOPTS"),
            "got: {diags:?}"
        );
    }

    // -- Probe-backed expression statements (p25) -----------------------------

    #[test]
    fn nopts_fires_on_double_quoted_string_and_literals() {
        // Probed: `"dq"` (line 2), `[1 2]`, `{1}`, `@sin`, `@(v) v` echo in a script.
        for source in [
            "x = 1;\n\"dq\"\n",
            "x = 1;\n[1 2]\n",
            "x = 1;\n{1}\n",
            "x = 1;\n@sin\n",
            "x = 1;\n@(v) v\n",
        ] {
            let diags = lint(source);
            assert!(has_id(&diags, "NOPTS"), "{source:?}: got: {diags:?}");
        }
    }

    #[test]
    fn nopts_fires_on_negation_and_range_statements() {
        // Probed: `~x` (`not_operator`), `~true`, `~false`, `x:2` echo; `!x` is silent.
        let diags = lint("x = 1;\n~x\n");
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
        let diags = lint("x = 1;\nx:2\n");
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
        let diags = lint("x = 1;\n!x\n");
        assert!(!has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_silent_on_bare_constant_names() {
        // Probed: `pi` and `true` are not variables, so they are silent.
        for source in ["pi\n", "true\n"] {
            let diags = lint(source);
            assert!(!has_id(&diags, "NOPTS"), "{source:?}: got: {diags:?}");
        }
    }

    #[test]
    fn nopts_bare_name_echoes_when_assigned_later() {
        // Probed: `y` before `y = 1;` echoes (flow-insensitive for bare names);
        // `ans` echoes only when it is assigned somewhere in the script.
        let diags = lint("y\ny = 1;\n");
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
        let diags = lint("ans\nans = 5;\n");
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_silent_on_ans_never_assigned() {
        // Probed: `2+2;` then `ans` is silent (`ans` is not a variable here).
        let diags = lint("x = 1;\n2+2;\nans\n");
        assert!(!has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_silent_inside_parfor_body() {
        // Probed: a parfor body gives no output, so its statements are silent.
        let diags = lint("parfor i=1:2\n  q = i\nend\nx = 1\n");
        assert!(
            !diags.iter().any(|d| d.rule_id == "NOPTS" && d.line == 2),
            "got: {diags:?}"
        );
        assert!(
            has_id(&diags, "NOPTS"),
            "the statement after the loop still echoes: {diags:?}"
        );
    }

    #[test]
    fn nopts_column_of_operator_bracket_and_quote() {
        // Probed: `x == 1` at the `==` (column 3), `x(1) + 1` at the `+` (6),
        // `x'` at the quote (2), `x(1)` at the `(` (2), `x:2` at the `:` (2),
        // `x(1)'` at the quote (5).
        let column = |source: &str| {
            lint(source)
                .iter()
                .find(|d| d.rule_id == "NOPTS")
                .map(|d| d.column)
        };
        assert_eq!(column("x = 1;\nx == 1\n"), Some(3));
        assert_eq!(column("x = 1;\nx(1) + 1\n"), Some(6));
        assert_eq!(column("x = 1;\nx'\n"), Some(2));
        assert_eq!(column("x = 1;\nx(1)\n"), Some(2));
        assert_eq!(column("x = 1;\nx:2\n"), Some(2));
        assert_eq!(column("x = 1;\nx(1)'\n"), Some(5));
    }

    #[test]
    fn nopts_silent_on_indexing_of_empty_or_copied_value() {
        // Probed: `y = []; y(1) = 3; y(1)` is silent, as is `y(end)` after growth;
        // `x = 2; x(1) = 3; x(1)` echoes, and so does `c{1}` after `c = {}`.
        let diags = lint("y = [];\ny(1) = 3;\ny(1)\n");
        assert!(!has_id(&diags, "NOPTS"), "got: {diags:?}");
        let diags = lint("y = [];\ny(end+1) = 3;\ny(end)\n");
        assert!(!has_id(&diags, "NOPTS"), "got: {diags:?}");
        let diags = lint("x = 2;\nx(1) = 3;\nx(1)\n");
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
        let diags = lint("c = {};\nc{1} = 2;\nc{1}\n");
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_silent_on_plain_call_then_bare_constant() {
        // Probed: a call of an unknown name (`f(1)` with no `f` variable) is silent.
        let diags = lint("disp(1)\nnumel([1 2])\n");
        assert!(!has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_silent_on_indexing_of_input_argument_in_function() {
        // Probed: `v(1)`, `x(end)` on an input are silent; `v{1}` and `v'` echo (NOPRT).
        let diags = lint("function f(v)\nv(1)\nend\n");
        assert!(!has_id(&diags, "NOPRT"), "got: {diags:?}");
        let diags = lint("function f(v)\nc = v;\nc(2)\nend\n");
        assert!(!has_id(&diags, "NOPRT"), "got: {diags:?}");
        let diags = lint("function f(v)\nv{1}\nend\n");
        assert!(has_id(&diags, "NOPRT"), "got: {diags:?}");
    }
}
