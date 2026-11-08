use super::*;

impl GoodPracticesEngine {
    /// CTCH: `catch` clause without an exception identifier.
    ///
    /// Best practice is to capture the error information with `catch ME`.
    pub(crate) fn check_ctch(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("CTCH") || node.kind() != "catch_clause" {
            return Vec::new();
        }

        // `catch ME` has an identifier child directly on the catch_clause;
        // a bare `catch` does not.
        if has_child_of_kind(node, "identifier") {
            return Vec::new();
        }

        let pos = node.start_position();
        let text = node_text(node, source);
        let keyword_len = text.find(|c: char| c.is_whitespace()).unwrap_or(5);
        vec![Diagnostic {
            rule_id: "CTCH",
            message: "Best practice is for CATCH to be followed by an identifier that gets the error information."
                .to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.start_byte() + keyword_len,
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
    fn test_ctch_fires_on_catch_without_identifier() {
        let source = "try\n    x = 1;\ncatch\n    disp('err');\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let try_node = find_child_of_kind(root, "try_statement").unwrap();
        let catch_node = find_child_of_kind(try_node, "catch_clause").unwrap();
        let diags = eng.check_ctch(catch_node, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "CTCH");
    }

    #[test]
    fn test_ctch_silent_on_catch_with_identifier() {
        let source = "try\n    x = 1;\ncatch ME\n    disp(ME);\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let try_node = find_child_of_kind(root, "try_statement").unwrap();
        let catch_node = find_child_of_kind(try_node, "catch_clause").unwrap();
        let diags = eng.check_ctch(catch_node, source);
        assert!(diags.is_empty());
    }
}
