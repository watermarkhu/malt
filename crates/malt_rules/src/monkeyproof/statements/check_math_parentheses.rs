//! MP_STAT_9: parentheses around mixed arithmetic operators.

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::{operator, walk};
use crate::monkeyproof::finding;

/// The arithmetic operator of `node`, if it is an arithmetic operation.
fn arithmetic_operator<'s>(node: Node, source: &'s str) -> Option<&'s str> {
    if node.kind() != "binary_operator" {
        return None;
    }
    operator(node, source).filter(|op| {
        matches!(
            *op,
            "+" | "-" | "*" | "/" | "\\" | "^" | ".*" | "./" | ".\\" | ".^" | ".+" | ".-"
        )
    })
}

/// Flag an arithmetic operation used, without parentheses, as an operand of a different
/// arithmetic operator (`a * b + c`, `f / 2 * g`). The finding covers the
/// unparenthesised operand.
///
/// Repeating one operator (`a * b * c`) needs no parentheses.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    walk(ctx.tree.root_node(), |node, _| {
        let Some(outer) = arithmetic_operator(node, ctx.source) else {
            return;
        };
        for side in ["left", "right"] {
            let Some(operand) = node.child_by_field_name(side) else {
                continue;
            };
            if arithmetic_operator(operand, ctx.source).is_some_and(|inner| inner != outer) {
                out.push(finding(
                    ctx.source,
                    "MP_STAT_9",
                    operand.byte_range(),
                    "Add parentheses to show the precedence of the arithmetic operators.",
                ));
            }
        }
    });
    out.sort_by_key(|d| d.byte_range.start);
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_mixed_arithmetic_operators() {
        let source = "d = a * b + c;\nh = f / 2 * g;\n";
        assert_eq!(hits("MP_STAT_9", source, ""), vec![(1, 5), (2, 5)]);
    }

    #[test]
    fn a_repeated_operator_and_parentheses_are_fine() {
        let source =
            "d = a * b * c;\ne = (a * b) + c;\nf = a + (b * c);\ng = a + b + c;\nh = x.^2;\n";
        assert!(hits("MP_STAT_9", source, "").is_empty());
    }

    #[test]
    fn each_mixed_pair_is_reported() {
        let source = "d = a + b * c - e;\n";
        assert_eq!(hits("MP_STAT_9", source, "").len(), 2);
    }

    #[test]
    fn logical_operators_do_not_count() {
        assert!(hits("MP_STAT_9", "d = a * b & c;\n", "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_9", "d = a * b +\n", "");
    }
}
