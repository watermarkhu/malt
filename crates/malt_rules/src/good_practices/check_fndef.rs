use super::*;

impl GoodPracticesEngine {
    /// FNDEF: Function not defined at expected location.
    pub(crate) fn check_fndef(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("FNDEF") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let mut diagnostics = Vec::new();

        // In a function file, check that the main function name matches the file name.
        if let Some(main_func) = meta.main_function() {
            // We don't have the file name in FileContext, so skip file-name matching.
            // Instead check that local functions come after the main function.
            for local in &meta.local_functions {
                if local.line < main_func.line {
                    diagnostics.push(Diagnostic {
                        rule_id: "FNDEF",
                        message: format!(
                            "Function name {} is known to MATLAB by its file name: VAR_FILE.",
                            local.name
                        ),
                        severity: Severity::Warning,
                        byte_range: local.byte_range.clone(),
                        line: local.line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}
