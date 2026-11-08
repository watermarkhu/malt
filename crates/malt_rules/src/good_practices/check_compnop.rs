use super::*;

impl GoodPracticesEngine {
    /// COMPNOP: `call(...) == true` simplifies to `call(...)`.
    pub(crate) fn check_compnop(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("COMPNOP") || node.kind() != "comparison_operator" {
            return Vec::new();
        }

        let Some((call_node, literal_node, op)) = self.comparison_with_literal(node, source) else {
            return Vec::new();
        };
        let literal = node_text(literal_node, source).trim();
        if op != "==" || literal != "true" {
            return Vec::new();
        }

        let call_text = node_text(call_node, source).to_string();
        let func_name = get_function_call_name(call_node, source).unwrap_or("function");
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "COMPNOP",
            message: format!(
                "This logical comparison simplifies to {call_text}. Did you mean to use {func_name} to evaluate function argument: {call_text}?"
            ),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: Some(Fix::new(
                node.start_byte()..node.end_byte(),
                call_text,
            )),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compnop_fires_on_call_eq_true() {
        let source = "if isa(x,'double') == true\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        let diags = eng.check_compnop(comp, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "COMPNOP");
        assert!(diags[0].fix.is_some());
    }

    #[test]
    fn test_compnop_fix_replaces_comparison_with_call() {
        let source = "if isa(x,'double') == true\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        let diags = eng.check_compnop(comp, source);
        let fix = diags[0].fix.as_ref().unwrap();
        assert_eq!(&source[fix.byte_range.clone()], "isa(x,'double') == true");
        assert_eq!(fix.replacement, "isa(x,'double')");
    }

    #[test]
    fn test_compnop_silent_on_identifier_eq_true() {
        let source = "if y == true\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        assert!(eng.check_compnop(comp, source).is_empty());
        assert!(eng.check_comnot(comp, source).is_empty());
    }

    #[test]
    fn test_compnop_silent_on_call_eq_false() {
        // `== false` belongs to COMPNOT, not COMPNOP.
        let source = "if isa(x,'double') == false\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        assert!(eng.check_compnop(comp, source).is_empty());
    }

    #[test]
    fn test_compnop_comnot_silent_without_comparison() {
        let source = "if isa(x,'double')\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        assert!(find_descendant_of_kind(root, "comparison_operator").is_none());
        assert!(eng.check_compnop(root, source).is_empty());
        assert!(eng.check_comnot(root, source).is_empty());
    }
}
