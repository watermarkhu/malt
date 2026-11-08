use super::*;

impl GoodPracticesEngine {
    /// SPEVB: EVALIN('base') or ASSIGNIN('base') inside an SPMD block.
    pub(crate) fn check_spevb(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("SPEVB") || node.kind() != "spmd_statement" {
            return Vec::new();
        }

        let body = match find_child_of_kind(node, "block") {
            Some(b) => b,
            None => return Vec::new(),
        };
        let mut calls = Vec::new();
        walk_body_for_function_calls(body, &mut calls);
        calls.retain(|c| is_evalin_assignin_base(*c, source));
        calls
            .into_iter()
            .map(|c| {
                let pos = c.start_position();
                Diagnostic {
                    rule_id: "SPEVB",
                    message: "Using EVALIN('base') or ASSIGNIN('base') inside an SPMD block refers to the worker machines' base workspaces.".to_string(),
                    severity: Severity::Warning,
                    byte_range: c.start_byte()..c.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spevb_fires_on_base_evalin_in_spmd() {
        let source = "spmd\n    evalin('base','x');\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let spmd = find_child_of_kind(root, "spmd_statement").unwrap();
        let diags = eng.check_spevb(spmd, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "SPEVB");
    }

    #[test]
    fn test_spevb_silent_on_caller_workspace_in_spmd() {
        let source = "spmd\n    evalin('caller','x');\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let spmd = find_child_of_kind(root, "spmd_statement").unwrap();
        assert!(eng.check_spevb(spmd, source).is_empty());
    }
}
