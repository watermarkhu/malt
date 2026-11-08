//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCSGA: Set/get methods should not be in methods blocks with attributes.
    pub(crate) fn check_mcsga(
        &self,
        _class: &ClassMeta,
        meta: &FileMeta,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for func in &meta.functions {
            if (func.is_setter || func.is_getter) && !func.method_attributes.is_empty() {
                // Check if the methods block has attributes other than Access
                let has_non_trivial_attrs = func.method_attributes.iter().any(|a| {
                    a.name != "Access"
                        || a.value
                            .as_deref()
                            .is_some_and(|v| v != "public" && v != "?")
                });
                if has_non_trivial_attrs {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCSGA",
                        message: "Set or get method must be defined in a METHODS block with no attributes.".to_string(),
                        severity: Severity::Error,
                        byte_range: func.byte_range.clone(),
                        line: func.line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }
    }
}
