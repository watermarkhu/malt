use super::*;

impl GoodPracticesEngine {
    /// PFEVB: EVALIN('base') or ASSIGNIN('base') inside a PARFOR loop.
    pub(crate) fn check_pfevb(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("PFEVB") || !is_parfor_node(node, source) {
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
                    rule_id: "PFEVB",
                    message: "Using EVALIN('base') or ASSIGNIN('base') inside a PARFOR loop refers to the worker machines' base workspaces.".to_string(),
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
    fn test_pfevb_fires_on_base_evalin_assignin() {
        let source = "parfor i = 1:10\n    evalin('base','x');\n    assignin('base','y',1);\n    x(i) = i;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let parfor = find_child_of_kind(root, "for_statement").unwrap();
        let diags = eng.check_pfevb(parfor, source);
        assert_eq!(diags.len(), 2);
        assert!(diags.iter().all(|d| d.rule_id == "PFEVB"));
    }

    #[test]
    fn test_pfevb_silent_on_caller_workspace() {
        let source = "parfor i = 1:10\n    evalin('caller','x');\n    x(i) = i;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let parfor = find_child_of_kind(root, "for_statement").unwrap();
        assert!(eng.check_pfevb(parfor, source).is_empty());
    }
}
