use super::*;

impl GoodPracticesEngine {
    /// MIPC1: `computer('arch')` returns a platform-specific value.
    pub(crate) fn check_mipc1(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MIPC1") || node.kind() != "function_call" {
            return Vec::new();
        }
        if get_function_call_name(node, source) != Some("computer") {
            return Vec::new();
        }

        let args_node = match find_child_of_kind(node, "arguments") {
            Some(a) => a,
            None => return Vec::new(),
        };
        let mut cursor = args_node.walk();
        let args: Vec<Node> = args_node
            .children(&mut cursor)
            .filter(|c| c.is_named())
            .collect();
        if args.len() != 1 {
            return Vec::new();
        }
        if args[0].kind() != "string" {
            return Vec::new();
        }
        if node_text(args[0], source).trim_matches('\'') != "arch" {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "MIPC1",
            message:
                "Calling the computer function with 'arch' returns 'win64', 'glnxa64', or 'maca64'."
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
    fn test_mipc1_fires_on_arch() {
        let source = "c = computer('arch');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_mipc1(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MIPC1");
    }

    #[test]
    fn test_mipc1_silent_on_other_argument() {
        let source = "c = computer('win');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_mipc1(fc, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_mipc1_silent_without_parens() {
        let source = "c = computer;\n";
        let tree = parse(source);
        let root = tree.root_node();

        assert!(find_descendant_of_kind(root, "function_call").is_none());
    }
}
