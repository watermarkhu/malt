//! FCNS-8: limit nesting depth.
//!
//! The first `if`, `for`, `parfor`, `while`, `switch` or `try` in a function is level
//! one, one inside it level two, and so on. A `case` or `catch` branch is not a level of
//! its own. Each statement that sits one level past `max_depth` (default 5) is reported,
//! the ones inside it are not. A nested function starts counting again from zero.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use super::common::head_range;
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_8";

/// Parameters of `MP_FCNS_8`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    max_depth: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self { max_depth: 5 }
    }
}

/// Run the check over a file.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    visit(ctx.tree.root_node(), 0, &params, ctx.source, out);
}

fn visit(node: Node, depth: usize, params: &Params, source: &str, out: &mut Vec<Diagnostic>) {
    let mut depth = depth;
    match node.kind() {
        "function_definition" => depth = 0,
        "if_statement" | "for_statement" | "while_statement" | "switch_statement"
        | "try_statement" => {
            depth += 1;
            if depth == params.max_depth + 1 {
                out.push(finding(
                    source,
                    ID,
                    head_range(node, source),
                    format!(
                        "This block is nested {depth} levels deep, more than {}. Move the inner code to a function.",
                        params.max_depth
                    ),
                ));
                return;
            }
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, depth, params, source, out);
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::run;
    use super::ID;

    fn nest(levels: usize) -> String {
        let mut s = String::from("function f(x)\n");
        for _ in 0..levels {
            s.push_str("if x\n");
        }
        s.push_str("y = 1;\n");
        for _ in 0..levels {
            s.push_str("end\n");
        }
        s.push_str("end\n");
        s
    }

    #[test]
    fn six_levels_fire_once_at_the_sixth() {
        let d = run(ID, &nest(7), "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (7, 1));
        assert!(run(ID, &nest(5), "").is_empty());
    }

    #[test]
    fn the_depth_is_a_parameter() {
        assert_eq!(
            run(ID, &nest(3), "[lint.rules.MP_FCNS_8]\nmax_depth = 2\n").len(),
            1
        );
    }

    #[test]
    fn all_block_kinds_count_and_cases_do_not() {
        let src = "function f(x)\nfor i = 1:2\nwhile x\nswitch x\ncase 1\ntry\nif x\nfor j = 1:2\nend\nend\ncatch\nend\nend\nend\nend\nend\n";
        let d = run(ID, src, "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].line, 8);
    }

    #[test]
    fn nested_functions_count_from_zero() {
        let src = format!(
            "function f(x)\nif x\nif x\nif x\n{}end\nend\nend\nend\n",
            nest(3).replacen("function f(x)", "function g(x)", 1)
        );
        assert!(run(ID, &src, "").is_empty(), "{:?}", run(ID, &src, ""));
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(
            ID,
            "if x\nif y\nfor\n",
            "[lint.rules.MP_FCNS_8]\nmax_depth = 1\n",
        );
    }
}
