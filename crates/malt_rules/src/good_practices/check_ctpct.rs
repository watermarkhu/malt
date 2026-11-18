use super::*;

impl GoodPracticesEngine {
    /// CTPCT: `sprintf`/`fprintf` format specifier count does not match the
    /// number of remaining arguments, and the same for a message passed to
    /// `error`/`warning`.
    pub(crate) fn check_ctpct(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("CTPCT") || node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match get_function_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };
        let is_message = func_name == "error" || func_name == "warning";
        if func_name != "sprintf" && func_name != "fprintf" && !is_message {
            return Vec::new();
        }

        let args_node = match find_child_of_kind(node, "arguments") {
            Some(a) => a,
            None => return Vec::new(),
        };

        let mut cursor = args_node.walk();
        // Continuations (`...`) and comments are named nodes but not arguments.
        let args: Vec<Node> = args_node
            .children(&mut cursor)
            .filter(|c| c.is_named() && !matches!(c.kind(), "comment" | "line_continuation"))
            .collect();
        // The format is the first argument (for `fprintf`, the second when the
        // first is a file identifier; for `error` and `warning`, the second when
        // the first is a message identifier). Only a single string literal is
        // checked: probed, MATLAB is silent for a concatenated `['%s' ...]`, a
        // variable or `strcat(...)` format.
        let mut format_index =
            if func_name == "fprintf" && args.first().is_some_and(|a| a.kind() != "string") {
                1
            } else {
                0
            };
        if is_message {
            // Probed (plan 6.2, 20 shapes): the first argument is the format unless it
            // is a message identifier (`a:b`, components that start with a letter) or,
            // for `warning`, a control word (`'off'`, `'on'`, `'query'`, ...).
            let Some(first) = args.first().filter(|a| a.kind() == "string") else {
                return Vec::new();
            };
            let first_text = node_text(*first, source).trim_matches(|c| c == '\'' || c == '"');
            if is_message_identifier_text(first_text) {
                format_index = 1;
            } else if func_name == "warning"
                && matches!(
                    first_text,
                    "on" | "off" | "query" | "error" | "verbose" | "backtrace" | "debug"
                )
            {
                return Vec::new();
            }
        }
        let format = match args.get(format_index) {
            Some(format) if format.kind() == "string" => *format,
            _ => return Vec::new(),
        };

