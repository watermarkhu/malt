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
        let format_index = match args.iter().position(|a| a.kind() == "string") {
            Some(i) => i,
            None => return Vec::new(),
        };

        let fmt_text = node_text(args[format_index], source).trim_matches('\'');
        let specifiers = count_format_specifiers(fmt_text);
        let arg_count = args.len() - format_index - 1;
        if specifiers == arg_count {
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

    #[test]
    fn test_ctpct_fires_on_mismatch() {
        let source = "fprintf('%d %d', x);\n";
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
        let source = "sprintf('%d %f', x);\n";
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
