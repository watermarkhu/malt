//! NO4LP check: use 4-space indentation in loop/conditional bodies.

use super::*;

// ---------------------------------------------------------------------------
// NO4LP: Use 4-space indentation in loop/conditional bodies
// ---------------------------------------------------------------------------

impl FormattingEngine {
    /// For `for_statement`, `while_statement`, `if_statement`, and
    /// `switch_statement`, check that the `block` body is indented by
    /// `indent_size` spaces relative to the parent keyword.
    pub(crate) fn check_no4lp(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let parent_col = node.start_position().column;
        let expected_indent = parent_col + self.no4lp_config.indent_size;

        // Find all `block` children (there may be multiple in if/switch).
        let child_count = node.child_count();
        for i in 0..child_count {
            let Some(child) = node.child(i) else {
                continue;
            };
            match child.kind() {
                "block" => {
                    self.check_block_indentation(child, expected_indent, source, diagnostics);
                }
                // elseif_clause and else_clause also contain blocks.
                "elseif_clause" | "else_clause" | "case_clause" | "otherwise_clause" => {
                    let inner_count = child.child_count();
                    for j in 0..inner_count {
                        if let Some(inner) = child.child(j) {
                            if inner.kind() == "block" {
                                self.check_block_indentation(
                                    inner,
                                    expected_indent,
                                    source,
                                    diagnostics,
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Check that each statement in a block starts at the expected column.
    fn check_block_indentation(
        &self,
        block: Node,
        expected_indent: usize,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let child_count = block.child_count();
        for i in 0..child_count {
            let Some(stmt) = block.child(i) else {
                continue;
            };
            // Skip non-statement nodes (e.g., semicolons, newlines).
            if is_punctuation(stmt.kind()) || stmt.kind() == "comment" {
                continue;
            }

            let actual_col = stmt.start_position().column;
            if actual_col != expected_indent {
                // Only flag if the line starts with spaces (not tabs or mixed).
                let line_start = line_start_byte(source, stmt.start_byte());
                let prefix = &source[line_start..stmt.start_byte()];
                if prefix.chars().all(|c| c == ' ') {
                    let pos = stmt.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "NO4LP",
                        message: "Parentheses are not needed in a FOR statement.".to_string(),
                        severity: Severity::Info,
                        byte_range: line_start..stmt.start_byte(),
                        line: pos.row + 1,
                        column: 1,
                        fix: Some(Fix::new(
                            line_start..stmt.start_byte(),
                            " ".repeat(expected_indent),
                        )),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::formatting::tests::{has_id, lint};

    // -- NO4LP ---------------------------------------------------------------

    #[test]
    fn no4lp_loop_body_unindented() {
        let source = "\
function f()
for i = 1:10
x = i;
end
end
";
        let diags = lint(source);
        assert!(has_id(&diags, "NO4LP"), "got: {diags:?}");
    }

    #[test]
    fn no4lp_loop_body_indented() {
        let source = "\
function f()
    for i = 1:10
        x = i;
    end
end
";
        let diags = lint(source);
        assert!(!has_id(&diags, "NO4LP"), "got: {diags:?}");
    }
}
