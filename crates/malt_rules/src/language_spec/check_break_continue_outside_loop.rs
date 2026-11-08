//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// BRKFOR / CONTFOR: break/continue must be inside a loop.
    pub(crate) fn check_break_continue_outside_loop(
        &self,
        root: tree_sitter::Node,
        _source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        Self::check_break_continue_dfs(root, false, diagnostics);
    }

    /// DFS to find break/continue outside loops.
    pub(crate) fn check_break_continue_dfs(
        node: tree_sitter::Node,
        in_loop: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "for_statement" | "while_statement" => {
                // Inside a loop now
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    Self::check_break_continue_dfs(child, true, diagnostics);
                }
                return;
            }
            "function_definition" => {
                // Reset loop context for new function scope
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    Self::check_break_continue_dfs(child, false, diagnostics);
                }
                return;
            }
            "break_statement" if !in_loop => {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "BRKFOR",
                    message: "BREAK statement can only be used in a FOR or WHILE loop.".to_string(),
                    severity: Severity::Error,
                    byte_range: node.start_byte()..node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
            "continue_statement" if !in_loop => {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "CONTFOR",
                    message: "CONTINUE statement can only be used in a FOR or WHILE loop."
                        .to_string(),
                    severity: Severity::Error,
                    byte_range: node.start_byte()..node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::check_break_continue_dfs(child, in_loop, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_brkfor_break_outside_loop() {
        let source = "\
function foo()
    break;
end
";
        let diags = check_source(source, "foo.m");
        let brkfor = filter_by_id(&diags, "BRKFOR");
        assert!(
            !brkfor.is_empty(),
            "BRKFOR should fire for break outside loop"
        );
    }

    #[test]
    fn test_contfor_continue_outside_loop() {
        let source = "\
function foo()
    continue;
end
";
        let diags = check_source(source, "foo.m");
        let contfor = filter_by_id(&diags, "CONTFOR");
        assert!(
            !contfor.is_empty(),
            "CONTFOR should fire for continue outside loop"
        );
    }

    #[test]
    fn test_no_fire_break_inside_loop() {
        let source = "\
function foo()
    for i = 1:10
        if i > 5
            break;
        end
    end
end
";
        let diags = check_source(source, "foo.m");
        let brkfor = filter_by_id(&diags, "BRKFOR");
        assert!(
            brkfor.is_empty(),
            "BRKFOR should NOT fire for break inside a loop"
        );
    }
}
