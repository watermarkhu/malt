use super::*;

impl GoodPracticesEngine {
    /// MHERM: a product `X'*A*X` or `X*A*X'`, which MATLAB evaluates left to
    /// right as `(X'*A)*X` and so does not guarantee a Hermitian result.
    /// Parenthesizing the inner product, `X'*(A*X)`, or the outer one, keeps it
    /// symmetric. A plain `X'*X` is already Hermitian and is not reported.
    pub(crate) fn check_mherm(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MHERM") || node.kind() != "binary_operator" {
            return Vec::new();
        }
        let Some((left, right)) = star_operands(node, source) else {
            return Vec::new();
        };
        // Only the outermost product of an unparenthesized chain: `(a*b)*c`.
        if left.kind() != "binary_operator" {
            return Vec::new();
        }
        let Some((first, _)) = star_operands(left, source) else {
            return Vec::new();
        };

        // One end is `X'` and the other is `X` (or also `X'`/`X.'`).
        let base = |n: Node| -> Option<String> {
            let target = if n.kind() == "postfix_operator" {
                first_named_child(n)?
            } else {
                n
            };
            (target.kind() == "identifier").then(|| node_text(target, source).to_string())
        };
        let (Some(a), Some(b)) = (base(first), base(right)) else {
            return Vec::new();
        };
        let one_transposed =
            (first.kind() == "postfix_operator") != (right.kind() == "postfix_operator");
        if a != b || !one_transposed {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "MHERM",
            message: format!(
                "Parenthesize the multiplication of {a} and its transpose to ensure the result is Hermitian."
            ),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

/// The operands of a `*` or `.*` binary operator.
fn star_operands<'a>(node: Node<'a>, source: &str) -> Option<(Node<'a>, Node<'a>)> {
    if node.kind() != "binary_operator" {
        return None;
    }
    let op = node.child(1)?;
    if !matches!(node_text(op, source), "*" | ".*") {
        return None;
    }
    Some((node.child(0)?, node.child(2)?))
}

#[cfg(test)]
mod tests {
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn mherm(src: &str) -> bool {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        has_id(&lint_nodes(&*e, src), "MHERM")
    }

    #[test]
    fn mherm_fires_on_x_transpose_a_x() {
        assert!(mherm("y = x' * A * x;\n"));
        assert!(mherm("y = x * A * x';\n"));
    }

    #[test]
    fn mherm_ignores_plain_products_and_parenthesized_chains() {
        assert!(!mherm("y = r' * r;\n"));
        assert!(!mherm("y = x' * (A * x);\n"));
        assert!(!mherm("y = a * b * c;\n"));
        assert!(!mherm("y = x' * A * z;\n"));
    }
}
