//! # FVINR: add an `(Input)` attribute to `arguments` blocks
//!
//! `check_fvinr` flags `arguments` blocks that declare no `(Input)` or
//! `(Output)` attribute and inserts `(Input)` for readability.

use super::*;

impl ReadabilityEngine {
    /// FVINR: Add `(Input)` attribute to `arguments` block for readability.
    pub(crate) fn check_fvinr(&self, ctx: &NodeContext, results: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("FVINR") {
            return;
        }
        let node = ctx.node;

        // If any `attributes` child exists, the block already has (Input) or (Output).
        let has_attributes = node
            .named_children(&mut node.walk())
            .any(|c| c.kind() == "attributes");
        if has_attributes {
            return;
        }

        // The `arguments` keyword is the first (unnamed) child.
        let arguments_keyword = match node.child(0) {
            Some(c) if c.kind() == "arguments" => c,
            _ => return,
        };

        // CRITICAL: fix text is " (Input)" — LEADING space, no trailing space.
        // The keyword is immediately followed by a newline, so inserting a
        // trailing space would produce `arguments(Input)` (invalid).
        let fix = Fix::insert(arguments_keyword.end_byte(), " (Input)");
        results.push(self.diag(
            "FVINR",
            "For readability, add Input attribute to the input arguments block.",
            arguments_keyword,
            Some(fix),
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

    // -- FVINR ---------------------------------------------------------------

    #[test]
    fn fvinr_no_attribute_fires() {
        let source = "function f(a)\n    arguments\n        a (1,1)\n    end\nend\n";
        let diags = lint_nodes(&*engine(), source);
        assert!(has_id(&diags, "FVINR"), "got: {diags:?}");
    }

    #[test]
    fn fvinr_with_input_attribute_does_not_fire() {
        let source = "function f(a)\n    arguments (Input)\n        a (1,1)\n    end\nend\n";
        let diags = lint_nodes(&*engine(), source);
        assert!(!has_id(&diags, "FVINR"), "got: {diags:?}");
    }

    #[test]
    fn fvinr_with_output_attribute_does_not_fire() {
        let source = "function f(a)\n    arguments (Output)\n        a\n    end\nend\n";
        let diags = lint_nodes(&*engine(), source);
        assert!(!has_id(&diags, "FVINR"), "got: {diags:?}");
    }

    #[test]
    fn fvinr_disabled_in_config_does_not_fire() {
        let config =
            Config::from_toml("[lint.rules.READABILITY_ENGINE]\ndisabled_checks = [\"FVINR\"]\n")
                .expect("valid config");
        let rule = ReadabilityEngine::from_config(&config);
        let source = "function f(a)\n    arguments\n        a (1,1)\n    end\nend\n";
        let diags = lint_nodes(&*rule, source);
        assert!(!has_id(&diags, "FVINR"), "got: {diags:?}");
    }

    #[test]
    fn fvinr_fix_inserts_input_attribute() {
        let source = "function f(a)\n    arguments\n        a (1,1)\n    end\nend\n";
        let diags = lint_nodes(&*engine(), source);
        let diag = diags
            .iter()
            .find(|d| d.rule_id == "FVINR")
            .expect("FVINR should fire");
        let fix = diag.fix.as_ref().expect("FVINR should carry an auto-fix");
        assert_eq!(fix.replacement, " (Input)");
        let mut fixed = String::from(source);
        fixed.replace_range(fix.byte_range.clone(), &fix.replacement);
        assert!(fixed.contains("arguments (Input)"), "got: {fixed:?}");
    }
}