        let fmt_text = node_text(format, source).trim_matches(|c| c == '\'' || c == '"');
        let specifiers = count_format_specifiers(fmt_text);
        let arg_count = args.len() - format_index - 1;
        // Probed MATLAB rule: a format with no specifiers that gets arguments is
        // reported. With specifiers, too few arguments are silent, except that
        // sprintf and fprintf report none at all (`fprintf('%d %d');` is the only
        // zero-argument case), and too many are reported only when their count is
        // not a multiple of the specifier count, because the format is reused
        // (`sprintf('%d', a, b)` and `sprintf('%d %d', a, b, a, b)` are silent).
        let mismatch = if specifiers == 0 {
            arg_count > 0
        } else if is_message {
            arg_count > specifiers && !arg_count.is_multiple_of(specifiers)
        } else {
            arg_count == 0 || (arg_count > specifiers && !arg_count.is_multiple_of(specifiers))
        };
        if !mismatch {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "CTPCT",
            message: "The format might not agree with the argument count.".to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

/// Whether a message argument reads as a message identifier: two or more
/// colon-separated components, each a letter followed by letters, digits or
/// underscores (probed: `a:`, `:b`, `1a:b`, `a.b:c` and `A_1:b-2` are formats).
fn is_message_identifier_text(text: &str) -> bool {
    let mut parts = 0;
    for part in text.split(':') {
        let mut chars = part.chars();
        let ok = chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !ok {
            return false;
        }
        parts += 1;
    }
    parts >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    // Probed (plan 6.2): `error`/`warning` take the first argument as the format
    // unless it is an identifier or a `warning` control word.
    #[test]
    fn test_ctpct_message_formats() {
        let n = |src: &str| {
            let tree = parse(src);
            let eng = engine();
            let mut count = 0;
            let mut stack = vec![tree.root_node()];
            while let Some(node) = stack.pop() {
                count += eng.check_ctpct(node, src).len();
                let mut c = node.walk();
                stack.extend(node.children(&mut c));
            }
            count
        };
        for src in [
            "error('msg', x);\n",
            "error('msg', 'text');\n",
            "warning('msg', x);\n",
            "error('ErrorID', x, y);\n",
            "error(\"msg\", x);\n",
            "error('A:b', 'fmt', 1);\n",
            "error('A:b', 'fmt %d %d', 1, 2, 3);\n",
            "error('a:', x);\n",
            "error('a.b:c', x);\n",
        ] {
            assert_eq!(n(src), 1, "{src}");
        }
        for src in [
            "warning('off', 'all');\n",
            "warning('on', 'MATLAB:foo');\n",
            "warning('query', 'MATLAB:foo');\n",
            "warning('error', x);\n",
            "error('msg %d', 1);\n",
            "error('msg');\n",
            "error('Mod:sub', x);\n",
        ] {
            assert_eq!(n(src), 0, "{src}");
        }
    }

    // Probe c4_B ct/c1, c2, c4, c7, c8 (k_continuations.m:49): MATLAB is silent
    // when the format is not one plain string literal.
    #[test]
    fn test_ctpct_silent_on_format_that_is_not_one_literal() {
        for source in [
            "s = sprintf(['%s: %.2f ' ...\n    '%d'], 'a', 1, 2);\n",
            "s = sprintf(['%s: %.2f ' '%d'], 'a', 1, 2);\n",
            "s = sprintf(strcat('%s', '%d'), 'a', 2);\n",
            "f = '%s: %.2f'; s = sprintf(f, 'a', 1);\n",
            "s = sprintf(['%s ' 'x'], 'a', 1);\n",
        ] {
            let tree = parse(source);
            let eng = engine();
            let fc = find_descendant_of_kind(tree.root_node(), "function_call").unwrap();
            assert!(eng.check_ctpct(fc, source).is_empty(), "{source:?}");
        }
    }

    #[test]
    fn test_ctpct_fires_on_mismatch() {
        let source = "fprintf('%d %d');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_ctpct(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "CTPCT");
    }

    #[test]
    fn test_ctpct_fires_on_sprintf_mismatch() {
        let source = "sprintf('hello', x);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_ctpct(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "CTPCT");
    }

    /// CTPCT diagnostics for every function call in `source`.
    fn ctpct(source: &str) -> usize {
        let tree = parse(source);
        let eng = engine();
        let mut count = 0;
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            count += eng.check_ctpct(n, source).len();
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        count
    }

    #[test]
    fn test_ctpct_fires_on_too_many_arguments_not_a_multiple() {
        // Probed: `sprintf('%s %d', 'a', 1, 2)`, `sprintf('%d %d', a, b, a)`,
        // `fprintf('%d %d %d', 1, 2, 3, 4)`, `sprintf('%d%d', a, b, c)`, and the
        // double-quoted `sprintf("%s %d", 'a', 1, 2)`.
        for source in [
            "s = sprintf('%s %d', 'a', 1, 2);\n",
            "s = sprintf('%d %d', a, b, a);\n",
            "fprintf('%d %d %d', 1, 2, 3, 4);\n",
            "s = sprintf('%d%d', a, b, c);\n",
            "s = sprintf(\"%s %d\", 'a', 1, 2);\n",
        ] {
            assert_eq!(ctpct(source), 1, "{source}");
        }
    }

    #[test]
    fn test_ctpct_silent_on_too_many_arguments_that_are_a_multiple() {
        // Probed: the format is reused, so `sprintf('%d', a, b)` is legitimate.
        for source in [
            "s = sprintf('%d', a, b);\n",
            "s = sprintf('%d', a, b, c);\n",
            "s = sprintf('%d %d', a, b, a, b);\n",
            "s = sprintf('%d %d', [1 2 3], 4);\n",
            "fprintf(1, '%d', a, b);\n",
            "s = sprintf('%s', 'a', 'b', 'c');\n",
        ] {
            assert_eq!(ctpct(source), 0, "{source}");
        }
    }

    #[test]
    fn test_ctpct_on_error_and_warning_messages() {
        // Probed: a message with a space and no specifiers that gets arguments
        // is reported. Action words and identifiers are not formats, and a
        // message with specifiers and matching arguments is silent.
        for source in [
            "error('Something failed', a);\n",
            "warning('hello there', a);\n",
            "error('bad id:here', 'x');\n",
        ] {
            assert_eq!(ctpct(source), 1, "{source}");
        }
        for source in [
            "error('x: %d', a, b, c);\n",
            "warning('x %d', a, b);\n",
            "error('Failed %d', x);\n",
            "warning('off', 'all');\n",
            "error('my:id', 'Something failed');\n",
        ] {
            assert_eq!(ctpct(source), 0, "{source}");
        }
    }

    #[test]
    fn test_ctpct_silent_on_nonzero_count_mismatch() {
        // MATLAB does not report this one-argument mismatch (probed).
        let source = "fprintf('%d %d', x);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        assert!(eng.check_ctpct(fc, source).is_empty());
    }

    #[test]
    fn test_ctpct_silent_on_escaped_percent() {
        let source = "sprintf('100%% %d', x);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_ctpct(fc, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_ctpct_counts_arguments_not_continuations() {
        // A `...` continuation is not an argument (fixtures corpus1, corpus4).
        let source = "s = sprintf('%d %d', ...\n    a, ...\n    b);\n";
        assert_eq!(ctpct(source), 0);
        let source = "s = sprintf('%d %d', ...\n    a, ...\n    b, a);\n";
        assert_eq!(ctpct(source), 1);
    }

    #[test]
    fn test_ctpct_silent_with_file_id() {
        let source = "fprintf(fid, '%d', x);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_ctpct(fc, source);
        assert!(diags.is_empty());
    }
}
