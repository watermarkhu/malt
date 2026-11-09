use super::*;

/// Functions that always return a logical (`true`/`false`) value.
const LOGICAL_FUNCTIONS: &[&str] = &[
    "any",
    "all",
    "isempty",
    "isnan",
    "isinf",
    "isfinite",
    "isequal",
    "isfield",
    "isreal",
    "islogical",
    "ischar",
    "isnumeric",
    "iscell",
    "isstruct",
    "isscalar",
    "isvector",
    "ismember",
    "strcmp",
    "strcmpi",
    "isa",
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

        let args = collect_call_args(call, source).join(", ");
        let other = node_text(if call_on_left { right } else { left }, source).trim();
        let inner = if call_on_left {
            format!("{args}{op}{other}")
        } else {
            format!("{other}{op}{args}")
        };
        let (rule_id, word) = if r0 {
            ("CTRUE", "true")
        } else {
            ("CFALSE", "false")
        };
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id,
            message: format!(
                "This logical comparison always returns {word}. Did you mean to use {name} to evaluate function argument: {name}(...{inner}...)?"
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
    fn ctrue_fires_on_any_le_one() {
        let diags = node_diags("B = any(A) <= 1;\n");
        assert!(has_id(&diags, "CTRUE"), "got: {diags:?}");
        let msg = &diags.iter().find(|d| d.rule_id == "CTRUE").unwrap().message;
        assert!(msg.contains("any(...A<=1...)"), "got: {msg}");
        assert!(!msg.contains("VAR_"), "got: {msg}");
    }

    #[test]
    fn cfalse_fires_on_any_gt_one() {
        let diags = node_diags("B = any(A) > 1;\n");
        assert!(has_id(&diags, "CFALSE"), "got: {diags:?}");
    }

    #[test]
    fn cfalse_fires_with_literal_on_left() {
        let diags = node_diags("B = 2 < isempty(A);\n");
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
}
