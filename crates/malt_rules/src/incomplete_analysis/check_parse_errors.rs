//! TMSMS: Too many parse errors.

use super::*;

impl IncompleteAnalysisEngine {
    /// TMSMS: Too many parse errors.
    ///
    /// MATLAB reports TMSMS once more than 1,000 parse errors are generated (probed:
    /// 1,000 errors give 1,000 SYNER and no TMSMS; 1,001 give TMSMS as well).
    ///
    /// EOFER is not emitted: in every probe with more than 1,000 errors MATLAB
    /// reported TMSMS and never EOFER.
    pub(crate) fn check_parse_errors(
        &self,
        metrics: &TreeMetrics,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
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
    }
}

#[cfg(test)]
mod tests {
    use super::super::{engine, engine_with};
    use crate::test_util::{has_id, lint_file};

    // -- TMSMS: too many parse errors ---------------------------------------

    #[test]
    fn tmsms_fires_above_1000_parse_errors() {
        // MATLAB probe: 1,001 lines of `x = ;` give TMSMS, 1,000 lines do not.
        let diags = lint_file(&*engine(), &"x = ;\n".repeat(1001));
        assert!(has_id(&diags, "TMSMS"), "got: {diags:?}");
    }

    #[test]
    fn tmsms_ok_at_1000_parse_errors() {
        let diags = lint_file(&*engine(), &"x = ;\n".repeat(1000));
        assert!(!has_id(&diags, "TMSMS"), "got: {diags:?}");
    }

    #[test]
    fn tmsms_fires_with_a_lowered_limit() {
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
    fn eofer_is_not_emitted() {
        // MATLAB probe: 1,500 errors (lines of `x = ;`, or one statement) give TMSMS
        // and never EOFER.
        let diags = lint_file(&*engine(), &"x = ;\n".repeat(1500));
        assert!(!has_id(&diags, "EOFER"), "got: {diags:?}");
    }
}
