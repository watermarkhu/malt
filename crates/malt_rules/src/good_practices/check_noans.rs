use super::*;

impl GoodPracticesEngine {
    /// NOANS: Statement result assigned to `ans` implicitly.
    ///
    /// Fires when a `function_call` at statement level has no assignment target
    /// and is not in the ignore list (disp, fprintf, etc.).
    pub(crate) fn check_noans(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("NOANS") || node.kind() != "function_call" {
            return Vec::new();
        }

        if !is_statement_level(node) {
            return Vec::new();
        }

        // If it is an assignment target, skip (the parent is `assignment`).
        if let Some(parent) = node.parent() {
            if parent.kind() == "assignment" {
                return Vec::new();
            }
        }

        // Suppress for known void-return functions.
        let func_name = get_function_call_name(node, source).unwrap_or("");
        let void_functions = [
            "disp", "fprintf", "sprintf", "warning", "error", "close", "clear", "clc", "clf",
            "delete", "mkdir", "rmdir", "cd", "addpath", "rmpath", "save", "fclose", "fopen",
            "pause", "drawnow", "figure", "set", "plot", "hold", "xlabel", "ylabel", "title",
            "legend", "grid", "axis", "subplot",
        ];
        if void_functions.contains(&func_name) {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "NOANS",
            message: "Using ANS as a variable is not recommended as ANS is frequently overwritten by MATLAB."
                .to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}
