//! MP_STAT_26: command syntax.

use std::collections::HashSet;

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;

use super::walk;
use crate::analysis::command::Command;
use crate::monkeyproof::finding;

/// Parameters of `MP_STAT_26`.
#[derive(Debug, Clone, Deserialize, Default)]
struct Params {
    /// Command names to accept, for example `["hold", "format"]`.
    #[serde(default)]
    allow: Vec<String>,
}

/// Flag a statement in command syntax with at least one argument (`hold on`,
/// `format long`, `close all`). The finding covers the statement.
///
/// A name on its own (`clc`, `tic`) is a call without parentheses and belongs to
/// `MP_STAT_3`. Shell escapes (`!ls`) are left to the security checks.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_STAT_26");
    let allowed: HashSet<&str> = params.allow.iter().map(String::as_str).collect();
    walk(ctx.tree.root_node(), |node, _| {
        if node.kind() != "command" {
            return;
        }
        let Some(command) = Command::from_node(node, ctx.source) else {
            return;
        };
        if !command.is_command()
            || command.is_shell_escape()
            || command.argument_nodes.is_empty()
            || allowed.contains(command.name)
        {
            return;
        }
        out.push(finding(
            ctx.source,
            "MP_STAT_26",
            node.byte_range(),
            format!(
                "Call {} with parentheses instead of command syntax.",
                command.name
            ),
        ));
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_command_syntax() {
        let source = "hold on\nformat long\nx = 1;\nclose all;\n";
        assert_eq!(hits("MP_STAT_26", source, ""), vec![(1, 1), (2, 1), (4, 1)]);
    }

    #[test]
    fn leaves_calls_and_lone_names_alone() {
        let source = "hold(\"on\")\nclc\ntic\nx = 3;\ny = x';\n!ls -la\n";
        assert!(hits("MP_STAT_26", source, "").is_empty());
    }

    #[test]
    fn names_can_be_allowed() {
        let toml = "[lint.rules.MP_STAT_26]\nallow = [\"hold\"]\n";
        assert_eq!(
            hits("MP_STAT_26", "hold on\nformat long\n", toml),
            vec![(2, 1)]
        );
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_26", "hold (\nformat 'long\n", "");
    }
}
