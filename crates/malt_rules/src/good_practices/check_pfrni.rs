use super::*;

impl GoodPracticesEngine {
    /// PFRNI: PARFOR loop with an explicitly specified increment.
    ///
    /// A parfor range with three parts (`start:step:end`) is flagged unless the
    /// increment is 1 or -1, the steps parfor supports. A parenthesised step
    /// (`1:(2):10`) and a range continued over lines (`1:...` then `2:10`) count
    /// the same way. MATLAB reports at the start of the step. There is no fix:
    /// the loop has to be rewritten by hand (changing the step changes which
    /// iterations run).
    pub(crate) fn check_pfrni(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("PFRNI") || !is_parfor_node(node, source) {
            return Vec::new();
        }

        let range = match find_iterator_range(node) {
            Some(r) => r,
            None => return Vec::new(),
        };
        let parts = range_parts(range);
        if parts.len() != 3 {
            return Vec::new();
        }
        let step = parts[1];
        let text = node_text(step, source).replace(' ', "");
        let value = text.trim_start_matches('(').trim_end_matches(')');
        match value.parse::<f64>() {
            Ok(v) if v != 1.0 && v != -1.0 => {}
            _ => return Vec::new(),
        }

        let pos = step.start_position();
        vec![Diagnostic {
            rule_id: "PFRNI",
            message: "The parfor loop can only use a step size of 1 or -1.".to_string(),
            severity: Severity::Warning,
            byte_range: step.start_byte()..step.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

/// The parts of a range (`start:step:end`): its named children, without the
/// `...` line continuation and comments.
fn range_parts(range: Node) -> Vec<Node> {
    let mut cursor = range.walk();
    range
        .named_children(&mut cursor)
        .filter(|c| !matches!(c.kind(), "line_continuation" | "comment"))
        .collect()
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
        // Probed: MATLAB points at the step (`parfor i = 1:2:10`, column 14).
        assert_eq!((diags[0].line, diags[0].column), (1, 14));
    }

    #[test]
    fn test_pfrni_fires_on_parenthesised_and_continued_steps() {
        // Probed: `1:(2):10` (pn21, column 14) and `1:...` / `2:10` (pn19, line 3).
        let eng = engine();
        let source = "parfor i = 1:(2):10\n    x(i) = i;\nend\n";
        let tree = parse(source);
        let parfor = find_child_of_kind(tree.root_node(), "for_statement").unwrap();
        let diags = eng.check_pfrni(parfor, source);
        assert_eq!((diags.len(), diags[0].column), (1, 14));

        let source = "parfor i = 1:...\n        2:10\n    x(i) = i;\nend\n";
        let tree = parse(source);
        let parfor = find_child_of_kind(tree.root_node(), "for_statement").unwrap();
        let diags = eng.check_pfrni(parfor, source);
        assert_eq!((diags.len(), diags[0].line, diags[0].column), (1, 2, 9));
    }

    #[test]
    fn test_pfrni_allows_steps_one_and_minus_one() {
        let eng = engine();
        for source in [
            "parfor i = n:-1:1\n    x(i) = i;\nend\n",
            "parfor i = 1:1:n\n    x(i) = i;\nend\n",
            "parfor i = 1:1.0:5\n    x(i) = i;\nend\n",
            "parfor i = 1:(n+1):10\n    x(i) = i;\nend\n",
            "parfor i = 1:(1):10\n    x(i) = i;\nend\n",
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
