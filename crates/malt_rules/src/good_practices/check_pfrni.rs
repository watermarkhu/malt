use super::*;

impl GoodPracticesEngine {
    /// PFRNI: PARFOR loop with an explicitly specified increment.
    ///
    /// A parfor range with three parts (`start:step:end`) is flagged unless the
    /// increment is 1 or -1, the steps parfor supports. There is no fix: the
    /// loop has to be rewritten by hand (changing the step changes which
    /// iterations run).
    pub(crate) fn check_pfrni(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("PFRNI") || !is_parfor_node(node, source) {
            return Vec::new();
        }

        let range = match find_iterator_range(node) {
            Some(r) => r,
            None => return Vec::new(),
        };
        if count_named_children(range) != 3 {
            return Vec::new();
        }
        let mut cursor = range.walk();
        let parts: Vec<Node> = range.named_children(&mut cursor).collect();
        let step = node_text(parts[1], source).replace(' ', "");
        if matches!(step.as_str(), "1" | "-1" | "+1") {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "PFRNI",
            message: "The parfor loop can only use a step size of 1 or -1.".to_string(),
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
    fn test_pfrni_fires_on_explicit_step() {
        let source = "parfor i = 1:2:10\n    x(i) = i;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let parfor = find_child_of_kind(root, "for_statement").unwrap();
        let diags = eng.check_pfrni(parfor, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "PFRNI");
        assert!(diags[0].fix.is_none(), "dropping the step changes the loop");
    }

    #[test]
    fn test_pfrni_allows_steps_one_and_minus_one() {
        let eng = engine();
        for source in [
            "parfor i = n:-1:1\n    x(i) = i;\nend\n",
            "parfor i = 1:1:n\n    x(i) = i;\nend\n",
        ] {
            let tree = parse(source);
            let parfor = find_child_of_kind(tree.root_node(), "for_statement").unwrap();
            assert!(eng.check_pfrni(parfor, source).is_empty(), "{source:?}");
        }
    }

    #[test]
    fn test_pfrni_silent_on_unit_step_and_regular_for() {
        let eng = engine();

        let source = "parfor i = 1:10\n    x(i) = i;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let parfor = find_child_of_kind(root, "for_statement").unwrap();
        assert!(eng.check_pfrni(parfor, source).is_empty());

        let source = "for i = 1:2:10\n    x(i) = i;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let parfor = find_child_of_kind(root, "for_statement").unwrap();
        assert!(eng.check_pfrni(parfor, source).is_empty());
    }
}
