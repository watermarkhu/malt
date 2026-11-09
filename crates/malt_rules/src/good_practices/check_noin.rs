use super::*;
use crate::analysis::metadata::AttributeMeta;

impl GoodPracticesEngine {
    /// NOIN: A non-static class method declares no input arguments, so it can
    /// never receive the object it is called on.
    ///
    /// Constructors (which create the object rather than receive it) and
    /// abstract method signatures are skipped. Methods defined in separate
    /// files of an `@ClassName` folder are not visible here and are not checked.
    pub(crate) fn check_noin(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("NOIN") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let Some(class) = &meta.class else {
            return Vec::new();
        };
        let mut diagnostics = Vec::new();

        for block in &class.methods_blocks {
            if is_static(&block.attributes) {
                continue;
            }
            for func in &block.methods {
                if func.is_constructor || func.is_abstract || !func.inputs.is_empty() {
                    continue;
                }
                diagnostics.push(Diagnostic {
                    rule_id: "NOIN",
                    message: format!(
                        "Method {} should either be a static method or have at least one input argument.",
                        func.name
                    ),
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

/// Whether a methods block's attributes make it `Static` (`Static`,
/// `Static = true`, but not `~Static` or `Static = false`).
fn is_static(attributes: &[AttributeMeta]) -> bool {
    attributes.iter().any(|a| {
        a.name == "Static" && !a.negated && a.value.as_deref().map(str::trim) != Some("false")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noin_count(source: &str) -> usize {
        let tree = parse(source);
        engine().check_noin(&tree, source).len()
    }

    #[test]
    fn test_noin_fires_on_method_without_inputs() {
        let source = "classdef A\n    methods\n        function r = f()\n            r = 1;\n        end\n    end\nend\n";
        assert_eq!(noin_count(source), 1);
    }

    #[test]
    fn test_noin_silent_when_method_takes_object() {
        let source = "classdef A\n    methods\n        function r = f(obj)\n            r = obj;\n        end\n    end\nend\n";
        assert_eq!(noin_count(source), 0);
    }

    #[test]
    fn test_noin_silent_for_static_method() {
        let source = "classdef A\n    methods (Static)\n        function r = f()\n            r = 1;\n        end\n    end\nend\n";
        assert_eq!(noin_count(source), 0);
    }

    #[test]
    fn test_noin_fires_when_static_is_false() {
        let source = "classdef A\n    methods (Static = false)\n        function r = f()\n            r = 1;\n        end\n    end\nend\n";
        assert_eq!(noin_count(source), 1);
    }

    #[test]
    fn test_noin_silent_for_constructor() {
        let source =
            "classdef A\n    methods\n        function obj = A()\n        end\n    end\nend\n";
        assert_eq!(noin_count(source), 0);
    }

    #[test]
    fn test_noin_silent_for_plain_function_file() {
        let source = "function r = f(a)\nr = a + 1;\nend\n";
        assert_eq!(noin_count(source), 0);
        let source = "function r = f()\nr = 1;\nend\n";
        assert_eq!(noin_count(source), 0);
    }
}
