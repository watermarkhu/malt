//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// GPNES: Global/persistent must be in outermost function (not nested).
    pub(crate) fn check_gpnes(&self, ctx: &FileContext, diagnostics: &mut Vec<Diagnostic>) {
        // Walk the tree to find nested functions with global/persistent
        let root = ctx.tree.root_node();
        Self::find_gp_in_nested(root, 0, diagnostics);
    }

    /// Recursively find global/persistent in nested functions.
    pub(crate) fn find_gp_in_nested(
        node: tree_sitter::Node,
        depth: usize,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_definition" {
                if depth > 0 {
                    // This is a nested function; check for global/persistent
                    Self::find_gp_statements(child, diagnostics);
                }
                Self::find_gp_in_nested(child, depth + 1, diagnostics);
            } else if child.kind() != "methods" && child.kind() != "class_definition" {
                Self::find_gp_in_nested(child, depth, diagnostics);
            }
        }
    }

    /// Find global/persistent statements within a function definition.
    pub(crate) fn find_gp_statements(
        func_node: tree_sitter::Node,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut cursor = func_node.walk();
        for child in func_node.children(&mut cursor) {
            match child.kind() {
                "global_operator" | "persistent_operator" => {
                    let pos = child.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "GPNES",
                        message: "A GLOBAL or PERSISTENT declaration must be in the outermost function where it is used.".to_string(),
                        severity: Severity::Error,
                        byte_range: child.start_byte()..child.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
                "block" => {
                    // Recurse into the block to find global/persistent
                    Self::find_gp_statements(child, diagnostics);
                }
                _ => {}
            }
        }
    }
}
