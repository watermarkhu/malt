use super::*;

impl GoodPracticesEngine {
    /// SHOCIRAA: Short-circuit operator (`&&`/`||`) used where element-wise is expected.
    pub(crate) fn check_shociraa(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("SHOCIRAA") || node.kind() != "boolean_operator" {
            return Vec::new();
        }

        let text = node_text(node, source);
        if !text.contains("&&") && !text.contains("||") {
            return Vec::new();
        }

        // Only fire inside array construction contexts (matrix, cell).
        let in_array_context = is_inside_array_context(node);
        if !in_array_context {
            return Vec::new();
        }

        let op_str = if text.contains("&&") { "&&" } else { "||" };

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "SHOCIRAA",
            message: format!(
                "Using the {op_str} operator in the expression {op_str}(A {op_str} B) is probably unintended."
            ),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}
