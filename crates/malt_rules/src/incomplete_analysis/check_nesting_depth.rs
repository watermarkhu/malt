//! NOSPC: File too complex (maximum nesting depth).

use super::*;

impl IncompleteAnalysisEngine {
    /// NOSPC: File too complex (nesting) — uses the maximum of all nesting depths.
    pub(crate) fn check_nesting_depth(
        &self,
        metrics: &TreeMetrics,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let max_depth = metrics
            .max_paren_depth
            .max(metrics.max_function_depth)
            .max(metrics.max_statement_depth);
        // Use the most restrictive threshold as a combined depth limit.
        let combined_limit = self
            .config
            .max_paren_depth
            .min(self.config.max_function_depth)
            .min(self.config.max_statement_depth);
        if max_depth > combined_limit {
            diagnostics.push(Diagnostic {
                rule_id: "NOSPC",
                message: "The file is too complex to analyze. Refactor the code to improve code maintainability. For example, reduce the nesting level of conditions or functions.".to_string(),
                severity: Severity::Error,
                byte_range: 0..source.len().min(1),
                line: 1,
                column: 1,
                fix: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{engine, engine_with};
    use crate::test_util::{has_id, lint_file};

    // -- NOSPC: nesting depth -----------------------------------------------

    #[test]
    fn nospc_fires_on_deep_nesting() {
        let engine = engine_with("max_statement_depth = 2");
        let diags = lint_file(
            &*engine,
            "if x\n    if x\n        if x\n            y = 1;\n        end\n    end\nend\n",
        );
        assert!(has_id(&diags, "NOSPC"), "got: {diags:?}");
    }

    #[test]
    fn nospc_ok_on_shallow_nesting() {
        let diags = lint_file(&*engine(), "if x\n    y = 1;\nend\n");
        assert!(!has_id(&diags, "NOSPC"), "got: {diags:?}");
    }
}
