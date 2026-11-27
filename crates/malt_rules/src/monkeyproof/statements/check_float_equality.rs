//! MP_STAT_19: `==` and `~=` on floating-point values.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use super::{call_identifier, name_list, operator, peel, text, variables, walk};
use crate::monkeyproof::finding;

/// Functions that return floating-point values.
const DEFAULT: &[&str] = &[
    "sqrt", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "sinh", "cosh", "tanh", "exp",
    "log", "log2", "log10", "mean", "median", "std", "var", "rand", "randn", "hypot", "nthroot",
];

/// Parameters of `MP_STAT_19`.
#[derive(Debug, Clone, Deserialize, Default)]
struct Params {
    /// Functions whose result is a floating-point value. Replaces the default list when set.
    functions: Option<Vec<String>>,
}

/// Flag `==` and `~=` when an operand is a number with a fractional part, `pi`, or a call
/// of a floating-point function such as `sqrt` or `mean`. The finding covers the
/// comparison.
///
/// Types are not known from one file, so a comparison of two variables is never reported.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_STAT_19");
    let functions = name_list(params.functions, DEFAULT);
    let root = ctx.tree.root_node();
    let vars = variables(root, ctx.source);
    let is_float = |operand: Node| -> bool {
        let mut node = peel(operand);
        while node.kind() == "unary_operator" {
            match node.child_by_field_name("operand") {
                Some(inner) => node = peel(inner),
                None => return false,
            }
        }
        match node.kind() {
            "number" => text(node, ctx.source)
                .trim_end_matches(['i', 'j'])
                .replace(['d', 'D'], "e")
                .parse::<f64>()
                .is_ok_and(|value| value.fract() != 0.0),
            "identifier" => text(node, ctx.source) == "pi" && !vars.contains("pi"),
            "function_call" => call_identifier(node, ctx.source)
                .is_some_and(|name| functions.contains(name) && !vars.contains(name)),
            _ => false,
        }
    };
    walk(root, |node, _| {
        if node.kind() != "comparison_operator"
            || !matches!(operator(node, ctx.source), Some("==" | "~="))
        {
            return;
        }
        let mut cursor = node.walk();
        let operands: Vec<Node> = node
            .named_children(&mut cursor)
            .filter(|child| !child.is_extra())
            .collect();
        if operands.len() == 2 && operands.iter().any(|operand| is_float(*operand)) {
            out.push(finding(
                ctx.source,
                "MP_STAT_19",
                node.byte_range(),
                "Do not compare floating-point values with == or ~=, use a tolerance.",
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_floating_point_operands() {
        let source = "a = f(x) == sqrt(3);\nb = y ~= 0.1;\nc = z == pi;\nd = -0.5 == w;\n";
        assert_eq!(
            hits("MP_STAT_19", source, ""),
            vec![(1, 5), (2, 5), (3, 5), (4, 5)]
        );
    }

    #[test]
    fn integers_and_unknown_types_are_fine() {
        let source = "a = x == 1;\nb = x == y;\nc = x == 2.0;\nd = x < 0.5;\ne = x == s.pi;\nf = n == numel(v);\n";
        assert!(hits("MP_STAT_19", source, "").is_empty());
    }

    #[test]
    fn a_variable_named_like_a_function_is_fine() {
        let source = "mean = [1 2];\na = mean(1) == b;\n";
        assert!(hits("MP_STAT_19", source, "").is_empty());
    }

    #[test]
    fn the_functions_are_a_parameter() {
        let toml = "[lint.rules.MP_STAT_19]\nfunctions = [\"myFloat\"]\n";
        let source = "a = myFloat(1) == b;\nc = sqrt(2) == d;\n";
        assert_eq!(hits("MP_STAT_19", source, toml), vec![(1, 5)]);
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_19", "a = x ==\nb = (y ~=\n", "");
    }
}
