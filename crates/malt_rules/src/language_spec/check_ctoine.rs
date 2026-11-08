//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// CTOINE: a constructed object of a class used as an input to that class's
    /// constructor (e.g., `obj = Foo(Foo(1))` inside the `Foo` constructor).
    pub(crate) fn check_ctoine(
        &self,
        class: &ClassMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        Self::check_ctoine_dfs(ctx.tree.root_node(), &class.name, ctx.source, diagnostics);
    }

    /// DFS helper for [`Self::check_ctoine`].
    pub(crate) fn check_ctoine_dfs(
        node: tree_sitter::Node,
        class_name: &str,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "function_call" {
            if let Some(name) = callee_name(node, source) {
                if name == class_name {
                    for arg in argument_nodes(node) {
                        if arg.kind() == "function_call" {
                            if let Some(inner) = callee_name(arg, source) {
                                if inner == class_name {
                                    let pos = arg.start_position();
                                    diagnostics.push(Diagnostic {
                                        rule_id: "CTOINE",
                                        message: "Use of constructed object as input to constructor is not supported."
                                            .to_string(),
                                        severity: Severity::Error,
                                        byte_range: arg.start_byte()..arg.end_byte(),
                                        line: pos.row + 1,
                                        column: pos.column + 1,
                                        fix: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::check_ctoine_dfs(child, class_name, source, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_ctoine_fires_constructed_object_as_input() {
        let source = "\
classdef Foo
    methods
        function obj = Foo(x)
            obj = Foo(Foo(1));
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let ctoine = filter_by_id(&diags, "CTOINE");
        assert!(
            !ctoine.is_empty(),
            "CTOINE should fire for Foo(Foo(1)) in constructor"
        );
    }

    #[test]
    fn test_ctoine_no_fire_plain_argument() {
        let source = "\
classdef Foo
    methods
        function obj = Foo(x)
            obj = Foo(x);
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let ctoine = filter_by_id(&diags, "CTOINE");
        assert!(ctoine.is_empty(), "CTOINE should NOT fire for Foo(x)");
    }
}
