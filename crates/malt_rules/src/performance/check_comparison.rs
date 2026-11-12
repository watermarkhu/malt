//! Node-level check for `comparison_operator` nodes: ISMT (`length(x) == 0`) and ISCL (`length(x) == 1`).

use super::*;

impl PerformanceEngine {
    /// Check `comparison_operator` for ISMT/ISCL patterns.
    pub(crate) fn check_comparison<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // Pattern: length(x) == 0 → isempty(x)
        // Pattern: length(x) == 1 → isscalar(x)
        if let Some((func, val)) = extract_length_comparison(node, source) {
            if func == "length" && self.is_enabled("ISMT") && val == "0" {
                diags.push(make_diag("ISMT", node));
            }
            let is_equality = node
                .child(1)
                .is_some_and(|op| node_text(op, source) == "==");
            if self.is_enabled("ISCL") && is_equality && val == "1" {
                diags.push(make_diag("ISCL", node));
            }
        }

        // Probed: `length(x) > 0` is ISMT too; `>= 1`, `< 1` and `0 < length(x)` are not.
        if self.is_enabled("ISMT") && is_length_greater_than_zero(node, source) {
            diags.push(make_diag("ISMT", node));
        }

        diags
    }
}

/// `length(x) > 0`, with the call on the left.
fn is_length_greater_than_zero(node: tree_sitter::Node, source: &str) -> bool {
    let (Some(left), Some(op), Some(right)) = (node.child(0), node.child(1), node.child(2)) else {
        return false;
    };
    node_text(op, source) == ">"
        && left.kind() == "function_call"
        && extract_func_name(left, source) == Some("length")
        && node_text(right, source).trim() == "0"
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- ISMT / ISCL ----------------------------------------------------------

    #[test]
    fn ismt_fires_on_length_zero() {
        let src = "y = (length(x) == 0);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "ISMT"), "got: {diags:?}");
    }

    #[test]
    fn ismt_fires_on_length_not_equal_zero() {
        // Probed: `(length(x) ~= 0)` is ISMT; ISCL is for `==` only.
        let diags = lint_nodes(&*engine(), "y = (length(x) ~= 0);\n");
        assert!(has_id(&diags, "ISMT"), "got: {diags:?}");
        let diags = lint_nodes(&*engine(), "y = (length(x) ~= 1);\n");
        assert!(!has_id(&diags, "ISCL"), "got: {diags:?}");
    }

    #[test]
    fn ismt_not_fire_on_other_length() {
        let src = "y = (length(x) == 2);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "ISMT"), "got: {diags:?}");
    }

    #[test]
    fn iscl_fires_on_length_one() {
        let src = "y = (length(x) == 1);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "ISCL"), "got: {diags:?}");
    }

    #[test]
    fn iscl_fires_on_numel_one() {
        let src = "y = (numel(x) == 1);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "ISCL"), "got: {diags:?}");
    }

    #[test]
    fn ismt_not_fire_on_numel_zero() {
        let src = "y = (numel(x) == 0);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "ISMT"), "got: {diags:?}");
    }

    #[test]
    fn iscl_not_fire_on_other_length() {
        let src = "y = (length(x) == 2);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "ISCL"), "got: {diags:?}");
    }
}

#[cfg(test)]
mod ismt_greater_tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    #[test]
    fn ismt_fires_on_length_greater_than_zero() {
        // Probed: `length(x) > 0` in a while condition is ISMT (undefined and constant x).
        for src in [
            "while length(stack) > 0\n    stack(end) = [];\nend\n",
            "y = (length(x) > 0);\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, "ISMT"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn ismt_silent_on_other_length_comparisons() {
        // Probed silent: `>= 1`, `< 1`, `0 < length(x)`, `numel(x) > 0`.
        for src in [
            "y = (length(x) >= 1);\n",
            "y = (length(x) < 1);\n",
            "y = (0 < length(x));\n",
            "y = (numel(x) > 0);\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "ISMT"), "{src}: got: {diags:?}");
        }
    }
}
