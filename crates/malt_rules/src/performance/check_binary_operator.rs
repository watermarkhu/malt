//! Node-level check for `binary_operator` nodes: MINV (`inv(A)*b`).

use super::*;

impl PerformanceEngine {
    /// Check `binary_operator` for the MINV pattern.
    pub(crate) fn check_binary_operator<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let op = extract_operator(node, source);

        // MINV: inv(A) * b or b * inv(A)
        if self.is_enabled("MINV") && op == "*" {
            if has_inv_operand(node, source) {
                diags.push(make_diag_with_fix("MINV", node, minv_fix(node, source)));
            } else if let Some(operand) = inv_variable_operand(node, source) {
                diags.push(make_diag("MINV", operand));
            }
        }

        diags
    }
}

/// `inv(A)*b` becomes `A\b` and `b*inv(A)` becomes `b/A`.
///
/// Unsafe: the solve gives the same answer up to rounding, and an exactly
/// singular `A` that `inv` warns about may behave differently. Offered when exactly one
/// operand is an `inv` call with one argument.
fn minv_fix(node: tree_sitter::Node, source: &str) -> Option<Fix> {
    let (left, right) = (node.child(0)?, node.child(2)?);
    if node.child_count() != 3 {
        return None;
    }
    fn inv_argument<'a>(
        operand: tree_sitter::Node<'a>,
        source: &str,
    ) -> Option<tree_sitter::Node<'a>> {
        if operand.kind() != "function_call"
            || call_path(operand, source, Dotted::Skip) != Some("inv")
        {
            return None;
        }
        let args = super::check_function_call::call_args(operand);
        (args.len() == 1).then(|| args[0])
    }
    let wrap = |arg: tree_sitter::Node| {
        let text = node_text(arg, source);
        if matches!(
            arg.kind(),
            "identifier" | "field_expression" | "function_call" | "cell_index" | "parenthesis"
        ) {
            text.to_string()
        } else {
            format!("({text})")
        }
    };
    let text = match (inv_argument(left, source), inv_argument(right, source)) {
        (Some(a), None) => format!("{}\\{}", wrap(a), node_text(right, source)),
        (None, Some(a)) => format!("{}/{}", node_text(left, source), wrap(a)),
        _ => return None,
    };
    Some(Fix::new(node.byte_range(), text).unsafe_fix())
}

/// A variable operand of `*` whose every reaching value is a plain `inv(...)` call
/// (`M = inv(A); M*b`), reported by MATLAB at the variable. Copies, products such as
/// `inv(A)*2` and parenthesised operands are not followed.
fn inv_variable_operand<'a>(
    node: tree_sitter::Node<'a>,
    source: &'a str,
) -> Option<tree_sitter::Node<'a>> {
    crate::analysis::symbols::children_of(node)
        .into_iter()
        .find(|child| {
            child.kind() == "identifier"
                && crate::analysis::reaching::reaching_values(
                    *child,
                    source,
                    &source[child.byte_range()],
                )
                .is_some_and(|values| {
                    values.iter().all(|v| {
                        v.kind() == "function_call"
                            && crate::analysis::callee::call_path(
                                *v,
                                source,
                                crate::analysis::callee::Dotted::Skip,
                            ) == Some("inv")
                    })
                })
        })
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{fix_of, has_id, lint_nodes, reported_without_fix};

    #[test]
    fn minv_fix_uses_backslash_and_slash() {
        let eng = engine();
        let (safe, fixed) = fix_of(&*eng, "x = inv(A) * b;\n", "MINV").unwrap();
        assert!(!safe);
        assert_eq!(fixed, "x = A\\b;\n");
        let (_, fixed) = fix_of(&*eng, "x = b' * inv(A);\n", "MINV").unwrap();
        assert_eq!(fixed, "x = b'/A;\n");
        let (_, fixed) = fix_of(&*eng, "x = inv(A + B) * c.d;\n", "MINV").unwrap();
        assert_eq!(fixed, "x = (A + B)\\c.d;\n");
        assert!(!has_id(&lint_nodes(&*eng, &fixed), "MINV"));
    }

    #[test]
    fn minv_no_fix_for_two_inverses_or_a_variable_holding_one() {
        let eng = engine();
        assert!(reported_without_fix(
            &*eng,
            "x = inv(A) * inv(B);\n",
            "MINV"
        ));
        assert!(reported_without_fix(
            &*eng,
            "function c = f(A, b)\nM = inv(A);\nc = M*b;\nend\n",
            "MINV"
        ));
    }

    // -- MINV -----------------------------------------------------------------

    #[test]
    fn minv_fires_on_inv_times_vector() {
        let src = "x = inv(A) * b;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MINV"), "got: {diags:?}");
    }

    #[test]
    fn minv_fires_on_vector_times_inv() {
        let src = "x = b * inv(A);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MINV"), "got: {diags:?}");
    }

    #[test]
    fn minv_not_fire_on_backslash_solve() {
        let src = "x = A \\ b;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MINV"), "got: {diags:?}");
    }

    #[test]
    fn minv_fires_on_variable_holding_inv() {
        // Probed (mi01, mi02, mi03, mi08, mi09, mi13): `M = inv(A); M*b` and `b*M`
        // are MINV at the variable, also inside an `if` and after a later reassignment.
        for src in [
            "function c = f(A, b)\nM = inv(A);\nc = M*b;\nend\n",
            "A = eye(2); b = [1; 2];\nM = inv(A);\nc = M*b;\n",
            "function c = f(A, b)\nM = inv(A);\nc = b*M;\nend\n",
            "function c = f(A, b)\nM = inv(A);\nif b(1) > 0\n    c = M*b;\nelse\n    c = b;\nend\nend\n",
            "function c = f(A, b)\nM = inv(A);\nc = M*b;\nM = 2;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, "MINV"), "{src:?} got: {diags:?}");
        }
    }

    #[test]
    fn minv_variable_is_silent_when_not_plainly_inv() {
        // Probed (mi05, mi06, mi10, mi11, mi14): a copy, a reassignment before the
        // product, pinv, parentheses and `.*`.
        for src in [
            "function c = f(A, b)\nM = inv(A);\nN = M;\nc = N*b;\nend\n",
            "function c = f(A, b)\nM = inv(A);\nM = A;\nc = M*b;\nend\n",
            "function c = f(A, b)\nM = pinv(A);\nc = M*b;\nend\n",
            "function c = f(A, b)\nM = inv(A);\nc = (M)*b;\nend\n",
            "function c = f(A, b)\nM = inv(A);\nc = M.*b;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "MINV"), "{src:?} got: {diags:?}");
        }
    }
}
