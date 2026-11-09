//! # DSPSY: `display(sprintf(...))`
//!
//! `check_display` flags `display(sprintf(...))`, which should print through
//! `fprintf` instead.

use super::*;

impl ReadabilityEngine {
    /// DSPSY: `display(sprintf(...))`
    pub(crate) fn check_display(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("DSPSY") {
            return;
        }
        let args = call_args(node);
        if args.len() == 1
            && args[0].kind() == "function_call"
            && call_name(args[0], source) == Some("sprintf")
        {
            results.push(self.diag(
                "DSPSY",
                "'display(sprintf(...))' can usually be replaced by 'fprintf(...\\n)'.",
                node,
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

    #[test]
    fn dspsy_display_sprintf_fires() {
        let diags = lint_nodes(&*engine(), "display(sprintf('%d', x));\n");
        assert!(has_id(&diags, "DSPSY"), "got: {diags:?}");
    }

    #[test]
    fn dspsy_plain_display_and_disp_do_not_fire() {
        for src in ["display(x);\n", "disp(x);\n", "display(sprintf('a'), 1);\n"] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "DSPSY"), "{src}");
        }
    }
}
