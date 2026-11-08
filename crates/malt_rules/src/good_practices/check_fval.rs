use super::*;

impl GoodPracticesEngine {
    /// FVAL: Function return value not used (assigned but never read).
    pub(crate) fn check_fval(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("FVAL") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let sym = SymbolTable::build(tree, source);
        let mut diagnostics = Vec::new();

        for func in meta.functions.iter().chain(meta.local_functions.iter()) {
            // Check each output argument: is it defined but never read in the function?
            if let Some(scope) = sym.scope_at(func.byte_range.start) {
                for output_name in &func.outputs {
                    if output_name == "~" {
                        continue;
                    }
                    // Output args are always "defined" as OutputArg. Check if also assigned.
                    let assigned = scope.defs.iter().any(|d| {
                        d.name == *output_name
                            && d.kind == crate::analysis::symbols::DefKind::Assignment
                    });
                    if !assigned && !scope.is_used(output_name) {
                        diagnostics.push(Diagnostic {
                            rule_id: "FVAL",
                            message: "Calling functions using 'feval' is usually not necessary. Call the function directly instead."
                                .to_string(),
                            severity: Severity::Warning,
                            byte_range: func.byte_range.clone(),
                            line: func.line,
                            column: 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        diagnostics
    }
}
