use super::*;

impl BugsEngine {
    /// LOGEMP: `isempty(...)` applied to a logical expression, which produces
    /// incorrect results for logical arrays.
    pub(crate) fn check_isempty_logical(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match extract_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };

        if func_name != "isempty" {
            return Vec::new();
        }

        let first_arg = match first_arg_node(node) {
            Some(n) => n,
            None => return Vec::new(),
        };

        let is_logical = matches!(
            first_arg.kind(),
            "comparison_operator" | "boolean_operator" | "not_operator"
        );

        if is_logical {
            let arg_text = node_text(first_arg, source);
            let pos = node.start_position();
            vec![Diagnostic {
                rule_id: "LOGEMP",
                message: "Using 'isempty' on a logical expression creates incorrect results. To determine if all the conditions are false, use '~any(..., \"all\")' instead.".to_string(),
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: Some(Fix::new(
                    node.start_byte()..node.end_byte(),
                    format!("~any({arg_text}, \"all\")"),
                )),
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

    // -- LOGEMP --------------------------------------------------------------

    #[test]
    fn logemp_fires_on_isempty_comparison() {
        let src = "y = isempty(x == 1);\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "LOGEMP"), "got: {diags:?}");
    }

    #[test]
    fn logemp_fires_on_isempty_boolean() {
        let src = "y = isempty(a && b);\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "LOGEMP"), "got: {diags:?}");
    }

    #[test]
    fn logemp_no_fire_on_plain_isempty() {
        let src = "y = isempty(x);\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "LOGEMP"), "got: {diags:?}");
    }
}
