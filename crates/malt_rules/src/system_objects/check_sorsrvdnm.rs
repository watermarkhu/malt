//! SORSRVDNM: Reserved names used for System object methods.

use super::*;

impl SystemObjectsEngine {
    /// Check method names against reserved System object names.
    pub(crate) fn check_sorsrvdnm(
        &self,
        methods_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        let count = methods_node.child_count();
        for i in 0..count {
            if let Some(child) = methods_node.child(i) {
                if child.kind() == "function_definition" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let method_name = node_text(name_node, source);
                        // Only flag if the method name is reserved but not an expected
                        // override (setupImpl, stepImpl, etc. are expected)
                        if RESERVED_NAMES.contains(&method_name) && !method_name.ends_with("Impl") {
                            diags.push(make_diag_named("SORSRVDNM", name_node, method_name));
                        }
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
    fn sorsrvdnm_reserved_method_name_fires() {
        let source = system_class("    methods\n        function step(obj)\n        end\n    end");
        let diags = lint_file(&*engine(), &source);
        assert!(has_id(&diags, "SORSRVDNM"), "got: {diags:?}");
    }

    #[test]
    fn sorsrvdnm_impl_suffixed_method_does_not_fire() {
        let source =
            system_class("    methods\n        function stepImpl(obj)\n        end\n    end");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SORSRVDNM"), "got: {diags:?}");
    }

    #[test]
    fn sorsrvdnm_non_reserved_method_does_not_fire() {
        let source =
            system_class("    methods\n        function doWork(obj)\n        end\n    end");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SORSRVDNM"), "got: {diags:?}");
    }

    #[test]
    fn sorsrvdnm_non_system_class_does_not_fire() {
        let source = "classdef MyRegular\n    methods\n        function step(obj)\n        end\n    end\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(!has_id(&diags, "SORSRVDNM"), "got: {diags:?}");
    }
}
