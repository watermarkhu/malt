use super::*;

impl GoodPracticesEngine {
    /// SIMPT: `import` command does not run before any other code in its
    /// enclosing function.
    pub(crate) fn check_simpt(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("SIMPT") || node.kind() != "command" {
            return Vec::new();
        }
        if get_command_name(node, source) != Some("import") {
            return Vec::new();
        }

        // Skip file/script-level imports (no enclosing function).
        let func = match enclosing_function(node) {
            Some(f) => f,
            None => return Vec::new(),
        };
        let block = match find_child_of_kind(func, "block") {
            Some(b) => b,
            None => return Vec::new(),
        };

        // Fire unless the import is the first non-comment statement in the block.
        let mut cursor = block.walk();
        let first_stmt = block
            .children(&mut cursor)
            .find(|c| c.is_named() && c.kind() != "comment");
        if let Some(first) = first_stmt {
            if first.start_byte() == node.start_byte() && first.end_byte() == node.end_byte() {
                return Vec::new();
            }
        }

        let func_name = func
            .child_by_field_name("name")
            .map(|n| node_text(n, source))
            .unwrap_or("");

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "SIMPT",
            message: format!(
                "This import statement runs before any other code in function {func_name}. Consider placing it at the top of the function body."
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
    fn test_simpt_fires_when_not_first_statement() {
        let source = "function f()\n  x = 1;\n  import foo.bar\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let cmd = find_descendant_of_kind(root, "command").unwrap();
        let diags = eng.check_simpt(cmd, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "SIMPT");
    }

    #[test]
    fn test_simpt_silent_when_first_statement() {
        let source = "function f()\n  import foo.bar;\n  x = 1;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let cmd = find_descendant_of_kind(root, "command").unwrap();
        let diags = eng.check_simpt(cmd, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_simpt_silent_at_script_level() {
        let source = "import foo.bar\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let cmd = find_child_of_kind(root, "command").unwrap();
        let diags = eng.check_simpt(cmd, source);
        assert!(diags.is_empty());
    }
}
