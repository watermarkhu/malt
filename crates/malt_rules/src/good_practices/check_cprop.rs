use super::*;

impl GoodPracticesEngine {
    /// CPROP: Constant property with complex initialization could be a method.
    pub(crate) fn check_cprop(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("CPROP") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let mut diagnostics = Vec::new();

        if let Some(ref class) = meta.class {
            for block in &class.properties_blocks {
                let is_constant = block
                    .attributes
                    .iter()
                    .any(|a| a.name == "Constant" && !a.negated);
                if !is_constant {
                    continue;
                }
                for prop in &block.properties {
                    // Flag if the default value is a function call (complex init).
                    if let Some(ref val) = prop.default_value {
                        if val.contains('(') {
                            diagnostics.push(Diagnostic {
                                rule_id: "CPROP",
                                message: format!(
                                    "{} is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename the property to improve readability.",
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
                }
            }
        }

        diagnostics
    }
}
