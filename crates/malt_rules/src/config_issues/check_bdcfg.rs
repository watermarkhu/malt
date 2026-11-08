//! BDCFG check: invalid configuration parameter names in `set_param`/`get_param` calls.

use super::*;

impl ConfigIssuesEngine {
    /// BDCFG: Check `set_param`/`get_param` calls for known-invalid parameter names.
    pub(crate) fn check_bdcfg(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        if self.is_check_disabled("BDCFG") {
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

        if func_name != "set_param" && func_name != "get_param" {
            return Vec::new();
        }

        // Look for string literal arguments that match known-bad parameters.
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
                    if KNOWN_BAD_PARAMS.contains(&unquoted) {
                        let start = arg.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "BDCFG",
                            message: format!(
                                "Code Analyzer configuration file is invalid. Factory configuration is used instead. Run matlab.codeanalysis.validateConfiguration({}) to identify specific issues.",
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

    // -- BDCFG: invalid configuration parameter (negative only) -------------

    #[test]
    fn bdcfg_ok_on_valid_param() {
        let diags = lint_nodes(
            &*engine(),
            "set_param(gcs, 'SimulationCommand', 'start');\n",
        );
        assert!(!has_id(&diags, "BDCFG"), "got: {diags:?}");
    }

    #[test]
    fn bdcfg_ok_on_unrelated_function() {
        let diags = lint_nodes(&*engine(), "myfunc('NotARealParam');\n");
        assert!(!has_id(&diags, "BDCFG"), "got: {diags:?}");
    }
}
