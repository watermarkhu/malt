//! # RPMTT, RPMTF: redundant boolean tautologies
//!
//! `check_boolean_tautology` flags `x | true` / `true | x` and
//! `x & false` / `false & x`, which simplify to `true` and `false`
//! respectively.

use super::*;

impl ReadabilityEngine {
    /// RPMTT, RPMTF: Boolean tautologies `x | true` → `true`, `x & false` → `false`
    pub(crate) fn check_boolean_tautology(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        // Need at least: lhs, operator_token, rhs
        if node.child_count() < 3 {
            return;
        }

        // Extract the operator token (unnamed child between named children)
        let op_text = extract_operator_text(node, source);

        let (lhs, rhs) = match (node.named_child(0), node.named_child(1)) {
            (Some(l), Some(r)) => (l, r),
            _ => return,
        };

        let lhs_text = &source[lhs.start_byte()..lhs.end_byte()];
        let rhs_text = &source[rhs.start_byte()..rhs.end_byte()];

        // x | true → true  OR  true | x → true
        if (op_text == "|" || op_text == "||")
            && self.is_check_enabled("RPMTT")
            && (rhs_text == "true" || lhs_text == "true")
        {
            results.push(self.diag(
                "RPMTT",
                "For readability, consider using 'true(x,y)' instead of 'repmat(true,x,y)'.",
                node,
                Some(Fix::new(node.start_byte()..node.end_byte(), "true")),
            ));
            return;
        }

        // x & false → false  OR  false & x → false
        if (op_text == "&" || op_text == "&&")
            && self.is_check_enabled("RPMTF")
            && (rhs_text == "false" || lhs_text == "false")
        {
            results.push(self.diag(
                "RPMTF",
                "For readability, consider using 'false(x,y)' instead of 'repmat(false,x,y)'.",
                node,
                Some(Fix::new(node.start_byte()..node.end_byte(), "false")),
            ));
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

    // -- RPMTT ---------------------------------------------------------------

    #[test]
    fn rpmtt_or_true_fires() {
        let diags = lint_nodes(&*engine(), "x = a || true;\n");
        assert!(has_id(&diags, "RPMTT"), "got: {diags:?}");
    }

    #[test]
    fn rpmtt_or_other_variable_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = a || b;\n");
        assert!(!has_id(&diags, "RPMTT"), "got: {diags:?}");
    }

    // -- RPMTF ---------------------------------------------------------------

    #[test]
    fn rpmtf_and_false_fires() {
        let diags = lint_nodes(&*engine(), "x = a && false;\n");
        assert!(has_id(&diags, "RPMTF"), "got: {diags:?}");
    }

    #[test]
    fn rpmtf_and_other_variable_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = a && b;\n");
        assert!(!has_id(&diags, "RPMTF"), "got: {diags:?}");
    }
}
