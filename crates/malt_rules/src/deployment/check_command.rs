//! Command-form deployment checks: MCCD, MCHLP, MCKBD, MCSVP.

use super::*;

impl DeploymentEngine {
    /// Check a `command` node for deployment-restricted commands.
    pub(crate) fn check_command<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let Some(cmd_name) = command_path(node, source) else {
            return Vec::new();
        };
        // `matlabroot` and `printdlg` alone are command-form calls with no arguments.
        let alone = node.named_child_count() == 1;
        let id = match cmd_name {
            "matlabroot" if alone => "MCMLR",
            "printdlg" if alone => "MCPRD",
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

impl DeploymentEngine {
    /// Check a bare `identifier` (`r = matlabroot;`, `f = @() keyboard;`, `fullfile(matlabroot, x)`):
    /// MATLAB reports MCMLR, MCKBD and MCPRD on the name used as a call without parentheses.
    pub(crate) fn check_identifier<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let id = match &source[node.start_byte()..node.end_byte()] {
            "matlabroot" => "MCMLR",
            "keyboard" => "MCKBD",
            "printdlg" => "MCPRD",
            _ => return Vec::new(),
        };
        if !self.is_enabled(id) {
            return Vec::new();
        }
        let Some(parent) = node.parent() else {
            return Vec::new();
        };
        let is_name_of_call = parent.kind() == "function_call"
            && parent.child_by_field_name("name").map(|n| n.id()) == Some(node.id());
        let is_assigned = parent.kind() == "assignment"
            && parent.child_by_field_name("left").map(|n| n.id()) == Some(node.id());
        if is_name_of_call
            || is_assigned
            || matches!(
                parent.kind(),
                "field_expression" | "command" | "command_name" | "function_definition"
            )
        {
            return Vec::new();
        }
        vec![make_diag(id, node)]
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
    fn bare_names_are_calls_without_parentheses() {
        for (src, id) in [
            ("matlabroot\n", "MCMLR"),
            ("r = matlabroot;\n", "MCMLR"),
            ("h = @() keyboard;\n", "MCKBD"),
            ("printdlg\n", "MCPRD"),
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, id), "{src}: got {diags:?}");
            assert_eq!(diags.iter().filter(|d| d.rule_id == id).count(), 1, "{src}");
        }
        for src in [
            "s.matlabroot = 1;\n",
            "x = obj.keyboard;\n",
            "keyboard = 3;\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(diags.is_empty(), "{src}: got {diags:?}");
        }
    }

    #[test]
    fn addpath_command_is_not_flagged() {
        let diags = lint_nodes(&*engine(), "rmpath src\n");
        assert!(diags.is_empty(), "got: {diags:?}");
    }
}
