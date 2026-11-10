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
        if self.is_check_enabled("EVLDUAL") && self.eval_is_command_concatenation(node, source) {
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

        // MATLAB reports EVLCS for a one-argument `eval` in statement and value
        // positions (probed); the two-argument form is the EV2IN suggestion instead.
        let two_arguments = find_child_of_kind(node, "arguments").is_some_and(|args| {
            let mut cursor = args.walk();
            let named = args.children(&mut cursor).filter(|c| c.is_named()).count();
            named > 1
        });
        // In a value position MATLAB reports only a literal string: it is silent on
        // `out = eval(expr)` but flags `y = eval('1')` (probed).
        let statement_level = node
            .parent()
            .is_some_and(|p| STATEMENT_PARENTS.contains(&p.kind()));
        let literal_argument = find_child_of_kind(node, "arguments")
            .and_then(|args| args.named_child(0))
            .is_some_and(|arg| arg.kind() == "string");
        // A concatenation without an assignment, `eval(['x' num2str(i)])`, is
        // silent in MATLAB (probed).
        let matrix_without_assignment = find_child_of_kind(node, "arguments")
            .and_then(|args| args.named_child(0))
            .is_some_and(|arg| arg.kind() == "matrix" && !node_text(arg, source).contains('='));
        if two_arguments || (!statement_level && !literal_argument) || matrix_without_assignment {
            return Vec::new();
        }

        // Determine which eval sub-check to fire based on argument patterns.
        let args_text = get_arguments_text(node, source);

        let (check_id, message) = if !self.is_check_enabled("EVLCS") {
            return Vec::new();
        } else {
            let literal = find_child_of_kind(node, "arguments")
                .map(|a| string_literal_text(a, source))
                .unwrap_or_default();
            classify_eval_usage(&literal, &args_text, &self.config)
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

    /// Whether an `eval(...)` call's single argument is a concatenation that
    /// starts with a command word and a space, such as `['load ' name]`.
    fn eval_is_command_concatenation(&self, node: Node, source: &str) -> bool {
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
        // MATLAB's EVLDUAL is for a concatenated command such as
        // `eval(['load ' name])`: the first piece is a command word followed by
        // a space (probed). `eval(['x' num2str(i)])` is not one.
        let arg = named[0];
        if arg.kind() != "matrix" {
            return false;
        }
        let text = node_text(arg, source).trim();
        let Some(rest) = text.strip_prefix('[').map(str::trim_start) else {
            return false;
        };
        let Some(word) = rest.strip_prefix('\'').and_then(|r| r.split('\'').next()) else {
            return false;
        };
        word.ends_with(' ')
            && !word.trim().is_empty()
            && word
                .trim()
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            && !text.contains('=')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evldual_fires_on_concatenated_command() {
        // MATLAB reports EVLDUAL for a concatenated command (probed).
        let source = "eval(['load ' filename]);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_child_of_kind(root, "function_call").unwrap();
        let diags = eng.check_eval(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "EVLDUAL");
    }

    #[test]
    fn test_evldual_silent_on_literal_call() {
        // A literal `eval('load(filename)')` is EVLCS in MATLAB, not EVLDUAL (probed).
        let source = "eval('load(filename)');\n";
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

    #[test]
    fn eval_classification_looks_at_string_literals_only() {
        use crate::test_util::lint_nodes as lint_file;
        use malt_core::Config;
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        let ids = |src: &str| -> Vec<&'static str> {
            lint_file(&*e, src)
                .iter()
                .map(|d| d.rule_id)
                .filter(|id| id.starts_with("EVL"))
                .collect()
        };
        // Constant statements and variables: plain EVLCS.
        assert_eq!(ids("eval('tmp = 2;');\n"), vec!["EVLCS"]);
        // MATLAB is silent on `out = eval(expr)` but flags a literal in a value
        // position (probed: `y = eval('1')`); the two-argument form is EV2IN.
        assert!(ids("out = eval(expr);\n").is_empty());
        assert_eq!(ids("y = eval('1');\n"), vec!["EVLCS"]);
        assert!(ids("eval('a = 1;', 'a = 0;');\n").is_empty());
        // A variable called `s` is not a field access.
        assert_eq!(ids("eval(s);\n"), vec!["EVLCS"]);
        // Built assignments and field names.
        assert_eq!(ids("eval([name ' = 1;']);\n"), vec!["EVLEQ"]);
        assert_eq!(ids("eval(['s.' field ' = 1;']);\n"), vec!["EVLDOT"]);
        // Probed: a constant statement with a field is EVLCS, not EVLDOT.
        assert_eq!(ids("eval('s.a = 1');\n"), vec!["EVLCS"]);
        // Probed: a sequentially numbered variable is EVLSEQVAR, not EVLEQ.
        assert_eq!(ids("eval(['a' num2str(3) ' = 1']);\n"), vec!["EVLSEQVAR"]);
        // Probed: a concatenation without an assignment is silent.
        assert!(ids("eval(['x' num2str(i)]);\n").is_empty());
        // Probed: EVLDUAL needs a command word followed by a space.
        assert_eq!(ids("eval(['load ' f]);\n"), vec!["EVLDUAL"]);
        assert!(!ids("eval(['x' f]);\n").contains(&"EVLDUAL"));
    }
}
