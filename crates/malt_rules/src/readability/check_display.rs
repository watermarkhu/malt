//! # DSPSY: `display(x)` → `disp(x)`
//!
//! `check_display` suggests `disp` over the equivalent `display` call.

use super::*;

impl ReadabilityEngine {
    /// DSPSY: `display(x)` → `disp(x)`
    pub(crate) fn check_display(&self, node: tree_sitter::Node, results: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("DSPSY") {
            return;
        }

        results.push(self.diag(
            "DSPSY",
            "'display(sprintf(...))' can usually be replaced by 'fprintf(...\\n)'.",
            node,
            None,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    // -- DSPSY ---------------------------------------------------------------

    #[test]
    fn dspsy_display_fires() {
        let diags = lint_nodes(&*engine(), "display(x);\n");
        assert!(has_id(&diags, "DSPSY"), "got: {diags:?}");
    }

    #[test]
    fn dspsy_disp_does_not_fire() {
        let diags = lint_nodes(&*engine(), "disp(x);\n");
        assert!(!has_id(&diags, "DSPSY"), "got: {diags:?}");
    }
}
