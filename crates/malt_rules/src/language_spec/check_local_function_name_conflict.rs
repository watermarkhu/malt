//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FCONF: Local function name same as file name.
    pub(crate) fn check_local_function_name_conflict(
        &self,
        meta: &FileMeta,
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

        for func in &meta.local_functions {
            if func.name == file_stem {
                diagnostics.push(Diagnostic {
                    rule_id: "FCONF",
                    message: format!(
                        "Unable to define local function {} because it has the same name as the file.",
                        func.name
                    ),
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

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fconf_local_function_same_as_file() {
        let source = "\
function y = main(x)
    y = helper(x);
end

function z = helper(x)
    z = x * 2;
end
";
        // File is named "helper.m" — the local function matches
        let diags = check_source(source, "helper.m");
        let fconf = filter_by_id(&diags, "FCONF");
        assert!(
            !fconf.is_empty(),
            "FCONF should fire when local function name matches file name"
        );
    }
}
