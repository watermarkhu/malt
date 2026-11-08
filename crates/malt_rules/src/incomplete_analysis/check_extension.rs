//! MDOTM/MDMCR: Invalid or deployed MATLAB file extension.

use super::*;

impl IncompleteAnalysisEngine {
    /// MDOTM: Invalid file extension (not .m).
    pub(crate) fn check_extension(
        &self,
        ctx: &FileContext,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if let Some(ext) = ctx.file_path.extension() {
            if ext != "m" {
                let ext_str = ext.to_string_lossy();
                // MDMCR: Deployed MATLAB file (.ctf or .p).
                if ext_str == "ctf" || ext_str == "p" {
                    diagnostics.push(Diagnostic {
                        rule_id: "MDMCR",
                        message: format!(
                            "Unable to run code analysis. {} is a deployed MATLAB file.",
                            ctx.file_path.display()
                        ),
                        severity: Severity::Error,
                        byte_range: 0..source.len().min(1),
                        line: 1,
                        column: 1,
                        fix: None,
                    });
                } else {
                    diagnostics.push(Diagnostic {
                        rule_id: "MDOTM",
                        message: format!(
                            "Unable to run code analysis. {} has an invalid file extension.",
                            ctx.file_path.display()
                        ),
                        severity: Severity::Error,
                        byte_range: 0..source.len().min(1),
                        line: 1,
                        column: 1,
                        fix: None,
                    });
                }
            }
        } else {
            // No extension at all — also flag as MDOTM.
            diagnostics.push(Diagnostic {
                rule_id: "MDOTM",
                message: format!(
                    "Unable to run code analysis. {} has an invalid file extension.",
                    ctx.file_path.display()
                ),
                severity: Severity::Error,
                byte_range: 0..source.len().min(1),
                line: 1,
                column: 1,
                fix: None,
            });
        }
    }
}
