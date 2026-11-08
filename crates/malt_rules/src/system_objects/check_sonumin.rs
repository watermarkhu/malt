//! SONUMIN: `step()` called with no arguments is suspicious for a System object.

use super::*;

impl SystemObjectsEngine {
    /// Check SONUMIN: `step()` called with no arguments.
    pub(crate) fn check_sonumin(
        &self,
        node: tree_sitter::Node,
        func_name: &str,
        method_name: &str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // SONUMIN: Check argument count for step() method
        // step() should have the System object + input signals
        if self.is_enabled("SONUMIN") && method_name == "step" {
            let arg_count = count_args(node);
            // step() with no arguments is suspicious (should have at least input)
            if arg_count == 0 && !func_name.contains('.') {
                diags.push(make_diag("SONUMIN", node));
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    #[test]
    fn sonumin_bare_step_no_args_fires() {
        let diags = lint_nodes(&*engine(), "step();\n");
        assert!(has_id(&diags, "SONUMIN"), "got: {diags:?}");
    }

    #[test]
    fn sonumin_step_with_input_does_not_fire() {
        let diags = lint_nodes(&*engine(), "step(input);\n");
        assert!(!has_id(&diags, "SONUMIN"), "got: {diags:?}");
    }
}
