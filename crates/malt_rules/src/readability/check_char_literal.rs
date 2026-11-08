//! # CHARTEN: `char(10)` → `newline`
//!
//! `check_char_literal` replaces `char(10)` with the clearer `newline`
//! function call.

use super::*;

impl ReadabilityEngine {
    /// CHARTEN: `char(10)` → `newline`
    pub(crate) fn check_char_literal(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("CHARTEN") {
            return;
        }

        let args = match find_arguments(node) {
            Some(a) => a,
            None => return,
        };

        let named_children: Vec<_> = (0..args.named_child_count())
            .filter_map(|i| args.named_child(i))
            .collect();

        if named_children.len() != 1 {
            return;
        }

        let arg_text = &source[named_children[0].start_byte()..named_children[0].end_byte()];
        if arg_text.trim() == "10" {
            results.push(self.diag(
                "CHARTEN",
                "For readability, consider using 'newline' instead of 'char(10)'.",
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

    // -- CHARTEN -------------------------------------------------------------

    #[test]
    fn charten_char_10_fires() {
        let diags = lint_nodes(&*engine(), "x = char(10);\n");
        assert!(has_id(&diags, "CHARTEN"), "got: {diags:?}");
    }

    #[test]
    fn charten_char_other_value_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = char(9);\n");
        assert!(!has_id(&diags, "CHARTEN"), "got: {diags:?}");
    }
}
