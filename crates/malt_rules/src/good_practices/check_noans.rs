use super::*;

impl GoodPracticesEngine {
    /// NOANS: a variable named `ans`, which MATLAB overwrites with the value of
    /// every expression statement that is not assigned.
    pub(crate) fn check_noans(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("NOANS") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            // A bare `ans` statement is a `command_name` in this grammar.
            if matches!(node.kind(), "identifier" | "command_name")
                && &source[node.byte_range()] == "ans"
                && !crate::analysis::symbols::is_field_name(node)
                // Probed: `s.ans(2)` and `obj.ans()` are fields, not the variable.
                && !source[..node.start_byte()].trim_end().ends_with('.')
                // Probed: the name of a function or method named `ans` is FCNANS or MTHANS,
                // not NOANS (`function r = ans(x)`, `function r = ans(obj)` in a classdef).
                // An output named `ans` (`function ans = f(x)`) is NOANS.
                && (node.parent().is_none_or(|p| p.kind() != "function_definition")
                    || source[node.end_byte()..].trim_start().starts_with('='))
            {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "NOANS",
                    message: "Using ANS as a variable is not recommended as ANS is frequently overwritten by MATLAB."
                        .to_string(),
                    severity: Severity::Warning,
                    byte_range: node.byte_range(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }
        diagnostics.sort_by_key(|d| d.byte_range.start);
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_noans_fires_on_bare_ans_statement() {
        // Probed inside a function: a bare `ans` statement is NOANS, like a read.
        // NOANS is a file-level check, so it runs through `check_file`.
        let source = "function f()\nans\nx = 1;\nans;\nend\n";
        let tree = crate::test_util::parse(source);
        let engine =
            crate::good_practices::GoodPracticesEngine::from_config(&malt_core::Config::default());
        let ctx = malt_core::FileContext {
            tree: &tree,
            source,
            file_path: std::path::Path::new("test.m"),
        };
        let diags: Vec<_> = engine
            .check_file(&ctx)
            .into_iter()
            .filter(|d| d.rule_id == "NOANS")
            .collect();
        assert_eq!(diags.len(), 2, "{diags:?}");
    }

    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn noans(src: &str) -> bool {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        has_id(&lint_file(&*e, src), "NOANS")
    }

    #[test]
    fn noans_fires_on_reading_or_assigning_ans() {
        assert!(noans("x = ans + 1;\n"));
        assert!(noans("ans = 3;\n"));
    }

    #[test]
    fn noans_ignores_calls_and_fields() {
        assert!(!noans("addParameter(p, 'a', 1);\n"));
        assert!(!noans("x = s.ans;\n"));
        // Probed (obj.ans()): a method call on an object is not the variable.
        assert!(!noans("function r = f(obj)\n    r = obj.ans();\nend\n"));
    }

    fn noans_lines(src: &str) -> Vec<usize> {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_file(&*e, src)
            .iter()
            .filter(|d| d.rule_id == "NOANS")
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn noans_silent_on_name_of_function_or_method_named_ans() {
        // Probed (N3, C1): `function r = ans(x)` gives FCNANS only, and a classdef method
        // `function r = ans(obj)` gives MTHANS only; neither gives NOANS.
        assert!(!noans("function r = ans(x)\n    r = x;\nend\n"));
        assert!(!noans(
            "classdef C1\n    methods\n        function r = ans(obj)\n            r = 1;\n        end\n    end\nend\n"
        ));
    }

    #[test]
    fn noans_fires_on_output_and_assignment_of_method_named_ans() {
        // Probed (C3): `function ans = ans(obj)` gives NOANS on the output (line 3) and on
        // the assignment (line 4), not on the method name.
        assert_eq!(
            noans_lines(
                "classdef C3\n    methods\n        function ans = ans(obj)\n            ans = 1;\n        end\n    end\nend\n"
            ),
            vec![3, 4]
        );
    }

    #[test]
    fn noans_fires_on_read_of_ans_in_function() {
        // Probed (N5): `r = ans;` inside a function is NOANS.
        assert!(noans("function r = f(x)\n    r = ans;\nend\n"));
    }
}
