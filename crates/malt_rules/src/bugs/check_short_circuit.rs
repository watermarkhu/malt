use super::*;
use crate::analysis::elementwise::elementwise_operands;

/// Functions that return `true` for a scalar logical produced by `&&`/`||`.
const ALWAYS_TRUE: &[&str] = &["isscalar", "islogical", "isreal"];
/// Functions that return `false` for a scalar logical produced by `&&`/`||`.
const ALWAYS_FALSE: &[&str] = &[
    "isempty",
    "isnan",
    "isinf",
    "ischar",
    "iscell",
    "isstruct",
    "isnumeric",
    "isfloat",
    "isinteger",
];

impl BugsEngine {
    /// SHOCIRT / SHOCIRF: a predicate such as `isscalar(A && B)` applied to the
    /// result of a short-circuit operator. `A && B` / `A || B` always yield a
    /// logical scalar (or error), so the predicate result is constant.
    pub(crate) fn check_short_circuit(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "function_call" {
            return Vec::new();
        }
        let Some(name) = extract_call_name(node, source) else {
            return Vec::new();
        };
        let (rule_id, word) = if ALWAYS_TRUE.contains(&name) {
            ("SHOCIRT", "true")
        } else if ALWAYS_FALSE.contains(&name) {
            ("SHOCIRF", "false")
        } else {
            return Vec::new();
        };
        // Probed: MATLAB reports LOGEMP, not SHOCIRF, for `isempty(a && b)`.
        if self
            .config
            .disabled_checks
            .iter()
            .any(|c| c.as_str() == rule_id)
        {
            return Vec::new();
        }
        let args = arg_nodes(node);
        let [arg] = args.as_slice() else {
            return Vec::new();
        };
        if arg.kind() != "boolean_operator" {
            return Vec::new();
        }
        let op = find_operator_text(*arg, source);
        if op != "&&" && op != "||" {
            return Vec::new();
        }
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id,
            message: format!(
                "The {op} operator is unexpected because {name}(A {op} B) always returns {word}."
            ),
            severity: Severity::Error,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }

    /// LBODUP: both operands of a logical operator are identical
    /// (`A && A`, `A || A`, `A & A`, `A | A`).
    pub(crate) fn check_identical_operands(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "boolean_operator" && node.kind() != "binary_operator" {
            return Vec::new();
        }
        let op = find_operator_text(node, source);
        if !matches!(op.as_str(), "&&" | "||" | "&" | "|") {
            return Vec::new();
        }
        // `&` and `|` bind looser than comparisons in MATLAB, not in the tree:
        // compare the operands MATLAB sees (`x & x > 1` is `x` and `x > 1`).
        let (left_text, right_text) = if op == "&" || op == "|" {
            let Some((left, right)) = elementwise_operands(node, source) else {
                return Vec::new();
            };
            (left.text(source), right.text(source))
        } else {
            let (Some(left), Some(right)) = (node.child(0), node.child(2)) else {
                return Vec::new();
            };
            (node_text(left, source), node_text(right, source))
        };
        if normalize_whitespace(left_text) != normalize_whitespace(right_text) {
            return Vec::new();
        }
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "LBODUP",
            message: format!(
                "Since both operands are identical, the second operand has no effect on the '{op}' operation. This indicates a bug in the code. Change one of the operands or remove the '{op}' operation."
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
    fn shocirt_fires_on_isscalar_of_and() {
        let diags = node_diags("y = isscalar(a && b);\n");
        let d = diags
            .iter()
            .find(|d| d.rule_id == "SHOCIRT")
            .expect("SHOCIRT");
        assert_eq!(
            d.message,
            "The && operator is unexpected because isscalar(A && B) always returns true."
        );
    }

    #[test]
    fn shocirf_fires_on_isempty_of_or() {
        let diags = node_diags("y = isempty(a || b);\n");
        assert!(has_id(&diags, "SHOCIRF"), "got: {diags:?}");
    }

    #[test]
    fn shocir_no_fire_without_short_circuit() {
        let diags = node_diags("y = isscalar(a & b);\nz = isempty(a);\n");
        assert!(
            !has_id(&diags, "SHOCIRT") && !has_id(&diags, "SHOCIRF"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn lbodup_fires_on_identical_operands() {
        let diags = node_diags("y = a && a;\nz = b | b;\n");
        assert_eq!(
            diags.iter().filter(|d| d.rule_id == "LBODUP").count(),
            2,
            "got: {diags:?}"
        );
        assert!(
            diags[0].message.contains("'&&' operation"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn lbodup_no_fire_on_distinct_operands() {
        let diags = node_diags("y = a && b;\n");
        assert!(!has_id(&diags, "LBODUP"), "got: {diags:?}");
    }

    #[test]
    fn lbodup_silent_on_mis_nested_elementwise_operands() {
        // Probed: MATLAB is silent on `x & x > 1` and `x > 1 & 1`. The tree has
        // `x & x` and `1 & 1` as nodes, but MATLAB's operands differ.
        for src in [
            "if x & x > 1\n    c = 1;\nend\n",
            "if x > 1 & 1\n    c = 1;\nend\n",
        ] {
            let diags = node_diags(src);
            assert!(!has_id(&diags, "LBODUP"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn lbodup_message_quotes_the_operator() {
        let diags = node_diags("x = a && a;\n");
        let d = diags
            .iter()
            .find(|d| d.rule_id == "LBODUP")
            .expect("LBODUP");
        assert_eq!(
            d.message,
            "Since both operands are identical, the second operand has no effect on the '&&' operation. This indicates a bug in the code. Change one of the operands or remove the '&&' operation."
        );
    }
}
