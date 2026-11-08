//! # PSIZE: `prod(size(x))` → `numel(x)`
//!
//! `check_prod_size` replaces `prod(size(x))` with the clearer `numel(x)`.

use super::*;

impl ReadabilityEngine {
    /// PSIZE: `prod(size(x))` → `numel(x)`
    pub(crate) fn check_prod_size(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("PSIZE") {
            return;
        }

        let args = match find_arguments(node) {
            Some(a) => a,
            None => return,
        };

        let named_children: Vec<_> = (0..args.named_child_count())
            .filter_map(|i| args.named_child(i))
            .collect();

        if named_children.len() != 1 {
            return;
        }

        let inner = named_children[0];
        if inner.kind() == "function_call" {
            if let Some(inner_name) = inner.child_by_field_name("name") {
                let inner_func = &source[inner_name.start_byte()..inner_name.end_byte()];
                if inner_func == "size" {
                    // Extract the variable passed to size
                    if let Some(inner_args) = find_arguments(inner) {
                        let inner_named: Vec<_> = (0..inner_args.named_child_count())
                            .filter_map(|i| inner_args.named_child(i))
                            .collect();
                        if inner_named.len() == 1 {
                            let var_text =
                                &source[inner_named[0].start_byte()..inner_named[0].end_byte()];
                            results.push(self.diag(
                                "PSIZE",
                                "NUMEL(x) is usually faster than PROD(SIZE(x)).",
                                node,
                                Some(Fix::new(
                                    node.start_byte()..node.end_byte(),
                                    format!("numel({var_text})"),
                                )),
                            ));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    // -- PSIZE ---------------------------------------------------------------

    #[test]
    fn psize_prod_size_fires() {
        let diags = lint_nodes(&*engine(), "x = prod(size(a));\n");
        assert!(has_id(&diags, "PSIZE"), "got: {diags:?}");
    }

    #[test]
    fn psize_prod_without_size_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = prod(a);\n");
        assert!(!has_id(&diags, "PSIZE"), "got: {diags:?}");
    }
}
