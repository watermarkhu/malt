use super::*;

impl GoodPracticesEngine {
    /// ADMTHDINV / ADPROP: App Designer member-access checks.
    ///
    /// Fires inside methods of a class derived from `matlab.apps.AppBase`:
    /// - ADMTHDINV: a class method is called without `app` as its first argument.
    /// - ADPROP: a class property is assigned through a bare identifier instead
    ///   of `app.PROP`.
    pub(crate) fn check_app_designer(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        let any_enabled = self.is_check_enabled("ADMTHDINV") || self.is_check_enabled("ADPROP");
        if !any_enabled {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let Some(class) = meta.class.as_ref() else {
            return Vec::new();
        };
        let is_app_designer = class.superclasses.iter().any(|s| s.contains("matlab.apps"));
        if !is_app_designer {
            return Vec::new();
        }

        let properties: std::collections::HashSet<String> = class
            .properties_blocks
            .iter()
            .flat_map(|pb| pb.properties.iter())
            .map(|p| p.name.clone())
            .collect();
        // Method names, excluding the constructor (the class name itself).
        let methods: std::collections::HashSet<String> = meta
            .all_methods()
            .iter()
            .map(|m| m.name.clone())
            .filter(|n| *n != class.name)
            .collect();
        let method_ranges: Vec<std::ops::Range<usize>> = meta
            .all_methods()
            .iter()
            .map(|m| m.byte_range.clone())
            .collect();

        let mut diagnostics = Vec::new();

        // ADMTHDINV: tree-walk each method body for bare calls to class methods.
        if self.is_check_enabled("ADMTHDINV") {
            let mut class_nodes = Vec::new();
            collect_nodes_of_kind(tree.root_node(), "class_definition", &mut class_nodes);
            if let Some(class_node) = class_nodes.first().copied() {
                let mut cursor = class_node.walk();
                for child in class_node.children(&mut cursor) {
                    if child.kind() != "methods" {
                        continue;
                    }
                    let mut m = child.walk();
                    for method in child.children(&mut m) {
                        if method.kind() != "function_definition" {
                            continue;
                        }
                        if let Some(block) = find_child_of_kind(method, "block") {
                            collect_admthdinv(block, source, &methods, &mut diagnostics);
                        }
                    }
                }
            }
        }

        // ADPROP: symbol-table based, restricted to the App Designer
        // class's method scopes.
        if self.is_check_enabled("ADPROP") {
            let sym = SymbolTable::build(tree, source);
            for scope in &sym.scopes {
                if scope.kind != crate::analysis::symbols::ScopeKind::Method {
                    continue;
                }
                let is_class_method = method_ranges
                    .iter()
                    .any(|r| r.start == scope.byte_range.start && r.end == scope.byte_range.end);
                if !is_class_method {
                    continue;
                }

                for d in &scope.defs {
                    if d.kind == crate::analysis::symbols::DefKind::Assignment
                        && properties.contains(&d.name)
                        && !scope.defs.iter().any(|o| {
                            o.name == d.name
                                && o.kind == crate::analysis::symbols::DefKind::InputArg
                        })
                    {
                        diagnostics.push(Diagnostic {
                            rule_id: "ADPROP",
                            message: format!(
                                "{} is also the name of a property, which may be confusing. Use app.PropertyName syntax to reference the property, or change one of the names to improve readability.",
                                d.name
                            ),
                            severity: Severity::Warning,
                            byte_range: d.byte_range.clone(),
                            line: d.line,
                            column: d.column,
                            fix: None,
                        });
                    }
                }
            }
        }

        diagnostics
    }
}

/// Collect ADMTHDINV diagnostics: bare `foo(...)` calls to class methods whose
/// first argument is not `app`.
fn collect_admthdinv(
    node: Node,
    source: &str,
    methods: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "function_definition" || node.kind() == "class_definition" {
        return;
    }
    if node.kind() == "function_call" {
        if let Some(name_node) = node.child_by_field_name("name") {
            if name_node.kind() == "identifier" {
                let name = node_text(name_node, source);
                if methods.contains(name) && !call_has_app_first_arg(node, source) {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "ADMTHDINV",
                        message: format!("Use {name}(app, ...) to call this function."),
                        severity: Severity::Warning,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            collect_admthdinv(child, source, methods, diagnostics);
        }
    }
}

/// Check whether a call's first argument is the literal identifier `app`.
fn call_has_app_first_arg(call: Node, source: &str) -> bool {
    let Some(args) = find_child_of_kind(call, "arguments") else {
        return false;
    };
    let Some(first) = first_named_child(args) else {
        return false;
    };
    first.kind() == "identifier" && node_text(first, source) == "app"
}

#[cfg(test)]
mod tests {
    use super::*;

    const APP_CLASS: &str = "classdef MyApp < matlab.apps.AppBase\n";

    fn app_designer_ids(source: &str) -> Vec<&'static str> {
        let tree = parse(source);
        let eng = engine();
        eng.check_app_designer(&tree, source)
            .iter()
            .map(|d| d.rule_id)
            .collect()
    }

    #[test]
    fn test_admthdinv_fires_on_bare_method_call_without_app() {
        let source = format!(
            "{APP_CLASS}    properties\n        Count = 0\n    end\n\
             methods\n        function results = compute(app, x)\n            helper(x);\n\
             results = x;\n        end\n        function helper(app, x)\n        end\n    end\nend\n"
        );
        let ids = app_designer_ids(&source);
        assert_eq!(ids.len(), 1, "got: {ids:?}");
        assert_eq!(ids[0], "ADMTHDINV");
    }

    #[test]
    fn test_admthdinv_silent_when_app_is_first_argument() {
        let source = format!(
            "{APP_CLASS}    methods\n        function results = compute(app, x)\n            helper(app, x);\n\
             results = x;\n        end\n        function helper(app, x)\n        end\n    end\nend\n"
        );
        let ids = app_designer_ids(&source);
        assert!(!ids.contains(&"ADMTHDINV"), "unexpected ADMTHDINV: {ids:?}");
    }

    #[test]
    fn test_admthdinv_silent_in_non_app_class() {
        let source = "classdef Plain\n    methods\n        function results = compute(obj, x)\n            helper(x);\n            results = x;\n        end\n        function helper(obj, x)\n        end\n    end\nend\n";
        let ids = app_designer_ids(source);
        assert!(ids.is_empty(), "got: {ids:?}");
    }

    #[test]
    fn test_adprop_fires_on_bare_property_assignment() {
        let source = format!(
            "{APP_CLASS}    properties\n        Count = 0\n    end\n\
             methods\n        function results = compute(app, x)\n            Count = x;\n            results = Count;\n        end\n    end\nend\n"
        );
        let ids = app_designer_ids(&source);
        assert!(ids.contains(&"ADPROP"), "got: {ids:?}");
    }

    #[test]
    fn test_adprop_silent_with_app_qualified_access() {
        let source = format!(
            "{APP_CLASS}    properties\n        Count = 0\n    end\n\
             methods\n        function results = compute(app, x)\n            app.Count = x;\n            results = app.Count;\n        end\n    end\nend\n"
        );
        let ids = app_designer_ids(&source);
        assert!(!ids.contains(&"ADPROP"), "got: {ids:?}");
    }

    #[test]
    fn test_app_designer_checks_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec!["ADMTHDINV".to_string(), "ADPROP".to_string()],
            },
        };
        let source = format!(
            "{APP_CLASS}    properties\n        Count = 0\n    end\n\
             methods\n        function results = compute(app, x)\n            helper(x);\n            Count = x;\n            results = Count;\n        end\n        function helper(app, x)\n        end\n    end\nend\n"
        );
        let tree = parse(&source);
        assert!(eng.check_app_designer(&tree, &source).is_empty());
    }
}
