//! # SPRINTFN: `sprintf('%d', x)` → `num2str(x)`
//!
//! `check_sprintf` suggests `num2str` for `sprintf` calls whose only purpose
//! is converting a number with a simple numeric format specifier.

use super::*;

impl ReadabilityEngine {
    /// SPRINTFN: `sprintf('%d', x)` → `num2str(x)`
    pub(crate) fn check_sprintf(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("SPRINTFN") {
            return;
        }

        let args = match find_arguments(node) {
            Some(a) => a,
            None => return,
        };

        let named_children: Vec<_> = (0..args.named_child_count())
            .filter_map(|i| args.named_child(i))
            .collect();

        if named_children.len() != 2 {
            return;
        }

        let fmt_text = &source[named_children[0].start_byte()..named_children[0].end_byte()];

        // Check for simple numeric format specifiers
        let fmt_stripped = fmt_text
            .trim_start_matches('\'')
            .trim_end_matches('\'')
            .trim_start_matches('"')
            .trim_end_matches('"');

        if matches!(fmt_stripped, "%d" | "%i" | "%f" | "%g" | "%e") {
            results.push(self.diag(
                "SPRINTFN",
                "For readability, consider using the 'newline' function instead of 'sprintf('\\n')'.",
                node,
                // num2str formats differently from sprintf: not a replacement.
                None,
            ));
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

    // -- SPRINTFN ------------------------------------------------------------

    #[test]
    fn sprintfn_simple_numeric_format_fires() {
        let diags = lint_nodes(&*engine(), "x = sprintf('%d', y);\n");
        assert!(has_id(&diags, "SPRINTFN"), "got: {diags:?}");
    }

    #[test]
    fn sprintfn_string_format_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = sprintf('%s', y);\n");
        assert!(!has_id(&diags, "SPRINTFN"), "got: {diags:?}");
    }
}
