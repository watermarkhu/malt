use super::*;

impl CodegenEngine {
    /// Check an `arguments_statement` node (EMRIFAV).
    pub(crate) fn check_arguments_statement(&self, node: tree_sitter::Node) -> Vec<Diagnostic> {
        if self.is_enabled("EMRIFAV") {
            vec![make_diag("EMRIFAV", node)]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    // -- EMRIFAV ---------------------------------------------------------------

    #[test]
    fn emrifav_fires_on_arguments_block() {
        let src =
            "function f(a)\n    arguments\n        a (1,1) double\n    end\n    x = a;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMRIFAV"), "got: {diags:?}");
    }

    #[test]
    fn emrifav_not_fire_without_arguments_block() {
        let src = "function f(a)\n    x = a;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMRIFAV"), "got: {diags:?}");
    }
}
