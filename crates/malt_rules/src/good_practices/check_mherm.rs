use super::*;

impl GoodPracticesEngine {
    /// MHERM: parenthesize the multiplication of a variable and its transpose
    /// to ensure the result is Hermitian.
    pub(crate) fn check_mherm(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MHERM") || node.kind() != "binary_operator" {
            return Vec::new();
        }

        // Already parenthesized — the rule is satisfied.
        if node
            .parent()
            .map(|p| p.kind() == "parenthesis")
            .unwrap_or(false)
        {
            return Vec::new();
        }

        let has_mul = (0..node.child_count()).any(|i| {
            node.child(i)
                .map(|c| {
                    let text = node_text(c, source);
                    text == "*" || text == ".*"
                })
                .unwrap_or(false)
        });
        if !has_mul {
            return Vec::new();
        }

        let postfix = (0..node.child_count()).find_map(|i| {
            let child = node.child(i)?;
            if child.kind() == "postfix_operator" {
                Some(child)
            } else {
                None
            }
        });
        let Some(pf) = postfix else {
            return Vec::new();
        };

        let var_name = first_named_child(pf)
            .map(|c| node_text(c, source))
            .unwrap_or("expression");

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "MHERM",
            message: format!(
                "Parenthesize the multiplication of {var_name} and its transpose to ensure the result is Hermitian."
            ),
            severity: Severity::Info,
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
    fn test_mherm_fires_on_unparenthesized_transpose_multiply() {
        let source = "classdef Foo\n    methods\n        function z = compute(obj, x)\n            z = x * x';\n        end\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        let nodes = all_descendants_of_kind(tree.root_node(), "binary_operator");
        let diags: Vec<Diagnostic> = nodes
            .iter()
            .flat_map(|n| eng.check_mherm(*n, source))
            .collect();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MHERM");
    }

    #[test]
    fn test_mherm_silent_when_parenthesized() {
        let source = "classdef Foo\n    methods\n        function z = compute(obj, x)\n            z = (x * x');\n        end\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        let nodes = all_descendants_of_kind(tree.root_node(), "binary_operator");
        let diags: Vec<Diagnostic> = nodes
            .iter()
            .flat_map(|n| eng.check_mherm(*n, source))
            .collect();
        assert!(diags.is_empty());
    }
}
