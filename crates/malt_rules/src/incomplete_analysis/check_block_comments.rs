//! DEEPC: Block comments nested too deeply.

use super::*;

/// MATLAB reports DEEPC at the eighth nested `%{`. Seven levels are fine (probed
/// with 1 to 7), and nesting of 8 to 1000 gives the same single report at the
/// eighth opener.
const DEEPC_DEPTH: usize = 8;

impl IncompleteAnalysisEngine {
    /// Scan source text for nested block comments (DEEPC).
    ///
    /// MATLAB block comments are `%{ ... %}` on lines of their own. They may
    /// nest, up to the limit above.
    pub(crate) fn check_nested_block_comments(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut depth: usize = 0;
        let mut byte_offset = 0;
        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed == "%{" {
                depth += 1;
                if depth == DEEPC_DEPTH {
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

    fn nested_comments(depth: usize) -> String {
        format!(
            "{}text\n{}x = 1;\n",
            "%{\n".repeat(depth),
            "%}\n".repeat(depth)
        )
    }

    // -- DEEPC: nested block comments ---------------------------------------

    #[test]
    fn deepc_fires_at_the_eighth_nested_block_comment() {
        // MATLAB probe: depth 8 gives one DEEPC at the eighth `%{` (line 8).
        let diags = lint_file(&*engine(), &nested_comments(8));
        let deepc: Vec<_> = diags.iter().filter(|d| d.rule_id == "DEEPC").collect();
        assert_eq!(deepc.len(), 1, "got: {diags:?}");
        assert_eq!(deepc[0].line, 8);
    }

    #[test]
    fn deepc_reported_once_for_deeper_nesting() {
        // MATLAB probe: depths 9, 10, 20, 1000 give the same single report.
        let diags = lint_file(&*engine(), &nested_comments(20));
        assert_eq!(diags.iter().filter(|d| d.rule_id == "DEEPC").count(), 1);
    }

    #[test]
    fn deepc_ok_up_to_seven_nested_block_comments() {
        // MATLAB probe: depths 1 to 7 are silent.
        let diags = lint_file(&*engine(), &nested_comments(7));
        assert!(!has_id(&diags, "DEEPC"), "got: {diags:?}");
    }

    #[test]
    fn deepc_ok_on_single_block_comment() {
        let diags = lint_file(&*engine(), "%{\ncomment\n%}\n");
        assert!(!has_id(&diags, "DEEPC"), "got: {diags:?}");
    }
}
