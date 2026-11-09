//! SODEPPROP: Dependent properties are not supported on MATLAB System blocks.

use super::*;

impl SystemObjectsEngine {
    /// Check SODEPPROP: every property declared in a `properties (Dependent)`
    /// block of a `matlab.System` class.
    pub(crate) fn check_sodeprop(
        &self,
        properties_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_enabled("SODEPPROP") || !block_has_attr(properties_node, "Dependent", source) {
            return;
        }
        let mut cursor = properties_node.walk();
        for child in properties_node.children(&mut cursor) {
            if child.kind() == "property" {
                diags.push(make_diag_named(
                    "SODEPPROP",
                    child,
                    property_name(child, source),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{engine, system_class};
    use crate::test_util::{has_id, lint_file};

    #[test]
    fn sodeprop_dependent_property_fires_with_name() {
        let source = system_class("    properties (Dependent)\n        Total\n    end\n");
        let diags = lint_file(&*engine(), &source);
        let d = diags
            .iter()
            .find(|d| d.rule_id == "SODEPPROP")
            .expect("fires");
        assert!(d.message.contains("Total property"), "got: {}", d.message);
        assert!(!d.message.contains("VAR_NAME"));
    }

    #[test]
    fn sodeprop_plain_property_does_not_fire() {
        let source = system_class("    properties\n        Total = 1;\n    end\n");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SODEPPROP"), "got: {diags:?}");
    }
}
