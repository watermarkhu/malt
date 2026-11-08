use super::*;

impl BugsEngine {
    /// DEBUGFUN: Debug function call in production code.
    pub(crate) fn check_debug_function(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match extract_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };

        let debug_fns = self.debug_functions();
        if debug_fns.contains(&func_name) {
            let pos = node.start_position();
            vec![Diagnostic {
                rule_id: "DEBUGFUN",
                message: "Debug functions are intended to be used at the command line. At runtime, they will generate an error. Remove the debug function.".to_string(),
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- DEBUGFUN ------------------------------------------------------------

    #[test]
    fn debugfun_fires_on_keyboard_call() {
        let src = "keyboard();\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "DEBUGFUN"), "got: {diags:?}");
    }

    #[test]
    fn debugfun_fires_on_dbcont_call() {
        let src = "dbcont();\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "DEBUGFUN"), "got: {diags:?}");
    }

    #[test]
    fn debugfun_no_fire_on_regular_call() {
        let src = "disp('hello');\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "DEBUGFUN"), "got: {diags:?}");
    }
}
