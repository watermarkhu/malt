use super::*;
use malt_core::ProjectIndex;
use std::path::Path;

impl GoodPracticesEngine {
    /// MCPO: `SetObservable`/`GetObservable`/`AbortSet` have no effect on
    /// properties of a value class.
    ///
    /// `project` is the path index when the user turned it on, and `file` the
    /// linted file. With it, a class whose superclasses are all value classes on
    /// the path is a value class too, and is reported as one with no superclass.
    pub(crate) fn check_mcpo(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        project: Option<&dyn ProjectIndex>,
        file: &Path,
    ) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MCPO") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let Some(class) = meta.class.as_ref() else {
            return Vec::new();
        };
        // Probed: silent with any superclass (`handle`, `matlab.mixin.Copyable`,
        // a value superclass, `handle & matlab.mixin.SetGet`), so only a class
        // with no superclass at all is reported. The path index, when on, adds
        // the subclasses of value classes (MATLAB loads them without a warning,
        // and the property has no more effect there than in its base).
        if !class.superclasses.is_empty() && !class.has_only_value_bases(project, file) {
            return Vec::new();
        }

        let class_node = find_child_of_kind(tree.root_node(), "class_definition");
        let mut diagnostics = Vec::new();

        // Probed: MATLAB never reports `AbortSet` in a value class, only
        // `SetObservable` and `GetObservable`.
        for (block_idx, block) in class.properties_blocks.iter().enumerate() {
            let attr_name = ["SetObservable", "GetObservable"].iter().find(|name| {
                block.attributes.iter().any(|a| {
                    &a.name == *name && !a.negated && a.value.as_deref().is_none_or(|v| v == "true")
                })
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
        let diags = eng.check_mcpo(&tree, source, None, Path::new("t.m"));
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MCPO");
    }

    #[test]
    fn test_mcpo_fires_on_get_observable_value_class() {
        // Probed: `GetObservable` is reported like `SetObservable`.
        let source =
            "classdef ValClass\n    properties (GetObservable)\n        Data\n    end\nend\n";
        let diags = engine().check_mcpo(&parse(source), source, None, Path::new("t.m"));
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
        assert!(engine()
            .check_mcpo(&parse(source), source, None, Path::new("t.m"))
            .is_empty());
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
                engine()
                    .check_mcpo(&parse(&source), &source, None, Path::new("t.m"))
                    .is_empty(),
                "{header}: reported"
            );
        }
    }

    #[test]
    fn test_mcpo_silent_on_handle_class() {
        let source = "classdef HClass < handle\n    properties (SetObservable)\n        Data\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng
            .check_mcpo(&tree, source, None, Path::new("t.m"))
            .is_empty());
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
        assert!(eng
            .check_mcpo(&tree, source, None, Path::new("t.m"))
            .is_empty());
    }

    /// The MCPO diagnostics of `source` with the search path `project`.
    fn mcpo_count(source: &str, project: &crate::test_util::StubProject) -> usize {
        let tree = parse(source);
        engine()
            .check_mcpo(&tree, source, Some(project), Path::new("sub.m"))
            .len()
    }

    #[test]
    fn test_mcpo_index_reports_observable_in_a_subclass_of_a_value_class() {
        // The subclass of a value class is a value class too (no effect, same message).
        let source = "classdef obs_v < base_v\n    properties (SetObservable)\n        y = 2;\n    end\nend\n";
        let project = crate::test_util::StubProject(vec![("base_v", false)]);
        assert_eq!(mcpo_count(source, &project), 1);
    }

    #[test]
    fn test_mcpo_index_silent_for_a_handle_base() {
        let source = "classdef obs_h < base_h\n    properties (SetObservable)\n        y = 2;\n    end\nend\n";
        let project = crate::test_util::StubProject(vec![("base_h", true)]);
        assert_eq!(mcpo_count(source, &project), 0);
    }

    #[test]
    fn test_mcpo_index_silent_for_an_unknown_base_and_without_the_index() {
        let source = "classdef obs_m < no_such_base\n    properties (SetObservable)\n        y = 2;\n    end\nend\n";
        let project = crate::test_util::StubProject(vec![("base_v", false)]);
        assert_eq!(mcpo_count(source, &project), 0);
        let tree = parse(source);
        assert_eq!(
            engine()
                .check_mcpo(&tree, source, None, Path::new("sub.m"))
                .len(),
            0
        );
    }
}

#[cfg(test)]
mod probe_p2a_tests {
    use crate::test_util::lint_file;
    use malt_core::Config;

    /// Lines of the `id` diagnostics that the default (mathworks) engine reports for `src`.
    fn lines(src: &str, id: &str) -> Vec<usize> {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_file(&*e, src)
            .iter()
            .filter(|d| d.rule_id == id)
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn mcpo_ignores_an_observable_attribute_set_to_false() {
        // Probed: `SetObservable = false` (32) and `~SetObservable` (31) are silent.
        let off = "classdef C\n properties (SetObservable = false)\n  Data\n end\nend\n";
        let negated = "classdef C\n properties (~SetObservable)\n  Data\n end\nend\n";
        let on = "classdef C\n properties (SetObservable)\n  Data\n end\nend\n";
        assert!(lines(off, "MCPO").is_empty());
        assert!(lines(negated, "MCPO").is_empty());
        assert_eq!(lines(on, "MCPO"), vec![2]);
    }
}
