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

        // (name, line, byte range) of each local function.
        let mut local: Vec<(String, usize, std::ops::Range<usize>)> = Vec::new();
        if meta.file_type == FileType::Script {
            // Every function in a script is local (probed: `x = 1;` then
            // `function fc_f03()` in fc_f03.m gives FCONF at the function's name).
            let root = ctx.tree.root_node();
            let mut cursor = root.walk();
            for child in root.children(&mut cursor) {
                if child.kind() != "function_definition" {
                    continue;
                }
                if let Some(name) = child.child_by_field_name("name") {
                    local.push((
                        node_text(name, ctx.source).to_string(),
                        name.start_position().row + 1,
                        child.byte_range(),
                    ));
                }
            }
        } else {
            local.extend(
                meta.local_functions
                    .iter()
                    .map(|f| (f.name.clone(), f.line, f.byte_range.clone())),
            );
        }

        for (name, line, byte_range) in local {
            if name == file_stem {
                diagnostics.push(Diagnostic {
                    rule_id: "FCONF",
                    message: format!(
                        "Unable to define local function '{name}' because it has the same name as the file."
                    ),
                    severity: Severity::Error,
                    byte_range,
                    line,
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

    #[test]
    fn test_fconf_script_function_same_as_file() {
        // Probed (f03, f12): a function after statements in a script named like the file.
        let source = "x = 1;\nfunction fc_f12()\nend\n";
        let diags = check_source(source, "fc_f12.m");
        let fconf = filter_by_id(&diags, "FCONF");
        assert_eq!(fconf.len(), 1, "got: {diags:?}");
        assert_eq!(fconf[0].line, 2);
        assert_eq!(
            fconf[0].message,
            "Unable to define local function 'fc_f12' because it has the same name as the file."
        );
    }

    #[test]
    fn test_fconf_silent_main_function_named_like_file() {
        // Probed (f10): the main function of a function file is not a local function.
        let source = "function y = fc_f10(x)\ny = x;\nend\n";
        let diags = check_source(source, "fc_f10.m");
        assert!(filter_by_id(&diags, "FCONF").is_empty(), "got: {diags:?}");
    }
}
