//! SODFLTVAL: System object property default value issues.

use super::*;

impl SystemObjectsEngine {
    /// Check property definitions for default value issues.
    pub(crate) fn check_sodfltval(
        &self,
        properties_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        let count = properties_node.child_count();
        for i in 0..count {
            if let Some(child) = properties_node.child(i) {
                if child.kind() == "property" {
                    // Check for function call as default value (potentially problematic)
                    let text = node_text(child, source);
                    if text.contains('(') && text.contains(')') && text.contains('=') {
                        // Heuristic: property with function call default value
                        // e.g., `Prop = someFunction()`
                        diags.push(make_diag_named(
                            "SODFLTVAL",
                            child,
                            property_name(child, source),
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{engine, system_class};
    use crate::test_util::{has_id, lint_file};

    #[test]
    fn sodfltval_function_call_default_fires() {
        let source = system_class("    properties\n        Gain = rand();\n    end");
        let diags = lint_file(&*engine(), &source);
        assert!(has_id(&diags, "SODFLTVAL"), "got: {diags:?}");
    }

    #[test]
    fn sodfltval_scalar_default_does_not_fire() {
        let source = system_class("    properties\n        Gain = 5;\n    end");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SODFLTVAL"), "got: {diags:?}");
    }

    #[test]
    fn sodfltval_string_default_does_not_fire() {
        let source = system_class("    properties\n        Name = 'rx';\n    end");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SODFLTVAL"), "got: {diags:?}");
    }
}
