use super::*;

use crate::analysis::symbols::{DefKind, ScopeKind, SymbolTable};
use std::collections::{HashMap, HashSet};

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
            // Probed: input arguments named like a property are silent (also in
            // `matlab.unittest.TestCase` test methods, see `scaleInvariance(testCase, scale)`).
            // A method that writes `obj.name`, or copies it with `x = obj.name;`, is silent.
            let body = &source[scope.byte_range.clone()];
            let receivers: Vec<&str> = scope
                .defs
                .iter()
                .filter(|d| d.kind == DefKind::InputArg)
                .map(|d| d.name.as_str())
                .collect();
            // Probed: a method of a `methods (Static)` block is silent (it has no object).
            if is_static_method(tree, source, scope.byte_range.start) {
                continue;
            }

            // Each store of the variable is reported, then each read after its first store.
            let mut first_store: HashMap<&str, usize> = HashMap::new();
            for def in &scope.defs {
                if !matches!(def.kind, DefKind::Assignment | DefKind::ForIterator)
                    || def.partial
                    || !properties.contains(def.name.as_str())
                    || receivers.iter().any(|r| {
                        body.contains(&format!("{r}.{} = ", def.name))
                            || body.contains(&format!("= {r}.{};", def.name))
                    })
                {
                    continue;
                }
                first_store
                    .entry(def.name.as_str())
                    .and_modify(|start| *start = (*start).min(def.byte_range.start))
                    .or_insert(def.byte_range.start);
                diagnostics.push(prop_diagnostic(
                    &def.name,
                    def.byte_range.clone(),
                    def.line,
                    def.column,
                ));
            }
            for usage in &scope.uses {
                let Some(&start) = first_store.get(usage.name.as_str()) else {
                    continue;
                };
                // `obj.name` is a property access, not a read of the variable.
                if usage.byte_range.start <= start
                    || source[..usage.byte_range.start].ends_with('.')
                {
                    continue;
                }
                diagnostics.push(prop_diagnostic(
                    &usage.name,
                    usage.byte_range.clone(),
                    usage.line,
                    usage.column,
                ));
            }
        }
        diagnostics
    }
}

/// The PROP diagnostic for one occurrence of a variable named like a property.
fn prop_diagnostic(
    name: &str,
    byte_range: std::ops::Range<usize>,
    line: usize,
    column: usize,
) -> Diagnostic {
    Diagnostic {
        rule_id: "PROP",
        message: format!(
            "{name} is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename this variable to improve readability."
        ),
        severity: Severity::Warning,
        byte_range,
        line,
        column,
        fix: None,
    }
}

/// Whether the method starting at `start` is in a `methods (Static)` block.
fn is_static_method(tree: &tree_sitter::Tree, source: &str, start: usize) -> bool {
    let mut current = tree.root_node().descendant_for_byte_range(start, start);
    while let Some(node) = current {
        if node.kind() == "methods" {
            let mut cursor = node.walk();
            return node
                .children(&mut cursor)
                .any(|c| c.kind() == "attributes" && source[c.byte_range()].contains("Static"));
        }
        current = node.parent();
    }
    false
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

    /// Lines of the PROP diagnostics for `src`.
    fn prop_lines(src: &str) -> Vec<usize> {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_file(&*e, src)
            .iter()
            .filter(|d| d.rule_id == "PROP")
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn prop_reports_every_store_and_read_of_the_variable() {
        // Probed: MATLAB reports the store (7) and the read (8), see cbp_pa.
        let src = "classdef C\n properties\n  a\n end\n methods\n  function m(obj)\n   a = 1;\n   disp(a);\n  end\n end\nend\n";
        assert_eq!(prop_lines(src), vec![7, 8]);
    }

    #[test]
    fn prop_skips_field_access_of_the_property() {
        // `obj.a` is the property, not a read of the variable (line 8 is silent).
        let src = "classdef C\n properties\n  a\n end\n methods\n  function m(obj)\n   a = 1;\n   b = obj.a + 1;\n   disp(a + b);\n  end\n end\nend\n";
        assert_eq!(prop_lines(src), vec![7, 9]);
    }

    #[test]
    fn prop_fires_with_unresolved_superclass() {
        // Probed: cbp_ph, an unknown superclass does not silence PROP.
        let src = "classdef C < cb_unknown\n properties\n  a\n end\n methods\n  function m(obj)\n   a = 1;\n   disp(a);\n  end\n end\nend\n";
        assert_eq!(prop_lines(src), vec![7, 8]);
    }

    #[test]
    fn prop_silent_for_input_argument_in_test_method() {
        // Probed: parameters named like a property (cb_param_b, cb_param_e) are silent.
        let src = "classdef C < matlab.unittest.TestCase\n properties\n  scale = 2\n end\n methods (Test)\n  function t(testCase, scale)\n   testCase.verifyEqual(scale, 2);\n  end\n end\nend\n";
        assert!(prop_lines(src).is_empty());
    }

    #[test]
    fn prop_silent_in_static_method() {
        // Probed: cbp_pf, a `methods (Static)` method has no object.
        let src = "classdef C\n properties\n  a\n end\n methods (Static)\n  function r = m()\n   a = 2;\n   r = a;\n  end\n end\nend\n";
        assert!(prop_lines(src).is_empty());
    }

    #[test]
    fn prop_silent_when_method_copies_or_writes_the_property() {
        // Probed: cbp_pi (`a = obj.a;`) and cbp_pj (`obj.a = a;`) are silent.
        let copy = "classdef C\n properties\n  a\n end\n methods\n  function m(obj)\n   a = obj.a;\n   disp(a);\n  end\n end\nend\n";
        let write = "classdef C\n properties\n  a\n end\n methods\n  function m(obj, b)\n   a = b;\n   obj.a = a;\n  end\n end\nend\n";
        assert!(prop_lines(copy).is_empty());
        assert!(prop_lines(write).is_empty());
    }

    #[test]
    fn prop_ignores_properties_themselves_and_other_names() {
        assert!(!prop("classdef C\n properties\n  a\n end\n methods\n  function m(obj)\n   b = obj.a;\n   disp(b);\n  end\n end\nend\n"));
        assert!(!prop("classdef C\n properties\n  a\n end\nend\n"));
    }
}
