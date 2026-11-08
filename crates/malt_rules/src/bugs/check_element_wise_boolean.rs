use super::*;

impl BugsEngine {
    /// CMDAND / CMDOR: Element-wise `&` or `|` used in a boolean context
    /// (e.g., `if` / `while` condition) where short-circuit `&&` / `||` was
    /// likely intended.
    pub(crate) fn check_element_wise_boolean(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "binary_operator" {
            return Vec::new();
        }

        let op_text = find_operator_text(node, source);
        let (rule_id, wrong_op, correct_op, message) = match op_text.as_str() {
            "&" => (
                "CMDAND",
                "&",
                "&&",
                "Use 'A && B' or 'A & B' to test whether A and B are both true in MATLAB."
                    .to_string(),
            ),
            "|" => (
                "CMDOR",
                "|",
                "||",
                "Use 'A || B' or 'A | B' to test whether either A or B is true in MATLAB."
                    .to_string(),
            ),
            _ => return Vec::new(),
        };

        // Only flag if inside a boolean context (if/while/elseif condition).
        if !is_in_boolean_context(node) {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id,
            message,
            severity: Severity::Error,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: Some(Fix::new(
                node.start_byte()..node.end_byte(),
                node_text(node, source).replacen(wrong_op, correct_op, 1),
            )),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- CMDAND / CMDOR ------------------------------------------------------

    #[test]
    fn cmdand_fires_on_ampersand_in_condition() {
        let src = "if (a & b)\n    x = 1;\nend\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "CMDAND"), "got: {diags:?}");
    }

    #[test]
    fn cmdand_no_fire_on_short_circuit() {
        let src = "if (a && b)\n    x = 1;\nend\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "CMDAND"), "got: {diags:?}");
    }

    #[test]
    fn cmdor_fires_on_pipe_in_condition() {
        let src = "while (a | b)\n    x = 1;\nend\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "CMDOR"), "got: {diags:?}");
    }

    #[test]
    fn cmdor_no_fire_on_short_circuit() {
        let src = "while (a || b)\n    x = 1;\nend\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "CMDOR"), "got: {diags:?}");
    }
}
