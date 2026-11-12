//! # LOGL, LOGSUM: logical helpers
//!
//! * LOGL: `logical(1)` / `logical(0)` -> `true` / `false`.
//! * LOGSUM: `sum(<logical expression>)` -> `nnz(<logical expression>)`,
//!   when the summed expression is a comparison or a logical operation.

use super::*;

/// A comparison, a short-circuit boolean operation, or an elementwise `&` / `|`.
fn is_logical_expression(n: tree_sitter::Node, source: &str) -> bool {
    match n.kind() {
        "comparison_operator" | "boolean_operator" => true,
        "binary_operator" => {
            let mut cursor = n.walk();
            let found = n.children(&mut cursor).any(|c| {
                !c.is_named() && matches!(&source[c.start_byte()..c.end_byte()], "&" | "|")
            });
            found
        }
        _ => false,
    }
}

impl ReadabilityEngine {
    /// LOGL: `logical(1)` / `logical(0)` -> `true` / `false`.
    pub(crate) fn check_logical_literal(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("LOGL") {
            return;
        }
        let args = call_args(node);
        if args.len() != 1 {
            return;
        }
        // Probed: MATLAB also reports `logical(zeros(...))` and `logical(ones(...))`,
        // which have no `true`/`false` rewrite.
        let replacement = match &source[args[0].start_byte()..args[0].end_byte()] {
            "1" => Some("true"),
            "0" => Some("false"),
            _ if is_zeros_or_ones_call(args[0], source) => None,
            _ => return,
        };
        results.push(self.diag(
            "LOGL",
            "Use 'true' or 'false' instead of 'logical(1)' or 'logical(0)'.",
            node,
            replacement.map(|r| Fix::new(node.start_byte()..node.end_byte(), r)),
        ));
    }

    /// LOGSUM: `sum(x > 0)` -> `nnz(x > 0)`.
    pub(crate) fn check_sum_logical(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("LOGSUM") {
            return;
        }
        let args = call_args(node);
        if args.len() != 1 {
            return;
        }
        if is_logical_expression(args[0], source) {
            results.push(self.diag(
                "LOGSUM",
                "Consider using 'nnz' instead of 'sum' for logical vectors to improve readability.",
                node,
                None,
            ));
        }
    }
}

/// `zeros(...)` or `ones(...)`: a logical array of constant values.
fn is_zeros_or_ones_call(n: tree_sitter::Node, source: &str) -> bool {
    n.kind() == "function_call"
        && n.child_by_field_name("name")
            .is_some_and(|name| matches!(&source[name.byte_range()], "zeros" | "ones"))
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
    fn logl_logical_literals_fire() {
        let src = "a = logical(1);\nb = logical(0);\n";
        let diags = lint_nodes(&*engine(), src);
        assert_eq!(diags.iter().filter(|d| d.rule_id == "LOGL").count(), 2);
        assert_eq!(
            malt_core::apply_fixes(src, &diags),
            "a = true;\nb = false;\n"
        );
    }

    #[test]
    fn logl_other_calls_and_find_do_not_fire() {
        for src in [
            "x = logical(2);\n",
            "x = logical(a);\n",
            "x = m(find(c));\n",
        ] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "LOGL"), "{src}");
        }
    }

    /// LOGSUM is off under the default preset (MATLAB reports it for
    /// `sum(x(:) <= 1)` only), so its tests run under `all`.
    fn engine_all() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::from_toml("[lint]\npreset = \"all\"\n").unwrap())
    }

    #[test]
    fn logsum_is_off_by_default() {
        assert!(!has_id(
            &lint_nodes(&*engine(), "n = sum(x > 3);\n"),
            "LOGSUM"
        ));
    }

    #[test]
    fn logsum_sum_of_comparison_fires() {
        for src in [
            "n = sum(x > 3);\n",
            "n = sum(a & b);\n",
            "if sum(x == 1) > 0, end\n",
        ] {
            assert!(has_id(&lint_nodes(&*engine_all(), src), "LOGSUM"), "{src}");
        }
    }

    #[test]
    fn logsum_plain_sum_does_not_fire() {
        for src in ["n = sum(x);\n", "n = sum(x) > 0;\n", "n = sum(x, 2);\n"] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "LOGSUM"), "{src}");
        }
    }
}

#[cfg(test)]
mod zeros_tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::from_toml("[lint]\npreset = \"all\"\n").unwrap())
    }

    #[test]
    fn logl_fires_on_logical_zeros_and_ones_without_fix() {
        // Probed: `logical(zeros(n, 1))`, `logical(ones(2))` and `logical(zeros(1,3))` are LOGL.
        for src in ["v = logical(zeros(n, 1));\n", "v = logical(ones(2));\n"] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, "LOGL"), "{src}: got: {diags:?}");
            assert!(
                diags.iter().all(|d| d.fix.is_none()),
                "{src}: got: {diags:?}"
            );
        }
    }

    #[test]
    fn logl_silent_on_logical_of_a_variable() {
        // Not probed as LOGL: a variable's value is not known to be constant.
        let diags = lint_nodes(&*engine(), "v = logical(mask);\n");
        assert!(!has_id(&diags, "LOGL"), "got: {diags:?}");
    }
}
