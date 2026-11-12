//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCEB: Events can only be defined in handle classes. Only a class with no
    /// superclass is reported: MATLAB is silent for any superclass, whether or
    /// not it resolves to a handle class (probed).
    pub(crate) fn check_mceb(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if class.superclasses.is_empty() {
            // Probed: one report per events block, an empty block included, at the
            // `events` keyword (not at `classdef`).
            for eb in &class.events_blocks {
                diagnostics.push(Diagnostic {
                    rule_id: "MCEB",
                    message: "Events can be defined only in a handle class.".to_string(),
                    severity: Severity::Error,
                    byte_range: class.byte_range.clone(),
                    line: eb.line,
                    column: 1,
                    fix: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mceb_events_in_non_handle() {
        let source = "\
classdef MyClass
    events
        DataChanged
    end
end
";
        let diags = check_source(source, "MyClass.m");
        let mceb = filter_by_id(&diags, "MCEB");
        assert!(
            !mceb.is_empty(),
            "MCEB should fire when events defined in non-handle class"
        );
    }

    #[test]
    fn test_mceb_reports_at_events_line() {
        // Probed (A1): MATLAB reports MCEB at the `events` line (5), not at `classdef`.
        let source = "classdef A1\n    properties\n        x = 1;\n    end\n    events\n        DataChanged\n    end\nend\n";
        let diags = check_source(source, "A1.m");
        let lines: Vec<usize> = filter_by_id(&diags, "MCEB")
            .iter()
            .map(|d| d.line)
            .collect();
        assert_eq!(lines, vec![5], "got: {diags:?}");
    }

    #[test]
    fn test_mceb_reports_each_events_block_including_empty() {
        // Probed (A4, A5): an empty `events` block is reported at its line, and each of
        // two blocks is reported (lines 2 and 5).
        let empty = check_source("classdef A4\n    events\n    end\nend\n", "A4.m");
        let lines: Vec<usize> = filter_by_id(&empty, "MCEB")
            .iter()
            .map(|d| d.line)
            .collect();
        assert_eq!(lines, vec![2], "got: {empty:?}");

        let two = check_source(
            "classdef A5\n    events\n        Start\n    end\n    events\n        Stop\n    end\nend\n",
            "A5.m",
        );
        let lines: Vec<usize> = filter_by_id(&two, "MCEB").iter().map(|d| d.line).collect();
        assert_eq!(lines, vec![2, 5], "got: {two:?}");
    }

    #[test]
    fn test_mceb_silent_with_superclass() {
        // Probes D, E, F: MATLAB is silent for any superclass, resolved or not.
        for superclass in ["Base", "matlab.mixin.SetGet"] {
            let source = format!(
                "classdef MyClass < {superclass}\n    events\n        DataChanged\n    end\nend\n"
            );
            let diags = check_source(&source, "MyClass.m");
            assert!(
                filter_by_id(&diags, "MCEB").is_empty(),
                "superclass {superclass}: MCEB should NOT fire"
            );
        }
    }

    #[test]
    fn test_mceb_no_fire_handle() {
        let source = "\
classdef MyClass < handle
    events
        DataChanged
    end
end
";
        let diags = check_source(source, "MyClass.m");
        let mceb = filter_by_id(&diags, "MCEB");
        assert!(
            mceb.is_empty(),
            "MCEB should NOT fire when events defined in handle class"
        );
    }
}
