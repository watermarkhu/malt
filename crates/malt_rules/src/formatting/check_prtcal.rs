//! PRTCAL check: bare function call statement without a trailing semicolon.

use super::*;

impl FormattingEngine {
    /// Flag a `function_call` used as a bare statement (producing output) that
    /// is not terminated by a semicolon.
    pub(crate) fn check_prtcal(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let is_statement_level = node
            .parent()
            .map(|p| STATEMENT_PARENTS.contains(&p.kind()))
            .unwrap_or(false);
        if !is_statement_level {
            return;
        }
        if has_trailing_semicolon(node, source) {
            return;
        }

        let start = node.start_position();
        let end_byte = node.end_byte();
        diagnostics.push(Diagnostic {
            rule_id: "PRTCAL",
            message: "Add a semicolon after the function call to hide the output.".to_string(),
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

    // -- PRTCAL --------------------------------------------------------------

    #[test]
    fn prtcal_function_call_without_semicolon() {
        let source = "disp('hello')\n";
        let diags = lint(source);
        assert!(has_id(&diags, "PRTCAL"), "got: {diags:?}");
    }

    #[test]
    fn prtcal_ok_with_semicolon() {
        let source = "disp('hello');\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "PRTCAL"), "got: {diags:?}");
    }

    #[test]
    fn prtcal_ok_when_not_statement_level() {
        let source = "x = compute(1);\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "PRTCAL"), "got: {diags:?}");
    }
}
