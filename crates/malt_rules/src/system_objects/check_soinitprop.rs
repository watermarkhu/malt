//! SOINITPROP: DiscreteState properties must be initialized in a `resetImpl` method.

use super::*;

impl SystemObjectsEngine {
    /// Check SOINITPROP: a `DiscreteState` property (without a default value,
    /// which is reported by SODFLTVAL) in a class that defines no `resetImpl`
    /// method to initialize it.
    pub(crate) fn check_soinitprop(
        &self,
        class_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_enabled("SOINITPROP") || class_has_method(class_node, "resetImpl", source) {
            return;
        }
        for property in class_properties(class_node) {
            let Some(block) = property.parent() else {
                continue;
            };
            if block_has_attr(block, "DiscreteState", source)
                && !property_has_default(property, source)
            {
                diags.push(make_diag_named(
                    "SOINITPROP",
                    property,
                    property_name(property, source),
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
    fn soinitprop_discretestate_without_resetimpl_fires() {
        let source = system_class("    properties (DiscreteState)\n        states\n    end\n");
        let diags = lint_file(&*engine(), &source);
        let d = diags
            .iter()
            .find(|d| d.rule_id == "SOINITPROP")
            .expect("fires");
        assert!(d.message.contains("property states "), "got: {}", d.message);
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
