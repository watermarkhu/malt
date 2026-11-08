use super::*;

impl GoodPracticesEngine {
    /// COMPNOT: `call(...) ~= true` or `call(...) == false` simplifies to
    /// `~call(...)`.
    pub(crate) fn check_comnot(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("COMPNOT") || node.kind() != "comparison_operator" {
            return Vec::new();
        }

        let Some((call_node, literal_node, op)) = self.comparison_with_literal(node, source) else {
            return Vec::new();
        };
        let literal = node_text(literal_node, source).trim();
        let is_comnot = (op == "~=" && literal == "true") || (op == "==" && literal == "false");
        if !is_comnot {
            return Vec::new();
        }

        let call_text = node_text(call_node, source).to_string();
        let func_name = get_function_call_name(call_node, source).unwrap_or("function");
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "COMPNOT",
            message: format!(
                "This logical comparison simplifies to ~{call_text}. Did you mean to use {func_name} to evaluate function argument: {call_text}?"
            ),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: Some(Fix::new(
                node.start_byte()..node.end_byte(),
                format!("~{call_text}"),
            )),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comnot_fires_on_call_neq_true() {
        let source = "if isa(x,'double') ~= true\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        let diags = eng.check_comnot(comp, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "COMPNOT");
        let fix = diags[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "~isa(x,'double')");
    }

    #[test]
    fn test_comnot_fires_on_call_eq_false() {
        let source = "if isa(x,'double') == false\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        let diags = eng.check_comnot(comp, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "COMPNOT");
        assert_eq!(
            diags[0].fix.as_ref().unwrap().replacement,
            "~isa(x,'double')"
        );
    }

    #[test]
    fn test_comnot_silent_on_call_neq_false() {
        // `~= false` is not a COMPNOT pattern.
        let source = "if isa(x,'double') ~= false\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        assert!(eng.check_comnot(comp, source).is_empty());
    }
}
