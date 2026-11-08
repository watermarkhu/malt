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
                    line: class.line,
                    column: 1,
                    fix: None,
                });
            }
        }
        for mb in &class.methods_blocks {
            if Self::block_has_access_conflict(&mb.attributes) {
                diagnostics.push(Diagnostic {
                    rule_id: "MTAGS3",
                    message: "Cannot use the Access attribute when using the SetAccess or GetAccess attribute."
                        .to_string(),
                    severity: Severity::Error,
                    byte_range: class.byte_range.clone(),
                    line: class.line,
                    column: 1,
                    fix: None,
                });
            }
        }
    }

    /// Whether an attribute list contains Access together with SetAccess or GetAccess.
    pub(crate) fn block_has_access_conflict(attrs: &[AttributeMeta]) -> bool {
        let has = |name: &str| attrs.iter().any(|a| a.name == name && !a.negated);
        has("Access") && (has("SetAccess") || has("GetAccess"))
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
}
