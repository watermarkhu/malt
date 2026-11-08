//! NCOMMA check: output variables not separated by commas.

use super::*;

impl FormattingEngine {
    /// For `multioutput_variable` nodes (assignment LHS or function output
    /// declaration), flag elements separated by whitespace instead of commas
    /// (e.g., `[a b] = f()` should be `[a, b]`).
    pub(crate) fn check_ncomma(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let mut elements: Vec<Node> = Vec::new();
        let child_count = node.child_count();
        for i in 0..child_count {
            let Some(child) = node.child(i) else {
                continue;
            };
            if matches!(
                child.kind(),
                "identifier" | "field_expression" | "ignored_argument" | "function_call"
            ) {
                elements.push(child);
            }
        }

        for pair in elements.windows(2) {
            let gap = &source[pair[0].end_byte()..pair[1].start_byte()];
            if gap.contains(',') {
                continue;
            }
            let pos = pair[1].start_position();
            let insert_pos = pair[1].start_byte();
            diagnostics.push(Diagnostic {
                rule_id: "NCOMMA",
                message: "Best practice is to separate output variables with commas.".to_string(),
                severity: Severity::Info,
                byte_range: pair[0].end_byte()..pair[1].start_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: Some(Fix::insert(insert_pos, ", ")),
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::formatting::tests::{has_id, lint};

    // -- NCOMMA --------------------------------------------------------------

    #[test]
    fn ncomma_space_separated_output() {
        let source = "[a b] = f();\n";
        let diags = lint(source);
        assert!(has_id(&diags, "NCOMMA"), "got: {diags:?}");
    }

    #[test]
    fn ncomma_comma_separated_output_ok() {
        let source = "[a, b] = f();\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "NCOMMA"), "got: {diags:?}");
    }

    #[test]
    fn ncomma_space_separated_function_output() {
        let source = "function [a b] = f()\n    a = 1;\n    b = 2;\nend\n";
        let diags = lint(source);
        assert!(has_id(&diags, "NCOMMA"), "got: {diags:?}");
    }
}
