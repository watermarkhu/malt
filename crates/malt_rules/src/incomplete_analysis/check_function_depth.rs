//! DEEPN: Functions nested too deeply.

use super::*;

impl IncompleteAnalysisEngine {
    /// DEEPN: Functions nested too deeply.
    ///
    /// MATLAB reports the function at the deepest level, on its header line
    /// (probed: 18, 19, 20 and 40 nested functions give the report on lines 18,
    /// 19, 20 and 40).
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
                line: metrics.max_function_line,
                column: 1,
                fix: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::engine;
    use crate::test_util::{has_id, lint_file};

    /// `depth` functions, each nested in the one before, with a body and `end`s.
    fn nested_functions(depth: usize) -> String {
        let headers: String = (0..depth).map(|i| format!("function f{i}()\n")).collect();
        format!("{headers}x = 1;\n{}", "end\n".repeat(depth))
    }

    // -- DEEPN: functions nested too deeply ---------------------------------

    #[test]
    fn deepn_fires_at_the_18th_nested_function_on_its_line() {
        // MATLAB probe: 18 nested functions give DEEPN on line 18.
        let diags = lint_file(&*engine(), &nested_functions(18));
        let deepn: Vec<_> = diags.iter().filter(|d| d.rule_id == "DEEPN").collect();
        assert_eq!(deepn.len(), 1, "got: {diags:?}");
        assert_eq!(deepn[0].line, 18);
    }

    #[test]
    fn deepn_reported_on_the_deepest_function_for_deeper_nesting() {
        // MATLAB probe: 40 nested functions give DEEPN on line 40.
        let diags = lint_file(&*engine(), &nested_functions(40));
        let deepn: Vec<_> = diags.iter().filter(|d| d.rule_id == "DEEPN").collect();
        assert_eq!(deepn.len(), 1, "got: {diags:?}");
        assert_eq!(deepn[0].line, 40);
    }

    #[test]
    fn deepn_ok_up_to_17_nested_functions() {
        // MATLAB probe: 17 nested functions are silent.
        let diags = lint_file(&*engine(), &nested_functions(17));
        assert!(!has_id(&diags, "DEEPN"), "got: {diags:?}");
    }

    #[test]
    fn deepn_ok_on_single_function() {
        let diags = lint_file(&*engine(), "function foo()\nend\n");
        assert!(!has_id(&diags, "DEEPN"), "got: {diags:?}");
    }
}
