//! TMMSG: Too many diagnostics generated.

use super::*;

impl IncompleteAnalysisEngine {
    /// TMMSG: Too many diagnostics generated.
    pub(crate) fn check_diagnostic_count(&self, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        // NOTE: This counts diagnostics emitted by THIS rule only. The linter
        // may also enforce this at a higher level across all rules.
        if diagnostics.len() > self.config.max_diagnostics {
            diagnostics.push(Diagnostic {
                rule_id: "TMMSG",
                message: "More than 10,000 Code Analyzer messages were generated, leading to some being deleted.".to_string(),
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

    // -- TMMSG: too many diagnostics ----------------------------------------

    #[test]
    fn tmmsg_fires_when_diagnostics_exceed() {
        // Force at least one diagnostic (a too-long line), then check TMMSG.
        let engine = engine_with("max_diagnostics = 0\nmax_line_length = 4");
        let diags = lint_file(&*engine, "x = 12345;\n");
        assert!(has_id(&diags, "TMMSG"), "got: {diags:?}");
    }

    #[test]
    fn tmmsg_ok_with_default_limit() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "TMMSG"), "got: {diags:?}");
    }
}
