//! MP_STAT_3: a call without input arguments written without parentheses.

use std::collections::HashSet;

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use super::{callee_uses, name_list, text, variables, walk, UseKind};
use crate::monkeyproof::finding;

/// Well-known functions that are usually called without inputs. Constants such as `pi`,
/// `eps`, `Inf` and `NaN` are not on the list: they read as values, not as calls.
const DEFAULT: &[&str] = &[
    "rand",
    "randn",
    "tic",
    "toc",
    "clock",
    "now",
    "cputime",
    "date",
    "pwd",
    "computer",
    "version",
    "mfilename",
    "gcf",
    "gca",
    "figure",
    "clc",
    "clf",
    "drawnow",
    "beep",
    "pause",
];

/// Parameters of `MP_STAT_3`.
#[derive(Debug, Clone, Deserialize, Default)]
struct Params {
    /// The well-known functions to report. Replaces the default list when set.
    functions: Option<Vec<String>>,
    /// Also report the functions defined in the same file. On unless set to `false`.
    local_functions: Option<bool>,
}

/// Whether an identifier at this place is read as a value, so a function name there is a
/// call without parentheses.
fn is_value_position(node: Node, parent: Node) -> bool {
    let field_is = |name: &str| parent.child_by_field_name(name) == Some(node);
    match parent.kind() {
        "binary_operator"
        | "boolean_operator"
        | "comparison_operator"
        | "not_operator"
        | "unary_operator"
        | "postfix_operator"
        | "parenthesis"
        | "range"
        | "row" => true,
        "assignment" => field_is("right"),
        "if_statement" | "elseif_clause" | "while_statement" | "switch_statement"
        | "case_clause" => field_is("condition"),
        "lambda" => field_is("expression"),
        "iterator" => parent.named_child(0) != Some(node),
        "arguments" => parent.parent().is_some_and(|p| p.kind() == "function_call"),
        _ => false,
    }
}

/// The names of the functions defined in the file.
fn local_function_names<'s>(root: Node, source: &'s str) -> HashSet<&'s str> {
    let mut names = HashSet::new();
    walk(root, |node, _| {
        if node.kind() == "function_definition" {
            if let Some(name) = node.child_by_field_name("name") {
                names.insert(text(name, source));
            }
        }
    });
    names
}

/// Flag the name of a known function used as a value or as a statement with no
/// parentheses after it (`x = rand * 2`, `tic`).
///
/// Only the names from the `functions` list and the functions defined in the same file
/// are trusted to be calls. A name the file also uses as a variable is skipped.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_STAT_3");
    let root = ctx.tree.root_node();
    let vars = variables(root, ctx.source);
    let mut known: HashSet<&str> = HashSet::new();
    let listed = name_list(params.functions, DEFAULT);
    let local = if params.local_functions.unwrap_or(true) {
        local_function_names(root, ctx.source)
    } else {
        HashSet::new()
    };
    let is_function =
        |name: &str| (listed.contains(name) || local.contains(name)) && !vars.contains(name);
    for name in listed
        .iter()
        .map(String::as_str)
        .chain(local.iter().copied())
    {
        if is_function(name) {
            known.insert(name);
        }
    }
    if known.is_empty() {
        return;
    }
    let message = |name: &str| format!("Add empty parentheses to the call of {name}.");
    for usage in callee_uses(root, ctx.source) {
        if usage.kind == UseKind::Bare && usage.statement && known.contains(usage.name) {
            out.push(finding(
                ctx.source,
                "MP_STAT_3",
                usage.name_node.byte_range(),
                message(usage.name),
            ));
        }
    }
    walk(root, |node, parent| {
        let Some(parent) = parent else { return };
        if node.kind() != "identifier" {
            return;
        }
        let name = text(node, ctx.source);
        if known.contains(name) && is_value_position(node, parent) {
            out.push(finding(
                ctx.source,
                "MP_STAT_3",
                node.byte_range(),
                message(name),
            ));
        }
    });
    out.sort_by_key(|d| d.byte_range.start);
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_known_functions_without_parentheses() {
        let source = "x = rand * 2;\ntic\nt = toc;\nif rand > 0.5\nend\n";
        assert_eq!(
            hits("MP_STAT_3", source, ""),
            vec![(1, 5), (2, 1), (3, 5), (4, 4)]
        );
    }

    #[test]
    fn leaves_calls_handles_and_constants_alone() {
        let source =
            "x = rand() * 2;\nf = @rand;\ny = pi + eps;\ns.rand = 3;\nz = obj.tic;\nw = rand(3);\n";
        assert!(hits("MP_STAT_3", source, "").is_empty());
    }

    #[test]
    fn flags_functions_of_the_same_file() {
        let source =
            "function r = main()\n  r = helper + 1;\nend\nfunction v = helper()\n  v = 3;\nend\n";
        assert_eq!(hits("MP_STAT_3", source, ""), vec![(2, 7)]);
    }

    #[test]
    fn a_variable_of_that_name_is_not_a_call() {
        let source = "rand = 4;\nx = rand * 2;\n";
        assert!(hits("MP_STAT_3", source, "").is_empty());
    }

    #[test]
    fn the_parameters_apply() {
        let toml = "[lint.rules.MP_STAT_3]\nfunctions = [\"clc\"]\nlocal_functions = false\n";
        let source = "clc\nx = rand;\nfunction v = helper()\n  v = 1;\nend\ny = helper;\n";
        assert_eq!(hits("MP_STAT_3", source, toml), vec![(1, 1)]);
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_3", "x = rand *\ntic (\n", "");
    }
}
