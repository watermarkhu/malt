//! # FLUDLR: `flipud(fliplr(x))` / `fliplr(flipud(x))` → `rot90(x, 2)`
//!
//! `check_fludlr` detects double flips (vertical + horizontal) and suggests
//! the equivalent 180-degree rotation `rot90(x, 2)`.

use super::*;

impl ReadabilityEngine {
    /// FLUDLR: `flipud(fliplr(x))` / `fliplr(flipud(x))` → `rot90(x, 2)`
    ///
    /// Flipping both vertically and horizontally is a 180-degree rotation,
    /// which reads more clearly as `rot90(x, 2)`.
    pub(crate) fn check_fludlr(
        &self,
        node: tree_sitter::Node,
        source: &str,
        func_name: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FLUDLR") {
            return;
        }

        let args = match find_arguments(node) {
            Some(a) => a,
            None => return,
        };

        let named_children: Vec<_> = (0..args.named_child_count())
            .filter_map(|i| args.named_child(i))
            .collect();

        // Outer call must have exactly one argument: the nested flip call
        if named_children.len() != 1 {
            return;
        }

        let inner = named_children[0];
        if inner.kind() != "function_call" {
            return;
        }

        let inner_name = match inner.child_by_field_name("name") {
            Some(n) => &source[n.start_byte()..n.end_byte()],
            None => return,
        };

        // Match the exact flipud/fliplr pair, each with exactly one argument
        let is_pair = match func_name {
            "flipud" => inner_name == "fliplr",
            "fliplr" => inner_name == "flipud",
            _ => false,
        };
        if !is_pair {
            return;
        }

        let inner_args = match find_arguments(inner) {
            Some(a) => a,
            None => return,
        };

        let inner_named: Vec<_> = (0..inner_args.named_child_count())
            .filter_map(|i| inner_args.named_child(i))
            .collect();

        if inner_named.len() != 1 {
            return;
        }

        let inner_arg_text = &source[inner_named[0].start_byte()..inner_named[0].end_byte()];

        results.push(self.diag(
            "FLUDLR",
            "For readability, consider using rot90(x,2) instead of flipud(fliplr(x)) or fliplr(flipud(x)).",
            node,
            Some(Fix::new(
                node.start_byte()..node.end_byte(),
                format!("rot90({inner_arg_text}, 2)"),
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

    // -- FLUDLR --------------------------------------------------------------

    #[test]
    fn fludlr_flipud_fliplr_fires() {
        let diags = lint_nodes(&*engine(), "y = flipud(fliplr(x));\n");
        assert!(has_id(&diags, "FLUDLR"), "got: {diags:?}");
    }

    #[test]
    fn fludlr_fliplr_flipud_fires() {
        let diags = lint_nodes(&*engine(), "y = fliplr(flipud(x));\n");
        assert!(has_id(&diags, "FLUDLR"), "got: {diags:?}");
    }

    #[test]
    fn fludlr_single_flipud_does_not_fire() {
        let diags = lint_nodes(&*engine(), "y = flipud(x);\n");
        assert!(!has_id(&diags, "FLUDLR"), "got: {diags:?}");
    }

    #[test]
    fn fludlr_single_fliplr_does_not_fire() {
        let diags = lint_nodes(&*engine(), "y = fliplr(x);\n");
        assert!(!has_id(&diags, "FLUDLR"), "got: {diags:?}");
    }

    #[test]
    fn fludlr_two_arg_outer_does_not_fire() {
        let diags = lint_nodes(&*engine(), "y = flipud(fliplr(x), 2);\n");
        assert!(!has_id(&diags, "FLUDLR"), "got: {diags:?}");
    }

    #[test]
    fn fludlr_same_flip_pair_does_not_fire() {
        let diags = lint_nodes(&*engine(), "y = flipud(flipud(x));\n");
        assert!(!has_id(&diags, "FLUDLR"), "got: {diags:?}");
    }

    #[test]
    fn fludlr_fix_replaces_with_rot90() {
        let source = "y = flipud(fliplr(x));\n";
        let diags = lint_nodes(&*engine(), source);
        let diag = diags
            .iter()
            .find(|d| d.rule_id == "FLUDLR")
            .expect("FLUDLR should fire");
        let fix = diag.fix.as_ref().expect("FLUDLR should carry an auto-fix");
        assert_eq!(fix.replacement, "rot90(x, 2)");
        let mut fixed = String::from(source);
        fixed.replace_range(fix.byte_range.clone(), &fix.replacement);
        assert_eq!(fixed, "y = rot90(x, 2);\n", "got: {fixed:?}");
    }

    #[test]
    fn fludlr_disabled_in_config_does_not_fire() {
        let config =
            Config::from_toml("[lint.rules.READABILITY_ENGINE]\ndisabled_checks = [\"FLUDLR\"]\n")
                .expect("valid config");
        let rule = ReadabilityEngine::from_config(&config);
        let diags = lint_nodes(&*rule, "y = flipud(fliplr(x));\n");
        assert!(!has_id(&diags, "FLUDLR"), "got: {diags:?}");
    }
}
