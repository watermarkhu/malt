//! EOFMI: Incomplete file — last node is ERROR or MISSING.

use super::*;

impl IncompleteAnalysisEngine {
    /// EOFMI is not emitted. MATLAB reports nothing of the kind for a file that
    /// ends in an unfinished statement (`x = 1; 2=`, `x = 1; 2+`): it reports the
    /// parse error where it occurs (SYNER) or only the statements before it.
    /// Trailing `last_node_is_error` tree shapes are therefore not a finding.
    pub(crate) fn check_incomplete_file(
        &self,
        _metrics: &TreeMetrics,
        _source: &str,
        _diagnostics: &mut Vec<Diagnostic>,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::super::engine;
    use crate::test_util::{has_id, lint_file};

    // -- EOFMI: incomplete file ---------------------------------------------

    #[test]
    fn eofmi_not_reported_for_unfinished_statement_at_eof() {
        // MATLAB probe: `x = 1; 2=` and `x = 1; 2+` give NOPTS or SYNER, never EOFMI.
        let diags = lint_file(&*engine(), "x = 1; 2");
        assert!(!has_id(&diags, "EOFMI"), "got: {diags:?}");
    }

    #[test]
    fn eofmi_ok_on_complete_file() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "EOFMI"), "got: {diags:?}");
    }
}
