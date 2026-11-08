//! MXASET: File too complex to analyze (node count).

use super::*;

impl IncompleteAnalysisEngine {
    /// MXASET: File too complex to analyze (node count).
    pub(crate) fn check_node_count(
        &self,
        metrics: &TreeMetrics,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if metrics.node_count > self.config.max_node_count {
            diagnostics.push(Diagnostic {
                rule_id: "MXASET",
                message: "The file is too complex to analyze. Simplify the code to improve code maintainability. For example, reduce the number of operations in expressions.".to_string(),
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

    // -- MXASET: file too complex (node count) ------------------------------

    #[test]
    fn mxaset_fires_when_node_count_exceeds() {
        let engine = engine_with("max_node_count = 1");
        let diags = lint_file(&*engine, "x = 1;\n");
        assert!(has_id(&diags, "MXASET"), "got: {diags:?}");
    }

    #[test]
    fn mxaset_ok_with_default_limit() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "MXASET"), "got: {diags:?}");
    }
}
