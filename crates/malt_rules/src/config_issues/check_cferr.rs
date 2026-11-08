//! CFERR check: configuration function calls with wrong argument counts.

use super::*;

impl ConfigIssuesEngine {
    /// CFERR: Check `coder.config` and similar calls for wrong argument count.
    pub(crate) fn check_cferr(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        if self.is_check_disabled("CFERR") {
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

        if !CONFIG_FUNCTIONS.contains(&func_name) {
            return Vec::new();
        }

        // These functions require exactly 1 argument (the config type string).
        // Zero arguments is invalid.
        let arg_list = match node.child_by_field_name("arguments") {
            Some(args) => args,
            None => {
                // No argument list at all — flag it.
                let start = node.start_position();
                return vec![Diagnostic {
                    rule_id: "CFERR",
                    message: "Cannot open or read the Code Analyzer settings from file VAR_FILE. Using default settings instead.".to_string(),
                    severity: Severity::Error,
                    byte_range: node.start_byte()..node.end_byte(),
                    line: start.row + 1,
                    column: start.column + 1,
                    fix: None,
                }];
            }
        };

        let arg_count = arg_list.named_child_count();
        if arg_count == 0 {
            let start = node.start_position();
            return vec![Diagnostic {
                rule_id: "CFERR",
                message: "Cannot open or read the Code Analyzer settings from file VAR_FILE. Using default settings instead.".to_string(),
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: start.row + 1,
                column: start.column + 1,
                fix: None,
            }];
        }

        Vec::new()
    }
}

#[cfg(test)]
mod tests {

    use crate::config_issues::tests::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- CFERR: configuration function error (negative only) ----------------

    #[test]
    fn cferr_ok_with_argument() {
        let diags = lint_nodes(&*engine(), "cfg = coder.config('lib');\n");
        assert!(!has_id(&diags, "CFERR"), "got: {diags:?}");
    }

    #[test]
    fn cferr_ok_on_plain_function() {
        let diags = lint_nodes(&*engine(), "x = config();\n");
        assert!(!has_id(&diags, "CFERR"), "got: {diags:?}");
    }
}
