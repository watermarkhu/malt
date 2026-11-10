use super::*;

impl GoodPracticesEngine {
    /// SHOCIRAA: a short-circuit operator (`&&`/`||`) passed directly to `any`
    /// or `all`, as in `any(A || B)`. MATLAB reports only these two functions
    /// (probed); the operator inside `[ ]`, `{ }` or an unknown call is silent.
    pub(crate) fn check_shociraa(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("SHOCIRAA") || node.kind() != "boolean_operator" {
            return Vec::new();
        }

        let Some(call) = node
            .parent()
            .filter(|p| p.kind() == "arguments")
            .and_then(|p| p.parent())
            .filter(|p| p.kind() == "function_call")
        else {
            return Vec::new();
        };
        let Some(name @ ("any" | "all")) = get_function_call_name(call, source) else {
            return Vec::new();
        };

        let text = node_text(node, source);
        let op = if text.contains("&&") {
            "&&"
        } else if text.contains("||") {
            "||"
        } else {
            return Vec::new();
        };

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "SHOCIRAA",
            message: format!(
                "Using the {op} operator in the expression {name}(A {op} B) is probably unintended."
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
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::{Config, Diagnostic};

    fn lint(source: &str) -> Vec<Diagnostic> {
        let engine = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_nodes(&*engine, source)
    }

    #[test]
    fn shociraa_fires_on_short_circuit_argument_of_any_or_all() {
        // Probed: MATLAB reports `any(A || B)` and `all(A && B)`.
        let d = lint("x = any(a || b);\n");
        let m = d.iter().find(|d| d.rule_id == "SHOCIRAA").unwrap();
        assert!(m.message.contains("any(A || B)"), "{}", m.message);
        let d = lint("x = all(a && b);\n");
        let m = d.iter().find(|d| d.rule_id == "SHOCIRAA").unwrap();
        assert!(m.message.contains("all(A && B)"), "{}", m.message);
    }

    #[test]
    fn shociraa_silent_outside_any_and_all() {
        // Probed silent: an unknown function, a matrix, a plain assignment.
        assert!(!has_id(&lint("x = f(a && b);\n"), "SHOCIRAA"));
        assert!(!has_id(&lint("y = [a && b];\n"), "SHOCIRAA"));
        assert!(!has_id(&lint("q = a || b;\n"), "SHOCIRAA"));
    }
}
