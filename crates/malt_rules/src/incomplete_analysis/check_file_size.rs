//! MBIG: File too large — source length exceeds the internal limit.

use super::*;

impl IncompleteAnalysisEngine {
    /// MBIG: File too large.
    pub(crate) fn check_file_size(
        &self,
        source: &str,
        file_name: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if source.len() > self.config.max_file_size {
            diagnostics.push(Diagnostic {
                rule_id: "MBIG",
                message: format!("Code analysis did not complete. File {file_name} is too large."),
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

    // -- MBIG: file too large -----------------------------------------------

    #[test]
    fn mbig_fires_when_file_exceeds_limit() {
        let engine = engine_with("max_file_size = 8");
        let diags = lint_file(&*engine, "x = 12345;\n");
        assert!(has_id(&diags, "MBIG"), "got: {diags:?}");
    }

    #[test]
    fn mbig_ok_with_default_limit() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "MBIG"), "got: {diags:?}");
    }
}
