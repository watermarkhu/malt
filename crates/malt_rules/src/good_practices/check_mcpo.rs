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
        // Probed: silent with any superclass (`handle`, `matlab.mixin.Copyable`,
        // a value superclass, `handle & matlab.mixin.SetGet`), so only a class
        // with no superclass at all is reported.
        if !class.superclasses.is_empty() {
            return Vec::new();
        }

        let class_node = find_child_of_kind(tree.root_node(), "class_definition");
        let mut diagnostics = Vec::new();

        // Probed: MATLAB never reports `AbortSet` in a value class, only
        // `SetObservable` and `GetObservable`.
        for (block_idx, block) in class.properties_blocks.iter().enumerate() {
            let attr_name = ["SetObservable", "GetObservable"].iter().find(|name| {
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
    fn test_mcpo_fires_on_get_observable_value_class() {
        // Probed: `GetObservable` is reported like `SetObservable`.
        let source =
            "classdef ValClass\n    properties (GetObservable)\n        Data\n    end\nend\n";
        let diags = engine().check_mcpo(&parse(source), source);
        assert_eq!(diags.len(), 1);
        assert_eq!(
            diags[0].message,
            "GetObservable property has no effect in a value class."
        );
    }

    #[test]
    fn test_mcpo_silent_on_abort_set_in_value_class() {
        // Probed: `AbortSet` is never reported, with or without a superclass.
        let source = "classdef ValClass\n    properties (AbortSet)\n        Data\n    end\nend\n";
        assert!(engine().check_mcpo(&parse(source), source).is_empty());
    }

    #[test]
    fn test_mcpo_silent_with_any_superclass() {
        // Probed (l_data_container.m:8, :13): each superclass list silences MCPO.
        for header in [
            "classdef C < matlab.mixin.Copyable & matlab.mixin.SetGet",
            "classdef C < matlab.mixin.SetGet",
            "classdef C < matlab.mixin.Copyable",
            "classdef C < handle & matlab.mixin.SetGet",
            "classdef C < SomeValueBase",
        ] {
            let source =
                format!("{header}\n    properties (SetObservable)\n        Data\n    end\nend\n");
            assert!(
                engine().check_mcpo(&parse(&source), &source).is_empty(),
                "{header}: reported"
            );
        }
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
