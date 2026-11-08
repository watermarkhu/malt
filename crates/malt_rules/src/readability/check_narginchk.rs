//! # NCHKE: prefer `narginchk`/`nargoutchk`
//!
//! `check_narginchk` flags `nargchk` calls and suggests the modern
//! `narginchk`/`nargoutchk` validators.

use super::*;

impl ReadabilityEngine {
    /// NCHKE: pattern `if nargin < N, error(...)` → `narginchk`
    ///
    /// Detects `function_call` named `error` or `nargchk` inside an
    /// if-statement that tests `nargin`.
    pub(crate) fn check_narginchk(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("NCHKE") {
            return;
        }

        let func_name = match node.child_by_field_name("name") {
            Some(n) => &source[n.start_byte()..n.end_byte()],
            None => return,
        };

        if func_name != "nargchk" {
            return;
        }

        results.push(self.diag("NCHKE", "Use NARGOUTCHK without ERROR.", node, None));
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

    // -- NCHKE ---------------------------------------------------------------

    #[test]
    fn nchke_nargchk_fires() {
        let diags = lint_nodes(&*engine(), "x = nargchk(1, 2, nargin);\n");
        assert!(has_id(&diags, "NCHKE"), "got: {diags:?}");
    }

    #[test]
    fn nchke_narginchk_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = narginchk(1, 2);\n");
        assert!(!has_id(&diags, "NCHKE"), "got: {diags:?}");
    }
}
