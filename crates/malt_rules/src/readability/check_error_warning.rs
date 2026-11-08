//! # SPERR / SPWRN: `error`/`warning` without a message ID
//!
//! `check_error_warning` flags single-string-literal `error('msg')` and
//! `warning('msg')` calls that omit the `component:mnemonic` message ID
//! recommended by Code Analyzer.

use super::*;

impl ReadabilityEngine {
    /// SPERR / SPWRN: `error('msg')` / `warning('msg')` without message ID
    pub(crate) fn check_error_warning(
        &self,
        node: tree_sitter::Node,
        source: &str,
        check_id: &'static str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled(check_id) {
            return;
        }

        let args = match find_arguments(node) {
            Some(a) => a,
            None => return,
        };

        let named_children: Vec<_> = (0..args.named_child_count())
            .filter_map(|i| args.named_child(i))
            .collect();

        // Single-argument error/warning call — no message ID
        if named_children.len() == 1 {
            let arg_text = &source[named_children[0].start_byte()..named_children[0].end_byte()];
            // Only flag if argument looks like a string literal
            if arg_text.starts_with('\'') || arg_text.starts_with('"') {
                let message = if check_id == "SPERR" {
                    "ERROR takes SPRINTF-like arguments directly."
                } else {
                    "WARNING takes SPRINTF-like arguments directly."
                };
                results.push(self.diag(check_id, message, node, None));
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

    // -- SPERR ---------------------------------------------------------------

    #[test]
    fn sperr_error_without_message_id_fires() {
        let diags = lint_nodes(&*engine(), "error('my message');\n");
        assert!(has_id(&diags, "SPERR"), "got: {diags:?}");
    }

    #[test]
    fn sperr_error_with_message_id_does_not_fire() {
        let diags = lint_nodes(&*engine(), "error('MyComp:myID', 'message');\n");
        assert!(!has_id(&diags, "SPERR"), "got: {diags:?}");
    }

    // -- SPWRN ---------------------------------------------------------------

    #[test]
    fn spwrn_warning_without_message_id_fires() {
        let diags = lint_nodes(&*engine(), "warning('my message');\n");
        assert!(has_id(&diags, "SPWRN"), "got: {diags:?}");
    }

    #[test]
    fn spwrn_warning_with_message_id_does_not_fire() {
        let diags = lint_nodes(&*engine(), "warning('MyComp:myID', 'message');\n");
        assert!(!has_id(&diags, "SPWRN"), "got: {diags:?}");
    }
}
