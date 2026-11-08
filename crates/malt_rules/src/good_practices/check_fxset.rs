use super::*;

impl GoodPracticesEngine {
    /// FXSETA: the `for` loop iterator variable is assigned inside the loop body.
    pub(crate) fn check_fxset(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("FXSETA") || node.kind() != "for_statement" {
            return Vec::new();
        }

        let iterator = match find_child_of_kind(node, "iterator") {
            Some(i) => i,
            None => return Vec::new(),
        };
        let var_node = match first_named_child(iterator) {
            Some(n) if n.kind() == "identifier" => n,
            _ => return Vec::new(),
        };
        let var_name = node_text(var_node, source).to_string();

        let body = match find_child_of_kind(node, "block") {
            Some(b) => b,
            None => return Vec::new(),
        };

        let mut diagnostics = Vec::new();
        collect_loop_assignments(body, &var_name, source, &mut diagnostics);
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fxseta_fires_on_iterator_assignment() {
        let source = "for i = 1:10\n  i = 5;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let for_node = find_child_of_kind(root, "for_statement").unwrap();
        let diags = eng.check_fxset(for_node, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "FXSETA");
    }

    #[test]
    fn test_fxseta_silent_on_read_only_iterator() {
        let source = "for i = 1:10\n  y = i;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let for_node = find_child_of_kind(root, "for_statement").unwrap();
        let diags = eng.check_fxset(for_node, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_fxseta_silent_on_other_variable_assignment() {
        let source = "for i = 1:10\n  j = 5;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let for_node = find_child_of_kind(root, "for_statement").unwrap();
        let diags = eng.check_fxset(for_node, source);
        assert!(diags.is_empty());
    }
}
