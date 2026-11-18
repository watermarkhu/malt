//! SODFLTVAL: DiscreteState properties must not be initialized with a default value.

use super::*;

impl SystemObjectsEngine {
    /// Check SODFLTVAL: a property in a `properties (DiscreteState)` block
    /// that carries a default value (`Name = ...`). DiscreteState properties
    /// must be initialized within `resetImpl` instead.
    pub(crate) fn check_sodfltval(
        &self,
        properties_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_enabled("SODFLTVAL")
            || !block_has_attr(properties_node, "DiscreteState", source)
        {
            return;
        }
        let mut cursor = properties_node.walk();
        for child in properties_node.children(&mut cursor) {
            if child.kind() == "property" && property_has_default(child, source) {
                diags.push(make_diag_named(
                    "SODFLTVAL",
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
    fn sodfltval_discretestate_default_fires_with_name() {
        let source = system_class("    properties (DiscreteState)\n        Count = 0;\n    end");
        let diags = lint_file(&*engine(), &source);
        let d = diags
            .iter()
            .find(|d| d.rule_id == "SODFLTVAL")
            .expect("fires");
        assert!(d.message.contains("property Count."), "got: {}", d.message);
    }

    #[test]
    fn sodfltval_regular_function_call_default_does_not_fire() {
        let source = system_class("    properties\n        Gain = rand();\n    end");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SODFLTVAL"), "got: {diags:?}");
    }

    #[test]
    fn sodfltval_discretestate_without_default_does_not_fire() {
        let source = system_class("    properties (DiscreteState)\n        Count\n    end");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SODFLTVAL"), "got: {diags:?}");
    }
}
