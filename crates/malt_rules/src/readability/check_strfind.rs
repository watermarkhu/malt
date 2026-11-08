//! # STRCLFH: `strfind` → `contains`
//!
//! `check_strfind` suggests `contains` for presence checks that currently use
//! `strfind`.

use super::*;

impl ReadabilityEngine {
    /// STRCLFH: `strfind` → `contains`
    pub(crate) fn check_strfind(
        &self,
        node: tree_sitter::Node,
        _source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("STRCLFH") {
            return;
        }

        results.push(self.diag(
            "STRCLFH",
            "For readability, use '~contains(str1, str2)' instead of 'cellfun(@isempty, strfind(str1, str2))'.",
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

    // -- STRCLFH -------------------------------------------------------------

    #[test]
    fn strclfh_strfind_fires() {
        let diags = lint_nodes(&*engine(), "x = strfind(a, b);\n");
        assert!(has_id(&diags, "STRCLFH"), "got: {diags:?}");
    }

    #[test]
    fn strclfh_contains_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = contains(a, b);\n");
        assert!(!has_id(&diags, "STRCLFH"), "got: {diags:?}");
    }
}
