//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// ERTXT / WTXT: `error`/`warning` called with a single string argument that
    /// is a message identifier (contains `:`) and no message text.
    pub(crate) fn check_error_warning_message(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(name) = callee_name(node, source) else {
            return;
        };
        let (rule_id, message) = match name.as_str() {
            "error" => (
                "ERTXT",
                "Specify an error message with the message identifier.",
            ),
            "warning" => (
                "WTXT",
                "Specify a warning message with the message identifier.",
            ),
            _ => return,
        };
        let args = argument_nodes(node);
        if args.len() != 1 {
            return;
        }
        let Some(content) = string_content(args[0], source) else {
            return;
        };
        // MATLAB reads a single argument as an identifier only when it has no
        // whitespace: 'Result: done' is a plain message.
        // MATLAB does not check statements that can never run.
        if content.contains(':')
            && !content.chars().any(char::is_whitespace)
            && !in_unreachable_code(node, source)
        {
            self.push_diag(node, rule_id, message, diagnostics);
        }
    }
}

/// Whether an earlier statement in an enclosing block ends control flow.
fn in_unreachable_code(node: tree_sitter::Node, source: &str) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        if matches!(parent.kind(), "block" | "source_file") {
            let script = parent.kind() == "source_file"
                && crate::analysis::control_flow::is_script_body(parent);
            let mut cursor = parent.walk();
            for sibling in parent.named_children(&mut cursor) {
                if sibling.id() == current.id() {
                    break;
                }
                if statement_terminates(sibling, source, script) {
                    return true;
                }
            }
        }
        current = parent;
    }
    false
}

fn statement_terminates(stmt: tree_sitter::Node, source: &str, script: bool) -> bool {
    match stmt.kind() {
        "return_statement" => true,
        "break_statement" | "continue_statement" => !script,
        "function_call" => callee_name(stmt, source).is_some_and(|name| {
            matches!(
                name.as_str(),
                "error" | "throw" | "rethrow" | "exit" | "quit"
            )
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_ertxt_fires_id_with_no_message() {
        let source = "error('MyTool:badInput');\n";
        let diags = check_source(source, "myscript.m");
        let ertxt = filter_by_id(&diags, "ERTXT");
        assert!(
            !ertxt.is_empty(),
            "ERTXT should fire for error ID with no message"
        );
    }

    #[test]
    fn test_ertxt_no_fire_plain_message_or_id_with_message() {
        let source = "\
error('plain message');
error('MyID', 'message here');
";
        let diags = check_source(source, "myscript.m");
        let ertxt = filter_by_id(&diags, "ERTXT");
        assert!(
            ertxt.is_empty(),
            "ERTXT should NOT fire for plain message or ID+message"
        );
    }

    #[test]
    fn test_wtxt_fires_id_with_no_message() {
        let source = "warning('MyTool:badInput');\n";
        let diags = check_source(source, "myscript.m");
        let wtxt = filter_by_id(&diags, "WTXT");
        assert!(
            !wtxt.is_empty(),
            "WTXT should fire for warning ID with no message"
        );
    }

    #[test]
    fn test_wtxt_no_fire_plain_message() {
        let source = "warning('plain message');\nwarning('MyID', 'msg');\n";
        let diags = check_source(source, "myscript.m");
        let wtxt = filter_by_id(&diags, "WTXT");
        assert!(
            wtxt.is_empty(),
            "WTXT should NOT fire for plain message or ID+message"
        );
    }

    #[test]
    fn test_ertxt_wtxt_no_fire_message_with_colon_and_space() {
        // MATLAB: `error('Result: done')` and `warning('hello: world')` are plain messages.
        let source = "\
error('msg:with spaces');
warning('hello: world');
";
        let diags = check_source(source, "myscript.m");
        assert!(filter_by_id(&diags, "ERTXT").is_empty());
        assert!(filter_by_id(&diags, "WTXT").is_empty());
    }

    #[test]
    fn test_ertxt_wtxt_not_reported_after_terminator() {
        // MATLAB: statements after `error(...)` are unreachable and not checked.
        let source = "\
error('a:b');
error('c:d');
warning('e:f');
";
        let diags = check_source(source, "myscript.m");
        assert_eq!(filter_by_id(&diags, "ERTXT").len(), 1);
        assert!(filter_by_id(&diags, "WTXT").is_empty());
    }
}
