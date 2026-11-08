//! NOPTS check: statement in a script that produces output without a semicolon.

use super::*;

impl FormattingEngine {
    /// Flag an `assignment` statement at script level that is not terminated by
    /// a semicolon, so it echoes its result to the console.
    pub(crate) fn check_nopts(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let is_statement_level = node
            .parent()
            .map(|p| STATEMENT_PARENTS.contains(&p.kind()))
            .unwrap_or(false);
        if !is_statement_level {
            return;
        }
        // Only scripts (not function bodies).
        if is_in_function(node) {
            return;
        }
        if has_trailing_semicolon(node, source) {
            return;
        }

        let start = node.start_position();
        let end_byte = node.end_byte();
        diagnostics.push(Diagnostic {
            rule_id: "NOPTS",
            message: "Add a semicolon after the statement to hide the output (in a script)."
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

    // -- NOPTS ---------------------------------------------------------------

    #[test]
    fn nopts_assignment_in_script_without_semicolon() {
        let source = "x = 5\n";
        let diags = lint(source);
        assert!(has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_ok_with_semicolon() {
        let source = "x = 5;\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "NOPTS"), "got: {diags:?}");
    }

    #[test]
    fn nopts_not_fire_in_function() {
        let source = "function f()\n    x = 5\nend\n";
        let diags = lint(source);
        assert!(!has_id(&diags, "NOPTS"), "got: {diags:?}");
    }
}
