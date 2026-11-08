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
                fix_replace_operator(node, source, "&", "&&"),
            ));
        }

        // OR2: | in boolean context
        if self.is_enabled("OR2") && op == "|" && is_in_condition(node) {
            diags.push(make_diag_with_fix(
                "OR2",
                node,
                fix_replace_operator(node, source, "|", "||"),
            ));
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- AND2 / OR2 ----------------------------------------------------------

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
