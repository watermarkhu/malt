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
            if func == "length" {
                if self.is_enabled("ISMT") && val == "0" {
                    diags.push(make_diag("ISMT", node));
                }
                if self.is_enabled("ISCL") && val == "1" {
                    diags.push(make_diag("ISCL", node));
                }
            }
        }

        diags
    }
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
    fn iscl_not_fire_on_other_length() {
        let src = "y = (length(x) == 2);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "ISCL"), "got: {diags:?}");
    }
}
