use super::*;

/// Per-method metadata needed by the OOP practice checks.
struct MethodInfo {
    /// Member name with any `set.`/`get.` prefix stripped.
    base_name: String,
    /// First non-ignored input argument (the object).
    first_input: Option<String>,
    /// Whether the method declares a non-`~` output.
    has_output: bool,
    /// Whether this is a property setter (`set.X`).
    is_setter: bool,
    /// Whether the enclosing methods block has the `Static` attribute.
    is_static: bool,
}

/// Class-level context shared by all per-method OOP checks.
struct ClassCtx {
    /// Declared property names.
    prop_names: std::collections::HashSet<String>,
    /// Declared member names: properties + methods + events.
    declared: std::collections::HashSet<String>,
    /// Names of `Constant` properties.
    constant_props: std::collections::HashSet<String>,
    /// Whether the class is a handle class.
    is_handle: bool,
}

impl GoodPracticesEngine {
    /// MCNPN / MCNPR / MCSNOV / MCSOH / MCVM / MCCSPS / MCSUP: OOP practice
    /// checks over every `class_definition` in the file.
    pub(crate) fn check_oop_practice(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        let any_enabled = self.is_check_enabled("MCNPN")
            || self.is_check_enabled("MCNPR")
            || self.is_check_enabled("MCSNOV")
            || self.is_check_enabled("MCSOH")
            || self.is_check_enabled("MCVM")
            || self.is_check_enabled("MCCSPS")
            || self.is_check_enabled("MCSUP");
        if !any_enabled {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let Some(class) = meta.class.as_ref() else {
            return Vec::new();
        };

        let mut class_nodes = Vec::new();
        collect_nodes_of_kind(tree.root_node(), "class_definition", &mut class_nodes);
        let Some(class_node) = class_nodes.first().copied() else {
            return Vec::new();
        };

        let ctx = ClassCtx {
            prop_names: class
                .properties_blocks
                .iter()
                .flat_map(|pb| pb.properties.iter())
                .map(|p| p.name.clone())
                .collect(),
            declared: class
                .properties_blocks
                .iter()
                .flat_map(|pb| pb.properties.iter())
                .map(|p| p.name.clone())
                .chain(meta.all_methods().iter().map(|m| strip_set_get(&m.name)))
                .chain(
                    class
                        .events_blocks
                        .iter()
                        .flat_map(|eb| eb.events.iter().cloned()),
                )
                .collect(),
            constant_props: collect_constant_properties(class_node, source),
            is_handle: class.is_handle(),
        };

        // Index methods by their function_definition start byte.
        let methods_by_start: std::collections::HashMap<usize, MethodInfo> = meta
            .all_methods()
            .iter()
            .map(|m| {
                (
                    m.byte_range.start,
                    MethodInfo {
                        base_name: strip_set_get(&m.name),
                        first_input: m.inputs.iter().find(|i| *i != "~").cloned(),
                        has_output: m.outputs.iter().any(|o| o != "~"),
                        is_setter: m.is_setter,
                        is_static: m
                            .method_attributes
                            .iter()
                            .any(|a| a.name == "Static" && !a.negated),
                    },
                )
            })
            .collect();

        let mut diagnostics = Vec::new();

        // MCCSPS: constant property name used as the object of a dot-access.
        if self.is_check_enabled("MCCSPS") {
            collect_mccsps(class_node, source, &ctx.constant_props, &mut diagnostics);
        }

        // Per-method checks.
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
                let Some(info) = methods_by_start.get(&method.start_byte()) else {
                    continue;
                };
                let Some(block) = find_child_of_kind(method, "block") else {
                    continue;
                };

                // MCSNOV: setter in a value class without an output.
                if self.is_check_enabled("MCSNOV")
                    && !ctx.is_handle
                    && info.is_setter
                    && !info.has_output
                {
                    let pos = method.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "MCSNOV",
                        message: "Set function in value class must return the modified object."
                            .to_string(),
                        severity: Severity::Warning,
                        byte_range: method.start_byte()..method.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }

                // MCSOH: setter in a handle class with an output.
                if self.is_check_enabled("MCSOH")
                    && ctx.is_handle
                    && info.is_setter
                    && info.has_output
                {
                    let pos = method.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "MCSOH",
                        message:
                            "Set function in handle class does not need to return the modified object."
                                .to_string(),
                        severity: Severity::Warning,
                        byte_range: method.start_byte()..method.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }

                // MCVM: value-class method without an output modifies the object.
                if self.is_check_enabled("MCVM")
                    && !ctx.is_handle
                    && !info.is_setter
                    && !info.is_static
                    && !info.has_output
                {
                    if let Some(lhs) = first_object_assignment(block, source, info) {
                        let pos = lhs.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "MCVM",
                            message: "Value class method that modifies the object must return the modified object."
                                .to_string(),
                            severity: Severity::Warning,
                            byte_range: lhs.start_byte()..lhs.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }

                // Per-node member-access checks (MCNPN / MCNPR / MCSUP).
                let mut mcsup_seen = std::collections::HashSet::new();
                check_method_body(
                    self,
                    block,
                    source,
                    info,
                    &ctx,
                    &mut diagnostics,
                    &mut mcsup_seen,
                );
            }
        }

        diagnostics
    }
}

