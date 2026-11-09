use super::*;

impl BugsEngine {
    /// BDSCA2: an operand of `&&` or `||` is a known non-scalar value, e.g.
    /// `[1 2] || false`.
    pub(crate) fn check_nonscalar_operand(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "boolean_operator" {
            return Vec::new();
        }
        let (Some(left), Some(right)) = (node.child(0), node.child(2)) else {
            return Vec::new();
        };
        let bad = [left, right]
            .into_iter()
            .find(|operand| is_array_expression(*operand, source));
        let Some(bad) = bad else {
            return Vec::new();
        };
        let pos = bad.start_position();
        vec![Diagnostic {
            rule_id: "BDSCA2",
            message: "Operands to '||' and '&&' must be scalar values. Use 'all' or 'any' to convert this value into a scalar value or use the element-wise operators '|' or '&' instead.".to_string(),
            severity: Severity::Error,
            byte_range: bad.start_byte()..bad.end_byte(),
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

    #[test]
    fn bdsca2_fires_on_matrix_operand() {
        let diags = node_diags("A = [1 2] || false;\n");
        assert!(has_id(&diags, "BDSCA2"), "got: {diags:?}");
    }

    #[test]
    fn bdsca2_fires_on_range_and_zeros() {
        let diags = node_diags("A = x && 1:3;\nB = zeros(2, 2) || y;\n");
        assert_eq!(
            diags.iter().filter(|d| d.rule_id == "BDSCA2").count(),
            2,
            "got: {diags:?}"
        );
    }

    #[test]
    fn bdsca2_no_fire_on_scalars() {
        let diags = node_diags("A = x && y;\nB = [1] || zeros(1);\nC = a * [1 2];\n");
        assert!(!has_id(&diags, "BDSCA2"), "got: {diags:?}");
    }
}
