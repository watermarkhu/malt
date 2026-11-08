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
        // Where the keyword *will* be once the statements around it are
        // indented, not where it is now: a nested statement is re-indented in
        // the same pass as its parent, and measuring from the old column would
        // move one nesting level per pass.
        let indent = self.no4lp_config.indent_size;
        let parent_col = target_column(node, indent);
        let expected_indent = parent_col + indent;

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
                    // A `case` body hangs off the `case` line, which this check
                    // leaves where it is; `elseif`/`else` bodies follow the `if`.
                    let expected_indent =
                        if matches!(child.kind(), "case_clause" | "otherwise_clause")
                            && line_prefix_is_blank(source, child)
                        {
                            child.start_position().column + indent
                        } else {
                            expected_indent
                        };
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

/// Whether only blanks precede `node` on its line.
fn line_prefix_is_blank(source: &str, node: Node) -> bool {
    let start = line_start_byte(source, node.start_byte());
    source[start..node.start_byte()].trim().is_empty()
}

/// The column a control statement ends up at: one level in from the statement
/// whose body it sits in, or where it is when it sits in no such body.
fn target_column(node: Node, indent: usize) -> usize {
    let owner = node
        .parent()
        .filter(|p| p.kind() == "block")
        .and_then(|block| block.parent())
        .map(|o| {
            if matches!(
                o.kind(),
                "elseif_clause" | "else_clause" | "case_clause" | "otherwise_clause"
            ) {
                o.parent().unwrap_or(o)
            } else {
                o
            }
        })
        .filter(|o| {
            matches!(
                o.kind(),
                "for_statement" | "while_statement" | "if_statement" | "switch_statement"
            )
        });
    match owner {
        Some(owner) => target_column(owner, indent) + indent,
        None => node.start_position().column,
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

    /// Apply the fixes of one pass.
    fn fix_once(source: &str) -> String {
        let diags: Vec<_> = lint(source)
            .into_iter()
            .filter(|d| d.rule_id == "NO4LP")
            .collect();
        malt_core::apply_fixes(source, &diags)
    }

    #[test]
    fn no4lp_deep_nesting_is_fixed_in_one_pass() {
        // Each level used to move one pass at a time and never settled within the
        // pass limit for deep nesting.
        let depth = 14;
        let source = format!(
            "{}y = 1;\n{}",
            "if x\n".repeat(depth),
            "end\n".repeat(depth)
        );
        let once = fix_once(&source);
        assert_eq!(fix_once(&once), once, "a second pass must change nothing");
        let deepest = once.lines().find(|l| l.trim() == "y = 1;").unwrap();
        assert_eq!(deepest.len() - deepest.trim_start().len(), depth * 4);
    }

    #[test]
    fn no4lp_case_bodies_hang_off_the_case_line() {
        let source = "switch x\n    case 1\n        a = 1;\n    otherwise\n        a = 2;\nend\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "NO4LP"), "got: {diags:?}");

        let flat = "switch x\n    case 1\n    a = 1;\nend\n";
        assert_eq!(
            fix_once(flat),
            "switch x\n    case 1\n        a = 1;\nend\n"
        );
    }
}
