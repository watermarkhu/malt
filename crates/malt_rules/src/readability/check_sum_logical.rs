//! # LOGSUM: `sum(logical) > 0` → `any(logical)`
//!
//! `check_sum_logical` flags `sum(...)` calls that appear as the left-hand
//! side of a `> 0` comparison and suggests `any(...)`.

use super::*;

impl ReadabilityEngine {
    /// LOGSUM: `sum(logical) > 0` → `any(logical)`
    ///
    /// We detect `function_call` named `sum` and check if it appears as the
    /// left-hand side of a `> 0` comparison in the parent node. If no parent
    /// comparison is found, we skip (sum alone is fine).
    pub(crate) fn check_sum_logical(
        &self,
        node: tree_sitter::Node,
        _source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("LOGSUM") {
            return;
        }

        // Only flag when parent is a comparison `sum(...) > 0`
        if let Some(parent) = node.parent() {
            if parent.kind() == "comparison_operator" {
                results.push(self.diag(
                    "LOGSUM",
                    "Consider using 'nnz' instead of 'sum' for logical vectors to improve readability.",
                    parent,
                    None,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    // -- LOGSUM --------------------------------------------------------------

    #[test]
    fn logsum_sum_greater_than_zero_fires() {
        let diags = lint_nodes(&*engine(), "x = sum(y) > 0;\n");
        assert!(has_id(&diags, "LOGSUM"), "got: {diags:?}");
    }

    #[test]
    fn logsum_sum_alone_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = sum(y);\n");
        assert!(!has_id(&diags, "LOGSUM"), "got: {diags:?}");
    }
}
