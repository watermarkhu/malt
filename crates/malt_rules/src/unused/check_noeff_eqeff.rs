//! NOEFF/EQEFF checks: statements with no effect.
//!
//! EQEFF is `==` with a reference on its left (`x == 1;`), reported at the `==`.
//! Operator expressions with any other value are VUNUS (`check_vunus`), so NOEFF
//! only covers the shapes VUNUS does not: constants, strings, field reads and
//! lambdas.

use super::check_vunus::{is_equality_on_reference, operator_token, unwrap_parens};
use super::*;

/// NOEFF is not emitted: MATLAB reported none of its 340 probe statements (constants,
/// strings, field reads, lambdas, indexing, `x.^2`, `x*2`, `s.a;`, `pi;`), and the
/// operator shapes it would share with VUNUS are VUNUS in MATLAB.
const NOEFF_REPORTED: bool = false;

impl UnusedEngine {
    /// Run NOEFF/EQEFF checks: statements with no effect.
    pub(crate) fn check_noeff_eqeff(&self, ctx: &FileContext, diagnostics: &mut Vec<Diagnostic>) {
        let noeff_disabled = !NOEFF_REPORTED || self.is_check_disabled("NOEFF");
        let eqeff_disabled = self.is_check_disabled("EQEFF");

        if noeff_disabled && eqeff_disabled {
            return;
        }

        // Walk the tree looking for expression nodes at statement level.
        walk_for_no_effect(
            ctx.tree.root_node(),
            false,
            ctx.source,
            diagnostics,
            noeff_disabled,
            eqeff_disabled,
        );
    }
}

// ---------------------------------------------------------------------------
// No-effect statement walker (free function to satisfy clippy)
// ---------------------------------------------------------------------------

/// Recursively walk tree to find statement-level expressions with no effect.
/// `statement_level` says whether the parent is a `source_file` or `block` (the
/// walk knows it; `Node::parent` would search from the root).
pub(crate) fn walk_for_no_effect(
    node: Node,
    statement_level: bool,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
    noeff_disabled: bool,
    eqeff_disabled: bool,
) {
    if statement_level {
        let expr = unwrap_parens(node);

        // EQEFF: `==` with a reference on its left (likely meant `=`), at the `==`.
        if !eqeff_disabled && is_equality_on_reference(expr, source) {
            if let Some(op) = operator_token(expr) {
                let pos = op.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "EQEFF",
                    message: "To assign values to variables, use =. The == operator compares equality of values.".to_string(),
                    severity: Severity::Warning,
                    byte_range: op.start_byte()..op.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: eqeff_fix(node, expr, op, source),
                });
            }
        }
        // NOEFF: the remaining expression nodes that are not assignments,
        // function_calls, commands, or operator expressions (VUNUS).
        else if !noeff_disabled
            && NO_EFFECT_EXPR_NODES.contains(&expr.kind())
            && !is_method_call(expr)
        {
            let pos = node.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "NOEFF",
                message: format!(
                    "The operation or expression {} has no evident effect.",
                    operator_or_text(expr, source)
                ),
                severity: Severity::Warning,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
    }

    // Recurse into children.
    let child_level = STATEMENT_PARENTS.contains(&node.kind());
    for child in crate::analysis::symbols::children_of(node) {
        walk_for_no_effect(
            child,
            child_level,
            source,
            diagnostics,
            noeff_disabled,
            eqeff_disabled,
        );
    }
}

/// `x == 1;` becomes `x = 1;`.
///
/// Unsafe: the statement may be a comparison someone meant to display, and the
/// assignment changes the variable. Offered for a plain variable or a field path
/// on the left (no call, no index) and no parentheses around the statement, where
/// `=` is a valid assignment.
fn eqeff_fix(node: Node, expr: Node, op: Node, source: &str) -> Option<malt_core::Fix> {
    if node.id() != expr.id() {
        return None;
    }
    let left = expr.named_child(0)?;
    let text = &source[left.byte_range()];
    let path = !text.is_empty() && text.split('.').all(crate::fix_util::is_plain_identifier);
    path.then(|| malt_core::Fix::new(op.byte_range(), "=").unsafe_fix())
}

/// `obj.method(args)` parses as a `field_expression` that ends in a call: a
/// statement that does something, not an expression without effect.
fn is_method_call(node: Node) -> bool {
    let mut current = node;
    while current.kind() == "field_expression" {
        let Some(last) =
            current.named_child((current.named_child_count().saturating_sub(1)) as u32)
        else {
            return false;
        };
        if matches!(last.kind(), "function_call" | "cell_index") {
            return true;
        }
        current = last;
    }
    false
}

