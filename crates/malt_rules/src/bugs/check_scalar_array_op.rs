use super::*;

impl BugsEngine {
    /// BDSCA2: Suspicious scalar/array operation.
    ///
    /// Flags arithmetic operations between known array-producing functions and
    /// scalars that may produce unintended results.
    pub(crate) fn check_scalar_array_op(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "binary_operator" {
            return Vec::new();
        }

        let op = find_operator_text(node, source);
        // Only flag matrix operations (*, /, \) not element-wise ones.
        if op != "*" && op != "/" && op != "\\" {
            return Vec::new();
        }

        let lhs = match node.child(0) {
            Some(c) => c,
            None => return Vec::new(),
        };
        let rhs = match node.child(2) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let lhs_is_array = is_array_expression(lhs, source);
        let rhs_is_array = is_array_expression(rhs, source);
        let lhs_is_scalar = is_scalar_literal(lhs, source);
        let rhs_is_scalar = is_scalar_literal(rhs, source);

        if (lhs_is_array && rhs_is_scalar) || (lhs_is_scalar && rhs_is_array) {
            let pos = node.start_position();
            vec![Diagnostic {
                rule_id: "BDSCA2",
                message: "Operands to '||' and '&&' must be scalar values. Use 'all' or 'any' to convert this value into a scalar value or use the element-wise operators '|' or '&' instead.".to_string(),
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

    // -- BDSCA2 --------------------------------------------------------------

    #[test]
    fn bdsca2_fires_on_array_times_scalar() {
        let src = "y = zeros(3) * 2;\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "BDSCA2"), "got: {diags:?}");
    }

    #[test]
    fn bdsca2_no_fire_on_identifier_ops() {
        let src = "y = a * b;\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "BDSCA2"), "got: {diags:?}");
    }
}
