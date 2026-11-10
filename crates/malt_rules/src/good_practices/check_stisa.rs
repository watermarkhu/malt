use super::*;

impl GoodPracticesEngine {
    /// STISA: Use `isa` instead of `strcmp(class(obj), '...')`.
    ///
    /// Fires for a `strcmp` call with two arguments where one is a `class(...)`
    /// call and the other is a single-quoted character literal, in either order.
    /// MATLAB does not report `strcmpi`, double-quoted strings, cell arrays, or a
    /// comparison of two `class` calls.
    pub(crate) fn check_stisa(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("STISA") || node.kind() != "function_call" {
            return Vec::new();
        }

        if get_function_call_name(node, source) != Some("strcmp") {
            return Vec::new();
        }

        let args_node = match find_child_of_kind(node, "arguments") {
            Some(a) => a,
            None => return Vec::new(),
        };
        let mut cursor = args_node.walk();
        let args: Vec<Node> = args_node.named_children(&mut cursor).collect();
        let [first, second] = args.as_slice() else {
            return Vec::new();
        };

        let is_class_call = |arg: &Node| get_function_call_name(*arg, source) == Some("class");
        // Probed: `char`, `cell`, `struct` and `logical` have their own IDs
        // (ISCHR, ISCELL, ISSTR, ISLOG), so STISA does not report them.
        let is_char_literal = |arg: &Node| {
            arg.kind() == "string"
                && node_text(*arg, source).starts_with('\'')
                && !matches!(
                    node_text(*arg, source),
                    "'char'" | "'cell'" | "'struct'" | "'logical'"
                )
        };
        let compares_class_name = (is_class_call(first) && is_char_literal(second))
            || (is_char_literal(first) && is_class_call(second));
        if !compares_class_name {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "STISA",
            message: "Consider using ISA instead of comparing the class name.".to_string(),
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

    /// Run STISA on the first function call in `source` (the `strcmp` in these tests).
    fn stisa_count(source: &str) -> usize {
        let tree = parse(source);
        let call = find_descendant_of_kind(tree.root_node(), "function_call").unwrap();
        engine().check_stisa(call, source).len()
    }

    #[test]
    fn test_stisa_fires_class_against_char_literal() {
        assert_eq!(stisa_count("b = strcmp(class(o), 'double');\n"), 1);
        assert_eq!(stisa_count("b = strcmp('double', class(o));\n"), 1);
        assert_eq!(stisa_count("b = strcmp(class(o.a), 'double');\n"), 1);
    }

    #[test]
    fn test_stisa_no_fire_strcmpi() {
        // Probed: MATLAB reports `strcmp` only.
        assert_eq!(stisa_count("b = strcmpi(class(o), 'double');\n"), 0);
    }

    #[test]
    fn test_stisa_no_fire_double_quoted_string() {
        // Probed: a string scalar ("double") is not reported.
        assert_eq!(stisa_count("b = strcmp(class(o), \"double\");\n"), 0);
    }

    #[test]
    fn test_stisa_no_fire_cell_or_two_class_calls() {
        assert_eq!(
            stisa_count("b = strcmp(class(o), {'double', 'single'});\n"),
            0
        );
        assert_eq!(stisa_count("b = strcmp(class(o), class(p));\n"), 0);
    }

    #[test]
    fn test_stisa_fires_on_other_classes() {
        // Probed: `single`, `int32` and `function_handle` are reported (fix/gp-b__t1.m).
        assert_eq!(stisa_count("b = strcmp(class(o), 'single');\n"), 1);
        assert_eq!(stisa_count("b = strcmp(class(o), 'int32');\n"), 1);
        assert_eq!(stisa_count("b = strcmp(class(o), 'function_handle');\n"), 1);
    }

    #[test]
    fn test_stisa_no_fire_on_classes_with_own_id() {
        // Probed: `char`, `cell`, `struct` and `logical` give ISCHR, ISCELL, ISSTR and ISLOG.
        assert_eq!(stisa_count("b = strcmp(class(o), 'char');\n"), 0);
        assert_eq!(stisa_count("b = strcmp('cell', class(o));\n"), 0);
        assert_eq!(stisa_count("b = strcmp(class(o), 'struct');\n"), 0);
        assert_eq!(stisa_count("b = strcmp(class(o), 'logical');\n"), 0);
    }

    #[test]
    fn test_stisa_no_fire_without_class_call() {
        assert_eq!(stisa_count("b = strcmp(o.name, 'x');\n"), 0);
        assert_eq!(stisa_count("b = strcmp(class(o));\n"), 0);
    }
}
