use super::*;

impl GoodPracticesEngine {
    /// DISPLAY: a `display` method of a classdef overloads the builtin.
    ///
    /// Probed: MATLAB reports the method at its name (`function display(obj)` in
    /// a classdef, column of the name). A plain function file named `display`
    /// gives no DISPLAY (gp-a__t7:1).
    pub(crate) fn check_display_override(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        if !self.is_check_enabled("DISPLAY") {
            return Vec::new();
        }
        let mut diagnostics = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if node.kind() == "function_definition" && within_classdef(node) {
                if let Some(name) = node.child_by_field_name("name") {
                    if node_text(name, source).trim() == "display" {
                        let pos = name.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "DISPLAY",
                            message: "Overloading DISPLAY is not recommended.".to_string(),
                            severity: Severity::Warning,
                            byte_range: name.byte_range(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }
        diagnostics
    }
}

/// Whether `node` lies inside a `classdef` block.
fn within_classdef(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "class_definition" {
            return true;
        }
        current = parent.parent();
    }
    false
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
    fn test_display_reported_at_the_method_name() {
        // Probed: MATLAB reports disp2.m:3:14, the column of `display`.
        let source = "classdef Foo\n    methods\n        function display(obj)\n            fprintf('x');\n        end\n    end\nend\n";
        let tree = parse(source);
        let diags = engine().check_display_override(&tree, source);
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!((diags[0].line, diags[0].column), (3, 18));
    }

    #[test]
    fn test_display_silent_on_standalone_display_function() {
        // Probed: MATLAB is silent on `function out = display(x)` in a function file.
        let source = "function display(x)\n    fprintf('x');\nend\n";
        let tree = parse(source);
        let eng = engine();

        let diags = eng.check_display_override(&tree, source);
        assert!(diags.is_empty(), "got: {diags:?}");
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
