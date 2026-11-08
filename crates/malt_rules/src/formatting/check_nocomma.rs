//! NOCOMMA check: extra (trailing) comma in a matrix/cell row.

use super::*;

impl FormattingEngine {
    /// For `matrix` and `cell` nodes, flag a trailing comma in a `row`
    /// (e.g., `[1, 2,]`) as unnecessary.
    pub(crate) fn check_nocomma(&self, node: Node, diagnostics: &mut Vec<Diagnostic>) {
        let child_count = node.child_count();
        for i in 0..child_count {
            let Some(child) = node.child(i) else {
                continue;
            };
            if child.kind() == "row" {
                self.check_row_trailing_comma(child, diagnostics);
            }
        }
    }

    /// Flag a `row` whose last element is a real (non-zero-width) comma.
    ///
    /// A zero-width comma is a missing separator inserted by the grammar for
    /// space-separated elements; a real comma at the end of a row is an extra
    /// trailing comma the user typed.
    fn check_row_trailing_comma(&self, row: Node, diagnostics: &mut Vec<Diagnostic>) {
        let count = row.child_count();
        if count == 0 {
            return;
        }
        let Some(last) = row.child(count - 1) else {
            return;
        };
        if last.kind() != "," {
            return;
        }
        // Zero-width commas are "missing" separators, not extra commas.
        if last.start_byte() >= last.end_byte() {
            return;
        }

        let pos = last.start_position();
        diagnostics.push(Diagnostic {
            rule_id: "NOCOMMA",
            message: "Extra comma is unnecessary.".to_string(),
            severity: Severity::Info,
            byte_range: last.start_byte()..last.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: Some(Fix::new(last.start_byte()..last.end_byte(), String::new())),
        });
    }
}

#[cfg(test)]
mod tests {

    use crate::formatting::tests::{has_id, lint};

    // -- NOCOMMA -------------------------------------------------------------

    #[test]
    fn nocomma_trailing_comma_in_matrix() {
        let source = "x = [1, 2,];\n";
        let diags = lint(source);
        assert!(has_id(&diags, "NOCOMMA"), "got: {diags:?}");
    }

    #[test]
    fn nocomma_trailing_comma_in_cell() {
        let source = "c = {1, 2,};\n";
        let diags = lint(source);
        assert!(has_id(&diags, "NOCOMMA"), "got: {diags:?}");
    }

    #[test]
    fn nocomma_ok_without_trailing_comma() {
        let source = "x = [1, 2, 3];\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "NOCOMMA"), "got: {diags:?}");
    }

    #[test]
    fn nocomma_ok_space_separated() {
        let source = "x = [1 2 3];\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "NOCOMMA"), "got: {diags:?}");
    }
}
