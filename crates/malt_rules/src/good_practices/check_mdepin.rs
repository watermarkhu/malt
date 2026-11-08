use super::*;

impl GoodPracticesEngine {
    /// MDEPIN: dependent properties do not store values, so default values
    /// should not be assigned to them.
    pub(crate) fn check_mdepin(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MDEPIN") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let Some(class) = meta.class.as_ref() else {
            return Vec::new();
        };

        let mut diagnostics = Vec::new();
        for block in &class.properties_blocks {
            let is_dependent = block
                .attributes
                .iter()
                .any(|a| a.name == "Dependent" && !a.negated);
            if !is_dependent {
                continue;
            }
            for prop in &block.properties {
                if prop.has_default {
                    diagnostics.push(Diagnostic {
                        rule_id: "MDEPIN",
                        message: "Default values should not be assigned to dependent properties because dependent properties do not store the values.".to_string(),
                        severity: Severity::Warning,
                        byte_range: prop.byte_range.clone(),
                        line: prop.line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdepin_fires_on_dependent_default() {
        let source = "classdef DepClass\n    properties (Dependent)\n        Y = 5\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_mdepin(&tree, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MDEPIN");
    }

    #[test]
    fn test_mdepin_silent_on_dependent_without_default() {
        let source = "classdef DepClass\n    properties (Dependent)\n        Y\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_mdepin(&tree, source).is_empty());
    }
}
