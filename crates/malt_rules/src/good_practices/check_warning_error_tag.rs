use super::*;

impl GoodPracticesEngine {
    /// WNTAG / ERTAG: `warning`/`error` called without a message ID.
    ///
    /// A message ID is a string argument of the form `'comp:tag'` (contains `:`).
    pub(crate) fn check_warning_error_tag(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match get_function_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };

        let (check_id, message) = match func_name {
            "warning" if self.is_check_enabled("WNTAG") => (
                "WNTAG",
                "The first argument of WARNING should be a message identifier. Using a message identifier allows users better control over the message.",
            ),
            "error" if self.is_check_enabled("ERTAG") => (
                "ERTAG",
                "The first argument of ERROR should be a message identifier.",
            ),
            _ => return Vec::new(),
        };

        // MATLAB (R2026b, probed over 40 shapes): the first argument is a string literal that is
        // not a message identifier (`component:mnemonic`, `[A-Za-z]\w*(:\w+)+`, so 'a:b c',
        // 'a: b', 'a:b-c', and 'Result: %d' are not); `error(x)` with a single plain variable is
        // flagged too (as are `error(s.message)` and `error(['a' x])`), while `error(x, 3)`, `error(sprintf(..), 3)`, a concatenation and a struct
        // are not. A statement after an unconditional `error` is not looked at.
        let args_node = find_child_of_kind(node, "arguments");
        let args: Vec<Node> = args_node.map(named_children_of).unwrap_or_default();
        let Some(first_arg) = args.first().copied() else {
            return Vec::new();
        };
        let flagged = match first_arg.kind() {
            "string" => {
                let body = string_body(node_text(first_arg, source));
                // `warning('off', ...)`, `warning('on', ...)`, `warning('query', ...)` and
                // `warning('error', ...)` change a warning state, and `warning('')` is a reset.
                let state_control = check_id == "WNTAG"
                    && (body.is_empty()
                        || matches!(
                            body,
                            "on" | "off" | "query" | "error" | "backtrace" | "verbose"
                        ));
                !state_control && !is_message_identifier(body)
            }
            "identifier" | "field_expression" | "matrix" => check_id == "ERTAG" && args.len() == 1,
            _ => false,
        };
        if !flagged || (check_id == "ERTAG" && after_terminating_statement(node, source)) {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: check_id,
            message: message.to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

/// The text of a string literal without its quotes.
fn string_body(text: &str) -> &str {
    let t = text.trim();
    t.strip_prefix(['\'', '"'])
        .and_then(|t| t.strip_suffix(['\'', '"']))
        .unwrap_or(t)
}

/// `[A-Za-z]\w*(:\w+)+`: a component, then one or more colon-separated mnemonics.
fn is_message_identifier(id: &str) -> bool {
    let word = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    let mut parts = id.split(':');
    let head = parts.next().unwrap_or("");
    word(head)
        && head.starts_with(|c: char| c.is_ascii_alphabetic())
        && id.contains(':')
        && parts.all(word)
}

fn named_children_of(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .filter(|c| c.kind() != "comment")
        .collect()
}

/// Whether `node` follows a statement that never falls through (`return`, `break`, `continue`,
/// `error`, `throw`, `rethrow`) in its block or in a block that encloses it, up to the function:
/// MATLAB does not analyse such unreachable code for these checks.
fn after_terminating_statement(node: Node, source: &str) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        if matches!(
            parent.kind(),
            "function_definition" | "source_file" | "block"
        ) {
            let mut cursor = parent.walk();
            for sibling in parent.named_children(&mut cursor) {
                if sibling.id() == current.id() || sibling.start_byte() >= current.start_byte() {
                    break;
                }
                let callee = match sibling.kind() {
                    "return_statement" | "break_statement" | "continue_statement" => {
                        return true;
                    }
                    "function_call" => sibling.child_by_field_name("name"),
                    "command" => sibling.named_child(0),
                    _ => None,
                };
                if callee
                    .is_some_and(|c| matches!(node_text(c, source), "error" | "throw" | "rethrow"))
                {
                    return true;
                }
            }
            if parent.kind() != "block" {
                break;
            }
        }
        current = parent;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::lint_nodes;
    use malt_core::Config;

    fn ids(src: &str) -> Vec<&'static str> {
        let config = Config::from_toml("[lint]\npreset = \"all\"\n").expect("valid config");
        lint_nodes(&*GoodPracticesEngine::from_config(&config), src)
            .iter()
            .map(|d| d.rule_id)
            .collect()
    }

    #[test]
    fn a_string_that_is_not_an_identifier_is_flagged() {
        for first in [
            "'hello'",
            "'Result: %d'",
            "'a: b'",
            "'a:b c'",
            "'a:b-c'",
            "\"abc %d\"",
        ] {
            let src = format!("function f(x)\nerror({first}, 1);\nwarning({first}, 1);\nend\n");
            let found = ids(&src);
            assert!(
                found.contains(&"ERTAG") && found.contains(&"WNTAG"),
                "{first}: {found:?}"
            );
        }
        for first in ["'a:b'", "'A1:b2'", "'a_b:c_d'", "'a:b:c'"] {
            let src = format!("function f(x)\nerror({first}, 1);\nwarning({first}, 1);\nend\n");
            let found = ids(&src);
            assert!(
                !found.contains(&"ERTAG") && !found.contains(&"WNTAG"),
                "{first}: {found:?}"
            );
        }
    }

    #[test]
    fn variable_and_computed_first_arguments() {
        assert!(ids("function f(x)\nerror(x);\nend\n").contains(&"ERTAG"));
        assert!(!ids("function f(x)\nerror(x, 3);\nend\n").contains(&"ERTAG"));
        assert!(!ids("function f(x)\nerror(sprintf('a'), 3);\nend\n").contains(&"ERTAG"));
        assert!(!ids("function f(x)\nwarning(x);\nend\n").contains(&"WNTAG"));
    }

    #[test]
    fn code_after_an_unconditional_error_is_not_flagged() {
        let src = "function f(x)\nerror('hello');\nerror('Result: %d', 3);\nend\n";
        let n = ids(src).iter().filter(|i| **i == "ERTAG").count();
        assert_eq!(n, 1);
        let nested = "function f(x)\nif x\nerror('a');\nend\nif x\nerror('b');\nend\nend\n";
        assert_eq!(ids(nested).iter().filter(|i| **i == "ERTAG").count(), 2);
    }
}
