//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// IDXCOLND: END operator used outside of an indexing expression.
    pub(crate) fn check_end_operator(
        &self,
        root: tree_sitter::Node,
        _source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        Self::check_end_dfs(root, false, diagnostics);
    }

    /// DFS to find `end` keywords used as values outside of indexing contexts.
    pub(crate) fn check_end_dfs(
        node: tree_sitter::Node,
        in_index: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "function_call" => {
                // The arguments of a function_call could be indexing context
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    // Arguments inside function_call are in indexing context
                    // (since function_call is also used for array indexing)
                    Self::check_end_dfs(child, true, diagnostics);
                }
                return;
            }
            "cell_index" => {
                // Cell indexing — end is valid here
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    Self::check_end_dfs(child, true, diagnostics);
                }
                return;
            }
            "end" | "end_operator" if !in_index => {
                // `end` used as a value (not as block terminator)
                let pos = node.start_position();
                // Only flag if this looks like an end used as a value expression,
                // not a block-closing end keyword
                let parent = node.parent();
                let is_block_end = parent
                    .map(|p| {
                        matches!(
                            p.kind(),
                            "function_definition"
                                | "if_statement"
                                | "for_statement"
                                | "while_statement"
                                | "switch_statement"
                                | "try_statement"
                                | "class_definition"
                                | "properties"
                                | "methods"
                                | "events"
                                | "enumeration"
                                | "parfor"
                                | "spmd_statement"
                                | "elseif_clause"
                                | "else_clause"
                                | "case_clause"
                                | "otherwise_clause"
                                | "catch_clause"
                        )
                    })
                    .unwrap_or(false);

                if !is_block_end {
                    diagnostics.push(Diagnostic {
                        rule_id: "IDXCOLND",
                        message: "The END operator must be used within an array index expression."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::check_end_dfs(child, in_index, diagnostics);
        }
    }
}
