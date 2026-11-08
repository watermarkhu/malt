//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// NPERS: Persistent declarations in script files.
    pub(crate) fn check_npers(
        &self,
        symbol_table: &SymbolTable,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let root_scope = symbol_table.root_scope();
        for def in &root_scope.defs {
            if def.kind == DefKind::Persistent {
                diagnostics.push(Diagnostic {
                    rule_id: "NPERS",
                    message: "A PERSISTENT declaration is not valid in scripts.".to_string(),
                    severity: Severity::Error,
                    byte_range: def.byte_range.clone(),
                    line: def.line,
                    column: def.column,
                    fix: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_npers_persistent_in_script() {
        let source = "persistent x;\nx = 1;\n";
        let diags = check_source(source, "myscript.m");
        let npers = filter_by_id(&diags, "NPERS");
        assert!(
            !npers.is_empty(),
            "NPERS should fire for persistent in script"
        );
    }
}
