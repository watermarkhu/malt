//! ALIGN check: elseif/else alignment with if.

use super::*;

// ---------------------------------------------------------------------------
// ALIGN: elseif/else alignment with if
// ---------------------------------------------------------------------------

impl FormattingEngine {
    /// Check that `elseif_clause` and `else_clause` are aligned with their
    /// `if_statement` parent.
    pub(crate) fn check_align(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let if_col = node.start_position().column;
        let child_count = node.child_count();

        for i in 0..child_count {
            let Some(child) = node.child(i) else {
                continue;
            };
            let kind = child.kind();
            if kind != "elseif_clause" && kind != "else_clause" {
                continue;
            }

            let clause_col = child.start_position().column;
            if clause_col != if_col {
                let pos = child.start_position();
                let line_start = line_start_byte(source, child.start_byte());
                diagnostics.push(Diagnostic {
                    rule_id: "ALIGN",
                    message: format!(
                        "This keyword might not be aligned with its matching END on line {}.",
                        pos.row + 1
                    ),
                    severity: Severity::Info,
                    byte_range: line_start..child.start_byte(),
                    line: pos.row + 1,
                    column: clause_col + 1,
                    fix: Some(Fix::new(line_start..child.start_byte(), " ".repeat(if_col))),
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
}
