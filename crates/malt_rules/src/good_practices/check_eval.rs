use super::*;

impl GoodPracticesEngine {
    /// EVLCS / EVLDOT / EVLEQ / EVLSYS / EVLSEQVAR / EVLDUAL: `eval` usage.
    pub(crate) fn check_eval(&self, node: Node, source: &str) -> Vec<Diagnostic> {
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

        if func_name != "eval" {
            return Vec::new();
        }

        let pos = node.start_position();

        // EVLDUAL: eval('funcname(args)') where the argument is a literal
        // function call that can be invoked directly.
        //
        // Limitation (heuristic): only a single string-literal argument whose
        // contents are a plain `name(args)` call is recognized. An `eval` of a
        // call built through concatenation, a variable, or a command-form
        // argument is not flagged as EVLDUAL (it falls through to the other
        // eval sub-checks).
        if self.is_check_enabled("EVLDUAL") && self.eval_is_literal_function_call(node, source) {
            return vec![Diagnostic {
                rule_id: "EVLDUAL",
                message: "This use of 'eval' is unnecessary and can be removed. Call the evaluated function directly using parentheses. For example, use 'load(filename)' instead of 'eval(['load ' filename])'.".to_string(),
                severity: Severity::Warning,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }];
        }

        // Determine which eval sub-check to fire based on argument patterns.
        let args_text = get_arguments_text(node, source);

        let (check_id, message) = if !self.is_check_enabled("EVLCS") {
            return Vec::new();
        } else {
            classify_eval_usage(&args_text, &self.config)
        };

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

    /// Whether an `eval(...)` call's single argument is a literal string whose
    /// contents are a plain function call (`name(args)`).
    fn eval_is_literal_function_call(&self, node: Node, source: &str) -> bool {
        let Some(args) = find_child_of_kind(node, "arguments") else {
            return false;
        };
        let mut named = Vec::new();
        let mut cursor = args.walk();
        for child in args.children(&mut cursor) {
            if child.is_named() {
                named.push(child);
            }
        }
        if named.len() != 1 {
            return false;
        }
        let arg = named[0];
        if arg.kind() != "string" {
            return false;
        }
        let text = node_text(arg, source).trim();
        let inner = text.trim_matches(|c| c == '\'' || c == '"');
        is_literal_function_call(inner)
    }
}

/// Check whether a string contains a single, plain function call of the form
/// `identifier(...)` with balanced parentheses and no string concatenation or
/// matrix construction.
fn is_literal_function_call(inner: &str) -> bool {
    let s = inner.trim();
    let Some(open) = s.find('(') else {
        return false;
    };
    let name = &s[..open];
    let mut chars = name.chars();
    let first_is_alpha = chars
        .next()
        .map(|c| c.is_ascii_alphabetic())
        .unwrap_or(false);
    if !first_is_alpha
        || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        || !s.ends_with(')')
    {
        return false;
    }

    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            '\'' | '"' | '[' => return false,
            _ => {}
        }
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evldual_fires_on_literal_function_call() {
        let source = "eval('load(filename)');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_child_of_kind(root, "function_call").unwrap();
        let diags = eng.check_eval(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "EVLDUAL");
    }

    #[test]
    fn test_evldual_silent_on_dynamic_eval() {
        let source = "eval(['load ' filename]);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_child_of_kind(root, "function_call").unwrap();
        let diags = eng.check_eval(fc, source);
        assert!(
            diags.iter().all(|d| d.rule_id != "EVLDUAL"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn test_evlcs_fires_on_eval() {
        let source = "eval('x = 1');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_child_of_kind(root, "function_call").unwrap();
        let diags = eng.check_eval(fc, source);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].rule_id.starts_with("EVL"));
    }
}
