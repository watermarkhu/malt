//! SOINITPROP: DiscreteState properties must be initialized in a `resetImpl` method.

use super::*;

impl SystemObjectsEngine {
    /// Check SOINITPROP: a `DiscreteState` property is not initialized inside a
    /// `resetImpl` method. The default value of a DiscreteState property is
    /// reset by `resetImpl`; if the class lacks that method the property may
    /// keep stale state across calls.
    pub(crate) fn check_soinitprop(
        &self,
        class_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_enabled("SOINITPROP") {
            return;
        }
        // A resetImpl method cancels the check.
        if class_has_method(class_node, "resetImpl", source) {
            return;
        }
        for property in class_properties(class_node) {
            let text = node_text(property, source);
            // DiscreteState properties are declared with the (DiscreteState)
            // attribute on the properties block, or named with a type. Detect
            // the attribute form: `properties (DiscreteState)`.
            if let Some(block) = property.parent() {
                if block.kind() == "properties" && block_text_has(block, "DiscreteState", source) {
                    diags.push(make_diag_named(
                        "SOINITPROP",
                        property,
                        property_name(property, source),
                    ));
                }
            }
            let _ = text;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{engine, system_class};
    use crate::test_util::{has_id, lint_file};

    #[test]
    fn soinitprop_discretestate_without_resetimpl_fires() {
        let source = system_class("    properties (DiscreteState)\n        states\n    end\n");
        let diags = lint_file(&*engine(), &source);
        assert!(has_id(&diags, "SOINITPROP"), "got: {diags:?}");
    }

    #[test]
    fn soinitprop_with_resetimpl_does_not_fire() {
        let source = system_class(
            "    properties (DiscreteState)\n        states\n    end\n    methods\n        function resetImpl(obj)\n            obj.states = 0;\n        end\n    end\n",
        );
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SOINITPROP"), "got: {diags:?}");
    }

    #[test]
    fn soinitprop_plain_properties_do_not_fire() {
        let source = system_class("    properties\n        Gain = 5;\n    end\n");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SOINITPROP"), "got: {diags:?}");
    }
}
