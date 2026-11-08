use super::*;

impl GoodPracticesEngine {
    /// MCPO: `SetObservable`/`GetObservable`/`AbortSet` have no effect on
    /// properties of a value class.
    pub(crate) fn check_mcpo(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MCPO") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let Some(class) = meta.class.as_ref() else {
            return Vec::new();
        };
        if class.is_handle() {
            return Vec::new();
        }

        let class_node = find_child_of_kind(tree.root_node(), "class_definition");
        let mut diagnostics = Vec::new();

        for (block_idx, block) in class.properties_blocks.iter().enumerate() {
            let attr_name = ["SetObservable", "GetObservable", "AbortSet"]
                .iter()
                .find(|name| {
                    block
                        .attributes
                        .iter()
                        .any(|a| &a.name == *name && !a.negated)
                });
            let Some(attr_name) = attr_name else {
                continue;
            };

            let (start_byte, end_byte, line, column) =
                block_diagnostic_position(class_node, block_idx, class, source);

            diagnostics.push(Diagnostic {
                rule_id: "MCPO",
                message: format!("{attr_name} property has no effect in a value class."),
                severity: Severity::Warning,
                byte_range: start_byte..end_byte,
                line,
                column,
                fix: None,
            });
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcpo_fires_on_value_class_observable() {
        let source =
            "classdef ValClass\n    properties (SetObservable)\n        Data\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_mcpo(&tree, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MCPO");
    }

    #[test]
    fn test_mcpo_silent_on_handle_class() {
        let source = "classdef HClass < handle\n    properties (SetObservable)\n        Data\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_mcpo(&tree, source).is_empty());
    }

    #[test]
    fn test_mcpo_respects_disabled_check() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec!["MCPO".to_string()],
            },
        };
        let source =
            "classdef ValClass\n    properties (SetObservable)\n        Data\n    end\nend\n";
        let tree = parse(source);
        assert!(eng.check_mcpo(&tree, source).is_empty());
    }
}
