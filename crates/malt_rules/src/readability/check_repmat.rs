//! # RPMT0, RPMT1, RPMTT, RPMTF, RPMTI, RPMTN: `repmat` of a constant
//!
//! `check_repmat` flags `repmat(c, ...)` where `c` is the scalar constant `0`,
//! `1`, `true`, `false`, `Inf` or `NaN`, and suggests the dedicated
//! constructor (`zeros`, `ones`, `true`, `false`, `Inf`, `NaN`) with the same
//! size arguments.

use super::*;

/// Constant first argument of `repmat` -> (check id, constructor).
const REPMAT_CONSTANTS: &[(&str, &str, &str)] = &[
    ("1", "RPMT1", "ones"),
    ("0", "RPMT0", "zeros"),
    ("true", "RPMTT", "true"),
    ("false", "RPMTF", "false"),
    ("Inf", "RPMTI", "Inf"),
    ("NaN", "RPMTN", "NaN"),
];

impl ReadabilityEngine {
    /// RPMT*: `repmat(c, x, y)` -> `ones(x, y)` / `zeros(x, y)` / ...
    pub(crate) fn check_repmat(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        let args = call_args(node);
        if args.len() < 2 {
            return;
        }
        let first = &source[args[0].start_byte()..args[0].end_byte()];
        let Some(&(_, id, ctor)) = REPMAT_CONSTANTS.iter().find(|(c, _, _)| *c == first) else {
            return;
        };
        if !self.is_check_enabled(id) {
            return;
        }
        let rest = &source[args[1].start_byte()..args[args.len() - 1].end_byte()];
        let message = format!(
            "For readability, consider using '{ctor}(x,y)' instead of 'repmat({first},x,y)'."
        );
        results.push(self.diag(
            id,
            &message,
            node,
            Some(Fix::new(
                node.start_byte()..node.end_byte(),
                format!("{ctor}({rest})"),
            )),
        ));
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
    fn each_constant_maps_to_its_id() {
        for (c, id) in [
            ("1", "RPMT1"),
            ("0", "RPMT0"),
            ("true", "RPMTT"),
            ("false", "RPMTF"),
            ("Inf", "RPMTI"),
            ("NaN", "RPMTN"),
        ] {
            let diags = lint_nodes(&*engine(), &format!("x = repmat({c}, 3, 4);\n"));
            assert!(has_id(&diags, id), "{c}: got {diags:?}");
        }
    }

    #[test]
    fn other_first_argument_or_arithmetic_does_not_fire() {
        for src in [
            "x = repmat(2, 3, 4);\n",
            "x = repmat(a, 3);\n",
            "y = a * 1;\n",
            "y = a | true;\n",
            "y = a + 0;\n",
            "y = repmat(1);\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(
                !diags.iter().any(|d| d.rule_id.starts_with("RPMT")),
                "{src}: got {diags:?}"
            );
        }
    }

    #[test]
    fn message_fills_in_the_constant_and_fix_keeps_size_args() {
        let src = "x = repmat(0, n, [2 3]);\n";
        let diags = lint_nodes(&*engine(), src);
        let d = diags.iter().find(|d| d.rule_id == "RPMT0").expect("RPMT0");
        assert_eq!(
            d.message,
            "For readability, consider using 'zeros(x,y)' instead of 'repmat(0,x,y)'."
        );
        assert_eq!(
            malt_core::apply_fixes(src, &diags),
            "x = zeros(n, [2 3]);\n"
        );
    }
}
