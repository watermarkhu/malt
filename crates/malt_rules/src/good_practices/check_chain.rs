use super::*;

impl GoodPracticesEngine {
    /// CHAIN: a chained comparison such as `a < b < c`. MATLAB evaluates it as
    /// `(a < b) < c`, comparing a logical value with `c`, which is rarely the
    /// intent.
    pub(crate) fn check_chain(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("CHAIN") || node.kind() != "comparison_operator" {
            return Vec::new();
        }
        // Report the outermost comparison of a chain once.
        if node
            .parent()
            .is_some_and(|p| p.kind() == "comparison_operator")
        {
            return Vec::new();
        }
        let Some(left) = node.named_child(0) else {
            return Vec::new();
        };
        if left.kind() != "comparison_operator" {
            return Vec::new();
        }
        let op = |n: Node| {
            let mut cursor = n.walk();
            let found = n
                .children(&mut cursor)
                .find(|c| !c.is_named())
                .map(|c| source[c.byte_range()].to_string());
            found.unwrap_or_default()
        };
        let (first, second) = (op(left), op(node));
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "CHAIN",
            message: format!(
                "Expressions like a {first} b {second} c are interpreted as (a {first} b) {second} c. Typically, to test a {first} b {second} c mathematically, if all arguments are numeric scalars, use (a {first} b) && (b {second} c), otherwise use (a {first} b) & (b {second} c)."
            ),
            severity: Severity::Info,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::{Config, Diagnostic};

    fn lint(source: &str) -> Vec<Diagnostic> {
        let engine = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_nodes(&*engine, source)
    }

    #[test]
    fn chain_fires_on_chained_comparison_once() {
        let d = lint("x = a < b < c;\n");
        assert_eq!(
            d.iter().filter(|d| d.rule_id == "CHAIN").count(),
            1,
            "{d:?}"
        );
        let d = lint("x = a < b < c < d;\n");
        assert_eq!(
            d.iter().filter(|d| d.rule_id == "CHAIN").count(),
            1,
            "{d:?}"
        );
    }

    #[test]
    fn chain_names_the_operators() {
        let d = lint("x = a == b ~= c;\n");
        let m = d.iter().find(|d| d.rule_id == "CHAIN").unwrap();
        assert!(m.message.contains("a == b ~= c"), "{}", m.message);
    }

    #[test]
    fn chain_ignores_single_comparisons_and_field_chains() {
        assert!(!has_id(&lint("x = a < b;\n"), "CHAIN"));
        assert!(!has_id(&lint("x = a.b.c.d.e(1);\n"), "CHAIN"));
        assert!(!has_id(&lint("x = (a < b) < c;\n"), "CHAIN"));
    }
}
