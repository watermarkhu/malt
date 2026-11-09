use super::*;

use crate::analysis::symbols::{DefKind, ScopeKind, SymbolTable};
use std::collections::HashSet;

impl GoodPracticesEngine {
    /// PROP: a variable in a method that has the same name as a property of the
    /// class, which makes `name` and `obj.name` easy to mix up.
    pub(crate) fn check_prop_validation(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        if !self.is_check_enabled("PROP") {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let properties: HashSet<&str> = meta
            .all_properties()
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        if properties.is_empty() {
            return Vec::new();
        }

        let table = SymbolTable::build(tree, source);
        let mut diagnostics = Vec::new();
        for scope in &table.scopes {
            if scope.kind != ScopeKind::Method {
                continue;
            }
            let mut reported: HashSet<&str> = HashSet::new();
            for def in &scope.defs {
                let is_variable = matches!(
                    def.kind,
                    DefKind::Assignment | DefKind::ForIterator | DefKind::InputArg
                ) && !def.partial;
                if !is_variable
                    || !properties.contains(def.name.as_str())
                    || !reported.insert(def.name.as_str())
                {
                    continue;
                }
                diagnostics.push(Diagnostic {
                    rule_id: "PROP",
                    message: format!(
                        "{} is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename this variable to improve readability.",
                        def.name
                    ),
                    severity: Severity::Warning,
                    byte_range: def.byte_range.clone(),
                    line: def.line,
                    column: def.column,
                    fix: None,
                });
            }
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn prop(src: &str) -> bool {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        has_id(&lint_file(&*e, src), "PROP")
    }

    #[test]
    fn prop_fires_on_variable_named_like_a_property() {
        assert!(prop("classdef C\n properties\n  a\n end\n methods\n  function m(obj)\n   a = 1;\n   disp(a);\n  end\n end\nend\n"));
    }

    #[test]
    fn prop_ignores_properties_themselves_and_other_names() {
        assert!(!prop("classdef C\n properties\n  a\n end\n methods\n  function m(obj)\n   b = obj.a;\n   disp(b);\n  end\n end\nend\n"));
        assert!(!prop("classdef C\n properties\n  a\n end\nend\n"));
    }
}
