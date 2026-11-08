//! # ISROW / ISCOL: `size(x, dim) == 1` → `isrow(x)` / `iscolumn(x)`
//!
//! `check_size_comparison` rewrites `size(x, 1) == 1` to `isrow(x)` and
//! `size(x, 2) == 1` to `iscolumn(x)`.

use super::*;

impl ReadabilityEngine {
    /// ISROW / ISCOL: `size(x, dim) == 1` patterns
    pub(crate) fn check_size_comparison(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        // We need at least 3 children: lhs, operator, rhs
        if node.named_child_count() < 2 {
            return;
        }

        // Get the comparison operator text
        let node_text = &source[node.start_byte()..node.end_byte()];
        if !node_text.contains("==") {
            return;
        }

        // Find a function_call to `size` in the children
        let mut size_call = None;
        let mut other_side = None;

        for i in 0..node.named_child_count() {
            if let Some(child) = node.named_child(i) {
                if child.kind() == "function_call" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = &source[name_node.start_byte()..name_node.end_byte()];
                        if name == "size" {
                            size_call = Some(child);
                            continue;
                        }
                    }
                }
                other_side = Some(child);
            }
        }

        let size_node = match size_call {
            Some(n) => n,
            None => return,
        };

        // Check the "other side" is `1`
        let other = match other_side {
            Some(n) => n,
            None => return,
        };
        let other_text = &source[other.start_byte()..other.end_byte()].trim();
        if *other_text != "1" {
            return;
        }

        // Extract the dimension argument from size(x, dim)
        let args = match find_arguments(size_node) {
            Some(a) => a,
            None => return,
        };

        let named_children: Vec<_> = (0..args.named_child_count())
            .filter_map(|i| args.named_child(i))
            .collect();

        if named_children.len() != 2 {
            return;
        }

        let var_text = &source[named_children[0].start_byte()..named_children[0].end_byte()];
        let dim_text = &source[named_children[1].start_byte()..named_children[1].end_byte()].trim();

        match *dim_text {
            "1" if self.is_check_enabled("ISROW") => {
                results.push(self.diag(
                    "ISROW",
                    "When checking if a variable is a row vector consider using ISROW.",
                    node,
                    Some(Fix::new(
                        node.start_byte()..node.end_byte(),
                        format!("isrow({var_text})"),
                    )),
                ));
            }
            "2" if self.is_check_enabled("ISCOL") => {
                results.push(self.diag(
                    "ISCOL",
                    "When checking if a variable is a column vector consider using ISCOLUMN.",
                    node,
                    Some(Fix::new(
                        node.start_byte()..node.end_byte(),
                        format!("iscolumn({var_text})"),
                    )),
                ));
            }
            _ => {}
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

    // -- ISROW ---------------------------------------------------------------

    #[test]
    fn isrow_size_dim1_equals_one_fires() {
        let diags = lint_nodes(&*engine(), "x = size(a, 1) == 1;\n");
        assert!(has_id(&diags, "ISROW"), "got: {diags:?}");
    }

    #[test]
    fn isrow_size_dim1_equals_two_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = size(a, 1) == 2;\n");
        assert!(!has_id(&diags, "ISROW"), "got: {diags:?}");
    }

    // -- ISCOL ---------------------------------------------------------------

    #[test]
    fn iscol_size_dim2_equals_one_fires() {
        let diags = lint_nodes(&*engine(), "x = size(a, 2) == 1;\n");
        assert!(has_id(&diags, "ISCOL"), "got: {diags:?}");
    }

    #[test]
    fn iscol_size_dim2_equals_zero_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = size(a, 2) == 0;\n");
        assert!(!has_id(&diags, "ISCOL"), "got: {diags:?}");
    }
}
