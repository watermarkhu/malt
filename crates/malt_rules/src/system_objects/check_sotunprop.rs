//! SOTUNPROP1/3/4: Tunable property constraints on MATLAB System blocks.
//!
//! - **SOTUNPROP1** — a logical attribute is not supported for tunable
//!   properties on MATLAB System blocks (the property is made Nontunable).
//! - **SOTUNPROP3** — tunable properties on System blocks must be numeric; a
//!   `char` (character-vector) property is made Nontunable.
//! - **SOTUNPROP4** — tunable properties on System blocks must be numeric; a
//!   `string` property is made Nontunable.

use super::*;

impl SystemObjectsEngine {
    /// Check SOTUNPROP1/3/4: a tunable property (no Nontunable attribute, and
    /// not Constant) whose declared type or default is logical / char / string.
    pub(crate) fn check_sotunprop(
        &self,
        properties_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_enabled("SOTUNPROP1")
            && !self.is_enabled("SOTUNPROP3")
            && !self.is_enabled("SOTUNPROP4")
        {
            return;
        }
        // Skip blocks that opt out of tunability.
        if block_text_has(properties_node, "Nontunable", source) {
            return;
        }
        let count = properties_node.child_count();
        for i in 0..count {
            if let Some(child) = properties_node.child(i) {
                if child.kind() != "property" {
                    continue;
                }
                let text = node_text(child, source);
                // Skip Constant / Nontunable / DiscreteState properties.
                if text.contains("Constant") || text.contains("Nontunable") {
                    continue;
                }
                if text.contains("logical") {
                    diags.push(make_diag_named(
                        "SOTUNPROP1",
                        child,
                        property_name(child, source),
                    ));
                } else if text.contains("char") && !text.contains("string") {
                    diags.push(make_diag_named(
                        "SOTUNPROP3",
                        child,
                        property_name(child, source),
                    ));
                } else if text.contains("string") {
                    diags.push(make_diag_named(
                        "SOTUNPROP4",
                        child,
                        property_name(child, source),
                    ));
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
    fn sotunprop1_logical_tunable_fires() {
        let source = system_class("    properties\n        Flag logical = false\n    end\n");
        let diags = lint_file(&*engine(), &source);
        assert!(has_id(&diags, "SOTUNPROP1"), "got: {diags:?}");
    }

    #[test]
    fn sotunprop3_char_tunable_fires() {
        let source = system_class("    properties\n        Label char = 'x'\n    end\n");
        let diags = lint_file(&*engine(), &source);
        assert!(has_id(&diags, "SOTUNPROP3"), "got: {diags:?}");
    }

    #[test]
    fn sotunprop4_string_tunable_fires() {
        let source = system_class("    properties\n        Name string = \"x\"\n    end\n");
        let diags = lint_file(&*engine(), &source);
        assert!(has_id(&diags, "SOTUNPROP4"), "got: {diags:?}");
    }

    #[test]
    fn sotunprop_numeric_tunable_does_not_fire() {
        let source = system_class("    properties\n        Gain double = 5\n    end\n");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SOTUNPROP1"), "got: {diags:?}");
        assert!(!has_id(&diags, "SOTUNPROP3"), "got: {diags:?}");
        assert!(!has_id(&diags, "SOTUNPROP4"), "got: {diags:?}");
    }

    #[test]
    fn sotunprop_nontunable_block_does_not_fire() {
        let source =
            system_class("    properties (Nontunable)\n        Label char = 'x'\n    end\n");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SOTUNPROP3"), "got: {diags:?}");
    }
}
