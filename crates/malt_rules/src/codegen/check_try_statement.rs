use super::*;

impl CodegenEngine {
    /// Check a `try_statement` node.
    pub(crate) fn check_try_statement(&self, node: tree_sitter::Node) -> Vec<Diagnostic> {
        if self.is_enabled("EMTC") {
            vec![make_diag("EMTC", node)]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    // -- EMTC -----------------------------------------------------------------

    #[test]
    fn emtc_fires_on_try_catch() {
        let src = "try\n    x = 1;\ncatch\n    y = 2;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMTC"), "got: {diags:?}");
    }

    #[test]
    fn emtc_not_fire_on_plain_block() {
        let src = "if x > 0\n    y = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMTC"), "got: {diags:?}");
    }
}
