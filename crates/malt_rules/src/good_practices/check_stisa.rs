use super::*;

impl GoodPracticesEngine {
    /// STISA: Use `isa` instead of `strcmp(class(obj), '...')`.
    pub(crate) fn check_stisa(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("STISA") || node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match get_function_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };

        if func_name != "strcmp" && func_name != "strcmpi" {
            return Vec::new();
        }

        // Check if either argument is `class(...)`.
        let args_node = match find_child_of_kind(node, "arguments") {
            Some(a) => a,
            None => return Vec::new(),
        };

        let has_class_call = args_has_function_call(args_node, source, "class");
        if !has_class_call {
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
