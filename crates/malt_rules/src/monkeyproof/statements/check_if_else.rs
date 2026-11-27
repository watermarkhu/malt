//! MP_STAT_5: an `if` without an `else`.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use super::{call_arguments, call_identifier, name_list, statements_of, text, walk};
use crate::monkeyproof::finding;

/// Functions that leave the current flow of control.
const EXIT_FUNCTIONS: &[&str] = &["error", "throw", "rethrow", "throwAsCaller"];

/// Predicates that check an input before it is used.
const INPUT_CHECKS: &[&str] = &[
    "isempty",
    "isfield",
    "isa",
    "ischar",
    "isstring",
    "isnumeric",
    "islogical",
    "iscell",
    "isstruct",
    "isscalar",
    "isvector",
    "isrow",
    "iscolumn",
    "ismatrix",
    "isreal",
    "isfloat",
    "isinteger",
    "iscellstr",
];

/// Parameters of `MP_STAT_5`.
#[derive(Debug, Clone, Deserialize, Default)]
struct Params {
    /// Functions that stop the flow of control. An `if` whose body only calls these (or
    /// returns) is error handling and needs no `else`. Replaces the default list when set.
    exit_functions: Option<Vec<String>>,
    /// Predicates that, applied to an input of the function, make the `if` input handling.
    /// Replaces the default list when set.
    input_checks: Option<Vec<String>>,
}

/// The inputs of the function around `node`.
fn enclosing_inputs<'s>(node: Node, source: &'s str) -> Vec<&'s str> {
    let mut current = node.parent();
    while let Some(ancestor) = current {
        if ancestor.kind() == "function_definition" {
            let mut cursor = ancestor.walk();
            let args = ancestor
                .children(&mut cursor)
                .find(|child| child.kind() == "function_arguments");
            return args
                .map(|args| {
                    statements_of(args)
                        .into_iter()
                        .filter(|arg| arg.kind() == "identifier")
                        .map(|arg| text(arg, source))
                        .collect()
                })
                .unwrap_or_default();
        }
        current = ancestor.parent();
    }
    Vec::new()
}

/// Whether the condition looks at the inputs of the function: `nargin`, `nargout`,
/// `exist(...)`, or an input predicate applied to an input.
fn is_input_condition(
    condition: Node,
    source: &str,
    inputs: &[&str],
    checks: &std::collections::HashSet<String>,
) -> bool {
    let mut found = false;
    walk(condition, |node, _| match node.kind() {
        "identifier" if matches!(text(node, source), "nargin" | "nargout") => found = true,
        "function_call" => {
            if let Some(name) = call_identifier(node, source) {
                if name == "exist" {
                    found = true;
                } else if checks.contains(name) {
                    let first = call_arguments(node).first().map(|arg| super::peel(*arg));
                    if first.is_some_and(|arg| {
                        arg.kind() == "identifier" && inputs.contains(&text(arg, source))
                    }) {
                        found = true;
                    }
                }
            }
        }
        _ => {}
    });
    found
}

/// Whether the body does nothing but leave: `return`, `error(...)` and the like.
fn only_exits(body: Option<Node>, source: &str, exits: &std::collections::HashSet<String>) -> bool {
    let Some(body) = body else { return false };
    let statements = statements_of(body);
    !statements.is_empty()
        && statements.iter().all(|statement| match statement.kind() {
            "return_statement" => true,
            "function_call" => {
                call_identifier(*statement, source).is_some_and(|n| exits.contains(n))
            }
            "command" => statement
                .child(0)
                .is_some_and(|name| exits.contains(text(name, source))),
            _ => false,
        })
}

/// Flag an `if` that has neither `elseif` nor `else`. The finding covers the `if` keyword.
///
/// An `if` that only errors or returns, and an `if` that checks the inputs of the
/// function, need no `else`. An `if` with an `elseif` is left to `MP_STAT_24`.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_STAT_5");
    let exits = name_list(params.exit_functions, EXIT_FUNCTIONS);
    let checks = name_list(params.input_checks, INPUT_CHECKS);
    walk(ctx.tree.root_node(), |node, _| {
        if node.kind() != "if_statement" {
            return;
        }
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();
        if children
            .iter()
            .any(|c| matches!(c.kind(), "else_clause" | "elseif_clause"))
        {
            return;
        }
        let body = children.iter().find(|c| c.kind() == "block").copied();
        if only_exits(body, ctx.source, &exits) {
            return;
        }
        if let Some(condition) = node.child_by_field_name("condition") {
            let inputs = enclosing_inputs(node, ctx.source);
            if is_input_condition(condition, ctx.source, &inputs, &checks) {
                return;
            }
        }
        out.push(finding(
            ctx.source,
            "MP_STAT_5",
            children
                .first()
                .map_or(node.byte_range(), |c| c.byte_range()),
            "Add an else branch to this if, even when it only holds a comment.",
        ));
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_an_if_without_else() {
        let source = "if x > 0\n  saveData()\nend\n";
        assert_eq!(hits("MP_STAT_5", source, ""), vec![(1, 1)]);
    }

    #[test]
    fn leaves_complete_ifs_and_elseif_chains_alone() {
        let source =
            "if x\n  a = 1;\nelse\n  a = 2;\nend\nif x\n  a = 1;\nelseif y\n  a = 2;\nend\n";
        assert!(hits("MP_STAT_5", source, "").is_empty());
    }

    #[test]
    fn error_handling_needs_no_else() {
        let source = "if x < 0\n  error('Pkg:neg', 'negative')\nend\nif y\n  return\nend\nif z\n  rethrow(ME)\nend\n";
        assert!(hits("MP_STAT_5", source, "").is_empty());
    }

    #[test]
    fn an_error_next_to_other_work_still_needs_else() {
        let source = "if x < 0\n  disp(x)\n  error('Pkg:neg', 'negative')\nend\n";
        assert_eq!(hits("MP_STAT_5", source, "").len(), 1);
    }

    #[test]
    fn input_handling_needs_no_else() {
        let source = "function f(a, b)\n  if nargin < 2\n    b = 1;\n  end\n  if isempty(a)\n    a = 0;\n  end\n  if exist('c', 'var')\n    c = 2;\n  end\nend\n";
        assert!(hits("MP_STAT_5", source, "").is_empty());
    }

    #[test]
    fn a_predicate_on_a_local_variable_is_not_input_handling() {
        let source = "function f(a)\n  v = a + 1;\n  if isempty(v)\n    v = 0;\n  end\nend\n";
        assert_eq!(hits("MP_STAT_5", source, "").len(), 1);
    }

    #[test]
    fn the_exit_functions_are_a_parameter() {
        let toml = "[lint.rules.MP_STAT_5]\nexit_functions = [\"abort\"]\n";
        let source = "if x\n  abort()\nend\nif y\n  error('a:b', 'c')\nend\n";
        assert_eq!(hits("MP_STAT_5", source, toml), vec![(4, 1)]);
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_5", "if\nif x\n", "");
    }
}
