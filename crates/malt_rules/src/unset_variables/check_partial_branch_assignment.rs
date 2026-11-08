use super::*;

impl UnsetVariablesEngine {
    /// Check for variables set in some branches of if/switch but not all (PSET).
    ///
    /// This check walks `if_statement` and `switch_statement` nodes looking for
    /// variables that are assigned in some branches but not others.
    pub(crate) fn check_partial_branch_assignment(
        &self,
        tree: &Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        self.walk_pset(tree.root_node(), source, &mut diagnostics);
        diagnostics
    }

    /// DFS walk to find if/switch statements for PSET analysis.
    fn walk_pset(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        match node.kind() {
            "if_statement" => {
                self.check_if_pset(node, source, diagnostics);
            }
            "switch_statement" => {
                self.check_switch_pset(node, source, diagnostics);
            }
            _ => {}
        }

        // Recurse into children.
        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                self.walk_pset(child, source, diagnostics);
            }
        }
    }
}
