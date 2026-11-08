//! ALIGN check: elseif/else alignment with if.

use super::*;

// ---------------------------------------------------------------------------
// ALIGN: elseif/else alignment with if
// ---------------------------------------------------------------------------

impl FormattingEngine {
    /// Check that `elseif_clause` and `else_clause` are aligned with their
    /// `if_statement` parent.
    ///
    /// Only a clause that starts its line is checked: in a one-liner such as
    /// `if a, b = 1; else, b = 2; end` there is no indentation to align, and
    /// the text before the keyword is code that must never be rewritten.
    pub(crate) fn check_align(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let if_col = node.start_position().column;
        let child_count = node.child_count();
        let end_line = node.end_position().row + 1;
        // Indent the clause like the `if` line itself (tabs included); fall back
        // to spaces when the `if` shares its line with other code.
        let if_line_start = line_start_byte(source, node.start_byte());
        let if_prefix = &source[if_line_start..node.start_byte()];
        let indent = if if_prefix.trim().is_empty() {
            if_prefix.to_string()
        } else {
            " ".repeat(if_col)
        };

        for i in 0..child_count {
            let Some(child) = node.child(i) else {
                continue;
            };
            let kind = child.kind();
            if kind != "elseif_clause" && kind != "else_clause" {
                continue;
            }

            let clause_col = child.start_position().column;
            let line_start = line_start_byte(source, child.start_byte());
            let before_keyword = &source[line_start..child.start_byte()];
            if !before_keyword.trim().is_empty() {
                continue; // shares its line with code: nothing to align
            }
            if before_keyword != indent {
                let pos = child.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "ALIGN",
                    message: format!(
                        "This keyword might not be aligned with its matching END on line {end_line}."
                    ),
                    severity: Severity::Info,
                    byte_range: line_start..child.start_byte(),
                    line: pos.row + 1,
                    column: clause_col + 1,
                    fix: Some(Fix::new(line_start..child.start_byte(), indent.clone())),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::formatting::tests::{has_id, lint};

    // -- ALIGN ---------------------------------------------------------------

    #[test]
    fn align_misaligned_elseif() {
        let source = "\
function f()
    if x > 0
        a = 1;
      elseif x < 0
        a = -1;
    else
        a = 0;
    end
end
";
        let diags = lint(source);
        assert!(has_id(&diags, "ALIGN"), "got: {diags:?}");
    }

    #[test]
    fn align_aligned_clauses() {
        let source = "\
function f()
    if x > 0
        a = 1;
    elseif x < 0
        a = -1;
    else
        a = 0;
    end
end
";
        let diags = lint(source);
        assert!(!has_id(&diags, "ALIGN"), "got: {diags:?}");
    }

    /// The reviewed bug: the fix replaced everything from the start of the line
    /// to `else`, which deleted `if a, b = 1;` from a one-liner.
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
    fn align_fix_reindents_without_touching_code() {
        let source = "function f()\n\tif x\n\t\ta = 1;\n  else\n\t\ta = 0;\n\tend\nend\n";
        let diags = lint(source);
        let fix = diags
            .iter()
            .find(|d| d.rule_id == "ALIGN")
            .and_then(|d| d.fix.as_ref())
            .expect("ALIGN with a fix");
        let mut fixed = source.to_string();
        fixed.replace_range(fix.byte_range.clone(), &fix.replacement);
        assert_eq!(
            fixed,
            "function f()\n\tif x\n\t\ta = 1;\n\telse\n\t\ta = 0;\n\tend\nend\n"
        );
    }

    #[test]
    fn align_reports_the_line_of_end() {
        let source = "function f()\n    if x\n      a = 1;\n  else\n      a = 0;\n    end\nend\n";
        let diags = lint(source);
        let d = diags.iter().find(|d| d.rule_id == "ALIGN").expect("ALIGN");
        assert!(d.message.ends_with("on line 6."), "{}", d.message);
    }
}
