use super::*;

impl BugsEngine {
    /// MEXCEP: `warning(E.message)` (or `warning(E.message, ...)`) passes the
    /// message of an `MException` as the format string, so backslashes and `%`
    /// in it are interpreted.
    pub(crate) fn check_warning_message(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if extract_call_name(node, source) != Some("warning") {
            return Vec::new();
        }
        let args = arg_nodes(node);
        let Some(first) = args.first() else {
            return Vec::new();
        };
        let is_message = first.kind() == "field_expression"
            && first
                .child_by_field_name("field")
                .is_some_and(|f| node_text(f, source).trim() == "message")
            && first
                .child_by_field_name("object")
                .is_some_and(|o| o.kind() == "identifier");
        // With a leading identifier argument the message is a format argument.
        if !is_message || args.len() != 1 {
            return Vec::new();
        }
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "MEXCEP",
            message: "To report an MException as a warning, use a format specifier to ensure the message is printed correctly. For example, 'warning(E.identifier, \"%s\", E.message)'.".to_string(),
            severity: Severity::Error,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    #[test]
    fn mexcep_fires_on_warning_with_exception_message() {
        let diags = node_diags("try\n    f();\ncatch E\n    warning(E.message);\nend\n");
        assert!(has_id(&diags, "MEXCEP"), "got: {diags:?}");
    }

    #[test]
    fn mexcep_no_fire_with_format_specifier() {
        let diags = node_diags(
            "warning(E.identifier, '%s', E.message);\nwarning('%s', E.message);\nwarning('abc');\n",
        );
        assert!(!has_id(&diags, "MEXCEP"), "got: {diags:?}");
    }
}
