//! BDOPT check: invalid option names in `optimset`/`optimoptions` calls.

use super::*;

impl ConfigIssuesEngine {
    /// BDOPT: Check `optimset`/`optimoptions` for known-invalid option names.
    pub(crate) fn check_bdopt(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        if self.is_check_disabled("BDOPT") {
            return Vec::new();
        }

        let node = ctx.node;
        if node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match extract_function_name(node, ctx.source) {
            Some(name) => name,
            None => return Vec::new(),
        };

        if func_name != "optimset" && func_name != "optimoptions" {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let arg_list = match node.child_by_field_name("arguments") {
            Some(args) => args,
            None => return Vec::new(),
        };

        let arg_count = arg_list.named_child_count();
        for i in 0..arg_count {
            if let Some(arg) = arg_list.named_child(i) {
                if arg.kind() == "string" {
                    let text = &ctx.source[arg.start_byte()..arg.end_byte()];
                    let unquoted = text.trim_matches('\'').trim_matches('"');
                    if KNOWN_BAD_OPTIM_OPTIONS.contains(&unquoted) {
                        let start = arg.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "BDOPT",
                            message: format!(
                                "Option {} is ignored because it is invalid.",
                                unquoted
                            ),
                            severity: Severity::Error,
                            byte_range: arg.start_byte()..arg.end_byte(),
                            line: start.row + 1,
                            column: start.column + 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {

    use crate::config_issues::tests::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- BDOPT: invalid option value (negative only) ------------------------

    #[test]
    fn bdopt_ok_on_valid_option() {
        let diags = lint_nodes(&*engine(), "opts = optimset('Display', 'off');\n");
        assert!(!has_id(&diags, "BDOPT"), "got: {diags:?}");
    }

    #[test]
    fn bdopt_ok_on_unrelated_function() {
        let diags = lint_nodes(&*engine(), "foo('NotAnOption');\n");
        assert!(!has_id(&diags, "BDOPT"), "got: {diags:?}");
    }
}
