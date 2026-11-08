//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// ROWLN: Matrix rows must be the same length.
    pub(crate) fn check_matrix_rows(
        &self,
        root: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.check_matrix_rows_dfs(root, source, diagnostics);
    }

    /// DFS to find matrix nodes and check row consistency.
    pub(crate) fn check_matrix_rows_dfs(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "matrix" {
            self.validate_matrix_rows(node, source, diagnostics);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_matrix_rows_dfs(child, source, diagnostics);
        }
    }

    /// Validate that all rows in a matrix have the same number of elements.
    pub(crate) fn validate_matrix_rows(
        &self,
        matrix_node: tree_sitter::Node,
        _source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut row_lengths: Vec<usize> = Vec::new();
        let mut cursor = matrix_node.walk();

        for child in matrix_node.children(&mut cursor) {
            if child.kind() == "row" {
                // Count the number of expression elements in this row
                let mut elem_count = 0;
                let mut inner_cursor = child.walk();
                for elem in child.children(&mut inner_cursor) {
                    let k = elem.kind();
                    // Skip punctuation/separators
                    if k != "," && k != ";" && k != "[" && k != "]" && k != " " && elem.is_named() {
                        elem_count += 1;
                    }
                }
                row_lengths.push(elem_count);
            }
        }

        // Check if all rows have the same length (only if there are multiple rows)
        if row_lengths.len() > 1 {
            let first_len = row_lengths[0];
            if row_lengths
                .iter()
                .any(|&len| len != first_len && first_len > 0 && len > 0)
            {
                let pos = matrix_node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "ROWLN",
                    message: "All matrix rows must be the same length.".to_string(),
                    severity: Severity::Error,
                    byte_range: matrix_node.start_byte()..matrix_node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }
    }
}
