//! DEEPC: Block comments nested too deeply.

use super::*;

impl IncompleteAnalysisEngine {
    /// Scan source text for nested block comments (DEEPC).
    ///
    /// MATLAB block comments are `%{ ... %}`. Nesting is not allowed — a
    /// second `%{` inside an open block comment is an error.
    pub(crate) fn check_nested_block_comments(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut depth: usize = 0;
        let mut byte_offset = 0;

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed == "%{" {
                depth += 1;
                if depth > 1 {
                    diagnostics.push(Diagnostic {
                        rule_id: "DEEPC",
                        message: "Block comments are nested too deeply.".to_string(),
                        severity: Severity::Error,
                        byte_range: byte_offset..byte_offset + line.len(),
                        line: line_idx + 1,
                        column: 1,
                        fix: None,
                    });
                }
            } else if trimmed == "%}" {
                depth = depth.saturating_sub(1);
            }
            byte_offset += line.len() + 1;
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::super::engine;
    use crate::test_util::{has_id, lint_file};

    // -- DEEPC: nested block comments ---------------------------------------

    #[test]
    fn deepc_fires_on_nested_block_comments() {
        let diags = lint_file(&*engine(), "%{\n%{\n%}\n%}\n");
        assert!(has_id(&diags, "DEEPC"), "got: {diags:?}");
    }

    #[test]
    fn deepc_ok_on_single_block_comment() {
        let diags = lint_file(&*engine(), "%{\ncomment\n%}\n");
        assert!(!has_id(&diags, "DEEPC"), "got: {diags:?}");
    }
}
