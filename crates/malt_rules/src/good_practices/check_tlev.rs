use super::*;

impl GoodPracticesEngine {
    /// TLEV: dynamic-code function (`eval`/`evalc`/`evalin`/`feval`) used as a
    /// sub-expression rather than a top-level statement.
    pub(crate) fn check_tlev(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("TLEV") || node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match get_function_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };
        if !["eval", "evalc", "evalin", "feval"].contains(&func_name) {
            return Vec::new();
        }

        // A call that is the entire right-hand side of an assignment is still a
        // top-level statement; anything deeper is a sub-expression.
        if let Some(parent) = node.parent() {
            if parent.kind() == "assignment" {
                if let Some(right) = parent.child_by_field_name("right") {
                    if right.start_byte() == node.start_byte()
                        && right.end_byte() == node.end_byte()
                    {
                        return Vec::new();
                    }
                }
            }
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "TLEV",
            message: format!(
                "{func_name} could be very inefficient unless it is a top-level statement in its function."
            ),
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
    fn test_tlev_fires_on_bare_eval_statement() {
        let source = "eval(x + y);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_child_of_kind(root, "function_call").unwrap();
        let diags = eng.check_tlev(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "TLEV");
    }

    #[test]
    fn test_tlev_fires_on_nested_eval() {
        let source = "x = f(eval(y));\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let eval_call = goodprac_g4_find_call(root, "eval", source).unwrap();
        let diags = eng.check_tlev(eval_call, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "TLEV");
    }

    #[test]
    fn test_tlev_silent_on_whole_rhs() {
        let source = "x = eval(y);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_tlev(fc, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_tlev_silent_on_evalc_whole_rhs() {
        let source = "x = evalc('y = 1');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_tlev(fc, source);
        assert!(diags.is_empty());
    }

    fn goodprac_g4_find_call<'a>(node: Node<'a>, name: &str, source: &'a str) -> Option<Node<'a>> {
        if node.kind() == "function_call" && get_function_call_name(node, source) == Some(name) {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = goodprac_g4_find_call(child, name, source) {
                return Some(found);
            }
        }
        None
    }
}
