//! CFIG check: configuration command-form calls with missing arguments.

use super::*;

impl ConfigIssuesEngine {
    /// CFIG: Check command-form config calls for missing arguments.
    pub(crate) fn check_cfig(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        if self.is_check_disabled("CFIG") {
            return Vec::new();
        }

        let node = ctx.node;
        if node.kind() != "command" {
            return Vec::new();
        }

        // Extract command name from first child.
        let name_node = match node.child(0) {
            Some(n) if n.kind() == "command_name" => n,
            _ => return Vec::new(),
        };

        let cmd_name = &ctx.source[name_node.start_byte()..name_node.end_byte()];

        if !CONFIG_COMMANDS.contains(&cmd_name) {
            return Vec::new();
        }

        // Command-form calls to toolbox functions should have at least one argument.
        // If there are no further children after the command name, flag it.
        let child_count = node.named_child_count();
        if child_count <= 1 {
            let start = node.start_position();
            return vec![Diagnostic {
                rule_id: "CFIG",
                message: format!(
                    "The Code Analyzer settings file, VAR_FILE, has an error on line {}.",
                    start.row + 1
                ),
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: start.row + 1,
                column: start.column + 1,
                fix: None,
            }];
        }

        Vec::new()
    }
}

#[cfg(test)]
mod tests {

    use crate::config_issues::tests::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- CFIG: configuration command issue (negative only) ------------------

    #[test]
    fn cfig_ok_with_command_args() {
        let diags = lint_nodes(
            &*engine(),
            "matlab.addons.toolbox.installToolbox myToolbox.mlappinstall\n",
        );
        assert!(!has_id(&diags, "CFIG"), "got: {diags:?}");
    }

    #[test]
    fn cfig_ok_on_plain_command() {
        let diags = lint_nodes(&*engine(), "cd myFolder\n");
        assert!(!has_id(&diags, "CFIG"), "got: {diags:?}");
    }
}
