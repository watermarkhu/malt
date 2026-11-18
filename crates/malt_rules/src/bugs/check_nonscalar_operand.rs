use super::*;

impl BugsEngine {
    /// BDSCA2: an operand of `&&` or `||` is a known non-scalar value, e.g.
    /// `[1 2] || false`. Each such operand is reported (probed: `[1 0] || [1 0]`
    /// gives two diagnostics).
    pub(crate) fn check_nonscalar_operand(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "boolean_operator"
            || self.config.disabled_checks.iter().any(|id| id == "BDSCA2")
        {
            return Vec::new();
        }
        let (Some(left), Some(right)) = (node.child(0), node.child(2)) else {
            return Vec::new();
        };
        [left, right]
            .into_iter()
            .filter(|operand| is_array_expression(*operand, source))
            .map(|bad| {
                let pos = bad.start_position();
                Diagnostic {
                    rule_id: "BDSCA2",
                    message: "Operands to '||' and '&&' must be scalar values.  Use 'all' or 'any' to convert this value into a scalar value or use the element-wise operators '|' or '&' instead.".to_string(),
                    severity: Severity::Error,
                    byte_range: bad.start_byte()..bad.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                }
            })
            .collect()
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
    fn bdsca2_fires_on_range_and_empty_matrix() {
        // Probed: `x && 1:3`, `x || []`, `[] && x`, `3:1:9` and `[1 0] && [2 3]`
        // (both operands) are all reported.
        let diags = node_diags("A = x && 1:3;\nB = x || [];\n");
        assert_eq!(
            diags.iter().filter(|d| d.rule_id == "BDSCA2").count(),
            2,
            "got: {diags:?}"
        );
        for (source, count) in [
            ("y = [] && x;\n", 1),
            ("y = x && 3:1:9;\n", 1),
            ("if [1 0] && [2 3], disp(1), end\n", 2),
            ("y = [x x] && true;\n", 1),
            ("y = [1 2; 3 4] && x;\n", 1),
            ("y = x && [1 0] && x;\n", 1),
            ("while x && [1 0], x = 0; end\n", 1),
        ] {
            let diags = node_diags(source);
            assert_eq!(
                diags.iter().filter(|d| d.rule_id == "BDSCA2").count(),
                count,
                "{source:?} got: {diags:?}"
            );
        }
    }

    #[test]
    fn bdsca2_no_fire_on_scalars() {
        let diags = node_diags("A = x && y;\nB = [1] || zeros(1);\nC = a * [1 2];\n");
        assert!(!has_id(&diags, "BDSCA2"), "got: {diags:?}");
    }

    #[test]
    fn bdsca2_silent_on_forms_probed_silent() {
        // Probed silent: a single-element matrix, `zeros`/`ones`/`rand`, a
        // parenthesised range, `{1}`, `'abc'`, comparisons, `-[1 0]`, `~[1 0]`,
        // element-wise operators, transposes and a variable holding a matrix.
        for source in [
            "y = x && [x];\n",
            "y = [1] && x;\n",
            "y = zeros(1,3) || false;\n",
            "y = x && ones(2);\n",
            "y = rand(1,3) && x;\n",
            "y = x && (1:3);\n",
            "y = {1} && true;\n",
            "y = 'abc' || x;\n",
            "y = [1 0] == 1 && x;\n",
            "y = x && -[1 0];\n",
            "y = ~[] || x;\n",
            "y = [1 0] & x;\n",
            "y = x && [1 0] & 1;\n",
            "d = [1 0];\ny = d && true;\n",
        ] {
            let diags = node_diags(source);
            assert!(!has_id(&diags, "BDSCA2"), "{source:?} got: {diags:?}");
        }
    }

    #[test]
    fn bdsca2_message_is_matlab_text() {
        let diags = node_diags("y = [1 2] && 3;\n");
        let d = diags
            .iter()
            .find(|d| d.rule_id == "BDSCA2")
            .expect("BDSCA2");
        assert_eq!(
            d.message,
            "Operands to '||' and '&&' must be scalar values.  Use 'all' or 'any' to convert this value into a scalar value or use the element-wise operators '|' or '&' instead."
        );
    }
}
