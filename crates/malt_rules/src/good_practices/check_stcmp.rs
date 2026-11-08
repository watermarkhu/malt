use super::*;

impl GoodPracticesEngine {
    /// STCMP: Use `strcmp`/`strcmpi` instead of `==` for string comparison.
    pub(crate) fn check_stcmp(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("STCMP") {
            return Vec::new();
        }
        if node.kind() != "comparison_operator" && node.kind() != "binary_operator" {
            return Vec::new();
        }

        let text = node_text(node, source);
        if !text.contains("==") {
            return Vec::new();
        }

        // Check if either operand is a string literal.
        let has_string_operand = node_has_string_child(node);
        if !has_string_operand {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "STCMP",
            message: "Use STRCMP instead of == or ~= to compare character vectors, or convert character vectors to string scalars for direct comparison.".to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}
