use super::*;

impl GoodPracticesEngine {
    /// STCI: Use `strcmpi` for case-insensitive comparison.
    ///
    /// Detects patterns like `lower(s) == '...'` or `strcmp(lower(s), '...')`.
    pub(crate) fn check_stci(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("STCI") || node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match get_function_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };

        if func_name != "strcmp" {
            return Vec::new();
        }

        // Check if either argument is wrapped in lower() or upper().
        let args_node = match find_child_of_kind(node, "arguments") {
            Some(a) => a,
            None => return Vec::new(),
        };

        let has_case_conversion = args_has_case_conversion(args_node, source);
        if !has_case_conversion {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "STCI",
            message: "Use STRCMPI(str1,str2) instead of using UPPER/LOWER in a call to STRCMP."
                .to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}
