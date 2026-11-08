use super::*;

impl CodegenEngine {
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

        // EMIMP: import command
        if self.is_enabled("EMIMP") && cmd_name == "import" {
            diags.push(make_diag("EMIMP", node));
        }

        // EMFCN: unsupported commands (clear, load, etc.)
        if self.is_enabled("EMFCN") && UNSUPPORTED_FUNCTIONS.contains(&cmd_name) {
            diags.push(make_diag("EMFCN", node));
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    // -- EMIMP ----------------------------------------------------------------

    #[test]
    fn emimp_fires_on_import_command() {
        let src = "import pkg.fcn;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMIMP"), "got: {diags:?}");
    }

    #[test]
    fn emimp_not_fire_on_plain_statement() {
        let src = "x = 1;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMIMP"), "got: {diags:?}");
    }
}
