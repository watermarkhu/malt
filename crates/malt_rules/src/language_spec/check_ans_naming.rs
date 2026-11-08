//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FCNANS / CLANS: Function or class named 'ans'.
    pub(crate) fn check_ans_naming(
        &self,
        meta: &FileMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // CLANS: Class named 'ans'
        if let Some(ref class) = meta.class {
            if class.name == "ans" {
                diagnostics.push(Diagnostic {
                    rule_id: "CLANS",
                    message: "Using ANS as a class name is not supported.".to_string(),
                    severity: Severity::Error,
                    byte_range: class.byte_range.clone(),
                    line: class.line,
                    column: 1,
                    fix: None,
                });
            }
        }

        // FCNANS: Function named 'ans'
        for func in meta.functions.iter().chain(meta.local_functions.iter()) {
            if func.name == "ans" {
                diagnostics.push(Diagnostic {
                    rule_id: "FCNANS",
                    message: "Using ANS as a function name is not supported.".to_string(),
                    severity: Severity::Error,
                    byte_range: func.byte_range.clone(),
                    line: func.line,
                    column: 1,
                    fix: None,
                });
            }
        }

        // Also check by file name for the main function
        if meta.file_type == FileType::FunctionFile {
            let file_stem = ctx
                .file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if file_stem == "ans" {
                if let Some(main) = meta.main_function() {
                    if main.name != "ans" {
                        // File is named 'ans' but function has a different name — still flag
                        diagnostics.push(Diagnostic {
                            rule_id: "FCNANS",
                            message: "Using ANS as a function name is not supported.".to_string(),
                            severity: Severity::Error,
                            byte_range: main.byte_range.clone(),
                            line: main.line,
                            column: 1,
                            fix: None,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fcnans_function_named_ans() {
        let source = "\
function y = ans(x)
    y = x;
end
";
        let diags = check_source(source, "ans.m");
        let fcnans = filter_by_id(&diags, "FCNANS");
        assert!(
            !fcnans.is_empty(),
            "FCNANS should fire for function named 'ans'"
        );
    }

    #[test]
    fn test_clans_class_named_ans() {
        let source = "\
classdef ans
end
";
        let diags = check_source(source, "ans.m");
        let clans = filter_by_id(&diags, "CLANS");
        assert!(!clans.is_empty(), "CLANS should fire for class named 'ans'");
    }
}