/// Strip a `set.`/`get.` prefix from a method name.
fn strip_set_get(name: &str) -> String {
    name.strip_prefix("set.")
        .or_else(|| name.strip_prefix("get."))
        .unwrap_or(name)
        .to_string()
}

/// Collect the names of `Constant` properties declared in a class.
fn collect_constant_properties(
    class_node: Node,
    source: &str,
) -> std::collections::HashSet<String> {
    let mut result = std::collections::HashSet::new();
    let mut cursor = class_node.walk();
    for child in class_node.children(&mut cursor) {
        if child.kind() != "properties" {
            continue;
        }
        let is_constant = find_child_of_kind(child, "attributes")
            .map(|attrs| {
                let mut c = attrs.walk();
                let mut found = false;
                for attr in attrs.children(&mut c) {
                    if attr.kind() == "attribute"
                        && !node_text(attr, source).trim_start().starts_with('~')
                        && find_child_of_kind(attr, "identifier")
                            .map(|id| node_text(id, source) == "Constant")
                            .unwrap_or(false)
                    {
                        found = true;
                        break;
                    }
                }
                found
            })
            .unwrap_or(false);
        if !is_constant {
            continue;
        }
        let mut c2 = child.walk();
        for prop in child.children(&mut c2) {
            if prop.kind() == "property" {
                if let Some(name) = prop.child_by_field_name("name") {
                    result.insert(node_text(name, source).to_string());
                }
            }
        }
    }
    result
}

