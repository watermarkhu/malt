use super::*;

impl GoodPracticesEngine {
    /// LNGNM: Variable name exceeds maximum length.
    pub(crate) fn check_lngnm(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("LNGNM") || node.kind() != "assignment" {
            return Vec::new();
        }

        let lhs = match node.child_by_field_name("left") {
            Some(l) => l,
            None => return Vec::new(),
        };

        if lhs.kind() != "identifier" {
            return Vec::new();
        }

        let name = node_text(lhs, source);
        if name.len() <= self.config.max_variable_name_length {
            return Vec::new();
        }

        let pos = lhs.start_position();
        vec![Diagnostic {
            rule_id: "LNGNM",
            message: format!(
                "Names longer than {} characters are not supported. This name has been truncated to {} characters.",
                self.config.max_variable_name_length, self.config.max_variable_name_length
            ),
            severity: Severity::Warning,
            byte_range: lhs.start_byte()..lhs.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}
