//! NOEFF/EQEFF checks: statements with no effect.

use super::*;

impl UnusedEngine {
    /// Run NOEFF/EQEFF checks: statements with no effect.
    pub(crate) fn check_noeff_eqeff(&self, ctx: &FileContext, diagnostics: &mut Vec<Diagnostic>) {
        let noeff_disabled = self.is_check_disabled("NOEFF");
        let eqeff_disabled = self.is_check_disabled("EQEFF");

        if noeff_disabled && eqeff_disabled {
            return;
        }

        // Walk the tree looking for expression nodes at statement level.
        walk_for_no_effect(
            ctx.tree.root_node(),
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

/// Returns `true` if a `comparison_operator` node is an equality (`==`)
/// comparison rather than `<`, `<=`, `>`, `>=`, or `~=`.
fn comparison_is_equality(node: Node) -> bool {
    // comparison_operator is `seq(expression, operator, expression)`; the
    // operator token is the middle child and its kind is the literal text.
    node.child(1).map(|op| op.kind() == "==").unwrap_or(false)
}

/// Recursively walk tree to find statement-level expressions with no effect.
pub(crate) fn walk_for_no_effect(
    node: Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
    noeff_disabled: bool,
    eqeff_disabled: bool,
) {
    // Check if this node is at statement level (parent is block or source_file).
    let is_statement_level = node
        .parent()
        .map(|p| STATEMENT_PARENTS.contains(&p.kind()))
        .unwrap_or(false);

    if is_statement_level {
        let kind = node.kind();

        // EQEFF: `==` comparison at statement level (likely meant `=`).
        if !eqeff_disabled && kind == COMPARISON_NODE && comparison_is_equality(node) {
            let pos = node.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "EQEFF",
                message: "To assign values to variables, use =. The == operator compares equality of values.".to_string(),
                severity: Severity::Warning,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
        // NOEFF: other expression nodes at statement level that are not
        // assignments, function_calls, or commands.
        else if !noeff_disabled && NO_EFFECT_EXPR_NODES.contains(&kind) {
            let pos = node.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "NOEFF",
                message: format!(
                    "The operation or expression {} has no evident effect.",
                    operator_or_text(node, source)
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
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            walk_for_no_effect(child, source, diagnostics, noeff_disabled, eqeff_disabled);
        }
    }
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
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    // -- NOEFF: statement with no effect ------------------------------------

    #[test]
    fn noeff_fires_on_discarded_expression() {
        let diags = lint_file(&*engine(), "function foo()\n    1 + 2;\nend\n");
        assert!(has_id(&diags, "NOEFF"), "got: {diags:?}");
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
    }
}
