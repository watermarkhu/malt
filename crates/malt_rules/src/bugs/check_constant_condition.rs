use super::*;

impl BugsEngine {
    /// CTRUE / CFALSE: Condition is always true or always false.
    ///
    /// Checks `if` and `while` statement conditions for literal `true`, `false`,
    /// numeric `0`, and numeric `1`.
    pub(crate) fn check_constant_condition(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        let kind = node.kind();
        if kind != "if_statement" && kind != "while_statement" {
            return Vec::new();
        }

        let cond = match node.child_by_field_name("condition") {
            Some(c) => c,
            None => {
                // Fallback: for if_statement, condition is typically the second child
                // (first child is the `if` keyword).
                match find_condition_child(node) {
                    Some(c) => c,
                    None => return Vec::new(),
                }
            }
        };

        let cond_text = node_text(cond, source).trim();
        let pos = cond.start_position();

        if is_always_true(cond_text) {
            vec![Diagnostic {
                rule_id: "CTRUE",
                message: "This logical comparison always returns true. Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)?".to_string(),
                severity: Severity::Error,
                byte_range: cond.start_byte()..cond.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }]
        } else if is_always_false(cond_text) {
            vec![Diagnostic {
                rule_id: "CFALSE",
                message: "This logical comparison always returns false. Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)?".to_string(),
                severity: Severity::Error,
                byte_range: cond.start_byte()..cond.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- CTRUE / CFALSE ------------------------------------------------------

    #[test]
    fn ctrue_fires_on_if_true() {
        let src = "if true\n    x = 1;\nend\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "CTRUE"), "got: {diags:?}");
    }

    #[test]
    fn ctrue_no_fire_on_real_condition() {
        let src = "if x > 0\n    x = 1;\nend\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "CTRUE"), "got: {diags:?}");
    }

    #[test]
    fn cfalse_fires_on_while_zero() {
        let src = "while 0\n    x = 1;\nend\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "CFALSE"), "got: {diags:?}");
    }

    #[test]
    fn cfalse_no_fire_on_real_condition() {
        let src = "while x > 0\n    x = 1;\nend\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "CFALSE"), "got: {diags:?}");
    }
}
