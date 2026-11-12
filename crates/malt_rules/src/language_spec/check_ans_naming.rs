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
        // Probed: a method named `ans` is MTHANS in a class, not FCNANS.
        for func in meta.functions.iter().chain(meta.local_functions.iter()) {
            let in_class = meta
                .class
                .as_ref()
                .is_some_and(|c| c.byte_range.contains(&func.byte_range.start));
            if func.name == "ans" && !in_class {
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

    #[test]
    fn test_fcnans_function_with_output_named_ans_in_file() {
        // Probed (N3): `function r = ans(x)` gives one FCNANS, on the name at line 1.
        let source = "function r = ans(x)\n    r = x;\nend\n";
        let diags = check_source(source, "N3.m");
        let fcnans = filter_by_id(&diags, "FCNANS");
        assert_eq!(fcnans.len(), 1, "got: {diags:?}");
        assert_eq!(fcnans[0].line, 1);
    }

    #[test]
    fn test_fcnans_silent_for_classdef_method_named_ans() {
        // Probed (C1): a classdef method named `ans` is MTHANS, not FCNANS.
        let source = "classdef C1\n    methods\n        function r = ans(obj)\n            r = 1;\n        end\n    end\nend\n";
        let diags = check_source(source, "C1.m");
        assert!(filter_by_id(&diags, "FCNANS").is_empty(), "got: {diags:?}");
    }
}
