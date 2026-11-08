//! # DSPSP: `disp(sprintf(...))` → `fprintf`
//!
//! `check_disp` rewrites `disp(sprintf(...))` into a direct `fprintf(...)`
//! call, skipping the intermediate string construction.

use super::*;

impl ReadabilityEngine {
    /// DSPSP: `disp(sprintf(...))` → `fprintf`
    pub(crate) fn check_disp(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("DSPSP") {
            return;
        }

        let args = match find_arguments(node) {
            Some(a) => a,
            None => return,
        };

        let named_children: Vec<_> = (0..args.named_child_count())
            .filter_map(|i| args.named_child(i))
            .collect();

        if named_children.len() != 1 {
            return;
        }

        let inner = named_children[0];
        if inner.kind() == "function_call" {
            if let Some(inner_name) = inner.child_by_field_name("name") {
                let inner_func = &source[inner_name.start_byte()..inner_name.end_byte()];
                if inner_func == "sprintf" {
                    if let Some(inner_args) = find_arguments(inner) {
                        results.push(self.diag(
                            "DSPSP",
                            "'disp(sprintf(...))' can usually be replaced by 'fprintf(...\\n)'.",
                            node,
                            fprintf_fix(node, inner_args, source),
                        ));
                    }
                }
            }
        }
    }
}

/// The rewrite of `disp(sprintf(fmt, ...))` to `fprintf(fmt\n, ...)`.
///
/// `disp` ends its output with a newline and `fprintf` does not, so the newline
/// goes into the format. That only works for a literal format, and only when the
/// call is a statement: as an expression `disp` returns the text it would print.
fn fprintf_fix(
    call: tree_sitter::Node,
    sprintf_args: tree_sitter::Node,
    source: &str,
) -> Option<Fix> {
    let is_statement = call
        .parent()
        .is_some_and(|p| matches!(p.kind(), "block" | "source_file"));
    let format = sprintf_args
        .named_child(0)
        .filter(|f| f.kind() == "string")?;
    let literal = &source[format.start_byte()..format.end_byte()];
    let quote = literal.chars().next().filter(|q| matches!(q, '\'' | '"'))?;
    if !is_statement || literal.len() < 2 || !literal.ends_with(quote) {
        return None;
    }
    let args_text = &source[sprintf_args.start_byte()..sprintf_args.end_byte()];
    let offset = format.end_byte() - sprintf_args.start_byte();
    // Insert `\n` in front of the closing quote of the format.
    let with_newline = format!(
        "{}\\n{}",
        &args_text[..offset - 1],
        &args_text[offset - 1..]
    );
    Some(Fix::new(
        call.start_byte()..call.end_byte(),
        format!("fprintf({with_newline})"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    // -- DSPSP ---------------------------------------------------------------

    #[test]
    fn dspsp_disp_sprintf_fires() {
        let diags = lint_nodes(&*engine(), "disp(sprintf('%d', x));\n");
        assert!(has_id(&diags, "DSPSP"), "got: {diags:?}");
    }

    #[test]
    fn dspsp_disp_direct_does_not_fire() {
        let diags = lint_nodes(&*engine(), "disp(x);\n");
        assert!(!has_id(&diags, "DSPSP"), "got: {diags:?}");
    }

    /// The source after applying the DSPSP fix, or `None` when it has none.
    fn dspsp_fixed(src: &str) -> Option<String> {
        let diags = lint_nodes(&*engine(), src);
        diags.iter().find(|d| d.rule_id == "DSPSP")?.fix.as_ref()?;
        Some(malt_core::apply_fixes(src, &diags))
    }

    #[test]
    fn dspsp_fix_is_valid_matlab_with_the_newline_of_disp() {
        assert_eq!(
            dspsp_fixed("disp(sprintf('hi %d', 3));\n").as_deref(),
            Some("fprintf('hi %d\\n', 3);\n")
        );
        assert_eq!(
            dspsp_fixed("disp(sprintf(\"hi\"));\n").as_deref(),
            Some("fprintf(\"hi\\n\");\n")
        );
    }

    #[test]
    fn dspsp_has_no_fix_when_it_cannot_be_exact() {
        // Non-literal format, or `disp` used for its return value.
        assert_eq!(dspsp_fixed("disp(sprintf(fmt, 3));\n"), None);
        assert_eq!(dspsp_fixed("w = disp(sprintf('hi'));\n"), None);
        // Still reported either way.
        assert!(has_id(
            &lint_nodes(&*engine(), "disp(sprintf(fmt, 3));\n"),
            "DSPSP"
        ));
    }
}
