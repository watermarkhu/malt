//! `MP_LAYOUT_3`: operators are surrounded by spaces.

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::text::{walk, Text};

/// Operators of binary expressions that need a space on both sides.
const BINARY: &[&str] = &["+", "-", "*", ".*", "/", "./", "^", ".^"];

/// Report operators glued to an operand: `=` of assignments, `for` headers and property
/// defaults, `:` of ranges, the arithmetic operators `+ - * .* / ./ ^ .^`, the comparisons
/// and the logical `& && | ||`. A space after a unary minus is reported too.
///
/// Decisions on the edges of the standard:
///
/// - `Name=Value` in a call is never reported, with or without spaces.
/// - A `:` inside the argument list of a call or index (`x(2:end)`) is left alone, as the
///   grammar cannot tell indexing from a call. Ranges everywhere else (`1:n`, `[1:3]`,
///   `for i = 1:n`) are checked.
/// - A line break or a `...` continuation next to the operator counts as a space.
/// - Unary operators other than the minus, `.^` with a unary operand (`x .^ -1` is fine) and
///   the transpose are not operators in this sense.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let text = Text::new(ctx.source, ctx.tree.root_node());
    let bytes = ctx.source.as_bytes();
    let mut calls: Vec<usize> = Vec::new();
    walk(ctx.tree.root_node(), |node, depth| {
        while calls.last().is_some_and(|d| *d >= depth) {
            calls.pop();
        }
        match node.kind() {
            "function_call" => calls.push(depth),
            "binary_operator" | "comparison_operator" | "boolean_operator" => {
                if let Some(op) = operator(node) {
                    let name = &ctx.source[op.start_byte()..op.end_byte()];
                    let relevant = node.kind() != "binary_operator" || BINARY.contains(&name);
                    if relevant {
                        check(&text, bytes, op, out);
                    }
                }
            }
            "range" if calls.is_empty() => {
                for op in children_of_kind(node, ":") {
                    check(&text, bytes, op, out);
                }
            }
            "assignment" | "iterator" | "default_value" => {
                for op in children_of_kind(node, "=") {
                    check(&text, bytes, op, out);
                }
            }
            "unary_operator" => {
                if let Some(op) = node.child(0).filter(|op| op.kind() == "-") {
                    if matches!(bytes.get(op.end_byte()), Some(b' ' | b'\t')) {
                        out.push(text.finding(
                            "MP_LAYOUT_3",
                            op.start_byte()..op.end_byte(),
                            "Do not put a space between a unary minus and its operand.",
                        ));
                    }
                }
            }
            _ => {}
        }
    });
}

/// The operator token of a binary-like expression: its second child, when that is a token.
fn operator(node: Node) -> Option<Node> {
    node.child(1).filter(|op| !op.is_named())
}

/// The direct children of `node` of kind `kind`.
fn children_of_kind<'t>(node: Node<'t>, kind: &str) -> Vec<Node<'t>> {
    super::text::children(node)
        .into_iter()
        .filter(|child| child.kind() == kind && child.start_byte() < child.end_byte())
        .collect()
}

/// Push a finding when `op` touches an operand on either side.
fn check(text: &Text, bytes: &[u8], op: Node, out: &mut Vec<Diagnostic>) {
    let (start, end) = (op.start_byte(), op.end_byte());
    let space = |b: u8| matches!(b, b' ' | b'\t' | b'\n' | b'\r');
    let glued_before = start > 0 && !space(bytes[start - 1]);
    let glued_after = end < bytes.len() && !space(bytes[end]) && !bytes[end..].starts_with(b"...");
    if glued_before || glued_after {
        let name = &text.source[start..end];
        out.push(text.finding(
            "MP_LAYOUT_3",
            start..end,
            format!("Surround the `{name}` operator with spaces."),
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    fn columns(source: &str) -> Vec<usize> {
        let mut columns: Vec<usize> = hits("MP_LAYOUT_3", source, "")
            .iter()
            .map(|d| d.column)
            .collect();
        columns.sort();
        columns
    }

    #[test]
    fn glued_operators_are_reported_at_the_operator() {
        assert_eq!(columns("isValid=y>0&&x==-1;\n"), [8, 10, 12, 15]);
    }

    #[test]
    fn spaced_operators_are_fine() {
        assert!(columns("isValid = y > 0 && x == -1;\n").is_empty());
        assert!(columns("z = a ^ 2 + b .* c - d ./ e;\n").is_empty());
    }

    #[test]
    fn one_missing_space_is_enough() {
        assert_eq!(columns("x = a+ b;\n"), [6]);
        assert_eq!(columns("x = a +b;\n"), [7]);
    }

    #[test]
    fn unary_minus_is_not_a_binary_operator() {
        assert!(columns("x = -a;\ny = [1, -2];\nz = a .^ -1;\n").is_empty());
        assert_eq!(columns("x = - a;\n"), [5]);
    }

    #[test]
    fn name_value_pairs_are_left_alone() {
        assert!(columns("f(a, Name=3);\n").is_empty());
        assert!(columns("f(a, Name = 3);\n").is_empty());
    }

    #[test]
    fn ranges_outside_an_index_are_checked() {
        assert_eq!(columns("v = 1:n;\n"), [6]);
        assert_eq!(columns("for i = 1:n\nend\n"), [10]);
        assert_eq!(columns("for i=1 : n\nend\n"), [6]);
        assert!(columns("v = 1 : n;\n").is_empty());
    }

    #[test]
    fn colons_inside_an_index_are_left_alone() {
        assert!(columns("y = x(2:end);\nz = c{1:2};\nw = x(1:2, :);\n").is_empty());
    }

    #[test]
    fn strings_comments_and_transposes_are_not_operators() {
        assert!(columns("s = 'a+b';  % x=y\nt = a';\nu = \"a==b\";\n").is_empty());
    }

    #[test]
    fn line_breaks_count_as_spaces() {
        assert!(columns("x = a + ...\n    b;\ny = a ...\n    + b;\n").is_empty());
    }

    #[test]
    fn property_defaults_are_checked() {
        assert_eq!(
            columns("classdef A\n    properties\n        P=1\n    end\nend\n"),
            [10]
        );
    }

    #[test]
    fn broken_code_does_not_panic() {
        let _ = columns("x=(1+;\nif a==\n");
    }
}