/// Collect MCCSPS diagnostics: a constant property name used as the object of
/// a dot-access chain, which creates a struct instead of accessing the class
/// member.
fn collect_mccsps(
    node: Node,
    source: &str,
    constant_props: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "field_expression" {
        if let Some(obj) = node.child_by_field_name("object") {
            if obj.kind() == "identifier" {
                let name = node_text(obj, source);
                if constant_props.contains(name) {
                    let pos = obj.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "MCCSPS",
                        message: format!(
                            "Constant property {name} is not modified. '{name}.{name}' creates a struct named {name} with a field named {name}."
                        ),
                        severity: Severity::Warning,
                        byte_range: obj.start_byte()..obj.end_byte(),
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
            collect_mccsps(child, source, constant_props, diagnostics);
        }
    }
}

/// Find the first assignment that writes a field of the object (`obj.X = ...`),
/// used for MCVM.
fn first_object_assignment<'a>(
    node: Node<'a>,
    source: &str,
    info: &MethodInfo,
) -> Option<Node<'a>> {
    if node.kind() == "function_definition" || node.kind() == "class_definition" {
        return None;
    }
    if node.kind() == "assignment" {
        if let Some(left) = node.child_by_field_name("left") {
            if left.kind() == "field_expression" && is_object_field_access(left, source, info) {
                return Some(left);
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            if let Some(found) = first_object_assignment(child, source, info) {
                return Some(found);
            }
        }
    }
    None
}

/// Check whether a `field_expression` accesses a field of the method's object
/// (`obj.X` where `obj` is the first input argument).
fn is_object_field_access(fe: Node, source: &str, info: &MethodInfo) -> bool {
    let Some(obj) = fe.child_by_field_name("object") else {
        return false;
    };
    if obj.kind() != "identifier" {
        return false;
    }
    info.first_input
        .as_deref()
        .map(|first| node_text(obj, source) == first)
        .unwrap_or(false)
}

/// Walk a method body emitting MCNPN / MCNPR / MCSUP diagnostics.
fn check_method_body(
    eng: &GoodPracticesEngine,
    node: Node,
    source: &str,
    info: &MethodInfo,
    ctx: &ClassCtx,
    diagnostics: &mut Vec<Diagnostic>,
    mcsup_seen: &mut std::collections::HashSet<String>,
) {
    if node.kind() == "function_definition" || node.kind() == "class_definition" {
        return;
    }

    // MCNPN: member access on the object whose name is not declared.
    if eng.is_check_enabled("MCNPN")
        && node.kind() == "field_expression"
        && is_object_field_access(node, source, info)
    {
        if let Some(field) = node.child_by_field_name("field") {
            if field.kind() == "identifier" {
                let name = node_text(field, source);
                if !ctx.declared.contains(name) {
                    let pos = field.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "MCNPN",
                        message: format!(
                            "{name} is referenced but is not a property, method, or event name defined in this class."
                        ),
                        severity: Severity::Warning,
                        byte_range: field.start_byte()..field.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }
    }

    // MCNPR: assignment to a field of the object that is not a property.
    if eng.is_check_enabled("MCNPR") && node.kind() == "assignment" {
        if let Some(left) = node.child_by_field_name("left") {
            if left.kind() == "field_expression" && is_object_field_access(left, source, info) {
                if let Some(field) = left.child_by_field_name("field") {
                    if field.kind() == "identifier" {
                        let name = node_text(field, source);
                        if !ctx.prop_names.contains(name) {
                            let pos = field.start_position();
                            diagnostics.push(Diagnostic {
                                rule_id: "MCNPR",
                                message: format!(
                                    "{name} is not a property, but is the target of an assignment."
                                ),
                                severity: Severity::Warning,
                                byte_range: field.start_byte()..field.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // MCSUP: a setter accesses a property other than the one it sets.
    if eng.is_check_enabled("MCSUP")
        && info.is_setter
        && node.kind() == "field_expression"
        && is_object_field_access(node, source, info)
    {
        if let Some(field) = node.child_by_field_name("field") {
            if field.kind() == "identifier" {
                let name = node_text(field, source);
                if ctx.prop_names.contains(name)
                    && name != info.base_name
                    && mcsup_seen.insert(name.to_string())
                {
                    let pos = field.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "MCSUP",
                        message: format!(
                            "The set method for the property {} should not access another property ({}).",
                            info.base_name, name
                        ),
                        severity: Severity::Warning,
                        byte_range: field.start_byte()..field.end_byte(),
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
            check_method_body(eng, child, source, info, ctx, diagnostics, mcsup_seen);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oop_ids(source: &str) -> Vec<&'static str> {
        let tree = parse(source);
        let eng = engine();
        eng.check_oop_practice(&tree, source)
            .iter()
            .map(|d| d.rule_id)
            .collect()
    }

    fn oop_ids_for(source: &str, check: &str) -> Vec<&'static str> {
        oop_ids(source)
            .iter()
            .copied()
            .filter(|id| *id == check)
            .collect()
    }

    #[test]
    fn test_mcnpn_fires_on_undeclared_member() {
        let source = "classdef Foo\n    properties\n        X\n    end\n    methods\n        function r = getIt(obj)\n            r = obj.Z;\n        end\n    end\nend\n";
        let ids = oop_ids_for(source, "MCNPN");
        assert_eq!(ids.len(), 1, "got: {:?}", oop_ids(source));
    }

    #[test]
    fn test_mcnpn_silent_on_declared_member() {
        let source = "classdef Foo\n    properties\n        X\n    end\n    methods\n        function r = getIt(obj)\n            r = obj.X;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCNPN").is_empty());
    }

    #[test]
    fn test_mcnpr_fires_on_undeclared_assignment_target() {
        let source = "classdef Foo\n    properties\n        X\n    end\n    methods\n        function setVal(obj, v)\n            obj.Z = v;\n        end\n    end\nend\n";
        let ids = oop_ids_for(source, "MCNPR");
        assert_eq!(ids.len(), 1, "got: {:?}", oop_ids(source));
    }

    #[test]
    fn test_mcnpr_silent_on_declared_property() {
        let source = "classdef Foo\n    properties\n        X\n    end\n    methods\n        function setVal(obj, v)\n            obj.X = v;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCNPR").is_empty());
    }

    #[test]
    fn test_mcsnov_fires_on_value_class_setter_without_output() {
        let source = "classdef Foo\n    properties\n        X\n    end\n    methods\n        function set.X(obj, val)\n            obj.X = val;\n        end\n    end\nend\n";
        let ids = oop_ids_for(source, "MCSNOV");
        assert_eq!(ids.len(), 1, "got: {:?}", oop_ids(source));
    }

    #[test]
    fn test_mcsnov_silent_when_setter_returns_object() {
        let source = "classdef Foo\n    properties\n        X\n    end\n    methods\n        function obj = set.X(obj, val)\n            obj.X = val;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCSNOV").is_empty());
    }

    #[test]
    fn test_mcsoh_fires_on_handle_class_setter_with_output() {
        let source = "classdef Foo < handle\n    properties\n        X\n    end\n    methods\n        function obj = set.X(obj, val)\n            obj.X = val;\n        end\n    end\nend\n";
        let ids = oop_ids_for(source, "MCSOH");
        assert_eq!(ids.len(), 1, "got: {:?}", oop_ids(source));
    }

    #[test]
    fn test_mcsoh_silent_on_handle_class_setter_without_output() {
        let source = "classdef Foo < handle\n    properties\n        X\n    end\n    methods\n        function set.X(obj, val)\n            obj.X = val;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCSOH").is_empty());
    }

    #[test]
    fn test_mcvm_fires_on_value_class_method_without_output() {
        let source = "classdef Foo\n    properties\n        X\n    end\n    methods\n        function bump(obj, v)\n            obj.X = v;\n        end\n    end\nend\n";
        let ids = oop_ids_for(source, "MCVM");
        assert_eq!(ids.len(), 1, "got: {:?}", oop_ids(source));
    }

    #[test]
    fn test_mcvm_silent_when_method_has_output() {
        let source = "classdef Foo\n    properties\n        X\n    end\n    methods\n        function r = bump(obj, v)\n            obj.X = v;\n            r = obj.X;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCVM").is_empty());
    }

    #[test]
    fn test_mccsps_fires_on_constant_property_dot_access() {
        let source = "classdef Foo\n    properties (Constant)\n        C = 1\n    end\n    methods\n        function r = f(obj)\n            r = C.value;\n        end\n    end\nend\n";
        let ids = oop_ids_for(source, "MCCSPS");
        assert_eq!(ids.len(), 1, "got: {:?}", oop_ids(source));
    }

    #[test]
    fn test_mccsps_silent_on_plain_constant_access() {
        let source = "classdef Foo\n    properties (Constant)\n        C = 1\n    end\n    methods\n        function r = f(obj)\n            r = obj.C;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCCSPS").is_empty());
    }

    #[test]
    fn test_mcsup_fires_when_setter_accesses_other_property() {
        let source = "classdef Foo\n    properties\n        X\n        Y\n    end\n    methods\n        function set.X(obj, val)\n            obj.X = val;\n            obj.Y = val;\n        end\n    end\nend\n";
        let ids = oop_ids_for(source, "MCSUP");
        assert_eq!(ids.len(), 1, "got: {:?}", oop_ids(source));
    }

    #[test]
    fn test_mcsup_silent_when_setter_only_touches_own_property() {
        let source = "classdef Foo\n    properties\n        X\n        Y\n    end\n    methods\n        function set.X(obj, val)\n            obj.X = val;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCSUP").is_empty());
    }

    #[test]
    fn test_oop_checks_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec![
                    "MCNPN".to_string(),
                    "MCNPR".to_string(),
                    "MCSNOV".to_string(),
                    "MCSOH".to_string(),
                    "MCVM".to_string(),
                    "MCCSPS".to_string(),
                    "MCSUP".to_string(),
                ],
            },
        };
        let source = "classdef Foo < handle\n    properties (Constant)\n        C = 1\n    end\n    methods\n        function r = f(obj)\n            r = obj.Z + C.value;\n        end\n        function obj = set.C(obj, val)\n            obj.C = val;\n        end\n    end\nend\n";
        let tree = parse(source);
        assert!(eng.check_oop_practice(&tree, source).is_empty());
    }
}
