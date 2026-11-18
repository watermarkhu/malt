//! SORSRVDNM: a property of a System object uses a reserved name.

use super::*;

impl SystemObjectsEngine {
    /// Check SORSRVDNM: a property whose name collides with a name reserved by
    /// `matlab.System` (e.g. `step`, `reset`, `release`).
    pub(crate) fn check_sorsrvdnm(
        &self,
        properties_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_enabled("SORSRVDNM") {
            return;
        }
        let mut cursor = properties_node.walk();
        for child in properties_node.children(&mut cursor) {
            if child.kind() != "property" {
                continue;
            }
            let name = property_name(child, source);
            if RESERVED_NAMES.contains(&name) {
                diags.push(make_diag_named("SORSRVDNM", child, name));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{engine, system_class};
    use crate::test_util::{has_id, lint_file};

    #[test]
    fn sorsrvdnm_reserved_property_name_fires() {
        let source = system_class("    properties\n        reset = 1;\n    end");
        let diags = lint_file(&*engine(), &source);
        let d = diags
            .iter()
            .find(|d| d.rule_id == "SORSRVDNM")
            .expect("fires");
        assert_eq!(d.message, "reset property is a reserved name.");
    }

    #[test]
    fn sorsrvdnm_method_with_reserved_name_does_not_fire() {
        let source = system_class("    methods\n        function step(obj)\n        end\n    end");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SORSRVDNM"), "got: {diags:?}");
    }

    #[test]
    fn sorsrvdnm_ordinary_property_does_not_fire() {
        let source = system_class("    properties\n        Gain = 1;\n    end");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SORSRVDNM"), "got: {diags:?}");
    }

    #[test]
    fn sorsrvdnm_non_system_class_does_not_fire() {
        let source = "classdef MyRegular\n    properties\n        reset = 1;\n    end\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(!has_id(&diags, "SORSRVDNM"), "got: {diags:?}");
    }
}
