//! DEEPN: Functions nested too deeply.

use super::*;

impl IncompleteAnalysisEngine {
    /// DEEPN: Functions nested too deeply.
    pub(crate) fn check_function_depth(
        &self,
        metrics: &TreeMetrics,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if metrics.max_function_depth > self.config.max_function_depth {
            diagnostics.push(Diagnostic {
                rule_id: "DEEPN",
                message: "Functions are nested too deeply.".to_string(),
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

    // -- DEEPN: functions nested too deeply ---------------------------------

    #[test]
    fn deepn_fires_on_deep_function_nesting() {
        let engine = engine_with("max_function_depth = 2");
        let diags = lint_file(
            &*engine,
            "function a()\n    function b()\n        function c()\n        end\n    end\nend\n",
        );
        assert!(has_id(&diags, "DEEPN"), "got: {diags:?}");
    }

    #[test]
    fn deepn_ok_on_single_function() {
        let diags = lint_file(&*engine(), "function foo()\nend\n");
        assert!(!has_id(&diags, "DEEPN"), "got: {diags:?}");
    }
}
