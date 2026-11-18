use super::*;
use crate::analysis::reaching::resolved_values;

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

        if let Some(diags) = self.check_eval_variable(node, source) {
            return diags;
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
        // A field access built by concatenation, `eval(['s.' f])`, is EVLDOT with
        // no assignment (probed); other concatenations without one are silent.
        // An operating-system command built by concatenation, `eval(['!' c])`, is EVLSYS.
        let literal_text = find_child_of_kind(node, "arguments")
            .map(|a| string_literal_text(a, source))
            .unwrap_or_default();
        let field_access = literal_text.contains('.');
        let command = literal_text.trim_start().starts_with('!');
        if two_arguments
            || (!statement_level && !literal_argument)
            || (matrix_without_assignment && !field_access && !command)
        {
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
        // EVLCS is for a constant statement only (probed): a dynamic argument is silent.
        if check_id == "EVLCS" && !is_constant_argument(&args_text) {
            return Vec::new();
        }
        // EVLCS is for one literal: a concatenation of several is silent (probed: `['x' ' = 1;']`).
        let mut pieces = Vec::new();
        if let Some(args) = find_child_of_kind(node, "arguments") {
            collect_nodes_of_kind(args, "string", &mut pieces);
        }
        if check_id == "EVLCS" && pieces.len() > 1 {
            return Vec::new();
        }

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

    /// EVLCS / EVLEQ for `eval(t)` where `t` holds a constant text or a concatenation
    /// built from constants (probed: `t = 'x = 1;'; eval(t)` is EVLCS, `t = ['x = '
    /// num2str(v)]; eval(t)` is EVLEQ, through copies and `t = [t ';']`). Every value
    /// that reaches the call must give the same message; a literal in a value position
    /// only (`y = eval(t)`). `None` when the argument is not a single variable.
    fn check_eval_variable(&self, node: Node, source: &str) -> Option<Vec<Diagnostic>> {
        let args = find_child_of_kind(node, "arguments")?;
        if args.named_child_count() != 1 {
            return None;
        }
        let arg = args.named_child(0)?;
        if arg.kind() != "identifier" {
            return None;
        }
        let silent = Some(Vec::new());
        if !self.is_check_enabled("EVLCS") {
            return silent;
        }
        let Some(values) = resolved_values(arg, source) else {
            return silent;
        };
        let statement_level = node
            .parent()
            .is_some_and(|p| STATEMENT_PARENTS.contains(&p.kind()));
        let mut found = Vec::new();
        for value in values {
            if !statement_level && value.kind() != "string" {
                return silent;
            }
            let (literal, args_text) = match value.kind() {
                "string" => (
                    string_literal_text(value, source),
                    format!("({})", node_text(value, source)),
                ),
                "matrix" => {
                    let mut literal = String::new();
                    let mut parts = Vec::new();
                    if !expand_matrix(value, source, 0, &mut literal, &mut parts) {
                        return silent;
                    }
                    (literal, format!("([{}])", parts.join(" ")))
                }
                _ => return silent,
            };
            if value.kind() == "matrix" && command_word_concatenation(value, source) {
                // `t = ['load ' x]; eval(t)` is EVLDUAL (probed), as for a direct argument.
                return Some(if self.is_check_enabled("EVLDUAL") {
                    vec![Diagnostic {
                    rule_id: "EVLDUAL",
                    message: "This use of 'eval' is unnecessary and can be removed. Call the evaluated function directly using parentheses. For example, use 'load(filename)' instead of 'eval(['load ' filename])'.".to_string(),
                    severity: Severity::Warning,
                    byte_range: node.start_byte()..node.end_byte(),
                    line: node.start_position().row + 1,
                    column: node.start_position().column + 1,
                    fix: None,
                }]
                } else {
                    Vec::new()
                });
            }
            let (id, message) = classify_eval_usage(&literal, &args_text, &self.config);
            // As for a direct argument: a constant matrix is EVLCS only with one
            // literal (probed), and a dynamic EVLCS is silent.
            if id == "EVLCS" && (value.kind() == "matrix" || !is_constant_argument(&args_text)) {
                return silent;
            }
            found.push((id, message));
        }
        let first = *found.first()?;
        if found.iter().any(|f| *f != first) {
            return silent;
        }
        let pos = node.start_position();
        Some(vec![Diagnostic {
            rule_id: first.0,
            message: first.1.to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }])
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
        command_word_concatenation(named[0], source)
    }
}

/// The pieces of a matrix, with the elements of its rows (`[a b; c]` gives a, b, c).
fn matrix_pieces(node: Node) -> Vec<Node> {
    let mut out = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "row" {
            out.extend(matrix_pieces(child));
        } else {
            out.push(child);
        }
    }
    out
}

