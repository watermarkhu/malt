//! NOPTS check: statement in a script that produces output without a semicolon.

use std::collections::HashMap;

use super::*;

impl FormattingEngine {
    /// Flag an assignment or expression statement at script level that is not
    /// terminated by a semicolon, so it echoes its result to the console.
    pub(crate) fn check_nopts(
        &self,
        node: Node,
        frame: &Frame,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !frame.statement_level {
            return;
        }
        // Only scripts (not function bodies), and not parfor bodies (probed: silent).
        if frame.in_function || frame.in_parfor {
            return;
        }
        if semicolon_follows(node, frame.following.iter().copied(), source)
            && !crate::syntax_errors::has_end_value_in_literal(node, source)
        {
            return;
        }
        // An assignment that a line break splits before `-x`/`+x` is reported by NOPRC
        // instead (probed: MATLAB gives NOPRC on `x = 1` and NOPTS on `-x` only).
        if node.kind() == "assignment" && noprc_follows_in(node, frame, source) {
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
            fix: Some(semicolon_fix(end_byte, source)),
        });
    }
}

/// The MATLAB-parity fix of NOPTS and NOPRT: add a `;` where the Code Analyzer's
/// own fix adds it (probed on the fixtures: MATLAB's `fix` on
/// `x = 1, y = 2` produces `x = 1; y = 2;`, replacing the separating comma).
/// Inserting in front of the comma instead produces `x = 1;, y = 2;`, which
/// parses and runs but makes the Code Analyzer report NOCOMMA, so the comma is
/// replaced. With no comma behind the statement, the `;` is inserted at its
/// end, before any trailing blanks and comment.
pub(crate) fn semicolon_fix(end_byte: usize, source: &str) -> Fix {
    let line = source[end_byte..].split('\n').next().unwrap_or("");
    if let Some(offset) = line.find(|c: char| !matches!(c, ' ' | '\t')) {
        if line.as_bytes()[offset] == b',' {
            return Fix::new(end_byte + offset..end_byte + offset + 1, ";");
        }
    }
    Fix::insert(end_byte, ";")
}

