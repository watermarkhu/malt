//! NOCOMMA check: an extra comma between statements.

use super::*;

impl FormattingEngine {
    /// Flag a comma token that only repeats a separator. MATLAB reports the
    /// second comma of `,,` and a comma after `;` or `,` (`x = 1;,`), and a comma
    /// right after `try` (`try, a = 1;`). A comma after an expression
    /// (`if x, disp(1), end`) or after `catch` is a normal separator.
    ///
    /// Commas inside matrices, cells and argument lists are left alone: a
    /// doubled comma there is a different error (TWOCM).
    pub(crate) fn check_nocomma(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // A zero-width comma is a separator the parser inserted, not one the user typed.
        if node.start_byte() >= node.end_byte() || Self::inside_bracket_or_call(node) {
            return;
        }
        let before = source[..node.start_byte()].trim_end_matches([' ', '\t']);
        let extra = before.ends_with(';') || before.ends_with(',') || ends_with_word(before, "try");
        if !extra {
            return;
        }

        let pos = node.start_position();
        diagnostics.push(Diagnostic {
            rule_id: "NOCOMMA",
            message: "Extra comma is unnecessary.".to_string(),
            severity: Severity::Info,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: Some(Fix::new(node.start_byte()..node.end_byte(), String::new())),
        });
    }

    /// True when the comma sits inside `[]`, `{}`, `()` of a call or an arguments block.
    fn inside_bracket_or_call(node: Node) -> bool {
        let mut current = node.parent();
        while let Some(ancestor) = current {
            if matches!(
                ancestor.kind(),
                "matrix" | "cell" | "function_call" | "arguments" | "arguments_statement"
            ) {
                return true;
            }
            current = ancestor.parent();
        }
        false
    }
}

/// True when `text` ends with the keyword `word` as a whole word.
fn ends_with_word(text: &str, word: &str) -> bool {
    text.strip_suffix(word).is_some_and(|rest| {
        !rest
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

#[cfg(test)]
mod tests {

    use crate::formatting::tests::{has_id, lint};

    // -- NOCOMMA -------------------------------------------------------------

    #[test]
    fn nocomma_doubled_comma_after_semicolon() {
        // MATLAB R2026b reports the comma after `;` (column 7).
        let source = "x = 1;,\n";
        let diags = lint(source);
        assert!(has_id(&diags, "NOCOMMA"), "got: {diags:?}");
    }

    #[test]
    fn nocomma_second_of_two_commas() {
        // MATLAB reports the second comma of `,,` (column 7).
        let source = "x = 1,, y = 2;\n";
        let diags = lint(source);
        assert_eq!(diags.iter().filter(|d| d.rule_id == "NOCOMMA").count(), 1);
    }

    #[test]
    fn nocomma_comma_after_try() {
        // MATLAB reports `try,` but not the comma after `catch`.
        let source = "try, a = 1; catch, b = 2; end\n";
        let diags = lint(source);
        let nocomma: Vec<_> = diags.iter().filter(|d| d.rule_id == "NOCOMMA").collect();
        assert_eq!(nocomma.len(), 1, "got: {diags:?}");
        assert_eq!(nocomma[0].column, 4);
    }

    #[test]
    fn nocomma_comma_after_semicolon_in_control_block() {
        let source = "x = 1;\nwhile k < 3, k = k + 1;, end\n";
        let diags = lint(source);
        assert!(has_id(&diags, "NOCOMMA"), "got: {diags:?}");
    }

    #[test]
    fn nocomma_ok_for_separator_commas() {
        assert!(!has_id(&lint("if x, disp(1), end\n"), "NOCOMMA"));
        assert!(!has_id(&lint("x = 1, y = 2\n"), "NOCOMMA"));
        assert!(!has_id(&lint("for k=1:3, disp(k), end\n"), "NOCOMMA"));
    }

    #[test]
    fn nocomma_ok_for_trailing_comma_in_matrix_and_cell() {
        // MATLAB R2026b does not report `[1, 2,]` or `{1, 2,}` as NOCOMMA.
        assert!(!has_id(&lint("x = [1, 2,];\n"), "NOCOMMA"));
        assert!(!has_id(&lint("c = {1, 2,};\n"), "NOCOMMA"));
    }

    #[test]
    fn nocomma_ok_space_separated() {
        assert!(!has_id(&lint("x = [1 2 3];\n"), "NOCOMMA"));
    }
}
