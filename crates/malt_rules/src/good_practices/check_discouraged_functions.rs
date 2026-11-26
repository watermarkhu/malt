use super::*;

/// `true` when the file assigns `name` somewhere: MATLAB then reads it as a variable.
fn assigns_name(node: Node, name: &str, source: &str) -> bool {
    let mut root = node;
    while let Some(parent) = root.parent() {
        root = parent;
    }
    collect_simple_lhs_assignments(root, source)
        .iter()
        .any(|(assigned, _)| assigned == name)
}

/// Whether a call or command has arguments.
fn has_arguments(node: Node) -> bool {
    find_child_of_kind(node, "arguments").is_some_and(|args| args.named_child_count() > 0)
        || find_child_of_kind(node, "command_argument").is_some()
}

impl GoodPracticesEngine {
    /// ADAPPREF: Avoid `addpref` (use settings API).
    /// KEYBOARDFUN: `keyboard` left in production code.
    pub(crate) fn check_discouraged_functions(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() == "lambda" {
            let mut cursor = node.walk();
            let body = node.named_children(&mut cursor).last();
            let Some(body) = body.filter(|b| {
                b.kind() == "identifier"
                    && &source[b.start_byte()..b.end_byte()] == "keyboard"
                    && self.is_check_enabled("KEYBOARDFUN")
            }) else {
                return Vec::new();
            };
            let pos = body.start_position();
            return vec![Diagnostic {
                rule_id: "KEYBOARDFUN",
                message: "Consider removing 'keyboard' function once you have finished debugging. This function may have security implications.".to_string(),
                severity: Severity::Warning,
                byte_range: body.start_byte()..body.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }];
        }
        if node.kind() != "function_call" && node.kind() != "command" {
            return Vec::new();
        }

        let func_name = match node.kind() {
            "function_call" => get_function_call_name(node, source),
            "command" => get_command_name(node, source),
            _ => None,
        };

        let func_name = match func_name {
            Some(n) => n,
            None => return Vec::new(),
        };
        if func_name == "keyboard" && assigns_name(node, "keyboard", source) {
            return Vec::new();
        }

        let (check_id, message): (&'static str, String) = match func_name {
            "addpref" if self.is_check_enabled("ADAPPREF") => (
                "ADAPPREF",
                format!("Use app as the first argument for {func_name}."),
            ),
            "keyboard" if self.is_check_enabled("KEYBOARDFUN") => (
                "KEYBOARDFUN",
                "Consider removing 'keyboard' function once you have finished debugging. This function may have security implications.".to_string(),
            ),
            _ => return Vec::new(),
        };

        // `keyboard` on a line of its own is removed (unsafe: it is code, not a
        // rewrite, and a user may have left it on purpose).
        let fix = (check_id == "KEYBOARDFUN" && !has_arguments(node))
            .then(|| crate::fix_util::delete_statement_line(node, source))
            .flatten()
            .map(malt_core::Fix::unsafe_fix);
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: check_id,
            message,
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{fix_of, reported_without_fix};

    #[test]
    fn keyboardfun_fix_deletes_the_line() {
        let (safe, fixed) =
            fix_of(&engine(), "x = 1;\n    keyboard;\ny = 2;\n", "KEYBOARDFUN").unwrap();
        assert!(!safe);
        assert_eq!(fixed, "x = 1;\ny = 2;\n");
        let (_, fixed) = fix_of(&engine(), "if a\n    keyboard\nend\n", "KEYBOARDFUN").unwrap();
        assert_eq!(fixed, "if a\nend\n");
        let still = crate::test_util::lint_all(&engine(), &fixed);
        assert!(
            still.iter().all(|d| d.rule_id != "KEYBOARDFUN"),
            "{still:?}"
        );
    }

    #[test]
    fn keyboardfun_no_fix_when_sharing_a_line_or_in_an_expression() {
        for source in [
            "x = 1; keyboard;\n",
            "keyboard; x = 1;\n",
            "if a, keyboard, end\n",
            "h = @() keyboard;\n",
        ] {
            assert!(
                reported_without_fix(&engine(), source, "KEYBOARDFUN"),
                "{source}"
            );
        }
    }

    #[test]
    fn keyboardfun_fires_in_anonymous_function_body() {
        // Probed: MATLAB reports KEYBOARDFUN at `keyboard` in `h = @() keyboard;`.
        let source = "h = @() keyboard;\n";
        let tree = parse(source);
        let eng = engine();
        fn find_kind<'a>(node: tree_sitter::Node<'a>, kind: &str) -> Option<tree_sitter::Node<'a>> {
            if node.kind() == kind {
                return Some(node);
            }
            let children: Vec<_> = {
                let mut cursor = node.walk();
                node.children(&mut cursor).collect()
            };
            children.into_iter().find_map(|c| find_kind(c, kind))
        }
        let lambda = find_kind(tree.root_node(), "lambda").expect("lambda node");
        let diags = eng.check_discouraged_functions(lambda, source);
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].rule_id, "KEYBOARDFUN");
    }

    #[test]
    fn test_keyboardfun_fires() {
        let source = "keyboard;\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        // `keyboard` may be parsed as a command or function_call depending on grammar.
        let node = find_child_of_kind(root, "function_call")
            .or_else(|| find_child_of_kind(root, "command"));
        let node = node.expect("keyboard should parse as function_call or command");
        let diags = eng.check_discouraged_functions(node, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "KEYBOARDFUN");
    }

    #[test]
    fn test_keyboardfun_silent_when_keyboard_is_a_variable() {
        let source = "keyboard;\nkeyboard = 3;\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let node = find_child_of_kind(root, "function_call")
            .or_else(|| find_child_of_kind(root, "command"));
        let node = node.expect("keyboard should parse as function_call or command");
        assert!(eng.check_discouraged_functions(node, source).is_empty());
    }
}
