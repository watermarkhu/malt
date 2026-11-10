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
    }
}
