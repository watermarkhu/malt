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
                // A debug call on a line of its own is deleted. Unsafe: it is code the
                // author wrote, even if it only works at the command line.
                fix: crate::fix_util::delete_statement_line(node, source)
                    .map(malt_core::Fix::unsafe_fix),
            }]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{fix_of, has_id, reported_without_fix};

    #[test]
    fn debugfun_fix_deletes_the_statement_line() {
        let (safe, fixed) = fix_of(&*engine(), "x = 1;\ndbstep();\ny = 2;\n", "DEBUGFUN").unwrap();
        assert!(!safe);
        assert_eq!(fixed, "x = 1;\ny = 2;\n");
        assert!(!has_id(&node_diags(&fixed), "DEBUGFUN"));
    }

    #[test]
    fn debugfun_no_fix_when_not_alone_on_its_line() {
        for src in [
            "x = 1; dbstep();\n",
            "dbup(); x = 1;\n",
            "if a, dbstep(), end\n",
        ] {
            assert!(reported_without_fix(&*engine(), src, "DEBUGFUN"), "{src}");
        }
    }

    // -- DEBUGFUN ------------------------------------------------------------

    #[test]
    fn debugfun_fires_on_dbstep_call() {
        let src = "dbstep();\n";
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
