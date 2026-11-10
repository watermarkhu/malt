use super::*;

impl GoodPracticesEngine {
    /// TLEV: `global` or `persistent` declared inside a nested block (an `if`,
    /// loop or `switch` body) rather than as a top-level statement of its function.
    ///
    /// MATLAB does not report `eval` or `feval` here (probed: `eval` in an `if`
    /// gets EVLCS only), so those calls are no longer candidates.
    pub(crate) fn check_tlev(&self, node: Node, _source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("TLEV") {
            return Vec::new();
        }
        let keyword = match node.kind() {
            "global_operator" => "GLOBAL",
            "persistent_operator" => "PERSISTENT",
            _ => return Vec::new(),
        };
        let Some(parent) = node.parent() else {
            return Vec::new();
        };
        // Top-level: a script statement, or a statement of a function's body block.
        let top_level = match parent.kind() {
            "source_file" | "function_definition" => true,
            "block" => parent
                .parent()
                .is_some_and(|p| p.kind() == "function_definition"),
            _ => false,
        };
        if top_level {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "TLEV",
            message: format!(
                "{keyword} could be very inefficient unless it is a top-level statement in its function."
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
    fn test_tlev_silent_on_top_level_global() {
        let source = "function f()\nglobal g\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let decl = find_descendant_of_kind(root, "global_operator").unwrap();
        assert!(eng.check_tlev(decl, source).is_empty());
    }

    #[test]
    fn test_tlev_fires_on_nested_global() {
        let source = "function f()\nif true\nglobal g\nend\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let decl = find_descendant_of_kind(root, "global_operator").unwrap();
        let diags = eng.check_tlev(decl, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "TLEV");
        assert!(diags[0].message.starts_with("GLOBAL"));
    }

    #[test]
    fn test_tlev_fires_on_nested_persistent() {
        let source = "function f()\nfor i = 1:2\npersistent p\nend\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let decl = find_descendant_of_kind(root, "persistent_operator").unwrap();
        let diags = eng.check_tlev(decl, source);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.starts_with("PERSISTENT"));
    }

    #[test]
    fn test_tlev_silent_on_eval_in_block() {
        // MATLAB reports EVLCS for this eval, never TLEV (probed).
        let source = "function f()\nif true\neval('x = 1;');\nend\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        assert!(eng.check_tlev(fc, source).is_empty());
    }
}
