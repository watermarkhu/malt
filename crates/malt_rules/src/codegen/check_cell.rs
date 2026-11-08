use super::*;

impl CodegenEngine {
    /// Check a `cell` node (EMCEL).
    pub(crate) fn check_cell(&self, node: tree_sitter::Node) -> Vec<Diagnostic> {
        if self.is_enabled("EMCEL") {
            vec![make_diag("EMCEL", node)]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    // -- EMCEL ----------------------------------------------------------------

    #[test]
    fn emcel_fires_on_cell_literal() {
        let src = "x = {1, 2};\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMCEL"), "got: {diags:?}");
    }

    #[test]
    fn emcel_not_fire_on_matrix_literal() {
        let src = "x = [1, 2];\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMCEL"), "got: {diags:?}");
    }
}
