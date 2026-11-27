//! MP_STAT_7: operands of different types around one operator.

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::{operator, peel, walk};
use crate::monkeyproof::finding;

/// Whether `node` is an arithmetic operation (not `&` or `|`).
fn is_arithmetic(node: Node, source: &str) -> bool {
    node.kind() == "binary_operator"
        && operator(node, source).is_some_and(|op| {
            matches!(
                op,
                "+" | "-" | "*" | "/" | "\\" | "^" | ".*" | "./" | ".\\" | ".^" | ".+" | ".-"
            )
        })
}

/// Whether `node` is a logical operation or a comparison (a logical result).
fn is_logical(node: Node, source: &str) -> bool {
    match node.kind() {
        "comparison_operator" | "boolean_operator" | "not_operator" => true,
        "binary_operator" => operator(node, source).is_some_and(|op| matches!(op, "&" | "|")),
        _ => false,
    }
}

/// Flag an arithmetic operand of a logical operator (`a * b && c`), and a logical operand
/// of an arithmetic operator (`(a > b) * c`). Parentheses around the operand do not hide
/// it. The finding covers the operand.
///
/// Only operands that are clearly arithmetic or clearly logical are judged. A variable or
/// a call may hold either type, so it is never reported.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    walk(ctx.tree.root_node(), |node, _| {
        let (wants_logical, wants_arithmetic) = match node.kind() {
            "boolean_operator" => (true, false),
            "binary_operator" => match operator(node, ctx.source) {
                Some("&" | "|") => (true, false),
                _ if is_arithmetic(node, ctx.source) => (false, true),
                _ => return,
            },
            _ => return,
        };
        for side in ["left", "right"] {
            let Some(operand) = node.child_by_field_name(side) else {
                continue;
            };
            let inner = peel(operand);
            if wants_logical && is_arithmetic(inner, ctx.source) {
                out.push(finding(
                    ctx.source,
                    "MP_STAT_7",
                    operand.byte_range(),
                    "Do not use an arithmetic expression as an operand of a logical operator, compare it first.",
                ));
            } else if wants_arithmetic && is_logical(inner, ctx.source) {
                out.push(finding(
                    ctx.source,
                    "MP_STAT_7",
                    operand.byte_range(),
                    "Do not use a logical expression as an operand of an arithmetic operator, convert it first.",
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
    fn flags_arithmetic_inside_logical_operators() {
        let source = "d = a * b && c;\ne = x || (y + 1);\n";
        assert_eq!(hits("MP_STAT_7", source, ""), vec![(1, 5), (2, 10)]);
    }

    #[test]
    fn flags_logical_inside_arithmetic_operators() {
        let source = "d = (a > b) * c;\ne = 3 + ~flag;\nf = (a && b) - 1;\n";
        assert_eq!(hits("MP_STAT_7", source, ""), vec![(1, 5), (2, 9), (3, 5)]);
    }

    #[test]
    fn lookalikes_are_fine() {
        let source = "d = (a * b > 0) && c;\ne = a && b || c;\nf = x + y * z;\ng = a & b;\nh = isempty(x) && y;\n";
        assert!(hits("MP_STAT_7", source, "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_7", "d = a * b &&\n", "");
    }
}
