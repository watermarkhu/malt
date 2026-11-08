use super::*;

impl GoodPracticesEngine {
    /// MEXCEP: `warning` is called with an MException object as its argument.
    ///
    /// Passing an exception directly truncates the message; a format specifier
    /// should be used instead.
    pub(crate) fn check_mexcep(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MEXCEP") || node.kind() != "function_call" {
            return Vec::new();
        }
        if get_function_call_name(node, source) != Some("warning") {
            return Vec::new();
        }

        // Fire when the first argument is a bare identifier (the exception).
        let args = find_child_of_kind(node, "arguments");
        let first = args.and_then(|a| first_named_child(a));
        let is_exception = first.map(|arg| arg.kind() == "identifier").unwrap_or(false);
        if !is_exception {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "MEXCEP",
            message: "To report an MException as a warning, use a format specifier to ensure the message is printed correctly. For example, 'warning(E.identifier, \"%s\", E.message)'."
                .to_string(),
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
    fn test_mexcep_fires_on_warning_with_exception() {
        let source = "try\n    x = 1;\ncatch ME\n    warning(ME);\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_mexcep(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MEXCEP");
    }

    #[test]
    fn test_mexcep_silent_on_warning_with_string() {
        let source = "warning('this is a warning');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_mexcep(fc, source);
        assert!(diags.is_empty());
    }
}
