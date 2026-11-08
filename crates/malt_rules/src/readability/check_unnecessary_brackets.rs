//! # NBRAK2: unnecessary brackets around a scalar expression
//!
//! `check_unnecessary_brackets` flags `[x]` where `x` is a scalar
//! identifier, number, or string literal.

use super::*;

impl ReadabilityEngine {
    /// NBRAK2: Unnecessary brackets `[x]` for scalar expression.
    pub(crate) fn check_unnecessary_brackets(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("NBRAK2") {
            return;
        }

        // A matrix node with exactly one named child that is a "row" with
        // exactly one element is `[scalar]`.
        let named_count = node.named_child_count();
        if named_count != 1 {
            return;
        }

        let row = match node.named_child(0) {
            Some(r) => r,
            None => return,
        };

        // The row should have exactly one named child (the scalar expression)
        if row.kind() == "row" && row.named_child_count() == 1 {
            if let Some(inner) = row.named_child(0) {
                // Don't flag complex expressions or function calls that might
                // rely on concatenation behavior
                if matches!(inner.kind(), "identifier" | "number" | "string") {
                    let inner_text = &source[inner.start_byte()..inner.end_byte()];
                    results.push(self.diag(
                        "NBRAK2",
                        "Use of brackets [] is unnecessary.",
                        node,
                        Some(Fix::new(
                            node.start_byte()..node.end_byte(),
                            inner_text.to_string(),
                        )),
                    ));
                }
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

    // -- NBRAK2 --------------------------------------------------------------

    #[test]
    fn nbrak2_brackets_around_scalar_fires() {
        let diags = lint_nodes(&*engine(), "x = [5];\n");
        assert!(has_id(&diags, "NBRAK2"), "got: {diags:?}");
    }

    #[test]
    fn nbrak2_brackets_around_identifier_fires() {
        let diags = lint_nodes(&*engine(), "x = [v];\n");
        assert!(has_id(&diags, "NBRAK2"), "got: {diags:?}");
    }

    #[test]
    fn nbrak2_multi_element_matrix_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = [1 2];\n");
        assert!(!has_id(&diags, "NBRAK2"), "got: {diags:?}");
    }
}
