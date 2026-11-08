use super::*;

impl GoodPracticesEngine {
    /// TRYNC: `try` without `catch` clause.
    pub(crate) fn check_trync(&self, node: Node, _source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("TRYNC") || node.kind() != "try_statement" {
            return Vec::new();
        }

        let has_catch = has_child_of_kind(node, "catch_clause");
        if has_catch {
            return Vec::new();
        }

        let pos = node.start_position();
        let keyword_end = node.start_byte() + "try".len();
        vec![Diagnostic {
            rule_id: "TRYNC",
            message: "TRY statement should have a CATCH statement to check for unexpected errors."
                .to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..keyword_end,
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
    fn test_trync_fires_on_try_without_catch() {
        let source = "try\n    x = 1;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        // Find the try_statement node.
        let try_node = find_child_of_kind(root, "try_statement").unwrap();
        let diags = eng.check_trync(try_node, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "TRYNC");
    }

    #[test]
    fn test_trync_silent_on_try_with_catch() {
        let source = "try\n    x = 1;\ncatch ME\n    disp(ME);\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let try_node = find_child_of_kind(root, "try_statement").unwrap();
        let diags = eng.check_trync(try_node, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_disabled_check_skipped() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec!["TRYNC".to_string()],
            },
        };

        let source = "try\n    x = 1;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let try_node = find_child_of_kind(root, "try_statement").unwrap();
        let diags = eng.check_trync(try_node, source);
        assert!(diags.is_empty());
    }
}
