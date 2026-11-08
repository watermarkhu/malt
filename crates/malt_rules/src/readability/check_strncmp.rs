//! # STRCL1: `strncmp`/`strncmpi` → `startsWith`/`endsWith`
//!
//! `check_strncmp` suggests the modern `startsWith`/`endsWith` functions in
//! place of `strncmp`/`strncmpi`.

use super::*;

impl ReadabilityEngine {
    /// STRCL1: `strncmp`/`strncmpi` → `startsWith`/`endsWith`
    pub(crate) fn check_strncmp(
        &self,
        node: tree_sitter::Node,
        _source: &str,
        _func_name: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("STRCL1") {
            return;
        }

        results.push(self.diag(
            "STRCL1",
            "For readability, use '~contains(str1, str2)' instead of 'cellfun('isempty', strfind(str1, str2))'.",
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

    // -- STRCL1 --------------------------------------------------------------

    #[test]
    fn strcl1_strncmp_fires() {
        let diags = lint_nodes(&*engine(), "x = strncmp(a, b, 3);\n");
        assert!(has_id(&diags, "STRCL1"), "got: {diags:?}");
    }

    #[test]
    fn strcl1_strncmpi_fires() {
        let diags = lint_nodes(&*engine(), "x = strncmpi(a, b, 3);\n");
        assert!(has_id(&diags, "STRCL1"), "got: {diags:?}");
    }

    #[test]
    fn strcl1_starts_with_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = startsWith(a, b);\n");
        assert!(!has_id(&diags, "STRCL1"), "got: {diags:?}");
    }
}
