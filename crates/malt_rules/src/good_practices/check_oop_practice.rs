use super::*;

/// Per-method metadata needed by the OOP practice checks.
struct MethodInfo {
    /// Member name with any `set.`/`get.` prefix stripped.
    base_name: String,
    /// First non-ignored input argument (the object).
    first_input: Option<String>,
    /// The object variable: the first non-ignored input, or the output of a
    /// constructor (which creates the object and returns it).
    object: Option<String>,
    /// Whether the method declares a non-`~` output.
    has_output: bool,
    /// Whether the method returns its object (an output has the object's name).
    output_is_object: bool,
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
    /// Names of `Dependent` properties (their setters are exempt from MCSUP).
    dependent_props: std::collections::HashSet<String>,
    /// Whether the class is a handle class.
    is_handle: bool,
    /// Whether the class has no superclass at all (MCVM, MCSNOV).
    no_superclass: bool,
    /// Whether the class's members are fully known from this file: no
    /// superclass, or `handle` as the only one (MCNPN, MCNPR). MATLAB reports
    /// nothing on those members when any other superclass is present, whether
    /// or not it resolves.
    members_known: bool,
    /// Whether any methods block declares a signature without a body (`f(obj)`).
    /// MATLAB reports no MCNPN anywhere in such a class.
    has_signature_only: bool,
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
            dependent_props: collect_attributed_properties(class_node, source, "Dependent"),
            is_handle: class.is_handle(),
            no_superclass: class.superclasses.is_empty(),
            members_known: class.superclasses.is_empty()
                || (class.superclasses.len() == 1 && class.is_handle()),
            has_signature_only: {
                let mut signatures = Vec::new();
                collect_nodes_of_kind(class_node, "function_signature", &mut signatures);
                !signatures.is_empty()
            },
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
                        object: if m.is_constructor {
                            m.outputs.iter().find(|o| *o != "~").cloned()
                        } else {
                            m.inputs.iter().find(|i| *i != "~").cloned()
                        },
                        has_output: m.outputs.iter().any(|o| o != "~"),
                        output_is_object: m
                            .inputs
                            .iter()
                            .find(|i| *i != "~")
                            .is_some_and(|first| m.outputs.contains(first)),
                        is_setter: m.is_setter,
                        is_static: Attributes(&m.method_attributes).is_true("Static"),
                    },
                )
            })
            .collect();

        let mut diagnostics = Vec::new();

        // MCCSPS: `ClassName.K = v` on a Constant property K (see collect_mccsps).
        if self.is_check_enabled("MCCSPS") {
            collect_mccsps(
                class_node,
                source,
                &class.name,
                &ctx.constant_props,
                &mut diagnostics,
            );
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

                // MCSNOV: setter in a value class that does not return the object.
                // MATLAB is silent when the class has any superclass (probed). An empty
                // setter has no block, so the block lookup comes after this check.
                if self.is_check_enabled("MCSNOV")
                    && ctx.no_superclass
                    && info.is_setter
                    && !info.output_is_object
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

                let Some(block) = find_child_of_kind(method, "block") else {
                    continue;
                };
                // MCVM: value-class method that writes `obj.F` and never reads the
                // object again. Silent when the class has any superclass, or when
                // the method returns the object (probed).
                if self.is_check_enabled("MCVM")
                    && ctx.no_superclass
                    && !info.is_setter
                    && !info.is_static
                    && !info.output_is_object
                {
                    if let Some(assign) = first_unread_object_assignment(block, source, info) {
                        let lhs = assign.child_by_field_name("left").unwrap_or(assign);
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
                check_method_body(self, block, source, info, &ctx, &mut diagnostics);
            }
        }

        diagnostics
    }
}

/// Whether `node` is the target of an assignment (`obj.X = ...`); MCNPR reports those, not MCNPN.
fn is_assignment_target(node: Node) -> bool {
    node.parent().is_some_and(|p| {
        p.kind() == "assignment"
            && p.child_by_field_name("left")
                .is_some_and(|l| l.id() == node.id())
    })
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
    collect_attributed_properties(class_node, source, "Constant")
}

