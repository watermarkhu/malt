use super::*;

impl GoodPracticesEngine {
    /// DUALC: a command is immediately followed by a comma, which can
    /// prematurely end the command. Heuristic — see docs for limitations.
    pub(crate) fn check_dualc(&self, node: Node, _source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("DUALC") || node.kind() != "command" {
            return Vec::new();
        }

        let comma_after = node
            .next_sibling()
            .map(|n| n.kind() == ",")
            .unwrap_or(false);
        if !comma_after {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "DUALC",
            message: "Command might be prematurely ended by comma.".to_string(),
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
    fn test_dualc_fires_on_comma_after_command() {
        let source = "disp hello, disp world\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let cmd = find_child_of_kind(root, "command").unwrap();
        let diags = eng.check_dualc(cmd, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "DUALC");
    }

    #[test]
    fn test_dualc_silent_without_comma() {
        let source = "disp hello\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let cmd = find_child_of_kind(root, "command").unwrap();
        let diags = eng.check_dualc(cmd, source);
        assert!(diags.is_empty());
    }
}
