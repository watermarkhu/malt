//! NOPRT check: statement in a function that produces output without a semicolon.

use super::*;

impl FormattingEngine {
    /// Flag an assignment or expression statement inside a function that is not
    /// terminated by a semicolon, so it echoes its result to the console.
    pub(crate) fn check_noprt(
        &self,
        node: Node,
        frame: &Frame,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !frame.statement_level {
            return;
        }
        // Only function bodies (not scripts), and not parfor bodies (probed: silent).
        if !frame.in_function || frame.in_parfor {
            return;
        }
        // A bare `end` in brackets (`[end + 1];`) is a parse error that MATLAB recovers
        // from as a statement without its semicolon: NOPRT, whatever the `;` (probed).
        let bare_end = crate::syntax_errors::has_end_value_in_literal(node, source);
        // `a = 1:2:3:4` parses as a range followed by an ERROR node, and MATLAB
        // reports M3COL for the chain, not NOPRT.
        if !bare_end && frame.following.first().is_some_and(|s| s.kind() == "ERROR") {
            return;
        }
        if !bare_end && semicolon_follows(node, frame.following.iter().copied(), source) {
            return;
        }
        // An assignment that a line break splits before `-x`/`+x` is reported by
        // NOPRC instead (`y = 3` then `-4`). Probed: an expression statement such
        // as `~x(1)` before `-x(1)` still gets NOPRT on its own line.
        if node.kind() == "assignment" && super::check_nopts::noprc_follows_in(node, frame, source)
        {
            return;
        }

        let start = super::check_nopts::report_position(node, source);
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
    fn noprt_expression_before_split_sign_is_still_reported() {
        // Probed: `~x(1)` on line 3 before `-x(1)` gets NOPRT (only an assignment is
        // split by the next line's sign; no NOPRC here).
        let diags = lint("function f(x)\nx = 1;\n~x(1)\n-x(1)\nend\n");
        assert!(
            diags.iter().any(|d| d.rule_id == "NOPRT" && d.line == 3),
            "got: {diags:?}"
        );
    }

    #[test]
    fn noprt_silent_inside_parfor_body() {
        // Probed: statements in a parfor body give no output (pf3, pf4, f24).
        let diags = lint("function f()\nparfor k=1:2\n  s = k\nend\nend\n");
        assert!(!has_id(&diags, "NOPRT"), "got: {diags:?}");
        let diags = lint("function f()\nparfor k=1:2\n  ~k\nend\nend\n");
        assert!(!has_id(&diags, "NOPRT"), "got: {diags:?}");
    }

    #[test]
    fn noprt_fires_on_negation_and_spmd_body() {
        // Probed: `~x` (`not_operator`) echoes in a function, and so does a
        // statement in an spmd body (pf5).
        let diags = lint("function f(x)\n~x\nend\n");
        assert!(has_id(&diags, "NOPRT"), "got: {diags:?}");
        let diags = lint("function f()\nspmd\n  v = 1\nend\nend\n");
        assert!(has_id(&diags, "NOPRT"), "got: {diags:?}");
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

    #[test]
    fn noprt_on_bare_end_statement_whatever_the_semicolon() {
        // Probed: MATLAB reports NOPRT on `[end + 1];` and `{end};` in a function (its
        // recovery drops the semicolon); a statement with no error and a semicolon is silent.
        assert!(has_id(
            &lint("function y = f()\n[end + 1];\nend\n"),
            "NOPRT"
        ));
        assert!(has_id(&lint("function y = f()\n{end};\nend\n"), "NOPRT"));
        assert!(!has_id(&lint("function f()\nx = [1 2];\nend\n"), "NOPRT"));
    }
}
