//! VUNUS check: an operator expression, or a variable, whose value is discarded.
//!
//! A statement that ends in a semicolon and whose value is an operator
//! expression (`a + b;`, `~x;`, `x';`, `1:n;`, `[1 2];`, `{1};`) computes a value
//! that nothing reads. MATLAB reports the outermost operator, after any
//! parentheses, at the operator's position. Without a semicolon the value is
//! echoed instead (NOPTS or NOPRT), so nothing is reported here.
//!
//! `==` with a plain reference on the left (`x == 1;`, `s.a == 1;`, `c{1} == 1;`)
//! is EQEFF, not VUNUS; any other left operand (`1 == x;`, `x' == 1;`) is VUNUS.
//! A bare identifier is VUNUS only inside a function, when it is a variable of
//! that function (`a;` with `a` an input or an earlier assignment). In a script
//! MATLAB is silent about it.

use super::*;
use crate::analysis::symbols::{DefKind, ScopeKind, SymbolTable};
use crate::formatting::has_trailing_semicolon;

/// Node kinds whose value is an operator: `a + b`, `~a`, `a'`, `1:n`, `[a b]`, `{a}`.
const OPERATOR_NODES: &[&str] = &[
    "binary_operator",
    "boolean_operator",
    "comparison_operator",
    "unary_operator",
    "not_operator",
    "postfix_operator",
    "range",
    "matrix",
    "cell",
];

/// Left operands that make `==` an EQEFF (a reference, not a computed value).
const REFERENCE_NODES: &[&str] = &["identifier", "field_expression", "function_call"];

impl UnusedEngine {
    /// Run VUNUS check: a discarded operator expression or variable.
    pub(crate) fn check_vunus(
        &self,
        table: &SymbolTable,
        tree: &tree_sitter::Tree,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if self.is_check_disabled("VUNUS") {
            return;
        }
        walk_vunus(tree.root_node(), table, source, diagnostics);
    }
}

/// Visit every statement of the file and report the discarded ones.
fn walk_vunus(node: Node, table: &SymbolTable, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let is_statement_level = node
        .parent()
        .map(|p| STATEMENT_PARENTS.contains(&p.kind()))
        .unwrap_or(false);
    // A statement with a parse error inside (`x = (1:2:3:4)';`) is not a discarded
    // value: MATLAB reports the grammar problem instead.
    if is_statement_level && !node.has_error() && has_trailing_semicolon(node, source) {
        check_discarded(node, table, source, diagnostics);
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            walk_vunus(child, table, source, diagnostics);
        }
    }
}

/// Report the value of one semicolon-terminated statement, if it is discarded.
fn check_discarded(
    node: Node,
    table: &SymbolTable,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let expr = unwrap_parens(node);
    if is_equality_on_reference(expr, source) {
        return;
    }
    if OPERATOR_NODES.contains(&expr.kind()) {
        if let Some(op) = operator_token(expr) {
            let message = vunus_message(&source[op.byte_range()]);
            push(diagnostics, op, message);
        }
    } else if let Some(name_node) = bare_name(expr) {
        let name = &source[name_node.byte_range()];
        if is_function_variable(table, name_node, name) {
            push(diagnostics, name_node, vunus_message(name));
        }
    }
}

/// The name of a statement that is only a name: `a;`. The grammar parses such a
/// statement as a command with a single `command_name` child.
fn bare_name(node: Node) -> Option<Node> {
    match node.kind() {
        "identifier" => Some(node),
        "command" if node.named_child_count() == 1 => node
            .named_child(0)
            .filter(|child| child.kind() == "command_name"),
        _ => None,
    }
}

/// The expression inside any parentheses around a statement: `(a + b);` is `a + b`.
pub(crate) fn unwrap_parens(node: Node) -> Node {
    let mut current = node;
    while current.kind() == "parenthesis" && current.named_child_count() == 1 {
        match current.named_child(0) {
            Some(inner) => current = inner,
            None => break,
        }
    }
    current
}

