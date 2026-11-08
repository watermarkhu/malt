use super::*;

impl GoodPracticesEngine {
    /// VALST: Function uses `nargin`/`nargout` checks instead of arguments block.
    pub(crate) fn check_valst(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("VALST") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let sym = SymbolTable::build(tree, source);
        let mut diagnostics = Vec::new();

        for func in meta.functions.iter().chain(meta.local_functions.iter()) {
            if func.has_arguments_block || func.is_abstract {
                continue;
            }

            // Check if the function uses nargin/nargout for validation.
            if let Some(scope) = sym.scope_at(func.byte_range.start) {
                let uses_nargin = scope.is_used("nargin") || scope.is_used("nargout");
                if uses_nargin {
                    diagnostics.push(Diagnostic {
                        rule_id: "VALST",
                        message: format!(
                            "{} must be the last argument in the argument list.",
                            func.name
                        ),
                        severity: Severity::Info,
                        byte_range: func.byte_range.clone(),
                        line: func.line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}
