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
            // The object is the first input of a method, the output of a constructor.
            let is_constructor = name == class.name;
            let method = class
                .methods_blocks
                .iter()
                .flat_map(|mb| mb.methods.iter())
                .find(|m| m.byte_range.start == node.start_byte());
            let object = method.and_then(|m| {
                if is_constructor {
                    m.outputs.iter().find(|o| *o != "~").map(String::as_str)
                } else {
                    m.inputs.iter().find(|i| *i != "~").map(String::as_str)
                }
            });
            Self::walk_constant_assignments(node, class, ctx.source, object, diagnostics);
        }
    }

    /// Walk a function body for assignments to Constant properties.
    pub(crate) fn walk_constant_assignments(
        node: tree_sitter::Node,
        class: &ClassMeta,
        source: &str,
        object: Option<&str>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        Self::walk_constant_assignments_dfs(node, class, source, object, diagnostics);
    }

    /// DFS helper for [`Self::walk_constant_assignments`].
    pub(crate) fn walk_constant_assignments_dfs(
        node: tree_sitter::Node,
        class: &ClassMeta,
        source: &str,
        object: Option<&str>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "assignment" {
            if let Some(prop_name) = assignment_field_property(node, source) {
                let is_constant = class.properties_blocks.iter().any(|pb| {
                    Attributes(&pb.attributes).is_present("Constant")
                        && pb.properties.iter().any(|p| p.name == prop_name)
                });
                // Only an assignment through the object is MCCSOP. `ClassName.K = v`
                // builds a struct instead: MATLAB reports that as MCCSPS (good_practices).
                let through = node
                    .child(0)
                    .and_then(|lhs| lhs.child_by_field_name("object"))
                    .map(|o| node_text(o, source));
                if is_constant && object.is_some() && through == object {
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
            Self::walk_constant_assignments_dfs(child, class, source, object, diagnostics);
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

    #[test]
    fn mccsop_only_through_the_object_and_not_in_constructors_with_class_name() {
        // Probed: `obj.K = 5` in a method is MCCSOP; `other.K = 5` (a new struct)
        // is silent; `Foo.K = 5` in a constructor is MCCSPS, not MCCSOP.
        let head = "classdef Foo\n    properties (Constant)\n        K = 1\n    end\n";
        let method = format!("{head}    methods\n        function m(obj)\n            obj.K = 5;\n        end\n    end\nend\n");
        assert_eq!(
            filter_by_id(&check_source(&method, "Foo.m"), "MCCSOP").len(),
            1
        );
        let other = format!("{head}    methods\n        function m(obj)\n            other.K = 5;\n            disp(other)\n        end\n    end\nend\n");
        assert!(filter_by_id(&check_source(&other, "Foo.m"), "MCCSOP").is_empty());
        let constructor = format!("{head}    methods\n        function obj = Foo()\n            Foo.K = 5;\n        end\n    end\nend\n");
        assert!(filter_by_id(&check_source(&constructor, "Foo.m"), "MCCSOP").is_empty());
    }
}
