use super::*;

impl BugsEngine {
    /// INCR / DECR: a statement `++x` or `--x`. MATLAB has no such operators:
    /// the statement is a double unary plus/minus that neither changes `x` nor
    /// stores a result.
    pub(crate) fn check_incr_decr(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "unary_operator" {
            return Vec::new();
        }
        let at_statement_level = node
            .parent()
            .is_some_and(|p| p.kind() == "source_file" || p.kind() == "block");
        if !at_statement_level {
            return Vec::new();
        }
        let Some(inner) = node.child_by_field_name("operand") else {
            return Vec::new();
        };
        if inner.kind() != "unary_operator" {
            return Vec::new();
        }
        let Some(var) = inner.child_by_field_name("operand") else {
            return Vec::new();
        };
        if var.kind() != "identifier" {
            return Vec::new();
        }
        let text: String = node_text(node, source).split_whitespace().collect();
        let name = node_text(var, source).trim();
        let (rule_id, message, sign) = if text.starts_with("++") {
            (
                "INCR",
                format!("++{name} operation does not increment the value of {name}. To increase the value by 1, use {name} = {name} + 1."),
                '+',
            )
        } else if text.starts_with("--") {
            (
                "DECR",
                format!("--{name} operation does not decrement the value of {name}. To decrease the value by 1, use {name} = {name} - 1."),
                '-',
            )
        } else {
            return Vec::new();
        };
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id,
            message,
            severity: Severity::Error,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            // Turns a no-op into a real update.
            fix: Some(
                Fix::new(
                    node.start_byte()..node.end_byte(),
                    format!("{name} = {name} {sign} 1"),
                )
                .unsafe_fix(),
            ),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    #[test]
    fn incr_fires_on_plus_plus() {
        let diags = node_diags("++x;\n");
        let d = diags.iter().find(|d| d.rule_id == "INCR").expect("INCR");
        assert!(d.message.starts_with("++x operation"), "got: {}", d.message);
    }

    #[test]
    fn decr_fires_on_minus_minus() {
        let diags = node_diags("--count;\n");
        assert!(has_id(&diags, "DECR"), "got: {diags:?}");
    }

    #[test]
    fn no_fire_on_self_assignment_or_single_unary() {
        let diags = node_diags("for i = 1:3\n    x = x + 1;\n    y = x - 1;\nend\n-x;\n");
        assert!(
            !has_id(&diags, "INCR") && !has_id(&diags, "DECR"),
            "got: {diags:?}"
        );
    }
}
