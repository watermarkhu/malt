//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCEB: Events can only be defined in handle classes.
    pub(crate) fn check_mceb(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !class.events_blocks.is_empty() && !class.is_handle() {
            for eb in &class.events_blocks {
                if !eb.events.is_empty() {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCEB",
                        message: "Events can be defined only in a handle class.".to_string(),
                        severity: Severity::Error,
                        byte_range: class.byte_range.clone(),
                        line: class.line,
                        column: 1,
                        fix: None,
                    });
                    break; // Only report once per class
                }
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
