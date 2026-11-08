use super::*;

impl GoodPracticesEngine {
    /// MOBSRV: `SetObservable`/`GetObservable` have no effect on `Constant`
    /// properties.
    pub(crate) fn check_mobsrv(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MOBSRV") {
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
            let has_observable = block
                .attributes
                .iter()
                .any(|a| !a.negated && (a.name == "SetObservable" || a.name == "GetObservable"));
            if !has_constant || !has_observable {
                continue;
            }

            let (start_byte, end_byte, line, column) =
                block_diagnostic_position(class_node, block_idx, class, source);

            diagnostics.push(Diagnostic {
                rule_id: "MOBSRV",
                message:
                    "Using SetObservable or GetObservable on a Constant property has no effect."
                        .to_string(),
                severity: Severity::Info,
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
    fn test_mobsrv_fires_on_constant_observable() {
        let source = "classdef MyConst\n    properties (Constant, GetObservable)\n        X = 1\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_mobsrv(&tree, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MOBSRV");
    }

    #[test]
    fn test_mobsrv_silent_on_constant_only() {
        let source = "classdef MyConst\n    properties (Constant)\n        X = 1\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_mobsrv(&tree, source).is_empty());
    }
}
