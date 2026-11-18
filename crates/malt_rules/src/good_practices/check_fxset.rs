use super::*;

impl GoodPracticesEngine {
    /// FXSETA: the `for` loop iterator variable is assigned inside the loop body.
    pub(crate) fn check_fxset(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        // Probed: inside a parfor, MATLAB reports PFXST for a loop-variable assignment, not FXSETA.
        if !self.is_check_enabled("FXSETA")
            || node.kind() != "for_statement"
            || node_text(node, source).trim_start().starts_with("parfor")
            || in_unreachable_code(node, source)
        {
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
    fn test_fxseta_silent_after_endless_loop() {
        let source = "while true\n  x = 1;\nend\nfor i = 1:10\n  i = 5;\nend\n";
        let tree = parse(source);
        let eng = engine();
        let for_node = find_child_of_kind(tree.root_node(), "for_statement").unwrap();
        assert!(eng.check_fxset(for_node, source).is_empty());

        let source = "while true\n  if x, break; end\nend\nfor i = 1:10\n  i = 5;\nend\n";
        let tree = parse(source);
        let for_node = find_child_of_kind(tree.root_node(), "for_statement").unwrap();
        assert_eq!(eng.check_fxset(for_node, source).len(), 1);
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

    #[test]
    fn test_fxseta_fires_on_indexed_and_multi_output_assignment() {
        // Probed: `k(2) = 1` and `[k, j] = deal(...)` write the loop variable.
        for source in [
            "for k = 1:3\n  k(2) = 1;\nend\n",
            "for k = 1:3\n  [k, j] = deal(1, 2);\nend\n",
        ] {
            let tree = parse(source);
            let eng = engine();
            let for_node = find_child_of_kind(tree.root_node(), "for_statement").unwrap();
            assert_eq!(eng.check_fxset(for_node, source).len(), 1, "{source}");
        }
    }

    #[test]
    fn test_fxseta_silent_on_loop_variable_as_index_of_an_output() {
        // Dogfooding (plan 6.2): `[a(k), b(k)] = f(k)` only reads `k`.
        let source = "for k = 1:3\n  [a(k), b{k}] = deal(1, 2);\nend\n";
        let tree = parse(source);
        let eng = engine();
        let for_node = find_child_of_kind(tree.root_node(), "for_statement").unwrap();
        assert!(eng.check_fxset(for_node, source).is_empty());
    }

    #[test]
    fn test_fxseta_silent_on_field_with_the_loop_name() {
        let source = "for k = 1:3\n  s.k = 1;\nend\n";
        let tree = parse(source);
        let eng = engine();
        let for_node = find_child_of_kind(tree.root_node(), "for_statement").unwrap();
        assert!(eng.check_fxset(for_node, source).is_empty());
    }

    #[test]
    fn test_fxseta_message_is_matlab_text() {
        let source = "for i = 1:10\n  i = 5;\nend\n";
        let tree = parse(source);
        let eng = engine();
        let for_node = find_child_of_kind(tree.root_node(), "for_statement").unwrap();
        let diags = eng.check_fxset(for_node, source);
        assert_eq!(
            diags[0].message,
            "Assigning a value to a loop variable within the loop statements is confusing and may not have an effect. The value of 'i' will be reset on the next iteration of the loop."
        );
    }
}
