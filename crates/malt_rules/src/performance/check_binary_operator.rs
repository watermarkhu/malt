//! Node-level check for `binary_operator` nodes: MINV (`inv(A)*b`) and MMTC (`x .* x`).

use super::*;

impl PerformanceEngine {
    /// Check `binary_operator` for MINV and MMTC patterns.
    pub(crate) fn check_binary_operator<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let op = extract_operator(node, source);

        // MINV: inv(A) * b or b * inv(A)
        if self.is_enabled("MINV") && op == "*" && has_inv_operand(node, source) {
            diags.push(make_diag("MINV", node));
        }

        // MMTC: x .* x → x.^2
        if self.is_enabled("MMTC") && op == ".*" && has_same_operands(node, source) {
            diags.push(make_diag("MMTC", node));
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- MINV -----------------------------------------------------------------

    #[test]
    fn minv_fires_on_inv_times_vector() {
        let src = "x = inv(A) * b;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MINV"), "got: {diags:?}");
    }

    #[test]
    fn minv_fires_on_vector_times_inv() {
        let src = "x = b * inv(A);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MINV"), "got: {diags:?}");
    }

    #[test]
    fn minv_not_fire_on_backslash_solve() {
        let src = "x = A \\ b;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MINV"), "got: {diags:?}");
    }

    // -- MMTC -----------------------------------------------------------------

    #[test]
    fn mmtc_fires_on_same_operand_dot_star() {
        let src = "z = x .* x;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MMTC"), "got: {diags:?}");
    }

    #[test]
    fn mmtc_not_fire_on_different_operands() {
        let src = "z = x .* y;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MMTC"), "got: {diags:?}");
    }
}
