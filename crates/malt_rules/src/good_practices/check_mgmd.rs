use super::*;

impl GoodPracticesEngine {
    /// MGMD: a `get` method is required for each dependent property that does
    /// not have private `GetAccess`.
    pub(crate) fn check_mgmd(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MGMD") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let Some(class) = meta.class.as_ref() else {
            return Vec::new();
        };

        let method_names: std::collections::HashSet<String> = class
            .methods_blocks
            .iter()
            .flat_map(|mb| mb.methods.iter())
            .map(|m| m.name.clone())
            .collect();

        let mut diagnostics = Vec::new();
        for block in &class.properties_blocks {
            let is_dependent = block
                .attributes
                .iter()
                .any(|a| a.name == "Dependent" && !a.negated);
            if !is_dependent {
                continue;
            }
            let get_access_private = block.attributes.iter().any(|a| {
                a.name == "GetAccess"
                    && a.value
                        .as_deref()
                        .is_some_and(|v| v.eq_ignore_ascii_case("private"))
            });
            if get_access_private {
                continue;
            }
            for prop in &block.properties {
                let getter = format!("get.{}", prop.name);
                if !method_names.contains(&getter) {
                    diagnostics.push(Diagnostic {
                        rule_id: "MGMD",
                        message: "'get' method should be implemented for each dependent property that does not also have private 'GetAccess' attribute."
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
    fn test_mgmd_fires_on_dependent_without_getter() {
        let source = "classdef DepClass\n    properties (Dependent)\n        Y\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_mgmd(&tree, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MGMD");
    }

    #[test]
    fn test_mgmd_silent_when_getter_exists() {
        let source = "classdef DepClass\n    properties (Dependent)\n        Y\n    end\n    methods\n        function val = get.Y(obj)\n            val = 1;\n        end\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_mgmd(&tree, source).is_empty());
    }

    #[test]
    fn test_mgmd_silent_on_private_getaccess() {
        let source = "classdef DepClass\n    properties (Dependent, GetAccess = private)\n        Y\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        assert!(eng.check_mgmd(&tree, source).is_empty());
    }
}
