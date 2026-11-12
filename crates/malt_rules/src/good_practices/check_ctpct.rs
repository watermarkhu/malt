use super::*;

impl GoodPracticesEngine {
    /// CTPCT: `sprintf`/`fprintf` format specifier count does not match the
    /// number of remaining arguments.
    pub(crate) fn check_ctpct(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("CTPCT") || node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match get_function_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };
        if func_name != "sprintf" && func_name != "fprintf" {
            return Vec::new();
        }

        let args_node = match find_child_of_kind(node, "arguments") {
            Some(a) => a,
            None => return Vec::new(),
        };

        let mut cursor = args_node.walk();
        let args: Vec<Node> = args_node
            .children(&mut cursor)
            .filter(|c| c.is_named())
            .collect();
        // The format is the first argument (for `fprintf`, the second when the
        // first is a file identifier). Only a single string literal is checked:
        // probed, MATLAB is silent for a concatenated `['%s' ...]`, a variable
        // or `strcat(...)` format.
        let format_index =
            if func_name == "fprintf" && args.first().is_some_and(|a| a.kind() != "string") {
                1
            } else {
                0
            };
        let format = match args.get(format_index) {
            Some(format) if format.kind() == "string" => *format,
            _ => return Vec::new(),
        };

        let fmt_text = node_text(format, source).trim_matches('\'');
        let specifiers = count_format_specifiers(fmt_text);
        let arg_count = args.len() - format_index - 1;
        // MATLAB flags only a format with no specifiers that gets arguments, or one
        // with specifiers and no arguments (probed). Two non-zero counts that
        // differ, such as `fprintf('%d %d', 1)`, are silent.
        if (specifiers == 0) == (arg_count == 0) {
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_ctpct_silent_on_match() {
        let source = "fprintf('%d', x);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_ctpct(fc, source);
        assert!(diags.is_empty());
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
