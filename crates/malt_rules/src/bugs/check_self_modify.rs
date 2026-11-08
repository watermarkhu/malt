use super::*;

impl BugsEngine {
    /// INCR / DECR: Suspicious self-increment/decrement pattern `x = x + 1` or
    /// `x = x - 1` inside a loop body where MATLAB indexing may be intended.
    pub(crate) fn check_self_modify(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "assignment" {
            return Vec::new();
        }

        // Get LHS (first child) and RHS.
        let lhs = match node.child_by_field_name("left").or_else(|| node.child(0)) {
            Some(c) if c.kind() == "identifier" => c,
            _ => return Vec::new(),
        };

        let rhs = match node.child_by_field_name("right").or_else(|| node.child(2)) {
            Some(c) => c,
            None => return Vec::new(),
        };

        if rhs.kind() != "binary_operator" {
            return Vec::new();
        }

        let lhs_name = node_text(lhs, source);

        // Check if the RHS is `lhs_name + 1` or `lhs_name - 1`.
        let rhs_left = match rhs.child(0) {
            Some(c) => c,
            None => return Vec::new(),
        };
        let rhs_op = find_operator_text(rhs, source);
        let rhs_right = match rhs.child(2) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let rhs_left_text = node_text(rhs_left, source).trim().to_string();
        let rhs_right_text = node_text(rhs_right, source).trim().to_string();

        // Only flag if inside a loop body.
        if !is_inside_loop(node) {
            return Vec::new();
        }

        let (rule_id, message) = match rhs_op.as_str() {
            "+" if rhs_left_text == lhs_name && rhs_right_text == "1" => (
                "INCR",
                "++x operation does not increment the value of x. To increase the value by 1, use x = x + 1.".to_string(),
            ),
            "-" if rhs_left_text == lhs_name && rhs_right_text == "1" => (
                "DECR",
                "--x operation does not decrement the value of x. To decrease the value by 1, use x = x - 1.".to_string(),
            ),
            _ => return Vec::new(),
        };

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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- INCR / DECR ---------------------------------------------------------

    #[test]
    fn incr_fires_on_self_increment_in_loop() {
        let src = "for i = 1:10\n    x = x + 1;\nend\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "INCR"), "got: {diags:?}");
    }

    #[test]
    fn incr_no_fire_outside_loop() {
        let src = "x = x + 1;\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "INCR"), "got: {diags:?}");
    }

    #[test]
    fn decr_fires_on_self_decrement_in_loop() {
        let src = "while x > 0\n    x = x - 1;\nend\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "DECR"), "got: {diags:?}");
    }

    #[test]
    fn decr_no_fire_outside_loop() {
        let src = "x = x - 1;\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "DECR"), "got: {diags:?}");
    }
}
