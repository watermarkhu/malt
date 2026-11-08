//! # STLOW: unnecessary UPPER/LOWER call in a string comparison
//!
//! `check_stlow` flags `strcmp(upper(x), 'ABC')` / `strcmp(lower(x), 'abc')`
//! style comparisons where the compared literal is already entirely in the
//! target case, making the conversion call unnecessary.

use super::*;

impl ReadabilityEngine {
    /// STLOW: `strcmp(upper(x), 'ABC')` — the UPPER/LOWER call is unnecessary
    /// when the compared literal is already entirely in that case.
    pub(crate) fn check_stlow(
        &self,
        named_children: &[tree_sitter::Node],
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("STLOW") {
            return;
        }

        for i in 0..2 {
            let arg = named_children[i];
            if arg.kind() != "function_call" {
                continue;
            }

            let func_name = match arg.child_by_field_name("name") {
                Some(n) => &source[n.start_byte()..n.end_byte()],
                None => continue,
            };

            let uppercase = match func_name {
                "upper" => true,
                "lower" => false,
                _ => continue,
            };

            let inner_args = match find_arguments(arg) {
                Some(a) => a,
                None => continue,
            };
            if inner_args.named_child_count() != 1 {
                continue;
            }

            let other = named_children[1 - i];
            if other.kind() != "string" {
                continue;
            }
            let other_text = &source[other.start_byte()..other.end_byte()];
            let literal = other_text.trim_matches('\'').trim_matches('"');

            if is_all_case(literal, uppercase) {
                results.push(self.diag(
                    "STLOW",
                    "In this comparison the call to UPPER/LOWER is unnecessary.",
                    arg,
                    // Dropping UPPER/LOWER from a case-sensitive strcmp changes its
                    // result, so there is no rewrite to offer.
                    None,
                ));
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

    // -- STLOW ---------------------------------------------------------------

    #[test]
    fn stlow_upper_with_uppercase_literal_fires() {
        let diags = lint_nodes(&*engine(), "x = strcmp(upper(str), 'ABC');\n");
        assert!(has_id(&diags, "STLOW"), "got: {diags:?}");
    }

    #[test]
    fn stlow_lower_with_lowercase_literal_fires() {
        let diags = lint_nodes(&*engine(), "x = strcmp(lower(str), 'abc');\n");
        assert!(has_id(&diags, "STLOW"), "got: {diags:?}");
    }

    #[test]
    fn stlow_mixed_case_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = strcmp(upper(str), 'AbC');\n");
        assert!(!has_id(&diags, "STLOW"), "got: {diags:?}");
    }

    #[test]
    fn stlow_opposite_case_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = strcmp(upper(str), 'abc');\n");
        assert!(!has_id(&diags, "STLOW"), "got: {diags:?}");
    }

    #[test]
    fn stlow_no_conversion_call_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = strcmp(str, 'ABC');\n");
        assert!(!has_id(&diags, "STLOW"), "got: {diags:?}");
    }

    #[test]
    fn stlow_non_literal_other_side_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = strcmp(upper(x), y);\n");
        assert!(!has_id(&diags, "STLOW"), "got: {diags:?}");
    }

    #[test]
    fn stlow_does_not_rewrite_a_case_sensitive_comparison() {
        let source = "x = strcmp(upper(x), 'ABC');\n";
        let diags = lint_nodes(&*engine(), source);
        let diag = diags
            .iter()
            .find(|d| d.rule_id == "STLOW")
            .expect("STLOW should fire");
        // Dropping upper() from a case-sensitive strcmp changes its result
        // (x = 'abc' matches before, not after): the check reports, never rewrites.
        assert!(diag.fix.is_none(), "STLOW must not carry an auto-fix");
    }

    #[test]
    fn stlow_disabled_in_config_does_not_fire() {
        let config =
            Config::from_toml("[lint.rules.READABILITY_ENGINE]\ndisabled_checks = [\"STLOW\"]\n")
                .expect("valid config");
        let rule = ReadabilityEngine::from_config(&config);
        let diags = lint_nodes(&*rule, "x = strcmp(upper(str), 'ABC');\n");
        assert!(!has_id(&diags, "STLOW"), "got: {diags:?}");
    }
}
