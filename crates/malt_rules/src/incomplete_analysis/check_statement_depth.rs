//! DEEPS: Statements nested too deeply.

use super::*;

impl IncompleteAnalysisEngine {
    /// DEEPS: Statements nested too deeply.
    pub(crate) fn check_statement_depth(
        &self,
        metrics: &TreeMetrics,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if metrics.max_statement_depth > self.config.max_statement_depth {
            diagnostics.push(Diagnostic {
                rule_id: "DEEPS",
                message: "Statements are nested too deeply.".to_string(),
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

    // -- DEEPS: statements nested too deeply --------------------------------

    #[test]
    fn deepe_fires_on_deep_statement_nesting() {
        let engine = engine_with("max_statement_depth = 2");
        let diags = lint_file(
            &*engine,
            "if x\n    if x\n        if x\n            y = 1;\n        end\n    end\nend\n",
        );
        assert!(has_id(&diags, "DEEPS"), "got: {diags:?}");
    }

    #[test]
    fn deeps_ok_on_shallow_statement_nesting() {
        let diags = lint_file(&*engine(), "if x\n    y = 1;\nend\n");
        assert!(!has_id(&diags, "DEEPS"), "got: {diags:?}");
    }
}