/// Names of the properties declared in a `properties` block carrying `attribute`.
fn collect_attributed_properties(
    class_node: Node,
    source: &str,
    attribute: &str,
) -> std::collections::HashSet<String> {
    let mut result = std::collections::HashSet::new();
    let mut cursor = class_node.walk();
    for child in class_node.children(&mut cursor) {
        if child.kind() != "properties" {
            continue;
        }
        let is_constant = attributes_of(child, source, AttrForms::Named)
            .iter()
            .any(|attr| attr.name == attribute && !attr.negated);
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

/// Collect MCCSPS diagnostics: `ClassName.K = v` (or `ClassName.K(i) = v`) where K
/// is a Constant property. MATLAB reads the class name as a new struct variable,
/// so it reports the first such assignment of each function; later ones use
/// that variable. Constructors are included (probed).
fn collect_mccsps(
    node: Node,
    source: &str,
    class_name: &str,
    constant_props: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "function_definition" {
        if let Some(field) =
            first_class_constant_assignment(node, source, class_name, constant_props)
        {
            let pos = field.start_position();
            let name = node_text(field, source);
            diagnostics.push(Diagnostic {
                rule_id: "MCCSPS",
                message: format!(
                    "Constant property '{name}' is not modified. '{class_name}.{name}' creates a struct named '{class_name}' with a field named '{name}'."
                ),
                severity: Severity::Warning,
                byte_range: field.start_byte()..field.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            collect_mccsps(child, source, class_name, constant_props, diagnostics);
        }
    }
}

/// The field name of the first `ClassName.K = v` assignment under `node` whose K
/// is a Constant property (see [`collect_mccsps`]).
fn first_class_constant_assignment<'a>(
    node: Node<'a>,
    source: &str,
    class_name: &str,
    constant_props: &std::collections::HashSet<String>,
) -> Option<Node<'a>> {
    if node.kind() == "assignment" {
        let target = node.child_by_field_name("left").and_then(|left| {
            if left.kind() == "function_call" {
                left.child_by_field_name("name")
            } else {
                Some(left)
            }
        });
        if let Some(fe) = target.filter(|t| t.kind() == "field_expression") {
            let object = fe.child_by_field_name("object");
            let field = fe.child_by_field_name("field").and_then(member_name_node);
            if let (Some(object), Some(field)) = (object, field) {
                if object.kind() == "identifier"
                    && node_text(object, source) == class_name
                    && constant_props.contains(node_text(field, source))
                {
                    return Some(field);
                }
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) =
            first_class_constant_assignment(child, source, class_name, constant_props)
        {
            return Some(found);
        }
    }
    None
}

/// The assignment `obj.F = ...` that MATLAB reports for MCVM: the first one after
/// which `obj` is never read on the same path. Probed: with `obj.X = x;
/// obj.Y = 2;` the second write is reported (the second write reads `obj`), and
/// with `if c, obj.X = 1; else, obj.X = 2; end` only the first branch is
/// reported, because the other branch is not a read on the same path.
fn first_unread_object_assignment<'a>(
    body: Node<'a>,
    source: &str,
    info: &MethodInfo,
) -> Option<Node<'a>> {
    let obj = info.first_input.as_deref()?;
    let mut writes = Vec::new();
    collect_object_assignments(body, source, info, &mut writes);
    writes
        .into_iter()
        .find(|write| !object_read_after(body, *write, obj, source))
}

/// Every assignment that writes a field of the method's object (`obj.X = ...`),
/// in source order.
fn collect_object_assignments<'a>(
    node: Node<'a>,
    source: &str,
    info: &MethodInfo,
    out: &mut Vec<Node<'a>>,
) {
    if node.kind() == "function_definition" || node.kind() == "class_definition" {
        return;
    }
    if node.kind() == "assignment" {
        if let Some(left) = node.child_by_field_name("left") {
            // Probed: an indexed write `obj.X(2) = x` is not reported.
            let indexed = node_text(left, source).contains(['(', '{']);
            if left.kind() == "field_expression"
                && !indexed
                && is_first_input_field_access(left, source, info)
            {
                out.push(node);
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            collect_object_assignments(child, source, info, out);
        }
    }
}

/// Whether `obj` is used after `write` in `body`, on a path that reaches the
/// end of the write: a later use in a sibling `elseif`/`else`/`case` branch of
/// the write's own branch does not count.
fn object_read_after(body: Node, write: Node, obj: &str, source: &str) -> bool {
    let mut uses = Vec::new();
    collect_identifiers(body, obj, source, &mut uses);
    // A read in a loop that holds the write runs again after it, in the next iteration
    // (probed: `for k, disp(obj), obj.x = v; end` is silent). The write's own `obj` is not a read.
    let loops = enclosing_loops(write, body);
    uses.into_iter().any(|u| {
        !contains(write, u)
            && !in_sibling_branch(u, write)
            && (u.start_byte() >= write.end_byte() || loops.iter().any(|l| contains(*l, u)))
    })
}

/// The loops that hold `node` within `body`, innermost first.
fn enclosing_loops<'a>(node: Node<'a>, body: Node<'a>) -> Vec<Node<'a>> {
    let mut loops = Vec::new();
    let mut ancestor = node.parent();
    while let Some(a) = ancestor {
        if a.id() == body.id() {
            break;
        }
        if matches!(a.kind(), "for_statement" | "while_statement") {
            loops.push(a);
        }
        ancestor = a.parent();
    }
    loops
}

/// Every identifier named `name` under `node`, in source order.
fn collect_identifiers<'a>(node: Node<'a>, name: &str, source: &str, out: &mut Vec<Node<'a>>) {
    if node.kind() == "identifier" && node_text(node, source) == name {
        out.push(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_identifiers(child, name, source, out);
    }
}

/// Whether `node` lies in a different branch of an `if` or `switch` statement
/// than `write` does.
fn in_sibling_branch(node: Node, write: Node) -> bool {
    let mut ancestor = write.parent();
    while let Some(stmt) = ancestor {
        if matches!(stmt.kind(), "if_statement" | "switch_statement") && contains(stmt, node) {
            if let (Some(a), Some(b)) = (
                branch_containing(stmt, write),
                branch_containing(stmt, node),
            ) {
                if a.id() != b.id() && is_branch(a) && is_branch(b) {
                    return true;
                }
            }
        }
        ancestor = stmt.parent();
    }
    false
}

fn contains(outer: Node, inner: Node) -> bool {
    outer.start_byte() <= inner.start_byte() && inner.end_byte() <= outer.end_byte()
}

/// The direct child of `stmt` that contains `node`.
fn branch_containing<'a>(stmt: Node<'a>, node: Node) -> Option<Node<'a>> {
    let mut cursor = stmt.walk();
    let found = stmt.children(&mut cursor).find(|c| contains(*c, node));
    found
}

