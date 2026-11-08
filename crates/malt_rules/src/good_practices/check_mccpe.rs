use super::*;

impl GoodPracticesEngine {
    /// MCCPE: attempting to call a property or event as a function.
    pub(crate) fn check_mccpe(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MCCPE") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let Some(class) = meta.class.as_ref() else {
            return Vec::new();
        };

        let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for block in &class.properties_blocks {
            for prop in &block.properties {
                names.insert(prop.name.clone());
            }
        }
        for block in &class.events_blocks {
            for ev in &block.events {
                names.insert(ev.clone());
            }
        }
        if names.is_empty() {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        if let Some(class_node) = find_child_of_kind(tree.root_node(), "class_definition") {
            collect_property_event_calls(class_node, source, &names, &mut diagnostics);
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mccpe_fires_on_property_called_as_function() {
        let source = "classdef Foo\n    properties\n        Color\n    end\n    methods\n        function go(obj)\n            y = Color(1);\n        end\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_mccpe(&tree, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MCCPE");
    }

    #[test]
    fn test_mccpe_silent_on_field_access() {
        let source = "classdef Foo\n    properties\n        Color\n    end\n    methods\n        function go(obj)\n            x = obj.Color;\n        end\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_mccpe(&tree, source).is_empty());
    }
}
