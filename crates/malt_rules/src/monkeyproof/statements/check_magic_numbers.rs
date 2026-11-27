//! MP_STAT_2: magic numbers.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use super::{text, variables};
use crate::monkeyproof::finding;

/// Parameters of `MP_STAT_2`.
#[derive(Debug, Clone, Deserialize, Default)]
struct Params {
    /// Values that are not magic, compared by absolute value. Default `[0, 1, 2]`.
    allowed: Option<Vec<f64>>,
    /// Also report the numbers inside an index (`x(5)`) of a name the file assigns.
    /// Off unless set to `true`.
    #[serde(default)]
    check_indices: bool,
}

/// Nodes whose numbers are declarations, not expressions.
const DECLARATIONS: &[&str] = &[
    "arguments_statement",
    "properties",
    "enumeration",
    "events",
    "attributes",
    "dimensions",
    "parfor_options",
    "class_property",
    "validation_functions",
];

/// The value of a number literal, `None` for hexadecimal and binary literals.
fn value_of(raw: &str) -> Option<f64> {
    let digits = raw.trim_end_matches(['i', 'j']).replace(['d', 'D'], "e");
    digits.parse().ok()
}

/// Whether `node` is made of literals only: numbers, strings, `true` and `false`,
/// operators, matrices, cells and ranges. Such an expression is a definition.
fn is_literal(node: Node, source: &str) -> bool {
    match node.kind() {
        "number" | "string" | "string_content" => true,
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

/// Flag a number literal in an expression whose value is not in the allowed list.
///
/// Not reported: the right-hand side of an assignment made of literals only (that is
/// the definition the standard asks for), `case` labels, declarations in `arguments`,
/// `properties` and `enumeration` blocks, and, unless `check_indices` is on, the
/// subscripts of a name the file assigns. A call with a number is reported, so
/// `x(5)` on a name the file does not assign reads as a call and is reported.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_STAT_2");
    let allowed = params.allowed.unwrap_or_else(|| vec![0.0, 1.0, 2.0]);
    let root = ctx.tree.root_node();
    let vars = variables(root, ctx.source);

    // Each entry is a node and whether its numbers are exempt.
    let mut stack: Vec<(Node, bool)> = vec![(root, false)];
    while let Some((node, exempt)) = stack.pop() {
        if node.kind() == "number" && !exempt {
            let value = value_of(text(node, ctx.source));
            if value.is_some_and(|v| !allowed.iter().any(|a| a.abs() == v.abs())) {
                out.push(finding(
                    ctx.source,
                    "MP_STAT_2",
                    node.byte_range(),
                    "Define this number as a named constant before using it.",
                ));
            }
        }
        let whole = exempt || DECLARATIONS.contains(&node.kind());
        let right = (node.kind() == "assignment")
            .then(|| node.child_by_field_name("right"))
            .flatten();
        let condition = (node.kind() == "case_clause")
            .then(|| node.child_by_field_name("condition"))
            .flatten();
        let indexes = !params.check_indices
            && node.kind() == "function_call"
            && super::call_identifier(node, ctx.source).is_some_and(|name| vars.contains(name));
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();
        for child in children.into_iter().rev() {
            let child_exempt = whole
                || (Some(child) == right && is_literal(child, ctx.source))
                || Some(child) == condition
                || (indexes && child.kind() == "arguments");
            stack.push((child, child_exempt));
        }
    }
    out.sort_by_key(|d| d.byte_range.start);
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_numbers_in_expressions() {
        let source = "for k = 1:52\n  y = randi(13) * 7;\nend\n";
        assert_eq!(
            hits("MP_STAT_2", source, ""),
            vec![(1, 11), (2, 13), (2, 19)]
        );
    }

    #[test]
    fn small_values_and_definitions_are_fine() {
        let source = "DECK = 52;\nlimits = [0.5 1.5];\nx = 2 * y + 1 - 0;\nnames = 'abc';\n";
        assert!(hits("MP_STAT_2", source, "").is_empty());
    }

    #[test]
    fn declarations_and_case_labels_are_fine() {
        let source = "classdef A\n  properties\n    Size = 10\n  end\nend\n";
        assert!(hits("MP_STAT_2", source, "").is_empty());
        let source = "function f(x)\n  arguments\n    x (1,1) double = 5\n  end\n  switch x\n    case 7\n      y = 1;\n    otherwise\n      y = 0;\n  end\nend\n";
        assert!(hits("MP_STAT_2", source, "").is_empty());
    }

    #[test]
    fn indexing_a_variable_is_skipped_by_default() {
        let source = "v = [1 2 3];\nw = v(5) + v(2);\n";
        assert!(hits("MP_STAT_2", source, "").is_empty());
        let toml = "[lint.rules.MP_STAT_2]\ncheck_indices = true\n";
        assert_eq!(hits("MP_STAT_2", source, toml), vec![(2, 7)]);
    }

    #[test]
    fn the_allowed_values_are_a_parameter() {
        let toml = "[lint.rules.MP_STAT_2]\nallowed = [0, 1, 2, 60, 100]\n";
        let source = "t = s / 60 + 100 * f(3);\n";
        assert_eq!(hits("MP_STAT_2", source, toml), vec![(1, 22)]);
    }

    #[test]
    fn negative_values_match_the_allowed_absolute_value() {
        assert!(hits("MP_STAT_2", "y = x(end) * -1;\n", "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_2", "x = 5 +\n(3 *\n", "");
    }
}
