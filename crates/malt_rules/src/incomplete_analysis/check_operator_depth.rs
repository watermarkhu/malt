//! MXASET: File too complex to analyze (binary operators nested too deeply).

use super::*;

/// MATLAB reports MXASET for an expression with this many binary operators nested
/// in one another: the chain `1+1+...+1` of 999 terms (998 operators) is reported,
/// 998 terms (997 operators) is not. The measure is depth: a balanced sum of 1000
/// leaves is silent, and so are matrices and cell literals of 10,000 elements.
const MXASET_OPERATOR_DEPTH: usize = 998;

impl IncompleteAnalysisEngine {
    /// MXASET: an expression nested too deeply through binary operators.
    pub(crate) fn check_operator_depth(
        &self,
        metrics: &TreeMetrics,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if metrics.max_operator_depth >= MXASET_OPERATOR_DEPTH {
            diagnostics.push(Diagnostic {
                rule_id: "MXASET",
                message: "The file is too complex to analyze. Simplify the code to improve code maintainability. For example, reduce the number of operations in expressions.".to_string(),
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
    use super::super::engine;
    use crate::test_util::{has_id, lint_file};

    fn chain(terms: usize) -> String {
        format!("x = {};\n", vec!["1"; terms].join("+"))
    }

    fn balanced(lo: usize, hi: usize) -> String {
        if hi - lo == 1 {
            return "1".to_string();
        }
        let mid = (lo + hi) / 2;
        format!("({}+{})", balanced(lo, mid), balanced(mid, hi))
    }

    // -- MXASET: operators nested too deeply --------------------------------

    #[test]
    fn mxaset_fires_on_chain_of_998_operators() {
        // MATLAB probe: 999 terms (998 operators) give MXASET, 998 terms do not.
        let diags = lint_file(&*engine(), &chain(999));
        assert!(has_id(&diags, "MXASET"), "got: {diags:?}");
        assert_eq!(
            diags.iter().find(|d| d.rule_id == "MXASET").unwrap().line,
            1
        );
    }

    #[test]
    fn mxaset_ok_on_chain_of_997_operators() {
        let diags = lint_file(&*engine(), &chain(998));
        assert!(!has_id(&diags, "MXASET"), "got: {diags:?}");
    }

    #[test]
    fn mxaset_ok_on_balanced_sum_of_1000_leaves() {
        // MATLAB probe: 999 operators in a balanced tree (depth 10) are silent.
        let source = format!("x = {};\n", balanced(0, 1000));
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "MXASET"), "got: {diags:?}");
    }
}
