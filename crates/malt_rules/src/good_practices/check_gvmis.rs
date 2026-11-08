use super::*;

impl GoodPracticesEngine {
    /// GVMIS: Global variable used but never declared.
    pub(crate) fn check_gvmis(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("GVMIS") {
            return Vec::new();
        }

        let sym = SymbolTable::build(tree, source);
        let mut diagnostics = Vec::new();

        // Collect all global variable declarations across all scopes.
        let mut global_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for scope in &sym.scopes {
            for def in &scope.defs {
                if def.kind == crate::analysis::symbols::DefKind::Global {
                    global_names.insert(def.name.clone());
                }
            }
        }

        // For each scope, check for usages of variables that are only defined
        // as `global` in other scopes but not declared `global` in this scope.
        for scope in &sym.scopes {
            let local_globals: std::collections::HashSet<&str> = scope
                .defs
                .iter()
                .filter(|d| d.kind == crate::analysis::symbols::DefKind::Global)
                .map(|d| d.name.as_str())
                .collect();

            for usage in &scope.uses {
                if global_names.contains(&usage.name)
                    && !local_globals.contains(usage.name.as_str())
                    && !scope.is_defined(&usage.name)
                {
                    diagnostics.push(Diagnostic {
                        rule_id: "GVMIS",
                        message: "Global variables are inefficient and make errors difficult to diagnose. Use a function with input variables instead."
                            .to_string(),
                        severity: Severity::Warning,
                        byte_range: usage.byte_range.clone(),
                        line: usage.line,
                        column: usage.column,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}
