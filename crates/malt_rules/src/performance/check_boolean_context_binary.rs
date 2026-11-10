//! Node-level check for elementwise `&` / `|` in boolean contexts: AND2 and OR2.

use super::*;

impl PerformanceEngine {
    /// Check `boolean_operator` for AND2/OR2 patterns.
    ///
    /// In tree-sitter-matlab, `&&` and `||` parse as `boolean_operator`, while
    /// `&` and `|` parse as `binary_operator`. This check fires on
    /// `binary_operator` nodes with `&` or `|` that are inside boolean
    /// contexts (if/while conditions).
    pub(crate) fn check_boolean_context_binary<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let op = extract_operator(node, source);

        // AND2: & in boolean context
        if self.is_enabled("AND2") && op == "&" && is_in_condition(node) {
            diags.push(make_diag_with_fix(
                "AND2",
                node,
                fix_replace_operator(node, source, "&", "&&").map(|f| f.unsafe_fix()),
            ));
        }

        // OR2: | in boolean context
        if self.is_enabled("OR2") && op == "|" && is_in_condition(node) && or2_applies(node, source)
        {
            diags.push(make_diag_with_fix(
                "OR2",
                node,
                fix_replace_operator(node, source, "|", "||").map(|f| f.unsafe_fix()),
            ));
        }

        diags
    }
}

/// Whether an OR2 candidate `|` is a plain condition operand. Probed: MATLAB is
/// silent inside parentheses or call arguments (`(a | b) == 1`, `any(a | b)`)
/// and when an operand is a call other than `numel` or `length`
/// (`strcmp(a, 'x') | b`). It fires for variables, literals, comparisons and `numel`.
fn or2_applies(node: tree_sitter::Node, source: &str) -> bool {
    let mut current = node;
    loop {
        let Some(parent) = current.parent() else {
            return false;
        };
        match parent.kind() {
            "if_statement" | "while_statement" | "elseif_clause" => break,
            "binary_operator" | "comparison_operator" | "boolean_operator" => current = parent,
            _ => return false,
        }
    }
    ["left", "right"].iter().all(|field| {
        node.child_by_field_name(field).is_none_or(|operand| {
            operand.kind() != "function_call"
                || operand
                    .child_by_field_name("name")
                    .map(|name| &source[name.start_byte()..name.end_byte()])
                    .is_some_and(|name| matches!(name, "numel" | "length"))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- AND2 / OR2 ----------------------------------------------------------

    #[test]
    fn or2_silent_on_call_operand_or_parenthesised_operand() {
        // Probed: MATLAB reports no OR2 for `strcmp(...) | b`, `(a | b) == 1`, `any(a | b)`.
        for src in [
            "if strcmp(a, 'x') | b\n    c = 1;\nend\n",
            "if (a | b) == 1\n    c = 1;\nend\n",
            "if any(a | b)\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "OR2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn or2_fires_on_plain_condition_operands() {
        let diags = lint_nodes(&*engine(), "if a | b\n    c = 1;\nend\n");
        assert!(has_id(&diags, "OR2"), "got: {diags:?}");
    }

    #[test]
    fn and2_fires_on_elementwise_and_in_condition() {
        let src = "if a & b\n    c = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "AND2"), "got: {diags:?}");
    }

    #[test]
    fn and2_not_fire_on_short_circuit() {
        let src = "if a && b\n    c = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "AND2"), "got: {diags:?}");
    }

    #[test]
    fn or2_fires_on_elementwise_or_in_condition() {
        let src = "if a | b\n    c = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "OR2"), "got: {diags:?}");
    }

    #[test]
    fn or2_not_fire_on_short_circuit() {
        let src = "if a || b\n    c = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "OR2"), "got: {diags:?}");
    }
}
