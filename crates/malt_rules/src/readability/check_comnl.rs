//! # COMNL: newline after comma acts as a row separator
//!
//! `check_comnl` flags trailing commas in matrix rows when the following
//! row is separated by a newline, suggesting a semicolon (or an ellipsis)
//! to make the row separation explicit.

use super::*;

impl ReadabilityEngine {
    /// COMNL: A newline following a comma in a matrix acts as a row
    /// separator; suggest replacing the comma with a semicolon or using an
    /// ellipsis to continue the row.
    pub(crate) fn check_comnl(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("COMNL") {
            return;
        }

        // Collect `row` children in source order using children() + kind
        // filter (NOT named_children): `line_continuation`/`...` is a NAMED
        // extra and would leak into named_children().
        let rows: Vec<tree_sitter::Node> = node
            .children(&mut node.walk())
            .filter(|c| c.kind() == "row")
            .collect();

        for i in 0..rows.len().saturating_sub(1) {
            let row = rows[i];
            let next_row = rows[i + 1];

            let Some(comma) = find_trailing_comma(row) else {
                continue;
            };

            // Source gap between this row and the next row.
            let gap = &source[row.end_byte()..next_row.start_byte()];

            // Fire ONLY when a NEWLINE acts as the row separator:
            //   - gap contains '\n' (row separator is a newline), AND
            //   - gap does NOT contain ';' (semicolon is explicit row
            //     separator; replacing the comma would create a broken `;;`),
            //   AND
            //   - gap does NOT contain '...' (ellipsis escapes the newline).
            if gap.contains('\n') && !gap.contains(';') && !gap.contains("...") {
                let range = comma.start_byte()..comma.end_byte();
                results.push(self.diag(
                    "COMNL",
                    "Newline following comma acts as a row separator. Replace the comma with a semicolon to make the row separation clearer. Alternatively, use an ellipsis (...) to continue the current row on the next line.",
                    comma,
                    Some(Fix::new(range, ";")),
                ));
            }
        }
    }
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
