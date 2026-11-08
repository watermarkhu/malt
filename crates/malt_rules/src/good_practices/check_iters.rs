use super::*;

impl GoodPracticesEngine {
    /// ITERS: Loop variable shadows an outer variable.
    pub(crate) fn check_iters(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("ITERS") {
            return Vec::new();
        }

        let sym = SymbolTable::build(tree, source);
        let mut diagnostics = Vec::new();

        for (idx, scope) in sym.scopes.iter().enumerate() {
            // For each for-iterator definition, check if the same name exists
            // in a parent scope.
            for def in &scope.defs {
                if def.kind != crate::analysis::symbols::DefKind::ForIterator {
                    continue;
                }
                // Walk parent scopes.
                let mut parent_idx = scope.parent;
                while let Some(pidx) = parent_idx {
                    let parent_scope = &sym.scopes[pidx];
                    // Skip if it's the same scope index (shouldn't happen).
                    if pidx == idx {
                        break;
                    }
                    if parent_scope.is_defined(&def.name) {
                        diagnostics.push(Diagnostic {
                            rule_id: "ITERS",
                            message: "The Code Analyzer type analysis may be incorrect here."
                                .to_string(),
                            severity: Severity::Warning,
                            byte_range: def.byte_range.clone(),
                            line: def.line,
                            column: def.column,
                            fix: None,
                        });
                        break;
                    }
                    parent_idx = parent_scope.parent;
                }
            }
        }

        diagnostics
    }
}
