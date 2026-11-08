use super::*;

impl GoodPracticesEngine {
    /// LOAD: `load` called without output variable.
    pub(crate) fn check_load(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("LOAD") {
            return Vec::new();
        }

        // Only fire on statement-level load calls (not `x = load(...)`)
        let is_statement = is_statement_level(node);
        if !is_statement {
            return Vec::new();
        }

        let func_name = match node.kind() {
            "function_call" => get_function_call_name(node, source),
            "command" => get_command_name(node, source),
            _ => return Vec::new(),
        };

        if func_name != Some("load") {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "LOAD",
            message: "To avoid conflicts with functions on the path, specify variables to load from file."
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
    fn test_load_fires_without_output() {
        let source = "load('data.mat');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_child_of_kind(root, "function_call").unwrap();
        let diags = eng.check_load(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "LOAD");
    }

    #[test]
    fn test_load_silent_with_output() {
        let source = "s = load('data.mat');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        // The function_call is inside an assignment, not at statement level.
        let assignment = find_child_of_kind(root, "assignment").unwrap();
        let fc = find_child_of_kind(assignment, "function_call");
        if let Some(fc) = fc {
            let diags = eng.check_load(fc, source);
            assert!(diags.is_empty());
        }
    }
}
