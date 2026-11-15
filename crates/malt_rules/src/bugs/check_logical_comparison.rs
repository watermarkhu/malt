use super::*;

/// Functions whose logical result MATLAB compares as constant (`true`/`false`).
/// Only the ones probed: MATLAB reports CFALSE for `all`, `any`, `isnan`, `isinf`,
/// `isfinite`, `isrow`, `isvector`, `isscalar`, `isequal`, `isequaln`, `isreal` and
/// `ismatrix` compared with `==`, `>=`, `<` or `== -1` against a value outside 0..1,
/// and reports nothing for `strcmp`, `strncmp`, `ismember`, `isfield`, `isempty`,
/// `ischar`, `islogical`, `isnumeric`, `isa`, `contains`, `xor`, `isspace`, `isKey`,
/// `exist` or a comparison of a logical expression such as `(n > 2) == 2`.
const LOGICAL_FUNCTIONS: &[&str] = &[
    "all", "any", "isnan", "isinf", "isfinite", "isrow", "isvector", "isscalar", "isequal",
    "isequaln", "isreal", "ismatrix",
];

/// Evaluate `lhs OP rhs` for the given comparison operator.
fn compare(op: &str, lhs: f64, rhs: f64) -> Option<bool> {
    Some(match op {
        "==" => lhs == rhs,
        "~=" => lhs != rhs,
        "<" => lhs < rhs,
        "<=" => lhs <= rhs,
        ">" => lhs > rhs,
        ">=" => lhs >= rhs,
        _ => return None,
    })
}

fn numeric_literal(node: Node, source: &str) -> Option<f64> {
    match node.kind() {
        "number" | "unary_operator" => node_text(node, source).trim().parse::<f64>().ok(),
        _ => None,
    }
}

impl BugsEngine {
    /// CTRUE / CFALSE: a comparison operator applied to a function that
    /// returns a logical value, e.g. `any(A) <= 1`. The logical result can only
    /// be 0 or 1, so the comparison against a numeric literal is constant.
    pub(crate) fn check_logical_comparison(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "comparison_operator" {
            return Vec::new();
        }
        let (Some(left), Some(right)) = (node.child(0), node.child(2)) else {
            return Vec::new();
        };
        let op = find_operator_text(node, source);

        // Function on one side, numeric literal on the other.
        let (call, literal, call_on_left) = if left.kind() == "function_call" {
            match numeric_literal(right, source) {
                Some(v) => (left, v, true),
                None => return Vec::new(),
            }
        } else if right.kind() == "function_call" {
            match numeric_literal(left, source) {
                Some(v) => (right, v, false),
                None => return Vec::new(),
            }
        } else {
            return Vec::new();
        };
        let Some(name) = extract_call_name(call, source) else {
            return Vec::new();
        };
        if !LOGICAL_FUNCTIONS.contains(&name) {
            return Vec::new();
        }

        let eval = |v: f64| {
            if call_on_left {
                compare(&op, v, literal)
            } else {
                compare(&op, literal, v)
            }
        };
        let (Some(r0), Some(r1)) = (eval(0.0), eval(1.0)) else {
            return Vec::new();
        };
        if r0 != r1 {
            return Vec::new();
        }

        let (rule_id, word) = if r0 {
            ("CTRUE", "true")
        } else {
            ("CFALSE", "false")
        };
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id,
            message: format!(
                "This logical comparison always returns {word}. Did you mean to use {op} to evaluate function argument: {name}(...{op}...)?"
            ),
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

    #[test]
    fn cfalse_and_ctrue_follow_the_probed_functions() {
        // Probed: CFALSE for `isreal`, `isequal` and `any` comparisons with
        // constants outside 0..1; nothing for `isempty`, `ischar`, `islogical`
        // or `ismember`, nor `isempty` with `>` or `~=`.
        for src in [
            "B = isreal(A) == -1;\n",
            "B = any(A) == 2;\n",
            "B = isequal(A, 1) == 2;\n",
            "B = all(A) >= 2;\n",
            "B = all(A) < 0;\n",
            "B = isnan(A) == -1;\n",
            "B = isinf(A) >= 2;\n",
            "B = isrow(A) == 2;\n",
            "B = ismatrix(A) == 2;\n",
        ] {
            let diags = node_diags(src);
            assert!(has_id(&diags, "CFALSE"), "{src:?}: {diags:?}");
        }
        for src in [
            "B = isempty(A) == 2;\n",
            "B = ischar(A) == 2;\n",
            "B = islogical(A) == 2;\n",
            "B = ismember(A, 1) == 2;\n",
            "B = isempty(A) > 1;\n",
            "B = isempty(A) ~= 2;\n",
        ] {
            let diags = node_diags(src);
            assert!(
                !has_id(&diags, "CFALSE") && !has_id(&diags, "CTRUE"),
                "{src:?}: {diags:?}"
            );
        }
    }

    #[test]
    fn ctrue_fires_on_any_le_one() {
        let diags = node_diags("B = any(A) <= 1;\n");
        assert!(has_id(&diags, "CTRUE"), "got: {diags:?}");
        let msg = &diags.iter().find(|d| d.rule_id == "CTRUE").unwrap().message;
        // MATLAB prints the operator, not the argument: "any(...<=...)".
        assert!(msg.contains("any(...<=...)"), "got: {msg}");
        assert!(!msg.contains("VAR_"), "got: {msg}");
    }

    #[test]
    fn cfalse_fires_on_any_gt_one() {
        let diags = node_diags("B = any(A) > 1;\n");
        assert!(has_id(&diags, "CFALSE"), "got: {diags:?}");
    }

    #[test]
    fn cfalse_fires_with_literal_on_left() {
        let diags = node_diags("B = -1 == isreal(A);\n");
        assert!(has_id(&diags, "CFALSE"), "got: {diags:?}");
    }

    #[test]
    fn no_fire_when_comparison_is_meaningful() {
        let diags = node_diags("B = any(A) == 1;\n");
        assert!(
            !has_id(&diags, "CTRUE") && !has_id(&diags, "CFALSE"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn no_fire_on_literal_conditions() {
        let diags = node_diags("if true\n    x = 1;\nend\nwhile 0\nend\n");
        assert!(
            !has_id(&diags, "CTRUE") && !has_id(&diags, "CFALSE"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn no_fire_on_non_logical_function() {
        let diags = node_diags("B = max(A) <= 1;\n");
        assert!(
            !has_id(&diags, "CTRUE") && !has_id(&diags, "CFALSE"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn ctrue_message_names_operator_and_function() {
        // MATLAB: "... Did you mean to use >= to evaluate function argument: any(...>=...)?"
        let diags = node_diags("B = any(A) >= 0;\n");
        let d = diags.iter().find(|d| d.rule_id == "CTRUE").expect("CTRUE");
        assert_eq!(
            d.message,
            "This logical comparison always returns true. Did you mean to use >= to evaluate function argument: any(...>=...)?"
        );
    }
}
