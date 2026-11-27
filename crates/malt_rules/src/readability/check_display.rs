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
            && call_path(args[0], source, Dotted::Skip) == Some("sprintf")
        {
            results.push(self.diag(
                "DSPSY",
                "'display(sprintf(...))' can usually be replaced by 'fprintf(...\\n)'.",
                node,
                fprintf_fix(node, args[0], source),
            ));
        }
    }
}

/// `display(sprintf('f', a))` becomes `fprintf('f\\n', a)`, for a literal format.
fn fprintf_fix(node: tree_sitter::Node, sprintf: tree_sitter::Node, source: &str) -> Option<Fix> {
    let args = call_args(sprintf);
    let (format, last) = (args.first()?, args.last()?);
    let text = &source[format.byte_range()];
    let quote = text.chars().next().filter(|q| matches!(q, '\'' | '"'))?;
    if format.kind() != "string" || !text.ends_with(quote) || text.len() < 2 {
        return None;
    }
    let body = &text[..text.len() - 1];
    let rest = &source[format.end_byte()..last.end_byte()];
    Some(Fix::new(
        node.byte_range(),
        format!("fprintf({body}\\n{quote}{rest})"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{fix_of, has_id, lint_all, lint_nodes, reported_without_fix};
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
    fn dspsy_fix_rewrites_to_fprintf_and_is_unsafe() {
        let (safe, fixed) = fix_of(&*engine(), "display(sprintf('%d', x));\n", "DSPSY").unwrap();
        assert!(!safe);
        assert_eq!(fixed, "fprintf('%d\\n', x);\n");
        let (_, fixed) = fix_of(&*engine(), "display(sprintf(\"a %s\", s, t))\n", "DSPSY").unwrap();
        assert_eq!(fixed, "fprintf(\"a %s\\n\", s, t)\n");
        let still = lint_all(&*engine(), &fixed);
        assert!(still.iter().all(|d| d.rule_id != "DSPSY"), "{still:?}");
    }

    #[test]
    fn dspsy_no_fix_for_a_computed_format() {
        assert!(reported_without_fix(
            &*engine(),
            "display(sprintf(fmt, x));\n",
            "DSPSY"
        ));
    }

    #[test]
    fn dspsy_plain_display_and_disp_do_not_fire() {
        for src in ["display(x);\n", "disp(x);\n", "display(sprintf('a'), 1);\n"] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "DSPSY"), "{src}");
        }
    }
}
