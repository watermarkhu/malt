//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVNST: Arguments blocks in nested functions are not allowed.
    pub(crate) fn check_fvnst(&self, ctx: &FileContext, diagnostics: &mut Vec<Diagnostic>) {
        // Walk the tree to find nested function_definitions with arguments blocks
        let root = ctx.tree.root_node();
        Self::find_nested_functions_with_args(root, 0, diagnostics);
    }

    /// Recursively find nested functions that have arguments blocks.
    pub(crate) fn find_nested_functions_with_args(
        node: tree_sitter::Node,
        depth: usize,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_definition" {
                if depth > 0 {
                    // This is a nested function; check for arguments blocks
                    let mut inner_cursor = child.walk();
                    for inner in child.children(&mut inner_cursor) {
                        if inner.kind() == "arguments_statement" {
                            let pos = inner.start_position();
                            diagnostics.push(Diagnostic {
                                rule_id: "FVNST",
                                message: "Arguments blocks in nested function declarations are not supported."
                                    .to_string(),
                                severity: Severity::Error,
                                byte_range: inner.start_byte()..inner.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    }
                }
                // Recurse into the function definition at depth+1
                Self::find_nested_functions_with_args(child, depth + 1, diagnostics);
            } else if child.kind() != "methods" && child.kind() != "class_definition" {
                // Continue recursing but don't increase depth for non-function nodes
                // Skip methods blocks (class methods are not nested functions)
                Self::find_nested_functions_with_args(child, depth, diagnostics);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvnst_arguments_in_nested() {
        let source = "\
function outer()
    x = inner(1);
    function y = inner(a)
        arguments
            a double
        end
        y = a + 1;
    end
end
";
        let diags = check_source(source, "outer.m");
        let fvnst = filter_by_id(&diags, "FVNST");
        assert!(
            !fvnst.is_empty(),
            "FVNST should fire when arguments block is in nested function"
        );
    }
}
