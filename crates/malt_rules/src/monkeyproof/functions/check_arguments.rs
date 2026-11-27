//! FCNS-2: limit the number of inputs and outputs of a function.
//!
//! Reports a function, or a method declaration, with more inputs or more outputs than
//! `max_inputs` and `max_outputs` (default 5 each). `varargin` and `varargout` count as
//! one each, and so does every `~` placeholder.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use super::common::{function_name, inputs, name_range, outputs};
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_2";

/// Parameters of `MP_FCNS_2`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    max_inputs: usize,
    max_outputs: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            max_inputs: 5,
            max_outputs: 5,
        }
    }
}

/// Run the check over a file.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    visit(ctx.tree.root_node(), ctx.source, &params, out);
}

fn visit(node: Node, source: &str, params: &Params, out: &mut Vec<Diagnostic>) {
    if matches!(node.kind(), "function_definition" | "function_signature") {
        let name = function_name(node, source);
        let n_in = inputs(node).len();
        let n_out = outputs(node).len();
        if n_in > params.max_inputs {
            out.push(finding(
                source,
                ID,
                name_range(node),
                format!(
                    "Function {name} has {n_in} inputs, more than {}. Group related inputs in a struct.",
                    params.max_inputs
                ),
            ));
        }
        if n_out > params.max_outputs {
            out.push(finding(
                source,
                ID,
                name_range(node),
                format!(
                    "Function {name} has {n_out} outputs, more than {}. Group related outputs in a struct.",
                    params.max_outputs
                ),
            ));
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, source, params, out);
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::run;
    use super::ID;

    #[test]
    fn six_inputs_fire_at_the_name() {
        let d = run(ID, "function r = f(a, b, c, d, e, g)\nend\n", "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (1, 14));
        assert!(run(ID, "function r = f(a, b, c, d, e)\nend\n", "").is_empty());
    }

    #[test]
    fn six_outputs_fire() {
        let d = run(ID, "function [a, b, c, d, e, g] = f()\nend\n", "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert!(run(ID, "function [a, b, c, d, e] = f()\nend\n", "").is_empty());
    }

    #[test]
    fn method_declarations_count_and_parameters_apply() {
        let src = "classdef C\nmethods\nr = m(obj, a, b)\nend\nend\n";
        assert!(run(ID, src, "").is_empty());
        assert_eq!(
            run(ID, src, "[lint.rules.MP_FCNS_2]\nmax_inputs = 2\n").len(),
            1
        );
        let src = "function f(a, ~, varargin)\nend\n";
        assert_eq!(
            run(ID, src, "[lint.rules.MP_FCNS_2]\nmax_inputs = 2\n").len(),
            1
        );
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(ID, "function [a, = f(\n", "");
    }
}
