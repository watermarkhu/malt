//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MTMAT: an attribute can only be set once within a single attribute list.
    pub(crate) fn check_mtmat(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MTMAT") {
            return;
        }
        Self::check_duplicate_attributes(
            &class.attributes,
            class.byte_range.clone(),
            class.line,
            diagnostics,
        );
        for pb in &class.properties_blocks {
            Self::check_duplicate_attributes(
                &pb.attributes,
                class.byte_range.clone(),
                class.line,
                diagnostics,
            );
        }
        for mb in &class.methods_blocks {
            Self::check_duplicate_attributes(
                &mb.attributes,
                class.byte_range.clone(),
                class.line,
                diagnostics,
            );
        }
    }

    /// Report MTMAT for duplicate attribute names in a single attribute list.
    pub(crate) fn check_duplicate_attributes(
        attrs: &[AttributeMeta],
        byte_range: std::ops::Range<usize>,
        line: usize,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut seen: Vec<&str> = Vec::new();
        for attr in attrs {
            if seen.contains(&attr.name.as_str()) {
                diagnostics.push(Diagnostic {
                    rule_id: "MTMAT",
                    message: "Attribute can only be set once.".to_string(),
                    severity: Severity::Error,
                    byte_range: byte_range.clone(),
                    line,
                    column: 1,
                    fix: None,
                });
            } else {
                seen.push(&attr.name);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mtmat_fires_duplicate_attribute() {
        let source = "\
classdef (Sealed, Sealed) Foo
    methods
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MTMAT");
        assert!(
            !hits.is_empty(),
            "MTMAT should fire for a duplicate class attribute"
        );
    }

    #[test]
    fn test_mtmat_no_fire() {
        let source = "\
classdef (Sealed, Abstract) Foo
    methods
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MTMAT");
        assert!(
            hits.is_empty(),
            "MTMAT should NOT fire for distinct attributes"
        );
    }
}
