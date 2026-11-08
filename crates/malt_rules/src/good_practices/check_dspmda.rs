use super::*;

impl GoodPracticesEngine {
    /// DSPMDA: distributed array constructed inside an SPMD block.
    pub(crate) fn check_dspmda(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("DSPMDA") || node.kind() != "spmd_statement" {
            return Vec::new();
        }

        let body = match find_child_of_kind(node, "block") {
            Some(b) => b,
            None => return Vec::new(),
        };
        let mut calls = Vec::new();
        walk_body_for_function_calls(body, &mut calls);
        calls.retain(|c| is_distributed_call(*c, source));
        calls
            .into_iter()
            .map(|c| {
                let pos = c.start_position();
                Diagnostic {
                    rule_id: "DSPMDA",
                    message: "Distributed array must be created outside of an SPMD block."
                        .to_string(),
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
    fn test_dspmda_fires_on_distributed_constructors_in_spmd() {
        let source = "spmd\n    d = distributed(zeros(100));\n    g = gpuArray(1);\n    c = codistributed(2);\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let spmd = find_child_of_kind(root, "spmd_statement").unwrap();
        let diags = eng.check_dspmda(spmd, source);
        assert_eq!(diags.len(), 3);
        assert!(diags.iter().all(|d| d.rule_id == "DSPMDA"));
    }

    #[test]
    fn test_dspmda_silent_when_created_outside_spmd() {
        let source = "d = distributed(zeros(100));\nspmd\n    work_with(d);\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let spmd = find_child_of_kind(root, "spmd_statement").unwrap();
        assert!(eng.check_dspmda(spmd, source).is_empty());
    }
}