/// Whether a child of an `if` or `switch` is one of its branches.
fn is_branch(node: Node) -> bool {
    node.kind() == "block" || node.kind().ends_with("_clause")
}

/// Check whether a `field_expression` accesses a field of the method's object
/// (`obj.X` where `obj` is the first input argument).
fn is_object_field_access(fe: Node, source: &str, info: &MethodInfo) -> bool {
    // Probed: a static method has no object, so `T.value` on its input is silent.
    if info.is_static {
        return false;
    }
    let Some(obj) = fe.child_by_field_name("object") else {
        return false;
    };
    if obj.kind() != "identifier" {
        return false;
    }
    info.object
        .as_deref()
        .map(|object| node_text(obj, source) == object)
        .unwrap_or(false)
}

/// Whether `fe` is a field access on the first non-ignored input (the object of a
/// method, but not the output a constructor creates). MCVM reads its object so.
fn is_first_input_field_access(fe: Node, source: &str, info: &MethodInfo) -> bool {
    if info.is_static {
        return false;
    }
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

/// The identifier naming a member in a field access: `X` in `obj.X`, and also in
/// `obj.X(i)` and `obj.X{i}`, which the grammar parses as a call on the field.
fn member_name_node(field: Node) -> Option<Node> {
    match field.kind() {
        "identifier" => Some(field),
        "function_call" => field
            .child_by_field_name("name")
            .filter(|name| name.kind() == "identifier"),
        _ => None,
    }
}

/// Walk a method body emitting MCNPN / MCNPR / MCSUP diagnostics.
fn check_method_body(
    eng: &GoodPracticesEngine,
    node: Node,
    source: &str,
    info: &MethodInfo,
    ctx: &ClassCtx,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "function_definition" || node.kind() == "class_definition" {
        return;
    }

    // MCNPN: member access on the object whose name is not declared. Silent in a
    // class with a signature-only declaration (MATLAB's behaviour, probed), and
    // in a class with a superclass other than `handle` (probed).
    if eng.is_check_enabled("MCNPN")
        && ctx.members_known
        && !ctx.has_signature_only
        && node.kind() == "field_expression"
        && !is_assignment_target(node)
        && is_object_field_access(node, source, info)
    {
        // `obj.X(i)` and `obj.X{i}` parse as a call on the field: the name is in it.
        if let Some(field) = node.child_by_field_name("field").and_then(member_name_node) {
            if field.kind() == "identifier" {
                let name = node_text(field, source);
                if !ctx.declared.contains(name) {
                    let pos = field.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "MCNPN",
                        message: format!(
                            "'{name}' is referenced but is not a property, method, or event name defined in this class."
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
    if eng.is_check_enabled("MCNPR") && ctx.members_known && node.kind() == "assignment" {
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
                                    "'{name}' is not a property, but is the target of an assignment."
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

    // MCSUP: a setter accesses a property other than the one it sets. Every
    // access is reported (reads too); constant properties are not (probed).
    if eng.is_check_enabled("MCSUP")
        && info.is_setter
        && !ctx.dependent_props.contains(info.base_name.as_str())
        && node.kind() == "field_expression"
        && is_object_field_access(node, source, info)
    {
        if let Some(name_node) = node.child_by_field_name("field").and_then(member_name_node) {
            let name = node_text(name_node, source);
            if ctx.prop_names.contains(name)
                && !ctx.constant_props.contains(name)
                && name != info.base_name
            {
                let pos = name_node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "MCSUP",
                    message: format!(
                        "The set method for the property '{}' should not access another property ('{}').",
                        info.base_name, name
                    ),
                    severity: Severity::Warning,
                    byte_range: name_node.start_byte()..name_node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            check_method_body(eng, child, source, info, ctx, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Probed: a static method has no object, so `T.value` on its input is silent.
    #[test]
    fn mcnpn_silent_on_input_field_in_static_method() {
        let source = "classdef mc\n    properties\n        Data = []\n    end\n    methods (Static)\n        function obj = fromTable(T)\n            obj = mc(T.value);\n        end\n    end\n    methods\n        function obj = mc(a)\n            obj.Data = a;\n        end\n    end\nend\n";
        assert!(
            oop_ids_for(source, "MCNPN").is_empty(),
            "got: {:?}",
            oop_ids(source)
        );
    }

    // Probed: `obj.Nope` on the object argument of a method is MCNPN.
    #[test]
    fn mcnpn_fires_on_undeclared_member_of_object_argument() {
        let source = "classdef mc\n    properties\n        Data = []\n    end\n    methods\n        function r = get(obj, T)\n            r = T.value;\n            r = obj.Nope;\n        end\n    end\nend\n";
        assert_eq!(
            oop_ids_for(source, "MCNPN").len(),
            1,
            "got: {:?}",
            oop_ids(source)
        );
    }

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
    fn test_mcnpn_fires_on_abstract_method_with_body() {
        // MATLAB reports this (probe K2).
        let source = "classdef Foo\n    methods (Abstract)\n        function f2(obj)\n            x = obj.undefinedThing;\n        end\n    end\nend\n";
        let ids = oop_ids_for(source, "MCNPN");
        assert_eq!(ids.len(), 1, "got: {:?}", oop_ids(source));
    }

    #[test]
    fn test_mcnpn_silent_when_abstract_block_has_signature_only() {
        // MATLAB is silent for the body method once a signature-only line is in the class (probe K8).
        let source = "classdef Foo\n    methods (Abstract)\n        f2(obj)\n        function g(obj)\n            y = obj.foo;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCNPN").is_empty());
    }

    #[test]
    fn test_mcnpn_silent_when_plain_block_has_signature_only() {
        // Probe L4: the signature-only line is in a plain methods block.
        let source = "classdef Foo\n    methods\n        function g(obj)\n            y = obj.foo;\n        end\n        f2(obj)\n    end\nend\n";
        assert!(oop_ids_for(source, "MCNPN").is_empty());
    }

    #[test]
    fn test_mcnpn_silent_in_class_with_signature_only_setter_and_getter() {
        // Shape of ls-c__V1.m: `v.b` in a method is silent because of the
        // `set.k(obj, v)` / `get.k2(obj)` signatures later in the class.
        let source = "classdef Foo\n    methods\n        function validateW(v)\n            s = struct('a', v.b);\n        end\n    end\n    methods (Abstract)\n        set.k(obj, v)\n        v = get.k2(obj)\n    end\nend\n";
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
    fn test_mccsps_fires_on_class_qualified_assignment() {
        // Probed: `ClassName.K = 5;` in a method gives MCCSPS at K (not MCCSOP).
        let source = "classdef Foo\n    properties (Constant)\n        K = 1\n    end\n    methods\n        function m(obj)\n            Foo.K = 5;\n        end\n    end\nend\n";
        let ids = oop_ids_for(source, "MCCSPS");
        assert_eq!(ids.len(), 1, "got: {:?}", oop_ids(source));
        assert!(oop_ids_for(source, "MCCSOP").is_empty());
    }

    #[test]
    fn test_mccsps_fires_once_per_function_and_in_constructors() {
        // Probed: the first `Foo.K = ...` of a function is reported; the next one
        // uses the struct that the first created. The constructor is no exception.
        let source = "classdef Foo\n    properties (Constant)\n        K = 1\n    end\n    methods\n        function obj = Foo()\n            Foo.K = 5;\n            Foo.K = 6;\n        end\n        function m(obj)\n            Foo.K(2) = 5;\n        end\n    end\nend\n";
        assert_eq!(
            oop_ids_for(source, "MCCSPS").len(),
            2,
            "got: {:?}",
            oop_ids(source)
        );
    }

    #[test]
    fn test_mccsps_silent_on_other_object_and_non_constant() {
        // `other.K` is a new struct, not the class (probed: silent); `Foo.Y` is not
        // a constant property.
        let source = "classdef Foo\n    properties (Constant)\n        K = 1\n    end\n    properties\n        Y = 1\n    end\n    methods\n        function m(obj)\n            other.K = 5;\n            Foo.Y = 5;\n        end\n    end\nend\n";
        assert!(
            oop_ids_for(source, "MCCSPS").is_empty(),
            "got: {:?}",
            oop_ids(source)
        );
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
    fn test_mcsup_reports_every_access_with_quoted_names() {
        // Probe K6: each access on its own line is reported, not once per name.
        let source = "classdef Foo < handle\n    properties\n        X\n        Y\n    end\n    methods\n        function set.X(obj, v)\n            a = obj.Y;\n            b = obj.Y + 1;\n            obj.X = v;\n        end\n    end\nend\n";
        let tree = parse(source);
        let diags: Vec<_> = engine()
            .check_oop_practice(&tree, source)
            .into_iter()
            .filter(|d| d.rule_id == "MCSUP")
            .collect();
        assert_eq!(diags.len(), 2, "got: {:?}", oop_ids(source));
        assert_eq!(
            diags[0].message,
            "The set method for the property 'X' should not access another property ('Y')."
        );
    }

    #[test]
    fn test_mcsup_fires_when_setter_reads_other_property_with_superclass() {
        // Probe K2 and the hT/hU shapes: MCSUP does not depend on the superclass.
        let source = "classdef Foo < Bar\n    properties\n        X\n        Y\n    end\n    methods\n        function set.X(obj, v)\n            t = obj.Y;\n            obj.X = v;\n        end\n    end\nend\n";
        assert_eq!(oop_ids_for(source, "MCSUP").len(), 1);
    }

    #[test]
    fn test_mcsup_fires_on_indexed_property_access() {
        // Probe K1 (c_config_class.m:47): `obj.History{end+1} = ...` in a setter.
        let source = "classdef Foo < handle\n    properties\n        X\n        History = {}\n    end\n    methods\n        function set.X(obj, v)\n            obj.History{end+1} = v;\n        end\n    end\nend\n";
        let tree = parse(source);
        let diags: Vec<_> = engine()
            .check_oop_practice(&tree, source)
            .into_iter()
            .filter(|d| d.rule_id == "MCSUP")
            .collect();
        assert_eq!(diags.len(), 1, "got: {:?}", oop_ids(source));
        assert_eq!((diags[0].line, diags[0].column), (8, 17));
        assert!(diags[0].message.ends_with("('History')."));
    }

    #[test]
    fn test_mcsup_silent_on_constant_property() {
        // Probe K5: reading a Constant property from a setter is not reported.
        let source = "classdef Foo < handle\n    properties (Constant)\n        Version = '1'\n    end\n    properties\n        X\n    end\n    methods\n        function set.X(obj, v)\n            t = obj.Version;\n            obj.X = v;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCSUP").is_empty());
    }

    #[test]
    fn test_mcsup_silent_in_setter_of_dependent_property() {
        // Dogfooding (plan 6.2): MATLAB is silent when the set method belongs to a
        // Dependent property (with or without a get method, with or without an output).
        let source = "classdef Foo\n    properties (Dependent)\n        V\n    end\n    properties (Access = private)\n        P\n    end\n    methods\n        function obj = set.V(obj, v)\n            obj.P = v;\n        end\n        function v = get.V(obj)\n            v = obj.P;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCSUP").is_empty());
    }

    #[test]
    fn test_mcsup_silent_in_getter() {
        // Probe K8: only setters are checked.
        let source = "classdef Foo < handle\n    properties\n        X\n        Y\n    end\n    methods\n        function v = get.X(obj)\n            v = obj.Y;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCSUP").is_empty());
    }

    #[test]
    fn test_mcnpn_fires_in_handle_class() {
        // Probe A: `handle` alone keeps the member check on.
        let source = "classdef Foo < handle\n    methods\n        function bad(obj)\n            y = obj.NoSuchMember;\n        end\n    end\nend\n";
        assert_eq!(oop_ids_for(source, "MCNPN").len(), 1);
    }

    #[test]
    fn test_mcnpn_silent_with_superclass_other_than_handle() {
        // Probes C, D, E, F, G, P: a superclass that resolves (handle or value)
        // or not, and handle combined with another superclass, all silence MCNPN.
        for superclass in ["Bar", "handle & Bar", "matlab.mixin.SetGet"] {
            let source = format!(
                "classdef Foo < {superclass}\n    methods\n        function bad(obj)\n            y = obj.NoSuchMember;\n        end\n    end\nend\n"
            );
            assert!(
                oop_ids_for(&source, "MCNPN").is_empty(),
                "superclass {superclass}: got {:?}",
                oop_ids(&source)
            );
        }
    }

    #[test]
    fn test_mcnpr_fires_in_handle_class_only_when_members_known() {
        // Probes S (handle fires) and T, U, W (other superclasses silence MCNPR).
        let handle = "classdef Foo < handle\n    methods\n        function bad(obj)\n            obj.Undecl = 1;\n        end\n    end\nend\n";
        assert_eq!(oop_ids_for(handle, "MCNPR").len(), 1);
        let other = "classdef Foo < Bar\n    methods\n        function bad(obj)\n            obj.Undecl = 1;\n        end\n    end\nend\n";
        assert!(oop_ids_for(other, "MCNPR").is_empty());
    }

    #[test]
    fn test_mcvm_silent_with_superclass() {
        // Probes C, D, E: a class with a superclass is not reported as a value class.
        let source = "classdef Foo < Bar\n    properties\n        X\n    end\n    methods\n        function bump(obj, v)\n            obj.X = v;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCVM").is_empty());
    }

    /// Source of a value class `Foo` with property `X` and one method body.
    fn value_method(signature: &str, body: &str) -> String {
        format!(
            "classdef Foo\n    properties\n        X (1,1) double = 0\n        Y (1,1) double = 0\n    end\n    methods\n        function {signature}\n{body}        end\n    end\nend\n"
        )
    }

    #[test]
    fn test_mcvm_fires_when_output_is_not_assigned() {
        // Probed (l_vector2.m:75): `function r = withX(obj, x)` with `r` never set.
        let source = value_method("r = withX(obj, x)", "            obj.X = x;\n");
        assert_eq!(
            oop_ids_for(&source, "MCVM").len(),
            1,
            "got: {:?}",
            oop_ids(&source)
        );
    }

    #[test]
    fn test_mcvm_fires_when_output_is_another_value() {
        // Probed: `r = []` and `r = 1` are not the object.
        for body in [
            "            obj.X = x;\n            r = [];\n",
            "            obj.X = x;\n            r = 1;\n",
        ] {
            let source = value_method("r = withX(obj, x)", body);
            assert_eq!(
                oop_ids_for(&source, "MCVM").len(),
                1,
                "{body}: {:?}",
                oop_ids(&source)
            );
        }
    }

    #[test]
    fn test_mcvm_fires_on_write_in_if_and_before_return() {
        // Probed: a write inside `if`, and a write before an early `return`.
        let source = value_method(
            "withX(obj, x)",
            "            if x > 0\n                obj.X = x;\n            end\n",
        );
        assert_eq!(
            oop_ids_for(&source, "MCVM").len(),
            1,
            "got: {:?}",
            oop_ids(&source)
        );
        let source = value_method(
            "r = withX(obj, x)",
            "            obj.X = x;\n            if x > 0\n                r = 1;\n                return\n            end\n            r = 2;\n",
        );
        assert_eq!(
            oop_ids_for(&source, "MCVM").len(),
            1,
            "got: {:?}",
            oop_ids(&source)
        );
    }

    #[test]
    fn test_mcvm_reports_only_the_write_nobody_reads() {
        // Probed: `obj.X = x; obj.Y = 2;` reports one write, the second.
        let source = value_method(
            "withX(obj, x)",
            "            obj.X = x;\n            obj.Y = 2;\n",
        );
        assert_eq!(
            oop_ids_for(&source, "MCVM").len(),
            1,
            "got: {:?}",
            oop_ids(&source)
        );
        // Probed: the first branch of an if/else is reported once.
        let source = value_method(
            "r = withX(obj, x)",
            "            if x > 0\n                obj.X = x;\n            else\n                obj.X = -x;\n            end\n            r = 1;\n",
        );
        assert_eq!(
            oop_ids_for(&source, "MCVM").len(),
            1,
            "got: {:?}",
            oop_ids(&source)
        );
    }

    #[test]
    fn test_mcvm_silent_when_object_is_read_after_write() {
        // Probed: `obj.X = x; r = obj.X;`, `numel(obj)`, `q = obj`, `if obj.X > 0`.
        for body in [
            "            obj.X = x;\n            r = obj.X;\n",
            "            obj.X = x;\n            r = numel(obj);\n",
            "            obj.X = x;\n            q = obj;\n            r = 1;\n",
            "            obj.X = x;\n            if obj.X > 0\n                r = 1;\n            end\n",
        ] {
            let source = value_method("r = withX(obj, x)", body);
            assert!(oop_ids_for(&source, "MCVM").is_empty(), "{body}: {:?}", oop_ids(&source));
        }
    }

    #[test]
    fn test_mcvm_silent_when_method_returns_object_or_writes_an_element() {
        // Probed: an output named like the object returns it; `obj.X(2) = x` is not reported.
        let source = value_method("obj = withX(obj, x)", "            obj.X = x;\n");
        assert!(
            oop_ids_for(&source, "MCVM").is_empty(),
            "got: {:?}",
            oop_ids(&source)
        );
        let source = value_method("r = withX(obj, x)", "            obj.X(2) = x;\n");
        assert!(
            oop_ids_for(&source, "MCVM").is_empty(),
            "got: {:?}",
            oop_ids(&source)
        );
    }

    #[test]
    fn test_mcsnov_silent_with_superclass() {
        // Probes X, Y: a setter without an output is silent once there is a superclass.
        let source = "classdef Foo < Bar\n    properties\n        X\n    end\n    methods\n        function set.X(obj, v)\n            obj.X = v;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCSNOV").is_empty());
    }

    #[test]
    fn test_mcsoh_fires_with_handle_among_superclasses() {
        // Probe O2: MCSOH keys on `handle` being listed, even with another superclass.
        let source = "classdef Foo < handle & Bar\n    properties\n        X\n    end\n    methods\n        function obj = set.X(obj, v)\n            obj.X = v;\n        end\n    end\nend\n";
        assert_eq!(oop_ids_for(source, "MCSOH").len(), 1);
    }

    #[test]
    fn test_mcsoh_silent_on_sibling_superclass_without_handle() {
        // Probe O3: a superclass named by a sibling file (not `handle`) is silent.
        let source = "classdef Foo < Bar\n    properties\n        X\n    end\n    methods\n        function obj = set.X(obj, v)\n            obj.X = v;\n        end\n    end\nend\n";
        assert!(oop_ids_for(source, "MCSOH").is_empty());
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

    #[test]
    fn test_mcnpn_fires_on_brace_and_paren_reads_of_undeclared_members() {
        // Probed: `obj.Undecl{1}` and `obj.Undecl(1)` both give MCNPN at the name.
        let source = "classdef Foo\n    properties\n        X\n    end\n    methods\n        function r = f(obj)\n            r = obj.Undecl{1};\n            r = obj.Undecl(1);\n        end\n    end\nend\n";
        assert_eq!(
            oop_ids_for(source, "MCNPN").len(),
            2,
            "got: {:?}",
            oop_ids(source)
        );
    }

    #[test]
    fn test_mcnpr_fires_in_constructor_on_the_created_object() {
        // Probed: a constructor's object is its output (`obj`), so `obj.Z = a` is
        // MCNPR there too; `obj.X = a` for a property is silent.
        let source = "classdef Foo\n    properties\n        X\n    end\n    methods\n        function obj = Foo(a)\n            obj.Z = a;\n            obj.X = a;\n        end\n    end\nend\n";
        assert_eq!(
            oop_ids_for(source, "MCNPR").len(),
            1,
            "got: {:?}",
            oop_ids(source)
        );
    }

    #[test]
    fn test_mcnpn_fires_in_constructor_on_undeclared_read() {
        // Probed: MCNPN on `z = obj.Foo2` in a constructor.
        let source = "classdef Foo\n    properties\n        X\n    end\n    methods\n        function obj = Foo(a)\n            obj.X = a;\n            z = obj.Foo2;\n            disp(z)\n        end\n    end\nend\n";
        assert_eq!(
            oop_ids_for(source, "MCNPN").len(),
            1,
            "got: {:?}",
            oop_ids(source)
        );
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
    fn mcsnov_fires_on_an_empty_setter_that_does_not_return_the_object() {
        // Probed: an empty `set.Y(obj, v)` (49) and `r = set.ans(obj, v)` (93) are MCSNOV;
        // `obj = set.Data(obj, v)` is silent (97).
        let empty = "classdef C\n properties (Dependent)\n  Y\n end\n methods\n  function set.Y(obj, v)\n  end\n end\nend\n";
        let returns = "classdef C\n properties\n  Data\n end\n methods\n  function obj = set.Data(obj, v)\n   obj.Data = v;\n  end\n end\nend\n";
        assert_eq!(lines(empty, "MCSNOV"), vec![6]);
        assert!(lines(returns, "MCSNOV").is_empty());
    }

    #[test]
    fn mcnpn_leaves_an_assigned_unknown_member_to_mcnpr() {
        // Probed: `obj.Missing = 1` (105) is MCNPR only; `r = obj.Missing;` (104) is MCNPN.
        let assign = "classdef C\n properties\n  Data\n end\n methods\n  function obj = m(obj)\n   obj.Missing = 1;\n  end\n end\nend\n";
        let read = "classdef C\n properties\n  Data\n end\n methods\n  function r = m(obj)\n   r = obj.Missing;\n  end\n end\nend\n";
        assert!(lines(assign, "MCNPN").is_empty());
        assert_eq!(lines(assign, "MCNPR"), vec![7]);
        assert_eq!(lines(read, "MCNPN"), vec![7]);
    }
}
