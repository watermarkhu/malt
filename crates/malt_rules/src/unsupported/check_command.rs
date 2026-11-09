//! Node-level `command` dispatch for the unsupported features engine.
//!
//! Fires the function checks for command syntax (`feature accel on`) and the
//! IMPKG / MIPKG checks for `import` commands naming internal namespaces.

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

        if cmd_name == "import" {
            let mut cursor = node.walk();
            for arg in node.children(&mut cursor) {
                if arg.kind() != "command_argument" {
                    continue;
                }
                let id = namespace_check(node_text(arg, source));
                if let Some(id) = id.filter(|id| self.is_enabled(id)) {
                    diags.push(make_diag(id, node));
                }
            }
            return diags;
        }

        let found = UNSUPPORTED_FUNCTIONS.iter().find(|(n, _)| *n == cmd_name);
        if let Some((_, id)) = found.filter(|(_, id)| self.is_enabled(id)) {
            diags.push(make_diag(id, node));
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    #[test]
    fn impkg_fires_on_internal_matlab_import() {
        let diags = lint_nodes(&*engine(), "import internal.matlab.*\n");
        assert!(has_id(&diags, "IMPKG"), "got: {diags:?}");
    }

    #[test]
    fn mipkg_fires_on_matlab_internal_import() {
        let diags = lint_nodes(&*engine(), "import matlab.internal.foo\n");
        assert!(has_id(&diags, "MIPKG"), "got: {diags:?}");
    }

    #[test]
    fn import_of_regular_package_does_not_fire() {
        let diags = lint_nodes(&*engine(), "import pkg.sub.*;\n");
        assert!(
            !has_id(&diags, "IMPKG") && !has_id(&diags, "MIPKG"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn featud_fires_on_command_syntax() {
        let diags = lint_nodes(&*engine(), "feature accel on\n");
        assert!(has_id(&diags, "FEATUD"), "got: {diags:?}");
    }
}
