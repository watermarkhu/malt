use super::*;

impl GoodPracticesEngine {
    /// DISPLAY: a function or method named `display` overloads the builtin.
    ///
    /// # Limitations (heuristic)
    ///
    /// This flags any function/method named `display` on the name match alone.
    /// Overloading `display` for a custom class is a common, legitimate pattern
    /// (the builtin `display` is dispatched on user objects by design); Code
    /// Analyzer's actual check is more nuanced. No static analysis verifies
    /// that the definition actually shadows the builtin for a fundamental type.
    pub(crate) fn check_display_override(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        if !self.is_check_enabled("DISPLAY") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let mut diagnostics = Vec::new();

        for func in meta.functions.iter().chain(meta.local_functions.iter()) {
            if func.name == "display" {
                diagnostics.push(Diagnostic {
                    rule_id: "DISPLAY",
                    message: "Overloading DISPLAY is not recommended.".to_string(),
                    severity: Severity::Warning,
                    byte_range: func.byte_range.clone(),
                    line: func.line,
                    column: 1,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_fires_on_display_method() {
        let source = "classdef Foo\n    methods\n        function display(obj)\n            fprintf('x');\n        end\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();

        let ids: Vec<&str> = eng
            .check_display_override(&tree, source)
            .iter()
            .map(|d| d.rule_id)
            .collect();
        assert!(ids.contains(&"DISPLAY"), "got: {ids:?}");
    }

    #[test]
    fn test_display_fires_on_standalone_display_function() {
        let source = "function display(x)\n    fprintf('x');\nend\n";
        let tree = parse(source);
        let eng = engine();

        let ids: Vec<&str> = eng
            .check_display_override(&tree, source)
            .iter()
            .map(|d| d.rule_id)
            .collect();
        assert!(ids.contains(&"DISPLAY"), "got: {ids:?}");
    }

    #[test]
    fn test_display_silent_on_disp_method() {
        let source = "classdef Foo\n    methods\n        function disp(obj)\n            fprintf('x');\n        end\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();

        let diags = eng.check_display_override(&tree, source);
        assert!(diags.is_empty(), "got: {diags:?}");
    }
}
