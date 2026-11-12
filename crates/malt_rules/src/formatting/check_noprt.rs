//! NOPRT check: statement in a function that produces output without a semicolon.

use super::*;
use crate::analysis::control_flow::is_split_statement_continuation;

impl FormattingEngine {
    /// Flag an `assignment` statement inside a function that is not terminated
    /// by a semicolon, so it echoes its result to the console.
    pub(crate) fn check_noprt(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let is_statement_level = node
            .parent()
            .map(|p| STATEMENT_PARENTS.contains(&p.kind()))
            .unwrap_or(false);
        if !is_statement_level {
            return;
        }
        // Only function bodies (not scripts).
        if !is_in_function(node) {
            return;
        }
        // `a = 1:2:3:4` parses as a range followed by an ERROR node, and MATLAB
        // reports M3COL for the chain, not NOPRT.
        if node.next_sibling().is_some_and(|s| s.kind() == "ERROR") {
            return;
        }
        // A statement that a line break splits before `-x`/`+x` is reported by
        // NOPRC instead (`y = 3` then `-4`).
        if node
            .next_named_sibling()
            .is_some_and(|s| is_split_statement_continuation(s, source))
        {
            return;
        }
        if has_trailing_semicolon(node, source) {
            return;
        }

        let start = super::check_nopts::assignment_report_position(node);
        let end_byte = node.end_byte();
        diagnostics.push(Diagnostic {
            rule_id: "NOPRT",
            message: "Add a semicolon after the statement to hide the output (in a function)."
                .to_string(),
            severity: Severity::Info,
            byte_range: node.start_byte()..node.end_byte(),
            line: start.row + 1,
            column: start.column + 1,
            fix: Some(Fix::insert(end_byte, ";")),
        });
    }
}

#[cfg(test)]
mod tests {

    use crate::formatting::tests::{has_id, lint};

    // -- NOPRT ---------------------------------------------------------------

    // Probed: `rv(j) = j` in a loop makes `rv` a variable, so `rv` on its own
    // echoes in a function (NOPRT on line 5).
    #[test]
    fn noprt_fires_on_bare_name_created_by_indexed_assignment() {
        let diags = lint("function f()\nfor j=1:3\n  rv(j)=j;\nend\nrv\nend\n");
        assert!(
            diags.iter().any(|d| d.rule_id == "NOPRT" && d.line == 5),
            "got: {diags:?}"
        );
    }

    #[test]
    fn noprt_not_reported_on_statement_split_before_unary_sign() {
        // Probed: MATLAB reports NOPRT on the `-4` line only.
        let diags = lint("function f(n)\ny = 3\n-4\nend\n");
        assert!(
            !diags.iter().any(|d| d.rule_id == "NOPRT" && d.line == 2),
            "got: {diags:?}"
        );
    }

    #[test]
    fn noprt_assignment_in_function_without_semicolon() {
        let source = "function f()\n    x = 5\nend\n";
        let diags = lint(source);
        assert!(has_id(&diags, "NOPRT"), "got: {diags:?}");
    }

    #[test]
    fn noprt_ok_with_semicolon() {
        let source = "function f()\n    x = 5;\nend\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "NOPRT"), "got: {diags:?}");
    }

    #[test]
    fn noprt_colon_chain_does_not_fire() {
        // Probed: MATLAB reports M3COL for `a:b:c:d`, never NOPRT.
        let source = "function f()\n    a = 1:2:3:4;\n    b = 1:2:3:4:5;\nend\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "NOPRT"), "got: {diags:?}");
    }

    #[test]
    fn noprt_not_fire_in_script() {
        let source = "x = 5\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "NOPRT"), "got: {diags:?}");
    }
}
