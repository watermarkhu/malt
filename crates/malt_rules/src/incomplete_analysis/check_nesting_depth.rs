//! NOSPC: File too complex (maximum nesting depth).

use super::*;

impl IncompleteAnalysisEngine {
    /// NOSPC: File too complex (nesting) — uses the maximum of all nesting depths.
    pub(crate) fn check_nesting_depth(
        &self,
        metrics: &TreeMetrics,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Only statement nesting counts: MATLAB reports no NOSPC for deep parentheses
        // or function nesting within the limits probed.
        let max_depth = metrics.max_statement_depth;
        let combined_limit = self.config.max_statement_depth;
        if max_depth > combined_limit {
            diagnostics.push(Diagnostic {
                rule_id: "NOSPC",
                message: "The file is too complex to analyze. Refactor the code to improve code maintainability. For example, reduce the nesting level of conditions or functions.".to_string(),
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

    // -- NOSPC: nesting depth -----------------------------------------------

    #[test]
    fn nospc_fires_on_deep_nesting() {
        let engine = engine_with("max_statement_depth = 2");
        let diags = lint_file(
            &*engine,
            "if x\n    if x\n        if x\n            y = 1;\n        end\n    end\nend\n",
        );
        assert!(has_id(&diags, "NOSPC"), "got: {diags:?}");
    }

    #[test]
    fn nospc_ok_on_240_nested_ifs() {
        // MATLAB probe: 240 nested `if` levels are not too complex.
        let source = format!("{}y = 1;\n{}", "if x\n".repeat(240), "end\n".repeat(240));
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "NOSPC"), "got: {diags:?}");
    }

    #[test]
    fn nospc_fires_on_250_nested_ifs() {
        // MATLAB probe: 250 nested `if` levels give NOSPC.
        let source = format!("{}y = 1;\n{}", "if x\n".repeat(250), "end\n".repeat(250));
        let diags = lint_file(&*engine(), &source);
        assert!(has_id(&diags, "NOSPC"), "got: {diags:?}");
    }

    #[test]
    fn nospc_ok_on_shallow_nesting() {
        let diags = lint_file(&*engine(), "if x\n    y = 1;\nend\n");
        assert!(!has_id(&diags, "NOSPC"), "got: {diags:?}");
    }
}
