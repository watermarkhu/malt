//! TMSMS/EOFER: Too many parse errors or syntax errors.

use super::*;

impl IncompleteAnalysisEngine {
    /// TMSMS: Too many parse errors.
    pub(crate) fn check_parse_errors(
        &self,
        metrics: &TreeMetrics,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // TMSMS: Too many parse errors.
        if metrics.error_count > self.config.max_parse_errors {
            diagnostics.push(Diagnostic {
                rule_id: "TMSMS",
                message: "More than 1,000 parse error messages were generated, leading to some being deleted.".to_string(),
                severity: Severity::Error,
                byte_range: 0..source.len().min(1),
                line: 1,
                column: 1,
                fix: None,
            });
        }

        // EOFER: Too many syntax errors (same as TMSMS but lower threshold).
        // Fires when error count exceeds max_parse_errors (same metric, distinct ID
        // for MATLAB compatibility).
        if metrics.error_count > 0 && metrics.error_count > self.config.max_parse_errors {
            diagnostics.push(Diagnostic {
                rule_id: "EOFER",
                message: "Code analysis did not complete. File contains too many syntax errors."
                    .to_string(),
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

    // -- TMSMS / EOFER: too many parse errors -------------------------------

    #[test]
    fn tmsms_fires_on_parse_errors() {
        let engine = engine_with("max_parse_errors = 0");
        let diags = lint_file(&*engine, "x = ;\n");
        assert!(has_id(&diags, "TMSMS"), "got: {diags:?}");
    }

    #[test]
    fn tmsms_ok_on_valid_source() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "TMSMS"), "got: {diags:?}");
    }

    #[test]
    fn eofer_fires_on_parse_errors() {
        let engine = engine_with("max_parse_errors = 0");
        let diags = lint_file(&*engine, "x = ;\n");
        assert!(has_id(&diags, "EOFER"), "got: {diags:?}");
    }

    #[test]
    fn eofer_ok_on_valid_source() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "EOFER"), "got: {diags:?}");
    }
}
