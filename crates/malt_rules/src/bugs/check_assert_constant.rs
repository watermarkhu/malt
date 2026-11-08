use super::*;

impl BugsEngine {
    /// ASSRT: `assert` whose first argument is not a logical condition.
    pub(crate) fn check_assert_condition(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match extract_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };

        if func_name != "assert" {
            return Vec::new();
        }

        let first_arg = match first_arg_node(node) {
            Some(n) => n,
            None => return Vec::new(),
        };

        // A condition must evaluate to a logical scalar. String and numeric
        // literals are not conditions.
        let is_non_condition = matches!(first_arg.kind(), "string" | "number");

        if is_non_condition {
            let pos = node.start_position();
            vec![Diagnostic {
                rule_id: "ASSRT",
                message: "The first input argument to 'assert' must be a condition. To always throw an error, use 'error(msg)' instead.".to_string(),
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- ASSRT ---------------------------------------------------------------

    #[test]
    fn assrt_fires_on_string_argument() {
        let src = "assert('some message');\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "ASSRT"), "got: {diags:?}");
    }

    #[test]
    fn assrt_fires_on_numeric_argument() {
        let src = "assert(5);\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "ASSRT"), "got: {diags:?}");
    }

    #[test]
    fn assrt_no_fire_on_logical_literal() {
        let src = "assert(true);\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "ASSRT"), "got: {diags:?}");
    }

    #[test]
    fn assrt_no_fire_on_real_condition() {
        let src = "assert(x > 0);\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "ASSRT"), "got: {diags:?}");
    }
}
