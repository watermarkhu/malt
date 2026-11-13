//! MDEEP: Parentheses/brackets nested too deeply.

use super::*;

impl IncompleteAnalysisEngine {
    /// MDEEP: Parentheses/brackets nested too deeply.
    ///
    /// MATLAB reports MDEEP from 33 nested levels on, for `(`, `[`, `{` and call
    /// arguments alike (probed: 32 levels are silent, 33 are reported).
    pub(crate) fn check_paren_depth(
        &self,
        metrics: &TreeMetrics,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if metrics.max_paren_depth > self.config.max_paren_depth {
            diagnostics.push(Diagnostic {
                rule_id: "MDEEP",
                message: "Parentheses, brackets, and braces are nested too deeply.".to_string(),
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

    fn nested_parens(open: &str, close: &str, depth: usize) -> String {
        format!("y = {}1{};\n", open.repeat(depth), close.repeat(depth))
    }

    // -- MDEEP: parens/brackets too deeply nested ---------------------------

    #[test]
    fn mdeep_fires_at_33_nested_parentheses() {
        // MATLAB probe: 33 levels of `(`, `[`, `{` and `abs(` give MDEEP; 32 do not.
        assert!(has_id(
            &lint_file(&*engine(), &nested_parens("(", ")", 33)),
            "MDEEP"
        ));
        assert!(has_id(
            &lint_file(&*engine(), &nested_parens("[", "]", 33)),
            "MDEEP"
        ));
        assert!(has_id(
            &lint_file(&*engine(), &nested_parens("{", "}", 33)),
            "MDEEP"
        ));
        let calls = format!("y = {}1{};\n", "abs(".repeat(33), ")".repeat(33));
        assert!(has_id(&lint_file(&*engine(), &calls), "MDEEP"));
    }

    #[test]
    fn mdeep_ok_at_32_nested_parentheses() {
        let diags = lint_file(&*engine(), &nested_parens("(", ")", 32));
        assert!(!has_id(&diags, "MDEEP"), "got: {diags:?}");
    }

    #[test]
    fn mdeep_fires_with_a_lowered_limit() {
        let engine = engine_with("max_paren_depth = 2");
        let diags = lint_file(&*engine, "y = ((((1))));\n");
        assert!(has_id(&diags, "MDEEP"), "got: {diags:?}");
    }

    #[test]
    fn mdeep_ok_on_shallow_parens() {
        let diags = lint_file(&*engine(), "y = (1);\n");
        assert!(!has_id(&diags, "MDEEP"), "got: {diags:?}");
    }
}
