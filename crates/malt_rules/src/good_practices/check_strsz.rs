use super::*;

impl GoodPracticesEngine {
    /// STRSZ: `==`/`~=` between string literals of different sizes.
    pub(crate) fn check_strsz(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("STRSZ") {
            return Vec::new();
        }
        if node.kind() != "comparison_operator" && node.kind() != "binary_operator" {
            return Vec::new();
        }

        let text = node_text(node, source);
        if !text.contains("==") && !text.contains("~=") {
            return Vec::new();
        }

        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();
        let op_idx = match children
            .iter()
            .position(|c| c.kind() == "==" || c.kind() == "~=")
        {
            Some(i) if i > 0 && i + 1 < children.len() => i,
            _ => return Vec::new(),
        };

        let left = children[op_idx - 1];
        let right = children[op_idx + 1];
        if left.kind() != "string" || right.kind() != "string" {
            return Vec::new();
        }
        if node_text(left, source).len() == node_text(right, source).len() {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "STRSZ",
            message: "Use STRCMP to compare character vectors that can have different sizes."
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
    fn test_strsz_fires_on_different_length_strings() {
        let source = "x = 'abc' == 'abcd';\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        let diags = eng.check_strsz(comp, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "STRSZ");
    }

    #[test]
    fn test_strsz_silent_on_same_length_strings() {
        let source = "x = 'abc' == 'abc';\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        let diags = eng.check_strsz(comp, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_strsz_silent_on_strcmp() {
        let source = "x = strcmp('abc','abcd');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_strsz(fc, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_strsz_fires_on_not_equal() {
        let source = "x = 'abc' ~= 'abcd';\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        let diags = eng.check_strsz(comp, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "STRSZ");
    }
}
