use super::*;

impl GoodPracticesEngine {
    /// MCSAC: `SetAccess` cannot be set on `Constant` properties.
    pub(crate) fn check_mcsac(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MCSAC") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let Some(class) = meta.class.as_ref() else {
            return Vec::new();
        };

        let class_node = find_child_of_kind(tree.root_node(), "class_definition");
        let mut diagnostics = Vec::new();

        for (block_idx, block) in class.properties_blocks.iter().enumerate() {
            let has_constant = block
                .attributes
                .iter()
                .any(|a| a.name == "Constant" && !a.negated);
            let has_set_access = block
                .attributes
                .iter()
                .any(|a| a.name == "SetAccess" && !a.negated);
            if !has_constant || !has_set_access {
                continue;
            }

            let (start_byte, end_byte, line, column) =
                block_diagnostic_position(class_node, block_idx, class, source);

            diagnostics.push(Diagnostic {
                rule_id: "MCSAC",
                message: "SetAccess cannot be set on Constant properties.".to_string(),
                severity: Severity::Warning,
                byte_range: start_byte..end_byte,
                line,
                column,
                fix: None,
            });
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcsac_fires_on_constant_setaccess() {
        let source = "classdef MyConst\n    properties (Constant, SetAccess = public)\n        X = 1\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_mcsac(&tree, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MCSAC");
    }

    #[test]
    fn test_mcsac_silent_without_setaccess() {
        let source = "classdef MyConst\n    properties (Constant)\n        X = 1\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_mcsac(&tree, source).is_empty());
    }
}
