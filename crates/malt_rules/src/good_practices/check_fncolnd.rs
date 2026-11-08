use super::*;

impl GoodPracticesEngine {
    /// FNCOLND: `end` used as column index without explicit dimension specification.
    pub(crate) fn check_fncolnd(&self, tree: &tree_sitter::Tree, _source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("FNCOLND") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        find_end_as_index(tree.root_node(), &mut diagnostics);
        diagnostics
    }
}
