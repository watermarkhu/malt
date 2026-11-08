use super::*;

impl GoodPracticesEngine {
    /// RMWRN: `warning` called with a message ID tag that has been removed from MATLAB.
    ///
    /// The tag denylist is [`REMOVED_WARNING_TAGS`], currently empty. The
    /// mechanism is implemented so tags can be populated as removals occur.
    pub(crate) fn check_rmwrn(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("RMWRN") || node.kind() != "function_call" {
            return Vec::new();
        }
        if get_function_call_name(node, source) != Some("warning") {
            return Vec::new();
        }

        let args_node = match find_child_of_kind(node, "arguments") {
            Some(a) => a,
            None => return Vec::new(),
        };
        let first_arg = match first_named_child(args_node) {
            Some(a) => a,
            None => return Vec::new(),
        };
        if first_arg.kind() != "string" {
            return Vec::new();
        }

        let tag = node_text(first_arg, source).trim_matches('\'');
        if !REMOVED_WARNING_TAGS.contains(&tag) {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "RMWRN",
            message: format!(
                "The warning with tag {tag} has been removed from MATLAB, so this statement has no effect."
            ),
            severity: Severity::Warning,
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

    #[test]
    fn test_rmwrn_mechanism_never_fires_with_empty_denylist() {
        let source = "warning('ident:tag','msg');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_child_of_kind(root, "function_call").unwrap();
        let diags = eng.check_rmwrn(fc, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_rmwrn_not_a_warning_call() {
        let source = "error('ident:tag','msg');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_child_of_kind(root, "function_call").unwrap();
        let diags = eng.check_rmwrn(fc, source);
        assert!(diags.is_empty());
    }
}