/// The operator token of an operator expression: the operator of a binary or
/// unary expression, the transpose of a postfix one, or the opening bracket of a
/// matrix or cell. `None` for any other node.
pub(crate) fn operator_token(node: Node) -> Option<Node> {
    if !OPERATOR_NODES.contains(&node.kind()) {
        return None;
    }
    let mut cursor = node.walk();
    let tokens: Vec<Node> = node
        .children(&mut cursor)
        .filter(|child| !child.is_named())
        .collect();
    match node.kind() {
        "postfix_operator" => tokens.last().copied(),
        _ => tokens.first().copied(),
    }
}

/// Whether `node` is `==` with a reference on its left: an EQEFF rather than a
/// VUNUS. Used by both checks, so one construct is reported under one ID.
pub(crate) fn is_equality_on_reference(node: Node, source: &str) -> bool {
    node.kind() == "comparison_operator"
        && operator_token(node).is_some_and(|op| &source[op.byte_range()] == "==")
        && node
            .named_child(0)
            .is_some_and(|left| REFERENCE_NODES.contains(&left.kind()))
}

/// The message for a discarded `op`. MATLAB prints the transpose without quotes.
fn vunus_message(op: &str) -> String {
    if op == "'" {
        "' produces a value that might be unused.".to_string()
    } else {
        format!("'{op}' produces a value that might be unused.")
    }
}

/// Whether `name`, at `node`, is a variable of the function around it. Scripts
/// are not counted. An input or output argument is always a variable; any other
/// name counts when the function assigns it anywhere (probed: `q;` before
/// `q = 2` is VUNUS too).
fn is_function_variable(table: &SymbolTable, node: Node, name: &str) -> bool {
    let start = node.start_byte();
    let Some(scope) = table
        .scopes
        .iter()
        .filter(|s| s.byte_range.start <= start && start < s.byte_range.end)
        .min_by_key(|s| s.byte_range.len())
    else {
        return false;
    };
    if !matches!(
        scope.kind,
        ScopeKind::Function | ScopeKind::LocalFunction | ScopeKind::Method
    ) {
        return false;
    }
    scope.defs.iter().any(|d| {
        d.name == name
            && match d.kind {
                DefKind::InputArg | DefKind::OutputArg | DefKind::Assignment => true,
                DefKind::Global | DefKind::Persistent => false,
                _ => d.effect_start <= start,
            }
    })
}

