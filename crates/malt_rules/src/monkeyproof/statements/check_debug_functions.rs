//! MP_STAT_23: functions that interact with the MATLAB debugger.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;

use super::{callee_uses, name_list, variables};
use crate::monkeyproof::finding;

/// The functions of the debugger the check reports by default.
const DEFAULT: &[&str] = &[
    "keyboard", "dbstop", "dbclear", "dbcont", "dbstep", "dbquit", "dbup", "dbdown", "dbstack",
    "dbstatus", "dbtype",
];

/// Parameters of `MP_STAT_23`.
#[derive(Debug, Clone, Deserialize, Default)]
struct Params {
    /// The debugger functions to report. Replaces the default list when set.
    functions: Option<Vec<String>>,
}

/// Flag a call, a bare statement or a command of a debugger function.
///
/// A name the file also uses as a variable is skipped, and so is a method or a
/// qualified name (`obj.keyboard`).
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_STAT_23");
    let names = name_list(params.functions, DEFAULT);
    let root = ctx.tree.root_node();
    let vars = variables(root, ctx.source);
    for usage in callee_uses(root, ctx.source) {
        if names.contains(usage.name) && !vars.contains(usage.name) {
            out.push(finding(
                ctx.source,
                "MP_STAT_23",
                usage.name_node.byte_range(),
                format!("Remove the debugging function {}.", usage.name),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_calls_bare_names_and_commands() {
        let source = "keyboard\ndbstop in foo\nx = 1;\ndbstack();\ndbcont;\n";
        assert_eq!(
            hits("MP_STAT_23", source, ""),
            vec![(1, 1), (2, 1), (4, 1), (5, 1)]
        );
    }

    #[test]
    fn leaves_lookalikes_alone() {
        let source = "obj.keyboard();\nkeyboard2();\nx = dbstopper(1);\n";
        assert!(hits("MP_STAT_23", source, "").is_empty());
    }

    #[test]
    fn a_variable_of_that_name_is_not_a_call() {
        assert!(hits("MP_STAT_23", "dbstack = 3;\ny = dbstack(1);\n", "").is_empty());
    }

    #[test]
    fn the_list_is_a_parameter() {
        let toml = "[lint.rules.MP_STAT_23]\nfunctions = [\"clc\"]\n";
        assert_eq!(hits("MP_STAT_23", "clc\nkeyboard\n", toml), vec![(1, 1)]);
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_23", "keyboard(\nif\n", "");
    }
}
