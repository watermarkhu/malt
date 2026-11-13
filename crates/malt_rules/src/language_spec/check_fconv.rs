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
            // A loop variable, global or persistent declaration of the script's name counts too;
            // MATLAB reports the first of them, at the declaration.
            if def.name == file_stem
                && matches!(
                    def.kind,
                    DefKind::Assignment
                        | DefKind::ForIterator
                        | DefKind::Global
                        | DefKind::Persistent
                )
            {
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
    fn test_fconv_loop_variable_and_global_declaration() {
        // Probed: a for-loop variable (line 1, column 5) and a global declaration (column 8)
        // of the script's name are reported at the declaration; the later assignment is not.
        let diags = check_source("for fconv03 = 1:3\ndisp(fconv03)\nend\n", "fconv03.m");
        let fconv = filter_by_id(&diags, "FCONV");
        assert_eq!(fconv.len(), 1, "got: {diags:?}");
        assert_eq!((fconv[0].line, fconv[0].column), (1, 5));

        let diags = check_source("global fconv06\nfconv06 = 1;\n", "fconv06.m");
        let fconv = filter_by_id(&diags, "FCONV");
        assert_eq!(fconv.len(), 1, "got: {diags:?}");
        assert_eq!((fconv[0].line, fconv[0].column), (1, 8));
    }

    #[test]
    fn test_fconv_no_fire_field_of_the_script_name() {
        // Probed: a field `s.fconv09` of a struct is not a variable of the script's name.
        let diags = check_source("s.fconv09 = 1;\n", "fconv09.m");
        assert!(filter_by_id(&diags, "FCONV").is_empty(), "got: {diags:?}");
    }
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
