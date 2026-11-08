//! Command-form deployment checks: MCCD, MCHLP, MCPRD.

use super::*;

impl DeploymentEngine {
    /// Check a `command` node for deployment-restricted commands.
    pub(crate) fn check_command<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let cmd_name = match extract_command_name(node, source) {
            Some(n) => n,
            None => return diags,
        };

        // MCCD: cd command
        if self.is_enabled("MCCD") && cmd_name == "cd" {
            diags.push(make_diag("MCCD", node));
        }

        // MCHLP: help / doc as commands
        if self.is_enabled("MCHLP") && (cmd_name == "help" || cmd_name == "doc") {
            diags.push(make_diag("MCHLP", node));
        }

        // MCPRD: addpath / rmpath as commands
        if self.is_enabled("MCPRD") && (cmd_name == "addpath" || cmd_name == "rmpath") {
            diags.push(make_diag("MCPRD", node));
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::super::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- MCCD ----------------------------------------------------------------

    #[test]
    fn mccd_fires_on_cd_command() {
        let src = "cd /tmp\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCCD"), "got: {diags:?}");
    }

    #[test]
    fn mccd_no_fire_on_regular_command() {
        let src = "ls\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MCCD"), "got: {diags:?}");
    }

    // -- MCPRD ---------------------------------------------------------------

    #[test]
    fn mcprd_fires_on_rmpath_command() {
        let src = "rmpath src\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCPRD"), "got: {diags:?}");
    }

    // -- MCHLP ---------------------------------------------------------------

    #[test]
    fn mchlp_fires_on_doc_command() {
        let src = "doc plot\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCHLP"), "got: {diags:?}");
    }
}