/// Push a VUNUS diagnostic on the position of `node`.
fn push(diagnostics: &mut Vec<Diagnostic>, node: Node, message: String) {
    let pos = node.start_position();
    diagnostics.push(Diagnostic {
        rule_id: "VUNUS",
        message,
        severity: Severity::Warning,
        byte_range: node.byte_range(),
        line: pos.row + 1,
        column: pos.column + 1,
        fix: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::lint_file;
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    /// The VUNUS diagnostics for `source` as (line, column, message).
    fn vunus(source: &str) -> Vec<(usize, usize, String)> {
        lint_file(&*engine(), source)
            .into_iter()
            .filter(|d| d.rule_id == "VUNUS")
            .map(|d| (d.line, d.column, d.message))
            .collect()
    }

    fn msg(op: &str) -> String {
        format!("'{op}' produces a value that might be unused.")
    }

    // -- Probes: MATLAB R2026b checkcode -------------------------------------

    #[test]
    fn vunus_arithmetic_reported_at_operator() {
        assert_eq!(vunus("a = 1; b = 2;\na + b;\n"), vec![(2, 3, msg("+"))]);
    }

    #[test]
    fn vunus_without_semicolon_is_not_reported() {
        // Echoed instead: NOPTS in a script, NOPRT in a function.
        assert!(vunus("a = 1; b = 2;\na + b\n").is_empty());
        assert!(vunus("function f(a, b)\n    a + b\nend\n").is_empty());
    }

    #[test]
    fn vunus_comparison_and_logical_operators() {
        let src = "x = 1; y = 2;\nx < 1;\nx ~= 1;\nx & y;\nx && y;\nx <= 1;\nx >= 1;\n";
        let ops: Vec<String> = vunus(src).into_iter().map(|d| d.2).collect();
        assert_eq!(
            ops,
            vec![
                msg("<"),
                msg("~="),
                msg("&"),
                msg("&&"),
                msg("<="),
                msg(">=")
            ]
        );
    }

    #[test]
    fn vunus_unary_and_transpose() {
        let src = "x = [1 2];\n-x;\n~x;\nx';\nx.';\n";
        let found = vunus(src);
        assert_eq!(found[0], (2, 1, msg("-")));
        assert_eq!(found[1], (3, 1, msg("~")));
        assert_eq!(
            found[2],
            (4, 2, "' produces a value that might be unused.".to_string())
        );
        assert_eq!(found[3], (5, 2, msg(".'")));
    }

    #[test]
    fn vunus_one_report_for_the_outermost_operator() {
        // `x + 1 + 2` is `(x + 1) + 2`: only the outer `+`, at column 7.
        assert_eq!(vunus("x = 1;\nx + 1 + 2;\n"), vec![(2, 7, msg("+"))]);
    }

    #[test]
    fn vunus_parentheses_are_transparent() {
        assert_eq!(vunus("x = 1;\n(x + 1);\n"), vec![(2, 4, msg("+"))]);
        assert_eq!(vunus("x = 1;\n(x + 1) * 2;\n"), vec![(2, 9, msg("*"))]);
    }

    #[test]
    fn vunus_range_matrix_and_cell() {
        let src = "x = 1;\n1:x;\n[1 2];\n{1};\n";
        let found = vunus(src);
        assert_eq!(found[0], (2, 2, msg(":")));
        assert_eq!(found[1], (3, 1, msg("[")));
        assert_eq!(found[2], (4, 1, msg("{")));
    }

    #[test]
    fn vunus_equality_on_non_reference_left() {
        // `1 == x;`, `x' == 1;` and `(x + 1) == 2;` compute a value: VUNUS.
        let src = "x = 1;\n1 == x;\n";
        assert_eq!(vunus(src), vec![(2, 3, msg("=="))]);
        assert_eq!(vunus("x = 1;\nx + 1 == 2;\n"), vec![(2, 7, msg("=="))]);
        assert_eq!(vunus("x = 1;\n~x == 1;\n"), vec![(2, 4, msg("=="))]);
    }

    #[test]
    fn vunus_equality_on_reference_is_eqeff_not_vunus() {
        assert!(vunus("a = 1; b = 2;\na == b;\n").is_empty());
        assert!(vunus("s.a = 1;\ns.a == 1;\n").is_empty());
        assert!(vunus("x = 1;\n(x == 1);\n").is_empty());
    }

    #[test]
    fn vunus_bare_variable_in_function_only() {
        let src = "function f()\n    a = 1;\n    a;\nend\n";
        assert_eq!(vunus(src), vec![(3, 5, msg("a"))]);
        // The read is not a dead store: MATLAB reports only the VUNUS.
        let diags = lint_file(&*engine(), src);
        assert!(
            !diags.iter().any(|d| d.rule_id == "NASGU"),
            "got: {diags:?}"
        );
        assert_eq!(
            vunus("function f(a)\n    a;\nend\n"),
            vec![(2, 5, msg("a"))]
        );
        // In a script, and for a name that is not a variable, MATLAB is silent.
        assert!(vunus("x = 1;\nx;\n").is_empty());
        assert!(vunus("function f()\n    disp;\nend\n").is_empty());
    }

    #[test]
    fn vunus_bare_variable_assigned_later() {
        // Probe (gp-a__t10 `keyboard;`, v4): `a;` before `a = 1` is VUNUS too.
        assert_eq!(
            vunus("function f()\n    a;\n    a = 1;\nend\n"),
            vec![(2, 5, msg("a"))]
        );
        // `ans` with no assignment in the function is not a variable here.
        assert!(vunus("function f()\n    ans;\nend\n").is_empty());
    }
}
