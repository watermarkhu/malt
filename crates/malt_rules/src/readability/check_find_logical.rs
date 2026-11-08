//! # LOGL: `x(find(condition))` → `x(condition)`
//!
//! `check_find_logical` flags `find` calls used as indexing arguments and
//! suggests plain logical indexing instead.

use super::*;

impl ReadabilityEngine {
    /// LOGL: `x(find(condition))` → `x(condition)`
    ///
    /// Detects `find` as an argument to array/cell indexing. Since tree-sitter
    /// cannot distinguish `f(x)` from `a(i)`, we flag `find` calls used as
    /// arguments inside any `function_call` parent, which is the typical
    /// indexing pattern.
    pub(crate) fn check_find_logical(
        &self,
        node: tree_sitter::Node,
        _source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("LOGL") {
            return;
        }

        // find() used as argument inside another function_call (array indexing)
        if let Some(parent) = node.parent() {
            // The parent of the function_call `find(...)` should be an
            // arguments node, and *its* parent should be a function_call
            if let Some(grandparent) = parent.parent() {
                if grandparent.kind() == "function_call" {
                    results.push(self.diag(
                        "LOGL",
                        "Use 'true' or 'false' instead of 'logical(1)' or 'logical(0)'.",
                        node,
                        None,
                    ));
                }
            }
        }
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

    // -- LOGL ----------------------------------------------------------------

    #[test]
    fn logl_find_in_indexing_fires() {
        let diags = lint_nodes(&*engine(), "x = m(find(c));\n");
        assert!(has_id(&diags, "LOGL"), "got: {diags:?}");
    }

    #[test]
    fn logl_find_at_statement_level_does_not_fire() {
        let diags = lint_nodes(&*engine(), "y = find(c);\n");
        assert!(!has_id(&diags, "LOGL"), "got: {diags:?}");
    }
}
