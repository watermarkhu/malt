//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FCONV: Variable name same as script file name.
    pub(crate) fn check_fconv(
        &self,
        symbol_table: &SymbolTable,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let file_stem = ctx
            .file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if file_stem.is_empty() {
            return;
        }

        let root_scope = symbol_table.root_scope();
        for def in &root_scope.defs {
            if def.name == file_stem && def.kind == DefKind::Assignment {
                diagnostics.push(Diagnostic {
                    rule_id: "FCONV",
                    message: format!(
                        "Unable to define variable {} because it has the same name as the script.",
                        def.name
                    ),
                    severity: Severity::Error,
                    byte_range: def.byte_range.clone(),
                    line: def.line,
                    column: def.column,
                    fix: None,
                });
                break; // Only report first occurrence
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fconv_variable_same_as_script() {
        let source = "myscript = 42;\ndisp(myscript);\n";
        let diags = check_source(source, "myscript.m");
        let fconv = filter_by_id(&diags, "FCONV");
        assert!(
            !fconv.is_empty(),
            "FCONV should fire when variable name matches script file name"
        );
    }
}
