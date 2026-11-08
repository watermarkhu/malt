//! EOFMI: Incomplete file — last node is ERROR or MISSING.

use super::*;

impl IncompleteAnalysisEngine {
    /// EOFMI: Incomplete file (last node is ERROR or MISSING).
    pub(crate) fn check_incomplete_file(
        &self,
        metrics: &TreeMetrics,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if metrics.last_node_is_error {
            diagnostics.push(Diagnostic {
                rule_id: "EOFMI",
                message: "Invalid syntax at end of file. File is incomplete.".to_string(),
                severity: Severity::Error,
                byte_range: source.len().saturating_sub(1)..source.len(),
                line: source.lines().count().max(1),
                column: 1,
                fix: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::engine;
    use crate::test_util::{has_id, lint_file};

    // -- EOFMI: incomplete file ---------------------------------------------

    #[test]
    fn eofmi_fires_on_incomplete_file() {
        let diags = lint_file(&*engine(), "x = 1; 2");
        assert!(has_id(&diags, "EOFMI"), "got: {diags:?}");
    }

    #[test]
    fn eofmi_ok_on_complete_file() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "EOFMI"), "got: {diags:?}");
    }
}
