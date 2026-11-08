use super::*;

impl GoodPracticesEngine {
    /// ELARLOG: Element-wise `&` / `|` in if/while condition (should use `&&` / `||`).
    pub(crate) fn check_elarlog(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("ELARLOG") {
            return Vec::new();
        }
        if node.kind() != "if_statement" && node.kind() != "while_statement" {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        find_element_wise_boolean_in_condition(node, source, &mut diagnostics);
        diagnostics
    }
}
