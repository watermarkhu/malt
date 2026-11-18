//! ALIGN check: a block keyword and its matching END are indented differently.

use super::*;

// ---------------------------------------------------------------------------
// ALIGN: block keyword and END alignment
// ---------------------------------------------------------------------------

impl FormattingEngine {
    /// Check that the `end` of an `if`, `for`, `while`, `switch` or `try` block is
    /// in the same column as its keyword.
    ///
    /// Probed: MATLAB reports the keyword, at its own line and column, when the
    /// matching END starts its line in a different column. A difference of one
    /// column is silent, and a tab counts as one column (`\tend` under a
    /// keyword in column 1 is silent). It does not check `else` or `elseif`
    /// clauses, `spmd` or `function` ends (formatting__b:18, :20, al14, fe1 are
    /// silent in MATLAB). An `end` that shares its line with other code is not
    /// compared.
    pub(crate) fn check_align(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let end_start = node.end_byte().saturating_sub(3);
        if source.get(end_start..node.end_byte()) != Some("end") {
            return;
        }
        let end_line = line_start_byte(source, end_start);
        let end_indent = &source[end_line..end_start];
        if !end_indent.trim().is_empty() {
            return;
        }
        let keyword_line = line_start_byte(source, node.start_byte());
        let keyword_prefix = &source[keyword_line..node.start_byte()];
        // Columns count characters, so a tab is one column.
        if keyword_prefix
            .chars()
            .count()
            .abs_diff(end_indent.chars().count())
            < 2
        {
            return;
        }
        // Only a keyword that starts its line can be re-indented to match.
        let fix = keyword_prefix
            .trim()
            .is_empty()
            .then(|| Fix::new(end_line..end_start, keyword_prefix.to_string()));

        let pos = node.start_position();
        let end_row = source[..end_start].matches('\n').count() + 1;
        diagnostics.push(Diagnostic {
            rule_id: "ALIGN",
            message: format!(
                "This keyword might not be aligned with its matching END on line {end_row}."
            ),
            severity: Severity::Info,
            byte_range: keyword_line..node.start_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix,
        });
    }
}

#[cfg(test)]
mod tests {

    use crate::formatting::tests::{has_id, lint};
    use malt_core::apply_fixes;

    // -- ALIGN ---------------------------------------------------------------

    #[test]
    fn align_misaligned_end_of_if() {
        let diags = lint("if x\n    a = 1;\n  end\n");
        assert!(has_id(&diags, "ALIGN"), "got: {diags:?}");
    }

    #[test]
    fn align_misaligned_end_of_for_and_while() {
        // Probed: MATLAB reports both (formatting__i:7 and a `while true` probe).
        for source in [
            "for k = 1:3\n    x = k;\n      end\n",
            "while true\n  break;\n  end\n",
        ] {
            let diags = lint(source);
            assert!(has_id(&diags, "ALIGN"), "{source:?} got: {diags:?}");
        }
    }

    #[test]
    fn align_aligned_blocks() {
        let source = "for k = 1:3\n  x = k;\nend\nif x\n    a = 1;\nelse\n    a = 2;\nend\n";
        assert!(!has_id(&lint(source), "ALIGN"));
    }

    #[test]
    fn align_ignores_misaligned_else_clause() {
        // Probed: MATLAB is silent on an `else` indented differently (formatting__b:18, :20).
        let source = "if x\n    a = 1;\n    else\n    a = 2;\nend\n";
        assert!(!has_id(&lint(source), "ALIGN"));
    }

    #[test]
    fn align_leaves_one_liners_alone() {
        for source in [
            "if a, b = 1; else, b = 2; end\n",
            "if a, b = 1; elseif c, b = 3; else, b = 2; end\n",
            "  if x, e=1; elseif x>2, e=2; else, e=3; end\n",
        ] {
            let diags = lint(source);
            assert!(!has_id(&diags, "ALIGN"), "{source:?} got: {diags:?}");
        }
    }

    #[test]
    fn align_fix_reindents_the_end() {
        let source = "function f()\n\tif x\n\t\ta = 1;\n    end\nend\n";
        let fixed = apply_fixes(source, &lint(source));
        assert_eq!(fixed, "function f()\n\tif x\n\t\ta = 1;\n\tend\nend\n");
    }

    #[test]
    fn align_reports_the_line_of_end() {
        let source = "function f()\n    if x\n      a = 1;\n  end\nend\n";
        let diags = lint(source);
        let d = diags.iter().find(|d| d.rule_id == "ALIGN").expect("ALIGN");
        assert!(d.message.ends_with("on line 4."), "{}", d.message);
    }

    // Probed (p25 grid, `if` in column 1 and column 3): a difference of one column
    // is silent, two or more is reported, in both directions.
    #[test]
    fn align_one_column_difference_is_silent() {
        for source in [
            "if x\n  y = 1;\n end\n",
            "  if x\n    y = 1;\n   end\n",
            "  if x\n    y = 1;\n  end\n",
        ] {
            let diags = lint(source);
            assert!(!has_id(&diags, "ALIGN"), "{source:?} got: {diags:?}");
        }
        for source in [
            "if x\n  y = 1;\n  end\n",
            "  if x\n    y = 1;\nend\n",
            "  if x\n    y = 1;\n     end\n",
        ] {
            let diags = lint(source);
            assert!(has_id(&diags, "ALIGN"), "{source:?} got: {diags:?}");
        }
    }

    #[test]
    fn align_tab_counts_as_one_column() {
        // Probed: `\tend` under `if` in column 1 is one column off (silent); `\t  end` is reported.
        assert!(!has_id(&lint("if x\n  y = 1;\n\tend\n"), "ALIGN"));
        assert!(has_id(&lint("if x\n\ty = 1;\n\t  end\n"), "ALIGN"));
    }

    #[test]
    fn align_checks_switch_and_try() {
        // Probed: `switch` and `try` ends are compared like `if` (sw3, a07, al12).
        let diags = lint("function f()\n  switch 1\n    case 1\n    end\nend\n");
        assert!(has_id(&diags, "ALIGN"), "got: {diags:?}");
        let diags = lint("function f()\ntry\n  y = 1;\n  end\nend\n");
        assert!(has_id(&diags, "ALIGN"), "got: {diags:?}");
        assert!(!has_id(&lint("try\n  y = 1;\nend\n"), "ALIGN"));
    }

    #[test]
    fn align_silent_on_spmd_and_function_ends() {
        // Probed: `spmd` (al14) and `function` ends (fe1) are not compared.
        assert!(!has_id(
            &lint("function f()\n  spmd\n    w = 1;\n    end\nend\n"),
            "ALIGN"
        ));
        assert!(!has_id(&lint("function f()\n  x = 1;\n  end\n"), "ALIGN"));
    }

    #[test]
    fn align_keyword_after_code_is_reported_without_fix() {
        // Probed: `x = 1; if x` with `end` in column 1 is reported at the `if` (column 8).
        let source = "x = 1; if x\n  y = 2;\nend\n";
        let diags = lint(source);
        let d = diags.iter().find(|d| d.rule_id == "ALIGN").expect("ALIGN");
        assert_eq!((d.line, d.column), (1, 8), "got: {diags:?}");
        assert!(d.fix.is_none(), "got: {d:?}");
    }

    #[test]
    fn align_silent_when_end_shares_its_line() {
        // `end` after code on its line is not compared (`if x, y = 2; end` one-liner).
        assert!(!has_id(&lint("if x\n  y = 2; end\n"), "ALIGN"));
    }
}
