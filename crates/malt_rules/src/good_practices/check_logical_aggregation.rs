use super::*;

impl GoodPracticesEngine {
    /// LOGPROD / LOGMIN / LOGMAX: Using `prod`/`min`/`max` on logical values.
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

        // Heuristic: check if the argument is likely logical (named with `is`, `has`,
        // or is a comparison result). We can't know types statically, so this is
        // limited to obvious patterns.
        let args_node = find_child_of_kind(node, "arguments");
        let first_arg = args_node.and_then(|a| first_named_child(a));
        let looks_logical = first_arg
            .map(|arg| {
                let kind = arg.kind();
                if kind == "comparison_operator" || kind == "boolean_operator" {
                    return true;
                }
                if kind == "identifier" {
                    let name = node_text(arg, source);
                    return name.starts_with("is")
                        || name.starts_with("has")
                        || name == "true"
                        || name == "false";
                }
                if kind == "unary_operator" {
                    let text = node_text(arg, source);
                    return text.starts_with('~');
                }
                false
            })
            .unwrap_or(false);

        if !looks_logical {
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