/// Whether `node` is an expression statement that MATLAB echoes when it has no
/// semicolon (probed): an operator expression, a range, a literal (number,
/// string, matrix, cell), a handle or lambda, or a variable or indexing of one.
/// Plain calls such as `disp(x)`, field reads such as `y.a`, `pi` and `true`
/// are silent.
pub(crate) fn is_expression_statement(
    node: Node,
    frame: &Frame,
    names: &mut ScopeCache,
    source: &str,
) -> bool {
    if !frame.statement_level || frame.in_parfor {
        return false;
    }
    let names = names
        .entry(frame.scope.id())
        .or_insert_with(|| ScopeNames::of(frame.scope, source));
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
        "identifier" => names.is_variable(node, text_of(node, source), true),
        // A bare word such as `x` parses as a command; it is a variable statement
        // when the word names one (probed: `x` in a function, with `x` an input).
        // The word must be the whole command: no arguments, no continuation.
        "command" => {
            crate::analysis::command::Command::from_node(node, source).is_some_and(|cmd| {
                let word = text_of(node, source).trim();
                word == cmd.name
                    && word.chars().all(|c| c.is_alphanumeric() || c == '_')
                    && names.is_variable(node, word, true)
            })
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
                if !brace && names.is_parameter_alias(node, name_text) {
                    return false;
                }
                names.is_variable(node, name_text, false)
                    && !names.is_handle_variable(node, name_text)
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
    let byte = byte.unwrap_or(node.start_byte());
    if byte < node.start_byte() {
        return point_at(source, byte);
    }
    // Count rows from the node, not from the start of the file.
    let line_start = line_start_byte(source, byte);
    tree_sitter::Point {
        row: node.start_position().row + source[node.start_byte()..byte].matches('\n').count(),
        column: byte - line_start,
    }
}

/// Whether NOPRC reports `node` instead ([`crate::bugs::noprc_follows_among`]),
/// with the siblings the walk already knows.
pub(crate) fn noprc_follows_in(node: Node, frame: &Frame, source: &str) -> bool {
    let following = frame.following.iter().copied().filter(|s| s.is_named());
    crate::bugs::noprc_follows_among(node, following, source)
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

/// The scope id of each [`ScopeNames`] built so far in one walk.
pub(crate) type ScopeCache = HashMap<usize, ScopeNames>;

/// The variables of one scope (a function, or the file outside functions),
/// gathered once for the bare-name statements of NOPTS and NOPRT, which would
/// otherwise each rescan the scope.
pub(crate) struct ScopeNames {
    /// The input parameters, when the scope is a function.
    params: Vec<String>,
    /// Name to the earliest end byte of a whole (`v = ...`), indexed
    /// (`v(i) = ...`) or field (`v.f = ...`) assignment target `v`.
    first_assigned: HashMap<String, usize>,
    /// Name to its whole assignments `v = ...`, as (end byte of `v`, the
    /// right-hand side is a handle or lambda, the right-hand side aliases a
    /// parameter: an input argument or `[]`).
    plain: HashMap<String, Vec<(usize, bool, bool)>>,
}

impl ScopeNames {
    /// Gather the names of `scope`, a `function_definition` or the root.
    fn of(scope: Node, source: &str) -> Self {
        let params = scope_parameters(scope, source);
        let mut first_assigned: HashMap<String, usize> = HashMap::new();
        let mut plain: HashMap<String, Vec<(usize, bool, bool)>> = HashMap::new();
        let mut assignments = Vec::new();
        crate::good_practices::collect_nodes_of_kind(scope, "assignment", &mut assignments);
        for assignment in assignments {
            let Some(lhs) = assignment.child_by_field_name("left") else {
                continue;
            };
            let target = match lhs.kind() {
                "identifier" => Some(lhs),
                "function_call" | "field_expression" => lhs
                    .named_child(0)
                    .filter(|base| base.kind() == "identifier"),
                _ => None,
            };
            let Some(target) = target else {
                continue;
            };
            let name = text_of(target, source);
            let end = target.end_byte();
            first_assigned
                .entry(name.to_string())
                .and_modify(|first| *first = (*first).min(end))
                .or_insert(end);
            if lhs.kind() == "identifier" {
                let rhs = assignment.child_by_field_name("right");
                let handle = rhs.is_some_and(|rhs| {
                    matches!(rhs.kind(), "lambda" | "handle_operator" | "function_handle")
                });
                let alias = rhs.is_some_and(|rhs| match rhs.kind() {
                    "identifier" => params.iter().any(|p| p == text_of(rhs, source)),
                    "matrix" => rhs.named_child_count() == 0,
                    _ => false,
                });
                plain
                    .entry(name.to_string())
                    .or_default()
                    .push((end, handle, alias));
            }
        }
        Self {
            params,
            first_assigned,
            plain,
        }
    }

    /// Whether `name` is a variable of the scope at `node`: an input argument,
    /// or assigned before `node`. With `anywhere`, an assignment after `node`
    /// counts too (bare names are flow-insensitive in MATLAB: probed `y` before
    /// `y = 1;` echoes, and `ans` echoes only when it is assigned somewhere).
    /// Probed: `r(i,j) = v` in a loop makes `r` a variable, so `r` on its own
    /// echoes (NOPTS).
    fn is_variable(&self, node: Node, name: &str, anywhere: bool) -> bool {
        self.params.iter().any(|p| p == name)
            || self
                .first_assigned
                .get(name)
                .is_some_and(|&end| anywhere || end <= node.start_byte())
    }

    /// The last whole assignment of `name` that ends before `node`.
    fn last_plain_before(&self, node: Node, name: &str) -> Option<(usize, bool, bool)> {
        self.plain
            .get(name)?
            .iter()
            .filter(|(end, _, _)| *end <= node.start_byte())
            .max_by_key(|(end, _, _)| *end)
            .copied()
    }

    /// Whether `name` holds a function handle or an anonymous function at `node`:
    /// its last whole assignment before `node` has a handle or lambda right-hand side.
    fn is_handle_variable(&self, node: Node, name: &str) -> bool {
        self.last_plain_before(node, name)
            .is_some_and(|(_, handle, _)| handle)
    }

    /// Whether a parenthesised index of `name` at `node` reads as a call because
    /// the value is unknown: an input argument, a local copied from one
    /// (`c = v; c(2)`), or a local started as an empty matrix
    /// (`y = []; y(1) = 3; y(1)`). Probed: MATLAB is silent on each of these;
    /// `x = 2; x(1)` and `c{1}` still echo.
    fn is_parameter_alias(&self, node: Node, name: &str) -> bool {
        self.params.iter().any(|p| p == name)
            || self
                .last_plain_before(node, name)
                .is_some_and(|(_, _, alias)| alias)
    }
}

/// Source text of `node`, empty when it is not valid UTF-8.
fn text_of<'a>(node: Node, source: &'a str) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}

/// The input arguments of `scope` when it is a function definition. Probed: a
/// bare `x` statement is an expression statement when `x` is an input argument.
fn scope_parameters(scope: Node, source: &str) -> Vec<String> {
    if scope.kind() != "function_definition" {
        return Vec::new();
    }
    let mut params = Vec::new();
    for args in crate::analysis::symbols::children_of(scope) {
        if args.kind() != "function_arguments" {
            continue;
        }
        for c in crate::analysis::symbols::children_of(args) {
            if c.kind() == "identifier" {
                params.push(source[c.start_byte()..c.end_byte()].to_string());
            }
        }
    }
    params
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
