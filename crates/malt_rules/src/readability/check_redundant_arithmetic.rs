//! # RPMT1, RPMT0, RPMTI, RPMTN: redundant arithmetic with constants
//!
//! `check_redundant_arithmetic` simplifies trivial arithmetic such as
//! `x * 1`, `x * 0`, `x + 0`, and `x - 0`.

use super::*;

impl ReadabilityEngine {
    /// RPMT1, RPMT0, RPMTI, RPMTN: Redundant arithmetic with constants
    pub(crate) fn check_redundant_arithmetic(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if node.child_count() < 3 {
            return;
        }

        let op_text = extract_operator_text(node, source);

        let (lhs, rhs) = match (node.named_child(0), node.named_child(1)) {
            (Some(l), Some(r)) => (l, r),
            _ => return,
        };

        let lhs_text = &source[lhs.start_byte()..lhs.end_byte()];
        let rhs_text = &source[rhs.start_byte()..rhs.end_byte()];

        match op_text {
            "*" | ".*" => {
                // RPMT1: x * 1 → x
                if self.is_check_enabled("RPMT1") && rhs_text == "1" {
                    results.push(self.diag(
                        "RPMT1",
                        "For readability, consider using 'ones(x,y)' instead of 'repmat(1,x,y)'.",
                        node,
                        Some(Fix::new(
                            node.start_byte()..node.end_byte(),
                            lhs_text.to_string(),
                        )),
                    ));
                    return;
                }
                if self.is_check_enabled("RPMT1") && lhs_text == "1" {
                    results.push(self.diag(
                        "RPMT1",
                        "For readability, consider using 'ones(x,y)' instead of 'repmat(1,x,y)'.",
                        node,
                        Some(Fix::new(
                            node.start_byte()..node.end_byte(),
                            rhs_text.to_string(),
                        )),
                    ));
                    return;
                }

                // RPMT0: x * 0 → zeros(size(x))
                if self.is_check_enabled("RPMT0") && rhs_text == "0" {
                    results.push(self.diag(
                        "RPMT0",
                        "For readability, consider using 'zeros(x,y)' instead of 'repmat(0,x,y)'.",
                        node,
                        Some(Fix::new(
                            node.start_byte()..node.end_byte(),
                            format!("zeros(size({lhs_text}))"),
                        )),
                    ));
                } else if self.is_check_enabled("RPMT0") && lhs_text == "0" {
                    results.push(self.diag(
                        "RPMT0",
                        "For readability, consider using 'zeros(x,y)' instead of 'repmat(0,x,y)'.",
                        node,
                        Some(Fix::new(
                            node.start_byte()..node.end_byte(),
                            format!("zeros(size({rhs_text}))"),
                        )),
                    ));
                }
            }
            "+" => {
                // RPMTI: x + 0 → x
                if self.is_check_enabled("RPMTI") && rhs_text == "0" {
                    results.push(self.diag(
                        "RPMTI",
                        "For readability, consider using 'Inf(x,y)' instead of 'repmat(Inf,x,y)'.",
                        node,
                        Some(Fix::new(
                            node.start_byte()..node.end_byte(),
                            lhs_text.to_string(),
                        )),
                    ));
                } else if self.is_check_enabled("RPMTI") && lhs_text == "0" {
                    results.push(self.diag(
                        "RPMTI",
                        "For readability, consider using 'Inf(x,y)' instead of 'repmat(Inf,x,y)'.",
                        node,
                        Some(Fix::new(
                            node.start_byte()..node.end_byte(),
                            rhs_text.to_string(),
                        )),
                    ));
                }
            }
            "-" if self.is_check_enabled("RPMTN") && rhs_text == "0" => {
                // RPMTN: x - 0 → x
                results.push(self.diag(
                    "RPMTN",
                    "For readability, consider using 'NaN(x,y)' instead of 'repmat(NaN,x,y)'.",
                    node,
                    Some(Fix::new(
                        node.start_byte()..node.end_byte(),
                        lhs_text.to_string(),
                    )),
                ));
            }
            _ => {}
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

    // -- RPMT1 ---------------------------------------------------------------

    #[test]
    fn rpmt1_mul_by_one_fires() {
        let diags = lint_nodes(&*engine(), "x = a * 1;\n");
        assert!(has_id(&diags, "RPMT1"), "got: {diags:?}");
    }

    #[test]
    fn rpmt1_one_times_x_fires() {
        let diags = lint_nodes(&*engine(), "x = 1 * a;\n");
        assert!(has_id(&diags, "RPMT1"), "got: {diags:?}");
    }

    #[test]
    fn rpmt1_mul_by_two_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = a * 2;\n");
        assert!(!has_id(&diags, "RPMT1"), "got: {diags:?}");
    }

    // -- RPMT0 ---------------------------------------------------------------

    #[test]
    fn rpmt0_mul_by_zero_fires() {
        let diags = lint_nodes(&*engine(), "x = a * 0;\n");
        assert!(has_id(&diags, "RPMT0"), "got: {diags:?}");
    }

    #[test]
    fn rpmt0_mul_by_two_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = a * 2;\n");
        assert!(!has_id(&diags, "RPMT0"), "got: {diags:?}");
    }

    // -- RPMTI ---------------------------------------------------------------

    #[test]
    fn rpmti_add_zero_fires() {
        let diags = lint_nodes(&*engine(), "x = a + 0;\n");
        assert!(has_id(&diags, "RPMTI"), "got: {diags:?}");
    }

    #[test]
    fn rpmti_add_two_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = a + 2;\n");
        assert!(!has_id(&diags, "RPMTI"), "got: {diags:?}");
    }

    // -- RPMTN ---------------------------------------------------------------

    #[test]
    fn rpmtn_sub_zero_fires() {
        let diags = lint_nodes(&*engine(), "x = a - 0;\n");
        assert!(has_id(&diags, "RPMTN"), "got: {diags:?}");
    }

    #[test]
    fn rpmtn_sub_two_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = a - 2;\n");
        assert!(!has_id(&diags, "RPMTN"), "got: {diags:?}");
    }
}
