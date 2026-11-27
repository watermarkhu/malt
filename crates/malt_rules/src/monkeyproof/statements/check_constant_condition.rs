//! MP_STAT_12: conditions that are always true or always false.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use super::{call_arguments, call_identifier, text, walk};
use crate::monkeyproof::finding;

/// Parameters of `MP_STAT_12`.
#[derive(Debug, Clone, Deserialize, Default)]
struct Params {
    /// Accept `while true` (and `while 1`), the usual way to write an endless loop.
    #[serde(default)]
    allow_endless_loop: bool,
}

/// The truth value of a condition that does not depend on any variable, if it has one.
///
/// Literals, `true` and `false` (also as `true()` and `false()`), `~`, and `&&` / `||`
/// where one side decides the outcome (`x > 0 || true`).
fn truth(node: Node, source: &str) -> Option<bool> {
    match node.kind() {
        "parenthesis" => truth(node.named_child(0)?, source),
        "number" => {
            let raw = text(node, source).replace(['d', 'D'], "e");
            raw.parse::<f64>().ok().map(|value| value != 0.0)
        }
        "identifier" => match text(node, source) {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        "function_call" if call_arguments(node).is_empty() => {
            match call_identifier(node, source)? {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            }
        }
        "not_operator" => truth(node.named_child(0)?, source).map(|value| !value),
        "boolean_operator" => {
            let left = truth(node.child_by_field_name("left")?, source);
            let right = truth(node.child_by_field_name("right")?, source);
            if operator_is(node, source, "||") {
                match (left, right) {
                    (Some(true), _) | (_, Some(true)) => Some(true),
                    (Some(false), Some(false)) => Some(false),
                    _ => None,
                }
            } else {
                match (left, right) {
                    (Some(false), _) | (_, Some(false)) => Some(false),
                    (Some(true), Some(true)) => Some(true),
                    _ => None,
                }
            }
        }
        _ => None,
    }
}

/// Whether the boolean operator node uses `op`.
fn operator_is(node: Node, source: &str, op: &str) -> bool {
    super::operator(node, source) == Some(op)
}

/// Flag the condition of an `if`, `elseif` or `while` whose value is fixed.
///
/// Conditions that mention a variable are only flagged when one side of `&&` / `||`
/// already decides the result. The finding covers the condition.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_STAT_12");
    walk(ctx.tree.root_node(), |node, _| {
        let is_while = match node.kind() {
            "if_statement" | "elseif_clause" => false,
            "while_statement" => true,
            _ => return,
        };
        let Some(condition) = node.child_by_field_name("condition") else {
            return;
        };
        let Some(value) = truth(condition, ctx.source) else {
            return;
        };
        if is_while && value && params.allow_endless_loop {
            return;
        }
        out.push(finding(
            ctx.source,
            "MP_STAT_12",
            condition.byte_range(),
            format!(
                "This condition is always {}, remove it.",
                if value { "true" } else { "false" }
            ),
        ));
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_constant_conditions() {
        let source = "if true\n  a = 1;\nelseif 0\n  a = 2;\nend\nwhile 1\n  break\nend\nif x > 0 || true\nend\nif false && x\nend\n";
        assert_eq!(
            hits("MP_STAT_12", source, ""),
            vec![(1, 4), (3, 8), (6, 7), (9, 4), (11, 4)]
        );
    }

    #[test]
    fn leaves_real_conditions_alone() {
        let source = "if x\nend\nif x || false\nend\nif x && true\nend\nif isempty(true)\nend\nif n > 0\nend\n";
        assert!(hits("MP_STAT_12", source, "").is_empty());
    }

    #[test]
    fn negation_and_calls_are_folded() {
        let source = "if ~false\nend\nif true()\nend\nif (0)\nend\n";
        assert_eq!(hits("MP_STAT_12", source, "").len(), 3);
    }

    #[test]
    fn the_endless_loop_can_be_allowed() {
        let toml = "[lint.rules.MP_STAT_12]\nallow_endless_loop = true\n";
        assert!(hits("MP_STAT_12", "while true\n  break\nend\n", toml).is_empty());
        assert_eq!(hits("MP_STAT_12", "while false\nend\n", toml).len(), 1);
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_12", "if\nwhile (\n", "");
    }
}
