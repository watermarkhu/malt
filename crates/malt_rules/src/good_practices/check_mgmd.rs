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
            // `Access = private` also makes the getter private, and MATLAB reads the value
            // case-sensitively (`GetAccess = PRIVATE` is ATPPP and still reports MGMD).
            let get_access_private = block.attributes.iter().any(|a| {
                (a.name == "GetAccess" || a.name == "Access")
                    && a.value.as_deref() == Some("private")
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

#[cfg(test)]
mod probe_p2a_tests {
    use crate::test_util::lint_file;
    use malt_core::Config;

    /// Lines of the `id` diagnostics that the default (mathworks) engine reports for `src`.
    fn lines(src: &str, id: &str) -> Vec<usize> {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_file(&*e, src)
            .iter()
            .filter(|d| d.rule_id == id)
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn mgmd_getter_privacy_is_case_sensitive_and_access_counts() {
        // Probed: `Access = private` gives a private getter (42); `GetAccess = PRIVATE` does not (51).
        let access = "classdef C\n properties (Dependent, Access = private)\n  Y = 5\n end\nend\n";
        let upper = "classdef C\n properties (Dependent, GetAccess = PRIVATE)\n  Y\n end\nend\n";
        assert!(lines(access, "MGMD").is_empty());
        assert_eq!(lines(upper, "MGMD"), vec![3]);
    }
}
