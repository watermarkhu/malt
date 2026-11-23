//! # ASGSL: Assignment to variable might be unnecessary
//!
//! `check_asgsl` flags a self-assignment `x = x;`, where the value is the same
//! plain identifier as the target. MATLAB reports it as a readability hint
//! (level 0); `a = b;` and other copies are not reported.

use super::*;

impl ReadabilityEngine {
    /// ASGSL: file-level walk over `assignment` nodes whose target and value are
    /// the same identifier.
    pub(crate) fn check_asgsl(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("ASGSL") {
            return;
        }
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
            if node.kind() != "assignment" || in_unreachable_code(node, source) {
                continue;
            }
            let (Some(left), Some(right)) = (
                node.child_by_field_name("left"),
                node.child_by_field_name("right"),
            ) else {
                continue;
            };
            if left.kind() != "identifier"
                || right.kind() != "identifier"
                || source[left.byte_range()] != source[right.byte_range()]
            {
                continue;
            }
            results.push(self.diag(
                "ASGSL",
                "Assignment to variable might be unnecessary.",
                node,
                None,
            ));
        }
    }
}
