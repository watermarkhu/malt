//! # IJCL: assignment to `i`/`j` shadows the complex unit
//!
//! `check_ij_shadow` flags assignments to `i` or `j` that shadow MATLAB's
//! built-in imaginary unit.

use super::*;

impl ReadabilityEngine {
    /// IJCL: Assignment to `i` or `j` shadows the complex unit.
    pub(crate) fn check_ij_shadow(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("IJCL") {
            return;
        }

        // The LHS of an assignment — the first named child or field "left"
        let lhs = match node.child_by_field_name("left") {
            Some(n) => n,
            None => {
                // Fallback: first named child
                match node.named_child(0) {
                    Some(n) => n,
                    None => return,
                }
            }
        };

        if lhs.kind() != "identifier" {
            return;
        }

        let lhs_text = &source[lhs.start_byte()..lhs.end_byte()];
        if lhs_text == "i" || lhs_text == "j" {
            results.push(self.diag(
                "IJCL",
                "For improved robustness, consider replacing i and j by 1i.",
                lhs,
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

    // -- IJCL ----------------------------------------------------------------

    #[test]
    fn ijcl_assign_to_i_fires() {
        let diags = lint_nodes(&*engine(), "i = 5;\n");
        assert!(has_id(&diags, "IJCL"), "got: {diags:?}");
    }

    #[test]
    fn ijcl_assign_to_j_fires() {
        let diags = lint_nodes(&*engine(), "j = zeros(3);\n");
        assert!(has_id(&diags, "IJCL"), "got: {diags:?}");
    }

    #[test]
    fn ijcl_assign_to_other_name_does_not_fire() {
        let diags = lint_nodes(&*engine(), "k = 5;\n");
        assert!(!has_id(&diags, "IJCL"), "got: {diags:?}");
    }
}
