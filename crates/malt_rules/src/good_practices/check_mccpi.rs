use super::*;

impl GoodPracticesEngine {
    /// MCCPI: a `Constant` property must be initialized or declared `Abstract`.
    pub(crate) fn check_mccpi(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MCCPI") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let Some(class) = meta.class.as_ref() else {
            return Vec::new();
        };
        if class.is_abstract() {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        for block in &class.properties_blocks {
            let has_constant = block
                .attributes
                .iter()
                .any(|a| a.name == "Constant" && !a.negated);
            let block_abstract = block
                .attributes
                .iter()
                .any(|a| a.name == "Abstract" && !a.negated);
            if !has_constant || block_abstract {
                continue;
            }
            for prop in &block.properties {
                if !prop.has_default {
                    diagnostics.push(Diagnostic {
                        rule_id: "MCCPI",
                        message: "Initialize the Constant property or make it an Abstract Constant property."
                            .to_string(),
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
    fn test_mccpi_fires_on_uninitialized_constant() {
        let source = "classdef MyConst\n    properties (Constant)\n        X\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_mccpi(&tree, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MCCPI");
    }

    #[test]
    fn test_mccpi_silent_when_initialized() {
        let source = "classdef MyConst\n    properties (Constant)\n        X = 1\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_mccpi(&tree, source).is_empty());
    }

    #[test]
    fn test_mccpi_silent_on_abstract_block() {
        let source =
            "classdef MyConst\n    properties (Constant, Abstract)\n        X\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_mccpi(&tree, source).is_empty());
    }

    #[test]
    fn test_mccpi_silent_on_abstract_class() {
        let source =
            "classdef (Abstract) AbsClass\n    properties (Constant)\n        X\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_mccpi(&tree, source).is_empty());
    }
}
