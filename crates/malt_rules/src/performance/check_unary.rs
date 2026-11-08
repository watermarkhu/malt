//! Node-level check for `unary_operator` nodes: STCCS (`~isempty(strfind(...))`).

use super::*;

impl PerformanceEngine {
    /// Check `unary_operator` for STCCS pattern: ~isempty(strfind(...))
    pub(crate) fn check_unary<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        if !self.is_enabled("STCCS") {
            return diags;
        }

        // Pattern: ~isempty(strfind(...))
        let op = extract_unary_operator(node, source);
        if op != "~" {
            return diags;
        }

        // The operand should be a function_call to isempty
        let operand = match node
            .child(1)
            .or_else(|| node.child_by_field_name("operand"))
        {
            Some(n) => n,
            None => return diags,
        };

        if operand.kind() == "function_call" {
            if let Some(name) = extract_func_name(operand, source) {
                if name == "isempty" {
                    // Check if the argument to isempty is strfind(...)
                    if let Some(args_node) = find_arguments(operand) {
                        if let Some(first_arg) = first_named_child(args_node) {
                            if first_arg.kind() == "function_call" {
                                if let Some(inner_name) = extract_func_name(first_arg, source) {
                                    if inner_name == "strfind" {
                                        diags.push(make_diag("STCCS", node));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        diags
    }
}
