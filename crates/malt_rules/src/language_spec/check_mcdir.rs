//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCDIR: Class name must match @directory name.
    pub(crate) fn check_mcdir(
        &self,
        class: &ClassMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if let Some(parent_dir) = ctx.file_path.parent() {
            if let Some(dir_name) = parent_dir.file_name().and_then(|n| n.to_str()) {
                if let Some(stripped) = dir_name.strip_prefix('@') {
                    if stripped != class.name {
                        diagnostics.push(Diagnostic {
                            rule_id: "MCDIR",
                            // MATLAB quotes the class name and names the file by its
                            // absolute path (probed, plan 6.2).
                            message: format!(
                                "Class name '{}' and @directory name do not agree: '{}'.",
                                class.name,
                                std::path::absolute(ctx.file_path)
                                    .unwrap_or_else(|_| ctx.file_path.to_path_buf())
                                    .display()
                            ),
                            severity: Severity::Error,
                            byte_range: class.byte_range.clone(),
                            line: class.line,
                            column: 1,
                            fix: None,
                        });
                    }
                }
            }
        }
    }
}
