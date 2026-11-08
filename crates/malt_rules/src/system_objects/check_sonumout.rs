//! SONUMOUT: `release()`/`reset()` called with output arguments on a System object.

use super::*;

impl SystemObjectsEngine {
    /// Check SONUMOUT: `release()`/`reset()` called with output arguments.
    pub(crate) fn check_sonumout(
        &self,
        node: tree_sitter::Node,
        method_name: &str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // SONUMOUT: Calling release()/reset() with output arguments
        if self.is_enabled("SONUMOUT")
            && (method_name == "release" || method_name == "reset")
            && has_output_assignment(node)
        {
            diags.push(make_diag("SONUMOUT", node));
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    #[test]
    fn sonumout_release_with_output_fires() {
        let diags = lint_nodes(&*engine(), "out = release();\n");
        assert!(has_id(&diags, "SONUMOUT"), "got: {diags:?}");
    }

    #[test]
    fn sonumout_reset_with_output_fires() {
        let diags = lint_nodes(&*engine(), "out = reset();\n");
        assert!(has_id(&diags, "SONUMOUT"), "got: {diags:?}");
    }

    #[test]
    fn sonumout_release_as_statement_does_not_fire() {
        let diags = lint_nodes(&*engine(), "release(obj);\n");
        assert!(!has_id(&diags, "SONUMOUT"), "got: {diags:?}");
    }

    #[test]
    fn sonumout_step_with_output_does_not_fire() {
        let diags = lint_nodes(&*engine(), "out = step();\n");
        assert!(!has_id(&diags, "SONUMOUT"), "got: {diags:?}");
    }
}
