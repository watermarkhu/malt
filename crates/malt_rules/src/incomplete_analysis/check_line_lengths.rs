//! TEXTL: Text too long — a line exceeds the maximum line length.

use super::*;

impl IncompleteAnalysisEngine {
    /// Check maximum line length (TEXTL).
    pub(crate) fn check_line_lengths(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut byte_offset = 0;

        for (line_idx, line) in source.lines().enumerate() {
            let char_len = line.chars().count();
            if char_len > self.config.max_line_length {
                diagnostics.push(Diagnostic {
                    rule_id: "TEXTL",
                    message: "Text is too long for MATLAB to parse.".to_string(),
                    severity: Severity::Error,
                    byte_range: byte_offset..byte_offset + line.len(),
                    line: line_idx + 1,
                    column: 1,
                    fix: None,
                });
            }
            // +1 for newline character.
            byte_offset += line.len() + 1;
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::super::{engine, engine_with};
    use crate::test_util::{has_id, lint_file};

    // -- TEXTL: line too long -----------------------------------------------

    #[test]
    fn textl_fires_on_long_line() {
        let engine = engine_with("max_line_length = 10");
        let diags = lint_file(&*engine, "x = 1234567890;\n");
        assert!(has_id(&diags, "TEXTL"), "got: {diags:?}");
    }

    #[test]
    fn textl_ok_on_short_line() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "TEXTL"), "got: {diags:?}");
    }
}
