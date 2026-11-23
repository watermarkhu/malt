//! See the parent module `super` for the shared engine and helpers.

use super::*;

/// A numeric literal, optionally signed: the only elements whose width MATLAB knows.
fn is_number_literal(node: tree_sitter::Node) -> bool {
    match node.kind() {
        "number" => true,
        "unary_operator" => {
            let mut cursor = node.walk();
            let operands: Vec<_> = node.named_children(&mut cursor).collect();
            operands.len() == 1 && operands[0].kind() == "number"
        }
        _ => false,
    }
}

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
        let mut rows: Vec<tree_sitter::Node> = Vec::new();
        let mut cursor = matrix_node.walk();

        for child in matrix_node.children(&mut cursor) {
            if child.kind() == "row" {
                // Count the number of expression elements in this row. Probed: MATLAB
                // reports ROWLN only when every element is a number literal; variables,
                // calls, indexing, strings, ranges and nested matrices have widths it
                // does not know, so any other element leaves the matrix alone.
                let mut elem_count = 0;
                let mut inner_cursor = child.walk();
                for elem in child.children(&mut inner_cursor) {
                    if !elem.is_named() || elem.kind() == "comment" {
                        continue;
                    }
                    if !is_number_literal(elem) {
                        return;
                    }
                    elem_count += 1;
                }
                row_lengths.push(elem_count);
                rows.push(child);
            }
        }

        // Check if all rows have the same length (only if there are multiple rows)
        if row_lengths.len() > 1 {
            let first_len = row_lengths[0];
            // MATLAB reports the first row whose length differs from the first
            // row's (probed: `[1 2; 3]` at the `3`, `[1 2 3; 4 5 6; 7 8]` at `7`).
            if let Some(index) = row_lengths
                .iter()
                .position(|&len| len != first_len && first_len > 0 && len > 0)
            {
                let row = rows[index];
                let pos = row.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "ROWLN",
                    message: "All matrix rows must be the same length.".to_string(),
                    severity: Severity::Error,
                    byte_range: row.start_byte()..row.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    #[test]
    fn rowln_is_reported_at_the_first_mismatching_row() {
        // Probed: MATLAB reports ROWLN at the `3` of `[1 2; 3]`, and at the third
        // row of `[1 2` / ` 3 4` / ` 5`, not at the start of the matrix.
        let engine = LanguageSpecEngine::from_config(&Config::default());
        let rowln = |src: &str| -> Vec<(usize, usize)> {
            lint_file(&*engine, src)
                .iter()
                .filter(|d| d.rule_id == "ROWLN")
                .map(|d| (d.line, d.column))
                .collect()
        };
        assert_eq!(rowln("x = [1 2; 3];\n"), vec![(1, 11)]);
        assert_eq!(rowln("x = [1 2\n 3 4\n 5];\n"), vec![(3, 2)]);
        // Silent when every row has the same length, also across a comment.
        assert!(rowln("x = [1 2; 3 4];\n").is_empty());
        assert!(rowln("x = [1 2 % c\n 3 4];\n").is_empty());
    }

    fn engine() -> Box<dyn Rule> {
        LanguageSpecEngine::from_config(&Config::default())
    }

    // Probed: MATLAB reports ROWLN for literal rows of different widths.
    #[test]
    fn rowln_fires_on_literal_rows_of_different_widths() {
        assert!(has_id(&lint_file(&*engine(), "m = [1 2; 3];\n"), "ROWLN"));
        assert!(has_id(
            &lint_file(&*engine(), "m = [1 2; -3 4 5];\n"),
            "ROWLN"
        ));
        assert!(has_id(
            &lint_file(&*engine(), "m = [1 2; 3 4 5]';\n"),
            "ROWLN"
        ));
    }

    // Probed: MATLAB is silent when an element's width is not a number literal.
    #[test]
    fn rowln_silent_on_elements_with_unknown_width() {
        for source in [
            "m = [a; b c];\n",
            "m = [stack; r2 c2];\n",
            "m = [1 2; 'ab'];\n",
            "m = [1:2 3; 4 5 6];\n",
            "m = [[1 2] 3; 4 5 6];\n",
            "m = [1+2 3; 4 5 6 7];\n",
            "m = [1 2; rand 3 4];\n",
            "m = [x(1) x(2); 1 2 3];\n",
        ] {
            assert!(
                !has_id(&lint_file(&*engine(), source), "ROWLN"),
                "unexpected ROWLN for {source:?}"
            );
        }
    }

    #[test]
    fn rowln_silent_when_continuation_joins_rows_to_equal_width() {
        let source = "m = [1 2 ...\n 3; 4 5 6];\n";
        assert!(!has_id(&lint_file(&*engine(), source), "ROWLN"));
    }
}
