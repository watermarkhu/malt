//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCCSOP: a function cannot modify Constant properties.
    pub(crate) fn check_mccsop(
        &self,
        class: &ClassMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("MCCSOP") {
            return;
        }
        // MATLAB reports the assignment in the constructor and in any other method alike.
        let functions = self.collect_c2_function_definitions(ctx.tree.root_node(), ctx.source);
        for (node, name) in functions {
            Self::walk_constant_assignments(
                node,
                class,
                ctx.source,
                name == class.name,
                diagnostics,
            );
        }
    }

    /// Walk a function body for assignments to Constant properties.
    pub(crate) fn walk_constant_assignments(
        node: tree_sitter::Node,
        class: &ClassMeta,
        source: &str,
        is_constructor: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        Self::walk_constant_assignments_dfs(node, class, source, is_constructor, diagnostics);
    }

    /// DFS helper for [`Self::walk_constant_assignments`].
    pub(crate) fn walk_constant_assignments_dfs(
        node: tree_sitter::Node,
        class: &ClassMeta,
        source: &str,
        is_constructor: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "assignment" {
            if let Some(prop_name) = assignment_field_property(node, source) {
                let is_constant = class.properties_blocks.iter().any(|pb| {
                    pb.attributes
                        .iter()
                        .any(|a| a.name == "Constant" && !a.negated)
                        && pb.properties.iter().any(|p| p.name == prop_name)
                });
                // `ClassName.K = v` in a method builds a struct; MATLAB reports that as MCCSPS.
                let class_qualified = node.child(0).is_some_and(|lhs| {
                    node_text(lhs, source).starts_with(&format!("{}.", class.name))
                });
                if is_constant && (!class_qualified || is_constructor) {
                    let pos = node.start_position();
                    let (rule_id, message) = (
                        "MCCSOP",
                        format!("Unable to modify Constant property '{prop_name}'."),
                    );
                    diagnostics.push(Diagnostic {
                        rule_id,
                        message,
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_constant_assignments_dfs(child, class, source, is_constructor, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mccsop_fires_constructor_modifies_constant() {
        let source = "\
classdef Foo
    properties (Constant)
        x = 5
    end
    methods
        function obj = Foo()
            obj.x = 6;
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCCSOP");
        assert!(
            !hits.is_empty(),
            "MCCSOP should fire when the constructor modifies a Constant property"
        );
    }

    #[test]
    fn test_mccsop_no_fire() {
        let source = "\
classdef Foo
    properties
        x = 5
    end
    methods
        function obj = Foo()
            obj.x = 6;
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCCSOP");
        assert!(
            hits.is_empty(),
            "MCCSOP should NOT fire for a non-Constant property"
        );
    }

    #[test]
    fn test_mccsop_fires_method_sets_constant() {
        let source = "\
classdef Foo
    properties (Constant)
        x = 5
    end
    methods
        function obj = Foo()
        end
        function f(obj)
            obj.x = 7;
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCCSOP");
        assert!(
            !hits.is_empty(),
            "MCCSOP should fire when a method sets a Constant property"
        );
    }

    #[test]
    fn test_mccsop_no_fire_non_constant() {
        let source = "\
classdef Foo
    properties
        x = 5
    end
    methods
        function obj = Foo()
        end
        function f(obj)
            obj.x = 7;
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCCSOP");
        assert!(
            hits.is_empty(),
            "MCCSOP should NOT fire for a non-Constant property"
        );
    }

    #[test]
    fn test_mccsop_no_fire_class_qualified_assignment() {
        let source = "classdef Foo\n    properties (Constant)\n        K = 1\n    end\n    methods (Static)\n        function s()\n            Foo.K = 3;\n        end\n    end\nend\n";
        let diags = crate::language_spec::tests::check_source(source, "Foo.m");
        assert!(crate::language_spec::tests::filter_by_id(&diags, "MCCSOP").is_empty());
        assert!(crate::language_spec::tests::filter_by_id(&diags, "MCSCN").is_empty());
    }
}
