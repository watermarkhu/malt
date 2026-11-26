use super::*;

impl GoodPracticesEngine {
    /// STRSZ: `==`/`~=` between string literals of different sizes.
    pub(crate) fn check_strsz(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("STRSZ") {
            return Vec::new();
        }
        if node.kind() != "comparison_operator" && node.kind() != "binary_operator" {
            return Vec::new();
        }
        // MATLAB is silent on comparisons in an if/while condition (probed).
        if in_condition(node) {
            return Vec::new();
        }

        let text = node_text(node, source);
        if !text.contains("==") && !text.contains("~=") {
            return Vec::new();
        }

        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();
        let op_idx = match children
            .iter()
            .position(|c| c.kind() == "==" || c.kind() == "~=")
        {
            Some(i) if i > 0 && i + 1 < children.len() => i,
            _ => return Vec::new(),
        };

        let left = children[op_idx - 1];
        let right = children[op_idx + 1];
        if left.kind() != "string" || right.kind() != "string" {
            return Vec::new();
        }
        if node_text(left, source).len() == node_text(right, source).len() {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "STRSZ",
            message: "Use STRCMP to compare character vectors that can have different sizes."
                .to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: super::check_stcmp::strcmp_fix(node, source),
        }]
    }
}

/// Whether the node is inside the condition of an if, elseif or while statement
/// (not inside one of its blocks).
fn in_condition(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "block" => return false,
            "if_statement" | "elseif_clause" | "while_statement" => return true,
            _ => current = parent.parent(),
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{fix_of, lint_all, reported_without_fix};

    #[test]
    fn strsz_fix_rewrites_to_strcmp_and_is_unsafe() {
        let (safe, fixed) = fix_of(&engine(), "x = 'abc' == 'abcd';\n", "STRSZ").unwrap();
        assert!(!safe);
        assert_eq!(fixed, "x = strcmp('abc', 'abcd');\n");
        let (_, fixed) = fix_of(&engine(), "x = 'abc' ~= 'abcd';\n", "STRSZ").unwrap();
        assert_eq!(fixed, "x = ~strcmp('abc', 'abcd');\n");
        let still = lint_all(&engine(), &fixed);
        assert!(still.iter().all(|d| d.rule_id != "STRSZ"), "{still:?}");
    }

    #[test]
    fn strsz_no_fix_in_other_contexts() {
        // The `~` would bind to the argument list, not to the comparison.
        assert!(reported_without_fix(
            &engine(),
            "disp('ab' ~= 'abc');\n",
            "STRSZ"
        ));
    }

    #[test]
    fn test_strsz_silent_in_if_condition() {
        let source = "if 'abc' == 'abcd'\n disp(1)\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        assert!(eng.check_strsz(comp, source).is_empty());
    }

    #[test]
    fn test_strsz_fires_on_different_length_strings() {
        let source = "x = 'abc' == 'abcd';\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        let diags = eng.check_strsz(comp, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "STRSZ");
    }

    #[test]
    fn test_strsz_silent_on_same_length_strings() {
        let source = "x = 'abc' == 'abc';\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        let diags = eng.check_strsz(comp, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_strsz_silent_on_strcmp() {
        let source = "x = strcmp('abc','abcd');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_strsz(fc, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_strsz_fires_on_not_equal() {
        let source = "x = 'abc' ~= 'abcd';\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        let diags = eng.check_strsz(comp, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "STRSZ");
    }
}
