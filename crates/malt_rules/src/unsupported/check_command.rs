//! Node-level `command` dispatch for the unsupported features engine.
//!
//! Fires the IMPKG check for `import` commands.

use super::*;

impl UnsupportedEngine {
    /// Check a `command` node for unsupported commands.
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

        // IMPKG: import command
        if self.is_enabled("IMPKG") && cmd_name == "import" {
            diags.push(make_diag("IMPKG", node));
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    // -- IMPKG ---------------------------------------------------------------

    #[test]
    fn impkg_fires_on_import_command() {
        let src = "import pkg.sub.*;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "IMPKG"), "got: {diags:?}");
    }

    #[test]
    fn impkg_no_fire_on_regular_command() {
        let src = "disp('hello');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "IMPKG"), "got: {diags:?}");
    }
}
