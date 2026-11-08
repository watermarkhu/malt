//! MDEEP: Parentheses/brackets nested too deeply.

use super::*;

impl IncompleteAnalysisEngine {
    /// MDEEP: Parentheses/brackets nested too deeply.
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

    // -- MDEEP: parens/brackets too deeply nested ---------------------------

    #[test]
    fn mdeep_fires_on_deep_parens() {
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
