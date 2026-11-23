//! # COMNL: newline after comma acts as a row separator
//!
//! `check_comnl` flags a comma that ends a line inside `[ ]` or `{ }`: the
//! newline acts as a row separator, so the comma should be a semicolon (or the
//! line should continue with an ellipsis).

use super::*;

impl ReadabilityEngine {
    /// COMNL: A newline following a comma in a matrix or cell acts as a row
    /// separator; suggest replacing the comma with a semicolon or using an
    /// ellipsis to continue the row. Probed: `[1,` + newline, `{1,` + newline,
    /// and a comma that ends a line inside a cell nested in a matrix all fire.
    /// A comma followed by a `%` comment is COMNC instead, and `...` escapes.
    pub(crate) fn check_comnl(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("COMNL") {
            return;
        }
        let mut commas = Vec::new();
        collect_element_commas(node, &mut commas);
        for comma in commas {
            if comma_ends_line(comma, source) {
                results.push(self.diag(
                    "COMNL",
                    "Newline following comma acts as a row separator. Replace the comma with a semicolon to make the row separation clearer. Alternatively, use an ellipsis (...) to continue the current row on the next line.",
                    comma,
                    Some(Fix::new(comma.start_byte()..comma.end_byte(), ";")),
                ));
            }
        }
    }
}

/// Collects the `,` tokens that separate the elements of `node`, a matrix or a
/// cell. Nested matrices and cells are visited on their own; calls and indexing
/// (`f(1,` ...) are skipped because their commas are not element separators.
fn collect_element_commas<'a>(node: tree_sitter::Node<'a>, out: &mut Vec<tree_sitter::Node<'a>>) {
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "," => out.push(child),
            "matrix" | "cell" | "function_call" => {}
            _ => collect_element_commas(child, out),
        }
    }
}

/// Whether only blanks separate `comma` from the line break that follows it.
/// A `%` comment, an ellipsis, or a `;` after the comma all return false.
fn comma_ends_line(comma: tree_sitter::Node, source: &str) -> bool {
    source[comma.end_byte()..]
        .trim_start_matches([' ', '\t'])
        .starts_with(['\n', '\r'])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    // -- COMNL ---------------------------------------------------------------

    #[test]
    fn comnl_trailing_comma_before_newline_fires() {
        let diags = lint_nodes(&*engine(), "x = [1, 2,\n3, 4];\n");
        assert!(has_id(&diags, "COMNL"), "got: {diags:?}");
    }

    #[test]
    fn comnl_multiple_rows_fires_each_comma() {
        let diags = lint_nodes(&*engine(), "x = [1, 2,\n3, 4,\n5, 6];\n");
        let count = diags.iter().filter(|d| d.rule_id == "COMNL").count();
        assert_eq!(count, 2, "got: {diags:?}");
    }

    #[test]
    fn comnl_fires_for_comma_before_newline_in_cell() {
        // Probed: `{ 'a',` + newline and `{` + `'a',` + newline both fire.
        let diags = lint_nodes(&*engine(), "c = { 'a',\n 'b' };\n");
        assert!(has_id(&diags, "COMNL"), "got: {diags:?}");
        let diags = lint_nodes(&*engine(), "c = {\n 'a',\n 'b' };\n");
        assert!(has_id(&diags, "COMNL"), "got: {diags:?}");
    }

    #[test]
    fn comnl_every_line_end_comma_in_cell_fires() {
        // Probed: `{1,` `2,` `3};` gives COMNL on the first two lines; the
        // comma before the closing brace does not end a line.
        let diags = lint_nodes(&*engine(), "q = {1,\n 2,\n 3};\n");
        let count = diags.iter().filter(|d| d.rule_id == "COMNL").count();
        assert_eq!(count, 2, "got: {diags:?}");
    }

    #[test]
    fn comnl_cell_comma_inside_matrix_fires() {
        // Probed: `[{1,` + newline fires once on the cell's comma.
        let diags = lint_nodes(&*engine(), "y = [{1,\n 2}, 3];\n");
        let count = diags.iter().filter(|d| d.rule_id == "COMNL").count();
        assert_eq!(count, 1, "got: {diags:?}");
    }

    #[test]
    fn comnl_comment_after_comma_does_not_fire() {
        // Probed: `[1, % c` gives COMNC, not COMNL.
        let diags = lint_nodes(&*engine(), "x = [1, % c\n 2];\n");
        assert!(!has_id(&diags, "COMNL"), "got: {diags:?}");
        let diags = lint_nodes(&*engine(), "x = {1, % c\n 2};\n");
        assert!(!has_id(&diags, "COMNL"), "got: {diags:?}");
    }

    #[test]
    fn comnl_cell_with_ellipsis_does_not_fire() {
        let diags = lint_nodes(&*engine(), "c = { 'a', ...\n 'b' };\n");
        assert!(!has_id(&diags, "COMNL"), "got: {diags:?}");
    }

    #[test]
    fn comnl_single_row_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = [1, 2, 3, 4];\n");
        assert!(!has_id(&diags, "COMNL"), "got: {diags:?}");
    }

    #[test]
    fn comnl_no_trailing_comma_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = [1, 2, 3\n4, 5, 6];\n");
        assert!(!has_id(&diags, "COMNL"), "got: {diags:?}");
    }

    #[test]
    fn comnl_semicolon_separator_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = [1, 2;\n3, 4];\n");
        assert!(!has_id(&diags, "COMNL"), "got: {diags:?}");
    }

    #[test]
    fn comnl_ellipsis_continuation_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = [1, 2, ...\n3, 4];\n");
        assert!(!has_id(&diags, "COMNL"), "got: {diags:?}");
    }

    #[test]
    fn comnl_semicolon_after_trailing_comma_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = [1, 2,;\n3, 4];\n");
        assert!(!has_id(&diags, "COMNL"), "got: {diags:?}");
    }

    #[test]
    fn comnl_disabled_via_config_does_not_fire() {
        let config =
            Config::from_toml("[lint.rules.READABILITY_ENGINE]\ndisabled_checks = [\"COMNL\"]\n")
                .expect("valid config");
        let rule = ReadabilityEngine::from_config(&config);
        let diags = lint_nodes(&*rule, "x = [1, 2,\n3, 4];\n");
        assert!(!has_id(&diags, "COMNL"), "got: {diags:?}");
    }

    #[test]
    fn comnl_fix_replaces_comma_with_semicolon() {
        let source = "x = [1, 2,\n3, 4];\n";
        let diags = lint_nodes(&*engine(), source);
        let comnl: Vec<_> = diags.iter().filter(|d| d.rule_id == "COMNL").collect();
        assert_eq!(comnl.len(), 1, "got: {diags:?}");
        let fix = comnl[0].fix.as_ref().expect("COMNL should provide a fix");
        assert_eq!(&source[fix.byte_range.clone()], ",");
        assert_eq!(fix.replacement, ";");
    }
}
