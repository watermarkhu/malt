//! MP_STAT_1: constant definitions at the top of the function.

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::{statements_of, text, walk};
use crate::monkeyproof::finding;

/// Whether `name` is written as a constant: capitals, digits and underscores, with at
/// least two characters and one letter.
fn is_constant_name(name: &str) -> bool {
    name.len() >= 2
        && name.starts_with(|c: char| c.is_ascii_uppercase())
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// Whether `node` is built from literals only.
fn is_literal(node: Node, source: &str) -> bool {
    match node.kind() {
        "number" | "string" => true,
        "identifier" => matches!(text(node, source), "true" | "false"),
        "binary_operator" | "unary_operator" | "parenthesis" | "matrix" | "cell" | "row"
        | "range" | "postfix_operator" => {
            let mut cursor = node.walk();
            let all = node
                .named_children(&mut cursor)
                .filter(|child| !child.is_extra())
                .all(|child| is_literal(child, source));
            all
        }
        _ => false,
    }
}

/// The name defined when `statement` is a constant definition: `NAME = literal`.
fn constant_name<'s>(statement: Node, source: &'s str) -> Option<&'s str> {
    if statement.kind() != "assignment" {
        return None;
    }
    let left = statement.child_by_field_name("left")?;
    let right = statement.child_by_field_name("right")?;
    let name = text(left, source);
    (left.kind() == "identifier" && is_constant_name(name) && is_literal(right, source))
        .then_some(name)
}

/// Flag a constant definition that comes after other code in the body of a function.
///
/// A constant definition is an assignment of literals to a name written in capitals
/// (`MAX_SIZE = 100`). `global` and `persistent` declarations may come first. Only the
/// statements directly in the function body are looked at, not those inside loops or
/// branches, and scripts have no function body to check.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    walk(ctx.tree.root_node(), |node, _| {
        if node.kind() != "function_definition" {
            return;
        }
        let mut cursor = node.walk();
        let Some(body) = node.children(&mut cursor).find(|c| c.kind() == "block") else {
            return;
        };
        let mut seen_code = false;
        for statement in statements_of(body) {
            match statement.kind() {
                "global_operator" | "persistent_operator" => {}
                "function_definition" => break,
                _ => {
                    if let Some(name) = constant_name(statement, ctx.source) {
                        if seen_code {
                            out.push(finding(
                                ctx.source,
                                "MP_STAT_1",
                                statement.byte_range(),
                                format!("Move the constant {name} to the top of the function."),
                            ));
                        }
                    } else {
                        seen_code = true;
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_a_constant_after_code() {
        let source = "function f(x)\n  y = 3 * x;\n  MAX_Z = 100;\n  z = min(y, MAX_Z);\nend\n";
        assert_eq!(hits("MP_STAT_1", source, ""), vec![(3, 3)]);
    }

    #[test]
    fn constants_first_are_fine() {
        let source = "function f(x)\n  persistent n\n  MAX_Z = 100;\n  LIMIT = [1 2 3];\n  y = min(3 * x, MAX_Z);\nend\n";
        assert!(hits("MP_STAT_1", source, "").is_empty());
    }

    #[test]
    fn ordinary_variables_are_not_constants() {
        let source = "function f(x)\n  y = 3 * x;\n  z = 100;\n  N = x;\n  Q = 5;\nend\n";
        assert!(hits("MP_STAT_1", source, "").is_empty());
    }

    #[test]
    fn scripts_and_each_function_are_judged_apart() {
        let source = "A_B = 1;\nx = 2;\nC_D = 3;\n";
        assert!(hits("MP_STAT_1", source, "").is_empty());
        let source = "function f()\n  x = 1;\nend\nfunction g()\n  K1 = 1;\n  y = 2;\nend\n";
        assert!(hits("MP_STAT_1", source, "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_1", "function f(\n  A_B = (\n", "");
    }
}
