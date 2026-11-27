//! MP_STAT_27: functions that access or depend on other workspaces.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;

use super::{callee_uses, name_list, string_value, variables, UseKind};
use crate::monkeyproof::finding;

/// The functions reported wherever they are called.
const DEFAULT: &[&str] = &[
    "evalin",
    "assignin",
    "inputname",
    "who",
    "whos",
    "evalc",
    "clc",
];

/// Parameters of `MP_STAT_27`.
#[derive(Debug, Clone, Deserialize, Default)]
struct Params {
    /// The functions reported wherever they are called. Replaces the default list when set.
    functions: Option<Vec<String>>,
    /// Also report the calls that only depend on the workspace in some forms:
    /// `exist(name, "var")`, `load` without an output, `clear` and `clearvars`
    /// without arguments or with `all`. On unless set to `false`.
    special_cases: Option<bool>,
}

/// The text of an argument word: string literals and command words.
fn is_word(word: Option<&str>, wanted: &[&str]) -> bool {
    word.is_some_and(|word| wanted.contains(&word))
}

/// Flag the use of a function that reads or writes another workspace.
///
/// A name the file also uses as a variable is skipped.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_STAT_27");
    let special = params.special_cases.unwrap_or(true);
    let names = name_list(params.functions, DEFAULT);
    let root = ctx.tree.root_node();
    let vars = variables(root, ctx.source);
    for usage in callee_uses(root, ctx.source) {
        if vars.contains(usage.name) {
            continue;
        }
        // The first two arguments as plain words, whichever way they were written.
        let word = |index: usize| -> Option<&str> {
            match usage.kind {
                UseKind::Command => usage.words.get(index).copied(),
                UseKind::Call => usage
                    .args
                    .get(index)
                    .and_then(|arg| string_value(*arg, ctx.source)),
                UseKind::Bare => None,
            }
        };
        let arg_count = match usage.kind {
            UseKind::Call => usage.args.len(),
            UseKind::Command => usage.words.len(),
            UseKind::Bare => 0,
        };
        let flagged = names.contains(usage.name)
            || (special
                && match usage.name {
                    "exist" => word(1) == Some("var"),
                    "load" => usage.statement,
                    "clear" | "clearvars" => {
                        arg_count == 0 || is_word(word(0), &["all", "-all", "global", "-global"])
                    }
                    _ => false,
                });
        if flagged {
            out.push(finding(
                ctx.source,
                "MP_STAT_27",
                usage.node.byte_range(),
                format!(
                    "Do not use {}, it depends on a workspace outside the function.",
                    usage.name
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_the_listed_functions() {
        let source =
            "x = evalin('base', 'a');\nassignin('base', 'b', 1);\nwhos\nclc\nn = inputname(1);\n";
        assert_eq!(
            hits("MP_STAT_27", source, ""),
            vec![(1, 5), (2, 1), (3, 1), (4, 1), (5, 5)]
        );
    }

    #[test]
    fn flags_the_workspace_forms() {
        let source =
            "exist('x', 'var')\nload('data.mat')\nload data.mat\nclear\nclear all\nclearvars()\n";
        assert_eq!(hits("MP_STAT_27", source, "").len(), 6);
    }

    #[test]
    fn leaves_the_harmless_forms_alone() {
        let source = "a = exist('file.m', 'file');\ns = load('data.mat');\nclear x\nclear('y')\nb = exist('f');\nobj.who();\n";
        assert!(hits("MP_STAT_27", source, "").is_empty());
    }

    #[test]
    fn a_variable_of_that_name_is_not_a_call() {
        assert!(hits("MP_STAT_27", "who = 1;\ny = who(1);\n", "").is_empty());
    }

    #[test]
    fn the_parameters_apply() {
        let toml = "[lint.rules.MP_STAT_27]\nfunctions = [\"clc\"]\nspecial_cases = false\n";
        assert_eq!(hits("MP_STAT_27", "clc\nwho\nclear\n", toml), vec![(1, 1)]);
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_27", "evalin(\nclear all (\n", "");
    }
}