/// The operator of an operator expression, else the start of its text.
fn operator_or_text(node: tree_sitter::Node, source: &str) -> String {
    let mut cursor = node.walk();
    let op = node
        .children(&mut cursor)
        .find(|c| !c.is_named())
        .map(|c| source[c.byte_range()].to_string());
    op.unwrap_or_else(|| source[node.byte_range()].chars().take(40).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{fix_of, has_id, lint_file, reported_without_fix};

    #[test]
    fn eqeff_fix_turns_the_comparison_into_an_assignment() {
        let (safe, fixed) = fix_of(&*engine(), "x == 1;\n", "EQEFF").unwrap();
        assert!(!safe);
        assert_eq!(fixed, "x = 1;\n");
        let (_, fixed) = fix_of(&*engine(), "s.a.b == y + 1\n", "EQEFF").unwrap();
        assert_eq!(fixed, "s.a.b = y + 1\n");
        assert!(!has_id(&lint_file(&*engine(), &fixed), "EQEFF"));
    }

    #[test]
    fn eqeff_no_fix_for_indexed_or_parenthesised_statements() {
        for src in ["x(2) == 1;\n", "(x == 1);\n", "c{1} == 1;\n"] {
            assert!(reported_without_fix(&*engine(), src, "EQEFF"), "{src}");
        }
    }

    fn engine() -> Box<dyn Rule> {
        // NOEFF is off under the mathworks preset (never emitted by MATLAB), so
        // build the engine with every check enabled.
        Box::new(UnusedEngine {
            config: UnusedConfig::default(),
        })
    }

    // -- NOEFF: statement with no effect ------------------------------------

    #[test]
    fn noeff_silent_on_discarded_constant() {
        // Probed: MATLAB reports no NOEFF for a discarded constant (or string,
        // field read, lambda), so the check is not emitted.
        let diags = lint_file(&*engine(), "function foo()\n    5;\nend\n");
        assert!(!has_id(&diags, "NOEFF"), "got: {diags:?}");
    }

    #[test]
    fn noeff_not_on_discarded_operator_expression() {
        // VUNUS owns operator expressions: one construct, one ID.
        let diags = lint_file(&*engine(), "function foo()\n    1 + 2;\nend\n");
        assert!(!has_id(&diags, "NOEFF"), "got: {diags:?}");
        assert!(has_id(&diags, "VUNUS"), "got: {diags:?}");
    }

    #[test]
    fn noeff_ok_on_assignment() {
        let diags = lint_file(&*engine(), "function foo()\n    x = 1 + 2;\nend\n");
        assert!(!has_id(&diags, "NOEFF"), "got: {diags:?}");
    }

    // -- EQEFF: comparison with no effect -----------------------------------

    #[test]
    fn eqeff_fires_on_discarded_comparison() {
        let diags = lint_file(&*engine(), "function foo()\n    a == b;\nend\n");
        assert!(has_id(&diags, "EQEFF"), "got: {diags:?}");
        assert!(!has_id(&diags, "VUNUS"), "got: {diags:?}");
    }

    #[test]
    fn eqeff_reported_at_the_operator() {
        let diags = lint_file(&*engine(), "a = 1; b = 2;\na == b;\n");
        let d: Vec<_> = diags.iter().filter(|d| d.rule_id == "EQEFF").collect();
        assert_eq!(d.len(), 1, "got: {diags:?}");
        assert_eq!((d[0].line, d[0].column), (2, 3));
    }

    #[test]
    fn eqeff_fires_in_parentheses() {
        let diags = lint_file(&*engine(), "function foo(a, b)\n    (a == b);\nend\n");
        assert!(has_id(&diags, "EQEFF"), "got: {diags:?}");
    }

    #[test]
    fn eqeff_ok_when_result_used() {
        let diags = lint_file(
            &*engine(),
            "function foo(a, b)\n    x = (a == b);\n    disp(x);\nend\n",
        );
        assert!(!has_id(&diags, "EQEFF"), "got: {diags:?}");
    }

    #[test]
    fn eqeff_not_fire_on_other_comparisons() {
        let diags = lint_file(&*engine(), "function foo(a, b)\n    a < b;\nend\n");
        assert!(!has_id(&diags, "EQEFF"), "got: {diags:?}");
        assert!(has_id(&diags, "VUNUS"), "got: {diags:?}");
    }
}
