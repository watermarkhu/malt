//! # ISMAT: `ndims(x) == 2` -> `ismatrix(x)`
//!
//! `check_ismat` flags `ndims(x) == 2`, `length(size(x)) == 2` and
//! `numel(size(x)) == 2`, which test for a 2-D array.

use super::*;

impl ReadabilityEngine {
    /// ISMAT: comparisons of the number of dimensions with 2.
    pub(crate) fn check_ismat(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("ISMAT") || !is_equality_operator(node, source) {
            return;
        }
        let (Some(a), Some(b)) = (node.named_child(0), node.named_child(1)) else {
            return;
        };
        for (call, lit) in [(a, b), (b, a)] {
            if &source[lit.start_byte()..lit.end_byte()] != "2" || call.kind() != "function_call" {
                continue;
            }
            let args = call_args(call);
            if args.len() != 1 {
                continue;
            }
            let operand = match call_name(call, source) {
                Some("ndims") => Some(&source[args[0].start_byte()..args[0].end_byte()]),
                Some("length" | "numel") => size_operand(args[0], source),
                _ => None,
            };
            if let Some(x) = operand {
                results.push(self.diag(
                    "ISMAT",
                    "When checking if a variable is a matrix consider using ISMATRIX.",
                    node,
                    Some(Fix::new(
                        node.start_byte()..node.end_byte(),
                        format!("ismatrix({x})"),
                    )),
                ));
                return;
            }
        }
    }
}

/// The operand of a one-argument `size(x)` call.
fn size_operand<'a>(n: tree_sitter::Node, source: &'a str) -> Option<&'a str> {
    if n.kind() != "function_call" || call_name(n, source) != Some("size") {
        return None;
    }
    let args = call_args(n);
    (args.len() == 1).then(|| &source[args[0].start_byte()..args[0].end_byte()])
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
    fn two_dimension_tests_fire() {
        for src in [
            "x = ndims(a) == 2;\n",
            "x = length(size(a)) == 2;\n",
            "x = numel(size(a)) == 2;\n",
            "x = 2 == ndims(a);\n",
        ] {
            assert!(has_id(&lint_nodes(&*engine(), src), "ISMAT"), "{src}");
        }
    }

    #[test]
    fn other_comparisons_and_isa_double_do_not_fire() {
        for src in [
            "x = ndims(a) == 3;\n",
            "x = ndims(a) > 2;\n",
            "x = length(a) == 2;\n",
            "x = isa(a, 'double');\n",
        ] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "ISMAT"), "{src}");
        }
    }

    #[test]
    fn fix_uses_ismatrix() {
        let src = "x = ndims(a) == 2;\n";
        let diags = lint_nodes(&*engine(), src);
        assert_eq!(malt_core::apply_fixes(src, &diags), "x = ismatrix(a);\n");
    }
}
