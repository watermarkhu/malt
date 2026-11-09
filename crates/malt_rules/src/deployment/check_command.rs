//! Command-form deployment checks: MCCD, MCHLP, MCKBD, MCSVP.

use super::*;

impl DeploymentEngine {
    /// Check a `command` node for deployment-restricted commands.
    pub(crate) fn check_command<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let Some(cmd_name) = extract_command_name(node, source) else {
            return Vec::new();
        };
        let id = match cmd_name {
            "cd" => "MCCD",
            "help" => "MCHLP",
            "keyboard" => "MCKBD",
            "savepath" => "MCSVP",
            _ => return Vec::new(),
        };
        if self.is_enabled(id) {
            vec![make_diag(id, node)]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::engine;
    use crate::test_util::{has_id, lint_nodes};

    #[test]
    fn mccd_fires_on_cd_command() {
        let diags = lint_nodes(&*engine(), "cd /tmp\n");
        assert!(has_id(&diags, "MCCD"), "got: {diags:?}");
    }

    #[test]
    fn mccd_no_fire_on_regular_command() {
        let diags = lint_nodes(&*engine(), "ls\n");
        assert!(!has_id(&diags, "MCCD"), "got: {diags:?}");
    }

    #[test]
    fn mchlp_fires_on_help_command() {
        let diags = lint_nodes(&*engine(), "help plot\n");
        assert!(has_id(&diags, "MCHLP"), "got: {diags:?}");
    }

    #[test]
    fn mchlp_no_fire_on_doc_command() {
        let diags = lint_nodes(&*engine(), "doc plot\n");
        assert!(!has_id(&diags, "MCHLP"), "got: {diags:?}");
    }

    #[test]
    fn addpath_command_is_not_flagged() {
        let diags = lint_nodes(&*engine(), "rmpath src\n");
        assert!(diags.is_empty(), "got: {diags:?}");
    }
}
