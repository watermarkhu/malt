use super::*;

impl BugsEngine {
    /// CMDAND / CMDOR: a statement such as `a &b` or `a |b` is parsed as a
    /// command-syntax call (`a('&b')`), not as a logical expression.
    pub(crate) fn check_command_logical(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "command" {
            return Vec::new();
        }
        let mut cursor = node.walk();
        let Some(arg) = node
            .children(&mut cursor)
            .find(|c| c.kind() == "command_argument")
        else {
            return Vec::new();
        };
        let text = node_text(arg, source);
        let (rule_id, message) = if text.starts_with('&') {
            (
                "CMDAND",
                "Use 'A && B' or 'A & B' to test whether A and B are both true in MATLAB.",
            )
        } else if text.starts_with('|') {
            (
                "CMDOR",
                "Use 'A || B' or 'A | B' to test whether either A or B is true in MATLAB.",
            )
        } else {
            return Vec::new();
        };
        let pos = node.start_position();
        // Separating the operator from its operand makes MATLAB parse the
        // statement as an expression instead of a command: a behaviour change.
        let fix = Fix::insert(
            arg.start_byte()
                + text
                    .chars()
                    .take_while(|c| *c == '&' || *c == '|')
                    .map(char::len_utf8)
                    .sum::<usize>(),
            " ",
        )
        .unsafe_fix();
        vec![Diagnostic {
            rule_id,
            message: message.to_string(),
            severity: Severity::Error,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: Some(fix),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    #[test]
    fn cmdand_fires_on_command_syntax_ampersand() {
        let diags = node_diags("a &b\n");
        assert!(has_id(&diags, "CMDAND"), "got: {diags:?}");
    }

    #[test]
    fn cmdor_fires_on_command_syntax_pipe() {
        let diags = node_diags("a |b\n");
        assert!(has_id(&diags, "CMDOR"), "got: {diags:?}");
    }

    #[test]
    fn no_fire_on_ordinary_command() {
        let diags = node_diags("hold on\n");
        assert!(
            !has_id(&diags, "CMDAND") && !has_id(&diags, "CMDOR"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn no_fire_on_expression_in_condition() {
        let diags = node_diags("if (a & b)\n    x = 1;\nend\n");
        assert!(!has_id(&diags, "CMDAND"), "got: {diags:?}");
    }
}
