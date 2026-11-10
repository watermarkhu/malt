use super::*;

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

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: check_id,
            message,
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
