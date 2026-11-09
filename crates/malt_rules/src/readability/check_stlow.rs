//! # STLOW: unnecessary UPPER/LOWER call in a case-insensitive comparison
//!
//! `check_stlow` flags `strcmpi(lower(x), y)` / `strncmpi(upper(x), y, n)`
//! style comparisons: the comparison already ignores case, so the call to
//! `upper`/`lower` on an argument is unnecessary.

use super::*;

impl ReadabilityEngine {
    /// STLOW: `strcmpi`/`strncmpi` with an `upper`/`lower` call as a compared
    /// argument (the first two arguments).
    pub(crate) fn check_stlow(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("STLOW") {
            return;
        }
        for arg in call_args(node).into_iter().take(2) {
            if arg.kind() != "function_call"
                || !matches!(call_name(arg, source), Some("upper" | "lower"))
            {
                continue;
            }
            let inner = call_args(arg);
            if inner.len() != 1 {
                continue;
            }
            let operand = &source[inner[0].start_byte()..inner[0].end_byte()];
            results.push(self.diag(
                "STLOW",
                "In this comparison the call to UPPER/LOWER is unnecessary.",
                arg,
                // Unsafe: upper/lower reject non-text input, strcmpi does not.
                Some(Fix::new(arg.start_byte()..arg.end_byte(), operand)),
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

    #[test]
    fn stlow_lower_in_strcmpi_fires_and_is_unsafe_fix() {
        let src = "x = strcmpi(lower(str), 'abc');\n";
        let diags = lint_nodes(&*engine(), src);
        let d = diags.iter().find(|d| d.rule_id == "STLOW").expect("STLOW");
        assert!(!d.fix.as_ref().expect("fix").is_safe());
        assert_eq!(
            malt_core::apply_fixes(src, &diags),
            src,
            "safe fixes must not apply it"
        );
    }

    #[test]
    fn stlow_upper_in_strncmpi_fires() {
        let diags = lint_nodes(&*engine(), "x = strncmpi('ABC', upper(s), 2);\n");
        assert!(has_id(&diags, "STLOW"), "got: {diags:?}");
    }

    #[test]
    fn stlow_case_sensitive_or_plain_comparison_does_not_fire() {
        for src in [
            "x = strcmp(upper(str), 'ABC');\n",
            "x = strcmpi(str, 'ABC');\n",
            "x = strcmpi(f(str), 'ABC');\n",
            "x = strncmpi(a, b, upper(n));\n",
        ] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "STLOW"), "{src}");
        }
    }

    #[test]
    fn stlow_disabled_in_config_does_not_fire() {
        let config =
            Config::from_toml("[lint.rules.READABILITY_ENGINE]\ndisabled_checks = [\"STLOW\"]\n")
                .expect("valid config");
        let rule = ReadabilityEngine::from_config(&config);
        let diags = lint_nodes(&*rule, "x = strcmpi(lower(str), 'abc');\n");
        assert!(!has_id(&diags, "STLOW"), "got: {diags:?}");
    }
}
