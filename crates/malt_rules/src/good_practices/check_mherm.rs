use super::*;
use crate::analysis::symbols::SymbolTable;

impl GoodPracticesEngine {
    /// MHERM: a product `X*X'` (or `X*X.'`) at the end of an unparenthesized
    /// chain of three or more factors, such as `A*X*X'` or `X'*X*X'`. MATLAB
    /// evaluates the chain left to right, so the result is not guaranteed to be
    /// Hermitian. Parenthesizing the last two factors, `A*(X*X')`, keeps it.
    /// MATLAB only sees names that are defined, so an undefined `X` is not
    /// reported. The diagnostic points at the factor `X` (or `X'`) before the
    /// transpose.
    pub(crate) fn check_mherm(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MHERM") || node.kind() != "binary_operator" {
            return Vec::new();
        }
        let Some((left, right)) = star_operands(node, source) else {
            return Vec::new();
        };
        // The left operand must be an unparenthesized product: `(A*X)*X'`.
        if left.kind() != "binary_operator" {
            return Vec::new();
        }
        let Some((_, factor)) = star_operands(left, source) else {
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
        let (Some(a), Some(b)) = (base(factor), base(right)) else {
            return Vec::new();
        };
        let one_transposed =
            (factor.kind() == "postfix_operator") != (right.kind() == "postfix_operator");
        if a != b || !one_transposed || !is_defined_at(node, &a, source) {
            return Vec::new();
        }

        let pos = factor.start_position();
        vec![Diagnostic {
            rule_id: "MHERM",
            message: format!(
                "Parenthesize the multiplication of {a} and its transpose to ensure the result is Hermitian."
            ),
            severity: Severity::Warning,
            byte_range: factor.start_byte()..factor.end_byte(),
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
    // Only the matrix product: `A.*X.*X'` is elementwise and MATLAB is silent (probed).
    if node_text(op, source) != "*" {
        return None;
    }
    Some((node.child(0)?, node.child(2)?))
}

/// Whether `name` is defined (a parameter or an assignment) in the scope of `node`.
fn is_defined_at(node: Node, name: &str, source: &str) -> bool {
    let table = SymbolTable::shared(
        || {
            let mut root = node;
            while let Some(parent) = root.parent() {
                root = parent;
            }
            root
        },
        source,
    );
    table
        .scope_at(node.start_byte())
        .is_some_and(|scope| scope.is_defined(name))
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
    fn mherm_fires_on_a_defined_factor_before_a_transpose_at_the_chain_end() {
        assert!(mherm("G = rand(3);\ny = A * G * G';\n"));
        assert!(mherm("G = rand(3);\ny = G' * G * G';\n"));
        assert!(mherm("G = rand(3);\ny = G * G * G.';\n"));
        assert!(mherm("G = rand(3);\ny = A * G' * G;\n"));
        assert!(mherm("x = rand(3, 1);\ny = x * x' * x;\n"));
        assert!(mherm("G = rand(3);\ny = 2 * A * G * G' + Q;\n"));
        assert!(mherm("function y = f(G, A)\ny = A * G * G';\nend\n"));
    }

    #[test]
    fn mherm_ignores_the_shapes_matlab_does_not_report() {
        // Two factors only, a transpose that is not at the end, a parenthesized
        // product, different names, and names that are not defined in the file.
        assert!(!mherm("G = rand(3);\ny = G * G';\n"));
        assert!(!mherm("G = rand(3);\ny = G' * G;\n"));
        assert!(!mherm("G = rand(3);\ny = G * G' * A;\n"));
        assert!(!mherm("G = rand(3);\ny = G * G' * G';\n"));
        assert!(!mherm("G = rand(3);\ny = (A * G) * G';\n"));
        assert!(!mherm("G = rand(3);\ny = A * (G * G');\n"));
        assert!(!mherm("G = rand(3);\ny = A * G * (G');\n"));
        assert!(!mherm("x = rand(3, 1);\ny = x' * A * x;\n"));
        assert!(!mherm("F = eye(3);\nP = F;\ny = F * P * F' + Q;\n"));
        assert!(!mherm("y = A * G * G';\n"));
    }
}

#[cfg(test)]
mod probe_p2a_tests {
    use crate::test_util::lint_nodes;
    use malt_core::Config;

    /// Lines of the `id` diagnostics that the default (mathworks) engine reports for `src`.
    fn lines(src: &str, id: &str) -> Vec<usize> {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_nodes(&*e, src)
            .iter()
            .filter(|d| d.rule_id == id)
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn mherm_reports_the_matrix_product_only_not_the_elementwise_one() {
        // Probed: `A.*X.*X'` (59) is silent; `A*X*X'` (57) is reported at the factor X.
        let elementwise = "function y = f(A, X)\ny = A.*X.*X';\nend\n";
        let product = "function y = f(A, X)\ny = A*X*X';\nend\n";
        assert!(lines(elementwise, "MHERM").is_empty());
        assert_eq!(lines(product, "MHERM"), vec![2]);
    }
}
