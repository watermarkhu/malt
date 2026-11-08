//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCCSOP: the constructor cannot modify Constant properties.
    pub(crate) fn check_mccsop(
        &self,
        class: &ClassMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("MCCSOP") {
            return;
        }
        let root = ctx.tree.root_node();
        let constructor = self.find_function_definition_by_name(root, &class.name, ctx.source);
        if let Some(node) = constructor {
            Self::walk_constant_assignments(node, class, ctx.source, true, diagnostics);
        }
    }

    /// MCSCN: any method that sets a Constant property.
    pub(crate) fn check_mcscn(
        &self,
        class: &ClassMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("MCSCN") {
            return;
        }
        let functions = self.collect_c2_function_definitions(ctx.tree.root_node(), ctx.source);
        for (node, _) in functions {
            Self::walk_constant_assignments(node, class, ctx.source, false, diagnostics);
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
        let method_name = node
            .child_by_field_name("name")
            .map(|n| node_text(n, source).to_string())
            .unwrap_or_default();
        Self::walk_constant_assignments_dfs(
            node,
            class,
            source,
            is_constructor,
            &method_name,
            diagnostics,
        );
    }

    /// DFS helper for [`Self::walk_constant_assignments`].
    pub(crate) fn walk_constant_assignments_dfs(
        node: tree_sitter::Node,
        class: &ClassMeta,
        source: &str,
        is_constructor: bool,
        method_name: &str,
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
                if is_constant {
                    let pos = node.start_position();
                    let (rule_id, message) = if is_constructor {
                        (
                            "MCCSOP",
                            format!("Unable to modify Constant property {prop_name}."),
                        )
                    } else {
                        (
                            "MCSCN",
                            format!("Method {method_name} tries to set a constant property."),
                        )
                    };
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
            Self::walk_constant_assignments_dfs(
                child,
                class,
                source,
                is_constructor,
                method_name,
                diagnostics,
            );
        }
    }

    /// Find a `function_definition` node by name in a tree.
    pub(crate) fn find_function_definition_by_name<'a>(
        &self,
        root: tree_sitter::Node<'a>,
        name: &str,
        source: &str,
    ) -> Option<tree_sitter::Node<'a>> {
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            if n.kind() == "function_definition" {
                let fname = n
                    .child_by_field_name("name")
                    .map(|f| node_text(f, source))
                    .unwrap_or("");
                if fname == name {
                    return Some(n);
                }
            }
            let mut cursor = n.walk();
            for child in n.children(&mut cursor) {
                stack.push(child);
            }
        }
        None
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
    fn test_mcscn_fires_method_sets_constant() {
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
        let hits = filter_by_id(&diags, "MCSCN");
        assert!(
            !hits.is_empty(),
            "MCSCN should fire when a method sets a Constant property"
        );
    }

    #[test]
    fn test_mcscn_no_fire() {
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
        let hits = filter_by_id(&diags, "MCSCN");
        assert!(
            hits.is_empty(),
            "MCSCN should NOT fire for a non-Constant property"
        );
    }
}
