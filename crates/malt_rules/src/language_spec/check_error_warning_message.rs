//! See the parent module `super` for the shared engine and helpers.

use super::*;

/// The rule ID and message for `error` (ERTXT) and `warning` (WTXT).
fn error_or_warning(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "error" => Some((
            "ERTXT",
            "Specify an error message with the message identifier.",
        )),
        "warning" => Some((
            "WTXT",
            "Specify a warning message with the message identifier.",
        )),
        _ => None,
    }
}

/// A message identifier is `component:component...`, each component a letter
/// followed by letters, digits or underscores. Probed: `a_b:c` and `A1:b2` are
/// identifiers; `a-b:c`, `a:b.c`, `a:b:`, `:b` and `1a:b` are plain messages.
fn is_message_identifier(text: &str) -> bool {
    let parts: Vec<&str> = text.split(':').collect();
    parts.len() >= 2
        && parts.iter().all(|part| {
            let mut chars = part.chars();
            chars.next().is_some_and(|c| c.is_ascii_alphabetic())
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

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
        let Some((rule_id, message)) = error_or_warning(&name) else {
            return;
        };
        let args = argument_nodes(node);
        if args.len() != 1 {
            return;
        }
        // Probed: `error("a:b")` is silent (MATLAB reads the string object as plain
        // text), but `warning("a:b")` is WTXT. Other argument kinds are not literals.
        let text = node_text(args[0], source);
        let double_quoted = text.starts_with('"');
        if (double_quoted && rule_id == "ERTXT") || !(double_quoted || text.starts_with('\'')) {
            return;
        }
        let Some(content) = string_content(args[0], source) else {
            return;
        };
        // MATLAB reads a single argument as an identifier only when it is one; see
        // is_message_identifier. MATLAB does not check statements that can never run.
        if is_message_identifier(&content) && !in_unreachable_code(node, source) {
            self.push_diag(node, rule_id, message, diagnostics);
        }
    }

    /// ERTXT / WTXT for the command form `error a:b` and `error 'a:b'` (probed: MATLAB
    /// reports both at the argument). Other command words and argument counts are silent.
    pub(crate) fn check_error_warning_command(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut name = None;
        let mut args = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "command_name" => name = Some(node_text(child, source)),
                "command_argument" => args.push(child),
                _ => {}
            }
        }
        let Some((rule_id, message)) = name.and_then(error_or_warning) else {
            return;
        };
        if args.len() != 1 {
            return;
        }
        let text = node_text(args[0], source);
        let content = if text.starts_with('"') {
            return;
        } else if text.len() >= 2 && text.starts_with('\'') && text.ends_with('\'') {
            &text[1..text.len() - 1]
        } else {
            text
        };
        if is_message_identifier(content) && !in_unreachable_code(node, source) {
            self.push_diag(args[0], rule_id, message, diagnostics);
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
        "if_statement" => always_taken_and_terminates(stmt, source),
        _ => false,
    }
}

/// An `if` whose condition is a true literal (`1`, `true`) with no `elseif` or
/// `else` always runs its body. When that body ends control flow, so does the
/// `if` (probed: `if 1 / return / end` makes the next statement unreachable).
fn always_taken_and_terminates(if_node: tree_sitter::Node, source: &str) -> bool {
    let is_true = if_node
        .child_by_field_name("condition")
        .is_some_and(|c| matches!(node_text(c, source), "1" | "true"));
    let mut cursor = if_node.walk();
    let children: Vec<_> = if_node.named_children(&mut cursor).collect();
    let has_branch = children
        .iter()
        .any(|c| matches!(c.kind(), "elseif_clause" | "else_clause"));
    let body_terminates = children
        .iter()
        .find(|c| c.kind() == "block")
        .is_some_and(|body| crate::analysis::control_flow::block_terminates(*body, source));
    is_true && !has_branch && body_terminates
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

    #[test]
    fn test_ertxt_silent_double_quoted_identifier() {
        // Probed (w04): MATLAB is silent on `error("Abc:def")`.
        let source = "function w04()\nerror(\"Abc:def\");\nend\n";
        let diags = check_source(source, "w04.m");
        assert!(filter_by_id(&diags, "ERTXT").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_wtxt_fires_double_quoted_identifier() {
        // Probed (dq1, dq2, dq5): MATLAB reports `warning("a:b")` as WTXT, unlike error.
        let source = "function f()\nwarning(\"Abc:def\");\nend\n";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "WTXT").len(), 1, "got: {diags:?}");
    }

    #[test]
    fn test_ertxt_silent_invalid_identifier_shapes() {
        // Probed (w12, w13, w14, w27, w28): none of these is a message identifier.
        for id in ["a:b:", ":b", "1a:b", "a-b:c", "a:b.c"] {
            let source = format!("function f()\nerror('{id}');\nend\n");
            let diags = check_source(&source, "f.m");
            assert!(
                filter_by_id(&diags, "ERTXT").is_empty(),
                "unexpected ERTXT for {id:?}: {diags:?}"
            );
        }
    }

    #[test]
    fn test_ertxt_fires_underscore_and_digit_identifiers() {
        // Probed (w26, w29): `a_b:c` and `A1:b2` are identifiers.
        let source = "function f()\nwarning('a_b:c');\nend\nfunction g()\nwarning('A1:b2');\nend\n";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "WTXT").len(), 2, "got: {diags:?}");
    }

    #[test]
    fn test_ertxt_wtxt_command_form() {
        // Probed (w10, w30, w32): `error a:b` and `error 'a:b'` give ERTXT at the
        // argument, `warning a:b` gives WTXT. Two arguments (w31) are silent.
        let source = "\
function f1()
error a:b
end
function f2()
error 'a:b';
end
function f3()
warning a:b
end
function f4()
error a:b c;
end
";
        let diags = check_source(source, "f.m");
        let ertxt = filter_by_id(&diags, "ERTXT");
        assert_eq!(ertxt.len(), 2, "got: {diags:?}");
        assert_eq!((ertxt[0].line, ertxt[0].column), (2, 7));
        assert_eq!(filter_by_id(&diags, "WTXT").len(), 1, "got: {diags:?}");
    }

    #[test]
    fn test_ertxt_silent_after_constant_true_if_returns() {
        // Probed (w33): MATLAB folds `if 1` and reports the statement after it as
        // unreachable, so no ERTXT there.
        let source = "function w33()\nif 1\nreturn\nend\nerror('a:b');\nend\n";
        let diags = check_source(source, "w33.m");
        assert!(filter_by_id(&diags, "ERTXT").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_ertxt_fires_after_if_without_constant_condition() {
        // A condition that is not a literal does not end control flow.
        let source = "function f(x)\nif x\nreturn\nend\nerror('a:b');\nend\n";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "ERTXT").len(), 1, "got: {diags:?}");
    }
}
