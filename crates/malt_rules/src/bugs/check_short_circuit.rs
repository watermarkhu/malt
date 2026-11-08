use super::*;

impl BugsEngine {
    /// SHOCIRT / SHOCIRF: Short-circuit operator with potentially non-scalar LHS.
    ///
    /// Flags `&&` and `||` when the left operand is a known array-producing call.
    pub(crate) fn check_short_circuit(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "boolean_operator" {
            return Vec::new();
        }

        let op_text = find_operator_text(node, source);
        let (rule_id, message) = match op_text.as_str() {
            "&&" => (
                "SHOCIRT",
                "The && operator is unexpected because &&(A && B) always returns true.".to_string(),
            ),
            "||" => (
                "SHOCIRF",
                "The || operator is unexpected because ||(A || B) always returns false."
                    .to_string(),
            ),
            _ => return Vec::new(),
        };

        // Check if the LHS is a known array-producing function call.
        let lhs = match node.child(0) {
            Some(c) => c,
            None => return Vec::new(),
        };

        if is_array_expression(lhs, source) {
            let pos = node.start_position();
            vec![Diagnostic {
                rule_id,
                message,
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- SHOCIRT / SHOCIRF ---------------------------------------------------

    #[test]
    fn shocirt_fires_on_zeros_short_circuit() {
        let src = "y = zeros(3) && x;\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "SHOCIRT"), "got: {diags:?}");
    }

    #[test]
    fn shocirt_no_fire_on_scalar_lhs() {
        let src = "y = a && b;\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "SHOCIRT"), "got: {diags:?}");
    }

    #[test]
    fn shocirf_fires_on_ones_short_circuit() {
        let src = "y = ones(3) || x;\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "SHOCIRF"), "got: {diags:?}");
    }

    #[test]
    fn shocirf_no_fire_on_scalar_lhs() {
        let src = "y = a || b;\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "SHOCIRF"), "got: {diags:?}");
    }
}
