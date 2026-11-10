//! ALIGN check: a block keyword and its matching END are indented differently.

use super::*;

// ---------------------------------------------------------------------------
// ALIGN: block keyword and END alignment
// ---------------------------------------------------------------------------

impl FormattingEngine {
    /// Check that the `end` of an `if`, `for` or `while` block is indented like
    /// its keyword.
    ///
    /// Probed: MATLAB reports the keyword, at its own line and column, when the
    /// matching END is indented differently (formatting__i:7 and the `if`, `for`
    /// and `while` probes). It does not check `else` or `elseif` clauses
    /// (formatting__b:18 and :20 are silent in MATLAB). Only a block whose keyword
    /// and `end` each start their own line is compared.
    pub(crate) fn check_align(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let end_start = node.end_byte().saturating_sub(3);
        if source.get(end_start..node.end_byte()) != Some("end") {
            return;
        }
        let keyword_line = line_start_byte(source, node.start_byte());
        let keyword_indent = &source[keyword_line..node.start_byte()];
        let end_line = line_start_byte(source, end_start);
        let end_indent = &source[end_line..end_start];
        if !keyword_indent.trim().is_empty()
            || !end_indent.trim().is_empty()
            || keyword_indent == end_indent
        {
            return;
        }
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
            fix: Some(Fix::new(end_line..end_start, keyword_indent.to_string())),
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
        let source = "function f()\n\tif x\n\t\ta = 1;\n  end\nend\n";
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
}
