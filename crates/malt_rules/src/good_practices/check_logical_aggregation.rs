use super::*;

impl GoodPracticesEngine {
    /// LOGPROD / LOGMIN / LOGMAX: Using `prod`/`min`/`max` on a logical expression.
    ///
    /// Probed: MATLAB reports these only when the single argument is a
    /// comparison (`max(a > b)`, `max(a.b > 2)`, `prod(x == y)`). It does not
    /// report a variable that holds a logical value (`isx`, `island`), a unary
    /// `~x`, a `&` or `|` expression, a parenthesised comparison, or a call
    /// with more than one argument (`max(a > b, [], 2)`).
    pub(crate) fn check_logical_aggregation(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match get_function_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };

        let (check_id, replacement) = match func_name {
            "prod" if self.is_check_enabled("LOGPROD") => ("LOGPROD", "all"),
            "min" if self.is_check_enabled("LOGMIN") => ("LOGMIN", "all"),
            "max" if self.is_check_enabled("LOGMAX") => ("LOGMAX", "any"),
            _ => return Vec::new(),
        };

        let Some(args_node) = find_child_of_kind(node, "arguments") else {
            return Vec::new();
        };
        if count_named_children(args_node) != 1 {
            return Vec::new();
        }
        // The grammar nests `a > b & a < 3` as a comparison over a binary operator,
        // so a `&` or `|` token anywhere in the argument rules the comparison out.
        let Some(arg) = first_named_child(args_node) else {
            return Vec::new();
        };
        if arg.kind() != "comparison_operator" || has_logical_connective(arg, source) {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: check_id,
            message: format!(
                "Using '{}' on a logical expression is hard to understand and might be incorrect. Consider using '{}' instead.",
                func_name, replacement
            ),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

/// Whether `node` or a descendant is a `&`, `|`, `&&` or `||` token.
fn has_logical_connective(node: Node, source: &str) -> bool {
    if node.child_count() == 0 && matches!(node_text(node, source), "&" | "|" | "&&" | "||") {
        return true;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if has_logical_connective(child, source) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run the check on the first function call in `source`.
    fn count(source: &str) -> Vec<&'static str> {
        let tree = parse(source);
        let call = find_descendant_of_kind(tree.root_node(), "function_call").unwrap();
        engine()
            .check_logical_aggregation(call, source)
            .iter()
            .map(|d| d.rule_id)
            .collect()
    }

    #[test]
    fn test_logmax_fires_on_comparison_argument() {
        // Probed: `max(a > b)`, `max(a == b)` and `max(a.b > 2)` are reported.
        assert_eq!(count("m = max(a > b);\n"), vec!["LOGMAX"]);
        assert_eq!(count("m = max(a == b);\n"), vec!["LOGMAX"]);
        assert_eq!(count("m = max(a.b > 2);\n"), vec!["LOGMAX"]);
    }

    #[test]
    fn test_logmin_and_logprod_fire_on_comparison_argument() {
        // Probed: `min(x > 2)` gives LOGMIN, `prod(x > 2)` gives LOGPROD.
        assert_eq!(count("m = min(x > 2);\n"), vec!["LOGMIN"]);
        assert_eq!(count("m = prod(x > 2);\n"), vec!["LOGPROD"]);
    }

    #[test]
    fn test_logmax_silent_on_logical_variable() {
        // Probed (fix/gp-a__t10.m): `max(isx)`, `max(island)`, `max(has_nan)` are not reported.
        assert!(count("m = max(isx);\n").is_empty());
        assert!(count("m = max(island);\n").is_empty());
    }

    #[test]
    fn test_logmax_silent_on_unary_not_and_boolean_operator() {
        // Probed: `max(~x)`, `prod(~x)`, `max(a & b)` and `max(a > b & a < 3)` are not reported.
        assert!(count("m = max(~x);\n").is_empty());
        assert!(count("m = prod(~x);\n").is_empty());
        assert!(count("m = max(a & b);\n").is_empty());
        assert!(count("m = max(a > b & a < 3);\n").is_empty());
    }

    #[test]
    fn test_logmax_silent_on_parenthesised_or_several_arguments() {
        // Probed: `max((a > b))` and `max(a > b, [], 2)` are not reported.
        assert!(count("m = max((a > b));\n").is_empty());
        assert!(count("m = max(a > b, [], 2);\n").is_empty());
    }
}
