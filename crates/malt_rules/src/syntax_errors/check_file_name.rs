//! `check_file_name` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_file_name(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if !self.is_check_enabled("BDFIL") {
            return diagnostics;
        }

        if let Some(stem) = ctx.file_path.file_stem() {
            let name = stem.to_string_lossy();

            let valid = !name.is_empty()
                && name.len() <= 63
                && name.starts_with(|c: char| c.is_ascii_alphabetic())
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

            if !valid {
                diagnostics.push(Diagnostic {
                    rule_id: "BDFIL",
                    message: "Invalid MATLAB file name. MATLAB file names must start with a letter, contain only letters, numbers, or underscores, and have no more than 63 characters.".to_string(),
                    severity: Severity::Error,
                    byte_range: 0..ctx.source.len().min(1),
                    line: 1,
                    column: 1,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}
