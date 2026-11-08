use super::*;

impl GoodPracticesEngine {
    /// PROP: Property without type/size validation.
    pub(crate) fn check_prop_validation(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        if !self.is_check_enabled("PROP") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let mut diagnostics = Vec::new();

        for prop in meta.all_properties() {
            if prop.type_constraint.is_none()
                && prop.validators.is_empty()
                && prop.dimensions.is_none()
            {
                diagnostics.push(Diagnostic {
                    rule_id: "PROP",
                    message: format!(
                        "{} is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename this variable to improve readability.",
                        prop.name
                    ),
                    severity: Severity::Info,
                    byte_range: prop.byte_range.clone(),
                    line: prop.line,
                    column: 1,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}
