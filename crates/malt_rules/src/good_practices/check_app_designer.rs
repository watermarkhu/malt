use super::check_prop_validation::property_name_uses;
use super::*;

impl GoodPracticesEngine {
    /// ADMTHDINV / ADPROP: App Designer member-access checks.
    ///
    /// Fires inside methods of a class derived from `matlab.apps.AppBase`:
    /// - ADMTHDINV: a class method is called without `app` as its first argument.
    /// - ADPROP: a class property is used through a bare identifier (read or stored) instead
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

        // Method names, excluding the constructor (the class name itself).
        let methods: std::collections::HashSet<String> = meta
            .all_methods()
            .iter()
            .map(|m| m.name.clone())
            .filter(|n| *n != class.name)
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

        // ADPROP: every bare use of a property name in a method. App Designer members
        // are reached through `app.`, so a bare name is reported whether it is read or stored.
        if self.is_check_enabled("ADPROP") {
            for name in property_name_uses(tree, source) {
                for usage in &name.uses {
                    diagnostics.push(Diagnostic {
                        rule_id: "ADPROP",
                        message: format!(
                            "'{}' is also the name of a property, which may be confusing. Use app.PropertyName syntax to reference the property, or change one of the names to improve readability.",
                            name.name
                        ),
                        severity: Severity::Warning,
                        byte_range: usage.byte_range.clone(),
                        line: usage.line,
                        column: usage.column,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

/// Whether an identifier is an operand of a binary operator (`helper + 1`).
fn is_binary_operand(node: Node) -> bool {
    node.parent().is_some_and(|p| p.kind() == "binary_operator")
}

/// Whether the function holding `node` assigns `name` itself (a local variable of that name).
fn assigned_in_function(node: Node, name: &str, source: &str) -> bool {
    let mut ancestor = node.parent();
    while let Some(a) = ancestor {
        if a.kind() == "function_definition" {
            return assigns_name(a, name, source);
        }
        ancestor = a.parent();
    }
    false
}

fn assigns_name(node: Node, name: &str, source: &str) -> bool {
    if node.kind() == "assignment"
        && node
            .child_by_field_name("left")
            .is_some_and(|l| l.kind() == "identifier" && node_text(l, source) == name)
    {
        return true;
    }
    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();
    children
        .into_iter()
        .any(|child| assigns_name(child, name, source))
}

/// Whether an identifier is a value rather than a call: the body of an anonymous
/// function (`@() helper`) or the right-hand side of an assignment (`m = helper`).
fn is_bare_value(node: Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        "lambda" => parent
            .named_child((parent.named_child_count().saturating_sub(1)) as u32)
            .is_some_and(|last| last.id() == node.id()),
        "assignment" => parent
            .child_by_field_name("right")
            .is_some_and(|rhs| rhs.id() == node.id()),
        _ => false,
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
    // Probed: a method named as a value, `@() helper` or `m = helper`, is reported too, and so
    // is an operand of an operator (`@() helper + 1`, `r = helper + 1`) unless the method
    // assigns the name itself (`helper = 2; r = helper + 1` is silent).
    if node.kind() == "identifier" && (is_bare_value(node) || is_binary_operand(node)) {
        let name = node_text(node, source);
        let qualified = source[..node.start_byte()].trim_end().ends_with('.');
        let shadowed = is_binary_operand(node) && assigned_in_function(node, name, source);
        if methods.contains(name) && !qualified && !shadowed {
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
    if node.kind() == "function_call" {
        if let Some(name_node) = node.child_by_field_name("name") {
            if name_node.kind() == "identifier" {
                let name = node_text(name_node, source);
                // `obj.name(...)` is a call through another object, not a bare call (probed).
                let qualified = source[..name_node.start_byte()].trim_end().ends_with('.');
                if methods.contains(name) && !qualified && !call_has_app_first_arg(node, source) {
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
    fn test_admthdinv_fires_on_handle_and_bare_reference_only() {
        // Probed (AppA): `@() helper` (7), `@(x) helper(1)` (8) and `m = helper;` (9) are
        // ADMTHDINV; `helper(app)`, `app.helper()` and `cellfun(@helper, ...)` are silent.
        let source = "classdef MyApp < matlab.apps.AppBase\n    methods (Access = private)\n        function r = helper(~, x)\n            r = x + 1;\n        end\n        function btn(app)\n            f = @() helper;\n            g = @(x) helper(1);\n            m = helper;\n            helper(app);\n            app.helper();\n            c = cellfun(@helper, {1, 2}, 'UniformOutput', false);\n        end\n    end\nend\n";
        let tree = parse(source);
        let lines: Vec<usize> = engine()
            .check_app_designer(&tree, source)
            .iter()
            .filter(|d| d.rule_id == "ADMTHDINV")
            .map(|d| d.line)
            .collect();
        assert_eq!(lines, vec![7, 8, 9]);
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
    fn admthdinv_ignores_a_qualified_static_call_and_adprop_reads_too() {
        // Probed: `P.helper2(1)` (160) is not a bare call; `r = Count;` (158) is ADPROP.
        let qualified = "classdef C < matlab.apps.AppBase\n properties\n  Count = 0\n end\n methods (Access = private)\n  function go(app)\n   r = C.helper2(1);\n  end\n end\n methods (Static)\n  function r = helper2(x)\n   r = x;\n  end\n end\nend\n";
        let read = "classdef C < matlab.apps.AppBase\n properties (Access = private)\n  Count = 0\n end\n methods (Access = private)\n  function r = go(app)\n   r = Count;\n  end\n end\nend\n";
        assert!(lines(qualified, "ADMTHDINV").is_empty());
        assert_eq!(lines(read, "ADPROP"), vec![7]);
    }
}
