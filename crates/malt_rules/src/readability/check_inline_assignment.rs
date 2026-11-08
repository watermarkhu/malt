//! # ASGSL: assignment inside a control-flow condition
//!
//! `check_inline_assignment` flags assignments that appear inside `if` /
//! `while` / `for` condition expressions and suggests assigning on a
//! separate line for clarity.

use super::*;

impl ReadabilityEngine {
    /// ASGSL: Assignment inside a control-flow condition.
    pub(crate) fn check_inline_assignment(
        &self,
        node: tree_sitter::Node,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("ASGSL") {
            return;
        }

        // Walk ancestors to see if this assignment is inside a condition
        let mut current = node.parent();
        while let Some(parent) = current {
            match parent.kind() {
                "if_statement" | "while_statement" | "for_statement" => {
                    // Check if the assignment is in the condition part
                    // (first child of if/while, second child of for)
                    results.push(self.diag(
                        "ASGSL",
                        "Assignment to variable might be unnecessary.",
                        node,
                        None,
                    ));
                    return;
                }
                "block" | "source_file" | "function_definition" => {
                    // Reached a block boundary — not in a condition
                    return;
                }
                _ => {
                    current = parent.parent();
                }
            }
        }
    }
}
