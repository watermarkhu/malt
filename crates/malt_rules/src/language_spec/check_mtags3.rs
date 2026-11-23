//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MTAGS3: the Access attribute cannot be combined with SetAccess/GetAccess.
    pub(crate) fn check_mtags3(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MTAGS3") {
            return;
        }
        for pb in &class.properties_blocks {
            if Self::block_has_access_conflict(&pb.attributes) {
                diagnostics.push(Diagnostic {
                    rule_id: "MTAGS3",
                    message: "Cannot use the Access attribute when using the SetAccess or GetAccess attribute."
                        .to_string(),
                    severity: Severity::Error,
                    byte_range: class.byte_range.clone(),
                    line: pb.line,
                    column: 1,
                    fix: None,
                });
            }
        }
        // Methods blocks have no SetAccess/GetAccess: MATLAB reports ATUNK there, not MTAGS3
        // (probed: `methods (Access = public, SetAccess = private)` gives ATUNK at SetAccess).
    }

    /// Whether an attribute list contains Access together with SetAccess or GetAccess.
    pub(crate) fn block_has_access_conflict(attrs: &[AttributeMeta]) -> bool {
        let attrs = Attributes(attrs);
        attrs.is_present("Access")
            && (attrs.is_present("SetAccess") || attrs.is_present("GetAccess"))
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mtags3_fires_access_with_setaccess() {
        let source = "\
classdef Foo
    properties (Access = private, SetAccess = private)
        x
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MTAGS3");
        assert!(
            !hits.is_empty(),
            "MTAGS3 should fire when Access is combined with SetAccess"
        );
    }

    #[test]
    fn test_mtags3_no_fire() {
        let source = "\
classdef Foo
    properties (Access = private)
        x
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MTAGS3");
        assert!(hits.is_empty(), "MTAGS3 should NOT fire for Access alone");
    }

    #[test]
    fn test_mtags3_reports_at_properties_line() {
        let source = "classdef Foo < handle\n    properties (Access = private, SetAccess = public)\n        p = 1\n    end\nend\n";
        let diags = crate::language_spec::tests::check_source(source, "Foo.m");
        let hits = crate::language_spec::tests::filter_by_id(&diags, "MTAGS3");
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|d| d.line == 2));
    }
}
