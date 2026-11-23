use super::*;
use crate::analysis::callee::{call_path, Dotted};
use crate::readability::find_arguments;

/// Functions whose single comparison argument MATLAB reports (ELARLOG). Probed in
/// R2026b: `abs`, `numel`, `length`, `floor` and `round` are reported; `sqrt`,
/// `double`, `logical`, `mod`, `min`, `sum`, `max`, `isempty` and `any` are silent.
const ELARLOG_FUNCTIONS: &[&str] = &["abs", "numel", "length", "floor", "round"];

impl GoodPracticesEngine {
    /// ELARLOG: a comparison passed alone to one of `ELARLOG_FUNCTIONS`, as in
    /// `numel(A ~= B)`. The message names the outermost operator of the argument.
    pub(crate) fn check_elarlog(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("ELARLOG") || node.kind() != "function_call" {
            return Vec::new();
        }
        let Some(name) = call_path(node, source, Dotted::Skip) else {
            return Vec::new();
        };
        if !ELARLOG_FUNCTIONS.contains(&name) {
            return Vec::new();
        }
        let Some(args) = find_arguments(node) else {
            return Vec::new();
        };
        let mut cursor = args.walk();
        let exprs: Vec<Node> = args.named_children(&mut cursor).collect();
        if exprs.len() != 1 || exprs[0].kind() != "comparison_operator" {
            return Vec::new();
        }
        let comparison = exprs[0];
        let mut op_cursor = comparison.walk();
        let Some(op) = comparison
            .children(&mut op_cursor)
            .find(|child| !child.is_named())
        else {
            return Vec::new();
        };
        let op_text = node_text(op, source);
        let pos = op.start_position();
        vec![Diagnostic {
            rule_id: "ELARLOG",
            message: format!(
                "The {op_text} operator in the expression {name}(A {op_text} B) is unexpected. Should this be {name}(A) {op_text} B?"
            ),
            severity: Severity::Warning,
            byte_range: op.start_byte()..op.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn elarlog(source: &str) -> Vec<Diagnostic> {
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();
        let call = find_descendant_of_kind(root, "function_call").unwrap();
        eng.check_elarlog(call, source)
    }

    #[test]
    fn test_elarlog_fires_on_comparison_in_abs() {
        let diags = elarlog("x = abs(a > 1);\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "ELARLOG");
        assert_eq!((diags[0].line, diags[0].column), (1, 11));
        assert!(diags[0].message.contains("abs(A > B)"));
    }

    #[test]
    fn test_elarlog_names_the_outermost_operator_of_a_chain() {
        let diags = elarlog("x = numel(a > 1 < 2);\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.starts_with("The < operator"));
    }

    #[test]
    fn test_elarlog_silent_for_other_functions_and_non_comparisons() {
        assert!(elarlog("x = sqrt(a > 1);\n").is_empty());
        assert!(elarlog("x = sum(a > 1 < 2);\n").is_empty());
        assert!(elarlog("x = abs(a + 1);\n").is_empty());
        assert!(elarlog("x = abs(a, b > 1);\n").is_empty());
    }
}
