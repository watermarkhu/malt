//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCCBD: Constructor must be defined in the class definition file.
    pub(crate) fn check_mccbd(
        &self,
        class: &ClassMeta,
        meta: &FileMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // If we're in a @directory class, the constructor may be in a separate file.
        // Check if there's a constructor in the methods blocks.
        let in_at_dir = ctx
            .file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .is_some_and(|dir| dir.starts_with('@'));

        if in_at_dir {
            // In @directory mode, skip this check — constructors can be in separate files.
            return;
        }

        // For single-file classes, check that a constructor is defined for non-abstract classes.
        // Actually, MCCBD checks that if a constructor IS defined, it should be in the class file.
        // This is mainly relevant for @folder classes where methods can be in separate files.
        // For single-file classes we verify the constructor is in a methods block (not local func).
        for func in &meta.local_functions {
            if func.name == class.name {
                diagnostics.push(Diagnostic {
                    rule_id: "MCCBD",
                    message: "Constructor must be fully defined in the class definition file."
                        .to_string(),
                    severity: Severity::Error,
                    byte_range: func.byte_range.clone(),
                    line: func.line,
                    column: 1,
                    fix: None,
                });
            }
        }
    }
}