/// The literal text and the source text of a matrix that builds an eval argument,
/// with each variable replaced by the single value it holds (`t = [t ';']`). `false`
/// when a variable has no single value that is a literal or such a matrix.
fn expand_matrix(
    node: Node,
    source: &str,
    depth: usize,
    literal: &mut String,
    parts: &mut Vec<String>,
) -> bool {
    if depth > 8 {
        return false;
    }
    for piece in matrix_pieces(node) {
        let value = if piece.kind() == "identifier" {
            let Some(values) = resolved_values(piece, source) else {
                return false;
            };
            match values.as_slice() {
                [value] => *value,
                _ => return false,
            }
        } else {
            piece
        };
        match value.kind() {
            "string" => {
                literal.push_str(&string_literal_text(value, source));
                parts.push(node_text(value, source).to_string());
            }
            "matrix" => {
                if !expand_matrix(value, source, depth + 1, literal, parts) {
                    return false;
                }
            }
            _ if piece.kind() == "identifier" => return false,
            _ => parts.push(node_text(value, source).to_string()),
        }
    }
    true
}

/// Whether a matrix is a concatenation that starts with a command word and a space,
/// such as `['load ' name]` (probed: EVLDUAL).
fn command_word_concatenation(arg: Node, source: &str) -> bool {
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
    fn eval_probed_variants_match_matlab() {
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
        // Probed: the constant-name form needs `name = ` as its first literal.
        assert!(ids("eval(['x = [' num2str(v) '];']);\n").is_empty());
        assert!(ids("eval(['x = {' num2str(v) '};']);\n").is_empty());
        assert!(ids("eval(['x = 5' num2str(v) ';']);\n").is_empty());
        assert!(ids("eval(['x' ' = 1;']);\n").is_empty());
        assert_eq!(ids("eval(['x = ' '[' num2str(v) ']']);\n"), vec!["EVLEQ"]);
        // Probed: a constant command is EVLCS; a built one is EVLSYS.
        assert_eq!(ids("eval('!ls');\n"), vec!["EVLCS"]);
        assert_eq!(ids("eval('system(''ls'')');\n"), vec!["EVLCS"]);
        assert_eq!(ids("eval(['!' c]);\n"), vec!["EVLSYS"]);
        // Probed: a field access built by concatenation is EVLDOT, with or without `=`.
        assert_eq!(ids("eval(['s.' f]);\n"), vec!["EVLDOT"]);
        assert_eq!(ids("eval(['s(2).' f]);\n"), vec!["EVLDOT"]);
        assert_eq!(ids("eval(['s.' f ';']);\n"), vec!["EVLDOT"]);
        assert!(ids("eval(['q' f]);\n").is_empty());
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
        // Probed: a variable argument is silent (`eval(s)` on an input is no EVLCS).
        assert!(ids("eval(s);\n").is_empty());
        // Probed: a constant one-literal concatenation is EVLCS, and a built
        // assignment with a constant name is EVLEQ.
        assert_eq!(ids("eval(['x = 1;']);\n"), vec!["EVLCS"]);
        assert_eq!(ids("eval(['x = ' num2str(v)]);\n"), vec!["EVLEQ"]);
        assert_eq!(ids("eval(['x = ' s ';']);\n"), vec!["EVLEQ"]);
        assert_eq!(
            ids("eval(['y = ' num2str(a) ' + ' num2str(b) ';']);\n"),
            vec!["EVLEQ"]
        );
        // Probed: `sprintf` outside a concatenation, `num2str` alone and a dynamic
        // name are silent; `sprintf` inside a concatenation is EVLEQ.
        assert!(ids("eval(sprintf('result_%s = out;', mode));\n").is_empty());
        assert!(ids("eval(num2str(v));\n").is_empty());
        assert!(ids("eval(['x_' name ' = ' num2str(v)]);\n").is_empty());
        assert!(ids("eval(['x(' num2str(i) ') = 1;']);\n").is_empty());
        assert_eq!(ids("eval(['x = ' sprintf('%d', v) ';']);\n"), vec!["EVLEQ"]);
        // Built assignments and field names.
        assert!(ids("eval([name ' = 1;']);\n").is_empty());
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

#[cfg(test)]
mod variable_tests {
    use crate::test_util::lint_nodes;
    use malt_core::Config;

    fn evl_ids(src: &str) -> Vec<&'static str> {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_nodes(&*e, src)
            .iter()
            .map(|d| d.rule_id)
            .filter(|id| id.starts_with("EVL"))
            .collect()
    }

    // Probed: a variable holding a constant text is EVLCS; every value that reaches
    // the call must give the same message (two literal branches, a loop back edge).
    #[test]
    fn eval_of_a_constant_variable_is_evlcs() {
        assert_eq!(evl_ids("t = 'x = 1;';\neval(t)\n"), vec!["EVLCS"]);
        assert_eq!(evl_ids("t = 'x = 1;';\ns = t;\neval(s)\n"), vec!["EVLCS"]);
        assert_eq!(evl_ids("t = 'a.b = 1;';\neval(t)\n"), vec!["EVLCS"]);
        assert_eq!(evl_ids("t = 'x = 1;';\ny = eval(t);\n"), vec!["EVLCS"]);
        assert_eq!(
            evl_ids("t = 'x = 1;'; t = 'y = 2;';\neval(t)\n"),
            vec!["EVLCS"]
        );
        assert_eq!(
            evl_ids("t = 'x = 1;';\nc = rand > 0.5;\nif c\n    t = 'y = 2;';\nend\neval(t)\n"),
            vec!["EVLCS"]
        );
        assert_eq!(
            evl_ids("t = 'x = 1;';\nfor k = 1:2\n    eval(t);\n    t = 'x = 2;';\nend\n"),
            vec!["EVLCS"]
        );
    }

    // Probed: a built assignment held in a variable is EVLEQ, through copies,
    // `t = [t ';']` and `num2str` pieces; a dotted name is EVLEQ too.
    #[test]
    fn eval_of_a_built_variable_is_evleq() {
        assert_eq!(
            evl_ids("v = 3;\nt = ['x = ' num2str(v)];\neval(t)\n"),
            vec!["EVLEQ"]
        );
        assert_eq!(
            evl_ids("t = ['x = ' num2str(3)];\nt = [t ';'];\neval(t)\n"),
            vec!["EVLEQ"]
        );
        assert_eq!(
            evl_ids("t = ['x = ' 'y;'];\nt = [t ' '];\neval(t)\n"),
            vec!["EVLEQ"]
        );
        assert_eq!(
            evl_ids("a.b = 1;\nt = ['a.b = ' num2str(5) ';'];\neval(t)\n"),
            vec!["EVLEQ"]
        );
    }

    // Probed: a concatenated command held in a variable is EVLDUAL.
    #[test]
    fn eval_of_a_command_variable_is_evldual() {
        assert_eq!(evl_ids("t = ['load ' 'x'];\neval(t)\n"), vec!["EVLDUAL"]);
    }

    // Probed: silent for an input, a cleared name, a value changed by indexing and the
    // two-argument form (EV2IN is another ID).
    #[test]
    fn eval_of_a_variable_is_silent_when_its_value_is_not_known() {
        assert!(evl_ids("eval(t)\n").is_empty());
        assert!(evl_ids("t = 'x = 1;';\nclear t\neval(t)\n").is_empty());
        assert!(evl_ids("t = 'x = 1;';\nt(end+1) = ' ';\neval(t)\n").is_empty());
        assert!(evl_ids("t = 'x = 1;';\neval(t, 'disp(2)')\n").is_empty());
    }
}
