//! # SPRINTFN: `sprintf('\n')` -> `newline`
//!
//! `check_sprintf` flags `sprintf('\n')` (a single newline escape) and
//! suggests the `newline` function.

use super::*;

impl ReadabilityEngine {
    /// SPRINTFN: `sprintf('\n')` -> `newline`
    pub(crate) fn check_sprintf(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("SPRINTFN") {
            return;
        }
        let args = call_args(node);
        if args.len() != 1 || args[0].kind() != "string" {
            return;
        }
        let text = &source[args[0].start_byte()..args[0].end_byte()];
        if text == "'\\n'" || text == "\"\\n\"" {
            results.push(self.diag(
                "SPRINTFN",
                "For readability, consider using the 'newline' function instead of 'sprintf('\\n')'.",
                node,
                Some(Fix::new(node.start_byte()..node.end_byte(), "newline")),
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

    #[test]
    fn sprintf_newline_fires_and_is_fixed() {
        let src = "x = [a sprintf('\\n')];\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "SPRINTFN"), "got: {diags:?}");
        assert_eq!(malt_core::apply_fixes(src, &diags), "x = [a newline];\n");
    }

    #[test]
    fn sprintf_numeric_format_or_other_text_does_not_fire() {
        for src in [
            "x = sprintf('%d', n);\n",
            "x = sprintf('a\\n');\n",
            "x = sprintf(fmt);\n",
        ] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "SPRINTFN"), "{src}");
        }
    }
}
