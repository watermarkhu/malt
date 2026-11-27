//! MP_STAT_8: parentheses around mixed logical operators.

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::{operator, walk};
use crate::monkeyproof::finding;

/// The logical operator of `node`, if it is a logical operation: `&&`, `||`, `&` or `|`.
fn logical_operator<'s>(node: Node, source: &'s str) -> Option<&'s str> {
    if !matches!(node.kind(), "boolean_operator" | "binary_operator") {
        return None;
    }
    operator(node, source).filter(|op| matches!(*op, "&&" | "||" | "&" | "|"))
}

/// Flag a logical operation used, without parentheses, as an operand of a different
/// logical operator (`a && b || c`). The finding covers the unparenthesised operand.
///
/// Repeating one operator (`a && b && c`) needs no parentheses.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    walk(ctx.tree.root_node(), |node, _| {
        let Some(outer) = logical_operator(node, ctx.source) else {
            return;
        };
        for side in ["left", "right"] {
            let Some(operand) = node.child_by_field_name(side) else {
                continue;
            };
            if logical_operator(operand, ctx.source).is_some_and(|inner| inner != outer) {
                out.push(finding(
                    ctx.source,
                    "MP_STAT_8",
                    operand.byte_range(),
                    "Add parentheses to show the precedence of the logical operators.",
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
    fn flags_mixed_logical_operators() {
        let source = "d = a && b || c;\ne = a || b && c;\nf = a & b | c;\n";
        assert_eq!(hits("MP_STAT_8", source, ""), vec![(1, 5), (2, 10), (3, 5)]);
    }

    #[test]
    fn a_repeated_operator_and_parentheses_are_fine() {
        let source = "d = a && b && c;\ne = (a && b) || c;\nf = a || (b && c);\ng = a | b | c;\n";
        assert!(hits("MP_STAT_8", source, "").is_empty());
    }

    #[test]
    fn comparisons_are_not_logical_operators() {
        assert!(hits("MP_STAT_8", "d = a > 1 && b < 2 || c;\n", "").len() == 1);
        assert!(hits("MP_STAT_8", "d = a > 1 && b < 2;\n", "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_8", "d = a && b ||\n", "");
    }
}
