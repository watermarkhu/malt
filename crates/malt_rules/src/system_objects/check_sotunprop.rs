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
    /// Check SOTUNPROP1/3/4: a tunable property (its block is not
    /// Nontunable / Constant / Dependent / DiscreteState) whose declared type
    /// or default literal is logical / char / string.
    pub(crate) fn check_sotunprop(
        &self,
        properties_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        if ["Nontunable", "Constant", "Dependent", "DiscreteState"]
            .iter()
            .any(|a| block_has_attr(properties_node, a, source))
        {
            return;
        }
        let mut cursor = properties_node.walk();
        for child in properties_node.children(&mut cursor) {
            if child.kind() != "property" {
                continue;
            }
            let text = node_text(child, source);
            let (decl, default) = match text.split_once('=') {
                Some((d, v)) => (d, v.trim()),
                None => (text, ""),
            };
            // Declared type: second token of the declaration, else infer it
            // from the default value literal.
            let ty = decl.split_whitespace().nth(1).unwrap_or("");
            let id = match ty {
                "logical" => "SOTUNPROP1",
                "char" => "SOTUNPROP3",
                "string" => "SOTUNPROP4",
                "" if default.starts_with("true") || default.starts_with("false") => "SOTUNPROP1",
                "" if default.starts_with('\'') => "SOTUNPROP3",
                "" if default.starts_with('"') => "SOTUNPROP4",
                _ => continue,
            };
            if self.is_enabled(id) {
                diags.push(make_diag_named(id, child, property_name(child, source)));
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
    fn sotunprop_infers_type_from_default_and_fills_name() {
        let source = system_class(
            "    properties
        Flag = true;
        Tag = 'a';
        Msg = \"b\";
    end
",
        );
        let diags = lint_file(&*engine(), &source);
        let msgs: Vec<_> = diags
            .iter()
            .map(|d| (d.rule_id, d.message.clone()))
            .collect();
        assert!(
            msgs.iter()
                .any(|(i, m)| *i == "SOTUNPROP1" && m.contains("Flag property")),
            "{msgs:?}"
        );
        assert!(
            msgs.iter()
                .any(|(i, m)| *i == "SOTUNPROP3" && m.contains("Tag property")),
            "{msgs:?}"
        );
        assert!(
            msgs.iter()
                .any(|(i, m)| *i == "SOTUNPROP4" && m.contains("Msg property")),
            "{msgs:?}"
        );
    }

    #[test]
    fn sotunprop_constant_block_does_not_fire() {
        let source = system_class(
            "    properties (Constant)
        Flag logical = true
    end
",
        );
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SOTUNPROP1"), "got: {diags:?}");
    }

    #[test]
    fn sotunprop_nontunable_block_does_not_fire() {
        let source =
            system_class("    properties (Nontunable)\n        Label char = 'x'\n    end\n");
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SOTUNPROP3"), "got: {diags:?}");
    }
}
