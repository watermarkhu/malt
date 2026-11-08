use super::*;

impl CodegenEngine {
    /// Check a `for_statement` node for parfor.
    pub(crate) fn check_for_statement<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // EMPFR: parfor detection — check if the node text starts with "parfor"
        if self.is_enabled("EMPFR") {
            let text = node_text(node, source);
            if text.starts_with("parfor") {
                diags.push(make_diag("EMPFR", node));
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    // -- EMPFR ----------------------------------------------------------------

    #[test]
    fn empfr_fires_on_parfor() {
        let src = "parfor i = 1:10\n    x = i;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMPFR"), "got: {diags:?}");
    }

    #[test]
    fn empfr_not_fire_on_regular_for() {
        let src = "for i = 1:10\n    x = i;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMPFR"), "got: {diags:?}");
    }
}
