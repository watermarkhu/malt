use super::*;

impl GoodPracticesEngine {
    /// MTHANS: `ans` is frequently overwritten by MATLAB, so it should not be
    /// used as a method name.
    pub(crate) fn check_mthans(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MTHANS") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let mut diagnostics = Vec::new();

        for func in meta.all_methods() {
            if func.name == "ans" {
                diagnostics.push(Diagnostic {
                    rule_id: "MTHANS",
                    message: "Using ANS as a method name is not recommended as ANS is frequently overwritten by MATLAB.".to_string(),
                    severity: Severity::Info,
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
    fn test_mthans_fires_on_method_named_ans() {
        let source = "classdef Foo\n    methods\n        function ans = ans(obj)\n            ans = 1;\n        end\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_mthans(&tree, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MTHANS");
    }

    #[test]
    fn test_mthans_silent_on_regular_method() {
        let source = "classdef Foo\n    methods\n        function z = calc(obj)\n            z = 1;\n        end\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_mthans(&tree, source).is_empty());
    }
}
