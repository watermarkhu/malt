use super::*;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

/// One property name that appears bare (not as `obj.name`) in a method body.
pub(crate) struct NameUses {
    /// The property name.
    pub(crate) name: String,
    /// Every bare occurrence of the name in the method, in source order.
    pub(crate) uses: Vec<NameUse>,
    /// Whether the method uses the name as a variable: a store (`x = `, `x(2) = `,
    /// `x.f = `, `[x, y] = `, `for x = `) or a brace index (`x{1}`). MATLAB then
    /// reports every use as PROP; otherwise a bare use refers to the property (CPROP).
    pub(crate) is_variable: bool,
    /// Whether the method copies the property into a variable of the same name, or
    /// writes it from one (`x = obj.x;`, `obj.x = x;`). PROP is silent there (probed).
    pub(crate) copy_or_write: bool,
}

/// One bare occurrence of a property name.
pub(crate) struct NameUse {
    /// Byte range of the identifier.
    pub(crate) byte_range: Range<usize>,
    /// 1-indexed line.
    pub(crate) line: usize,
    /// 1-indexed column.
    pub(crate) column: usize,
}

/// Whether the class derives from an App Designer base class. MATLAB reports
/// ADPROP there instead of PROP and CPROP (probed).
pub(crate) fn is_app_class(tree: &tree_sitter::Tree, source: &str) -> bool {
    FileMeta::build(tree, source)
        .class
        .as_ref()
        .is_some_and(|c| c.superclasses.iter().any(|s| s.contains("matlab.apps")))
}

/// The bare uses of each property name in each non-static method of the class.
/// A name that is also an input or output of the method, or a method name, is
/// skipped (probed: input arguments named like a property are silent).
pub(crate) fn property_name_uses(tree: &tree_sitter::Tree, source: &str) -> Vec<NameUses> {
    let meta = FileMeta::build(tree, source);
    if meta.class.is_none() {
        return Vec::new();
    }
    let props: HashSet<&str> = meta
        .all_properties()
        .iter()
        .map(|p| p.name.as_str())
        .collect();
    if props.is_empty() {
        return Vec::new();
    }
    let method_names: HashSet<&str> = meta.all_methods().iter().map(|m| m.name.as_str()).collect();
    let mut result = Vec::new();
    for method in meta.all_methods() {
        // Probed: a `methods (Static)` method has no object.
        if is_static_method(tree, source, method.byte_range.start) {
            continue;
        }
        let Some(node) = function_node(tree, method.byte_range.clone()) else {
            continue;
        };
        let body = &source[method.byte_range.clone()];
        // Only the body: the signature names the method (`set.Data`) and its arguments.
        let Some(block) = find_child_of_kind(node, "block") else {
            continue;
        };
        let mut found: HashMap<String, Vec<tree_sitter::Node>> = HashMap::new();
        collect_bare_names(block, source, &props, &mut found);
        for (name, mut nodes) in found {
            let is_param = method
                .inputs
                .iter()
                .chain(method.outputs.iter())
                .any(|v| v.as_str() == name);
            if is_param || method_names.contains(name.as_str()) {
                continue;
            }
            nodes.sort_by_key(|n| n.start_byte());
            let copy_or_write = method.inputs.iter().filter(|r| r.as_str() != "~").any(|r| {
                body.contains(&format!("{name} = {r}.{name};"))
                    || body.contains(&format!("{r}.{name} = {name};"))
            });
            let is_variable = nodes
                .iter()
                .any(|n| is_store(*n) || source.as_bytes().get(n.end_byte()) == Some(&b'{'));
            result.push(NameUses {
                name,
                uses: nodes
                    .iter()
                    .map(|n| NameUse {
                        byte_range: n.byte_range(),
                        line: n.start_position().row + 1,
                        column: n.start_position().column + 1,
                    })
                    .collect(),
                is_variable,
                copy_or_write,
            });
        }
    }
    result
}

impl GoodPracticesEngine {
    /// PROP: a variable in a method that has the same name as a property of the
    /// class, which makes `name` and `obj.name` easy to mix up. MATLAB reports every
    /// bare use of a name that the method stores to or indexes with braces, including
    /// reads before the first store (probed).
    pub(crate) fn check_prop_validation(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        if !self.is_check_enabled("PROP") || is_app_class(tree, source) {
            return Vec::new();
        }
        property_name_uses(tree, source)
            .iter()
            .filter(|n| n.is_variable && !n.copy_or_write)
            .flat_map(|n| n.uses.iter().map(|u| prop_diagnostic(&n.name, u)))
            .collect()
    }
}

/// Collect every identifier named like a property under `node`, except the field
/// of an `obj.name` access.
fn collect_bare_names<'a>(
    node: tree_sitter::Node<'a>,
    source: &str,
    props: &HashSet<&str>,
    out: &mut HashMap<String, Vec<tree_sitter::Node<'a>>>,
) {
    if node.kind() == "identifier" {
        let text = node_text(node, source);
        // `obj.name` is a field, whether the grammar parses it as a field expression or as
        // a call such as `obj.name(1)`.
        let is_field = source[..node.start_byte()].trim_end().ends_with('.')
            || node.parent().is_some_and(|p| {
                p.kind() == "field_expression"
                    && p.child_by_field_name("field")
                        .is_some_and(|f| f.id() == node.id())
            });
        if props.contains(text) && !is_field {
            out.entry(text.to_string()).or_default().push(node);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_bare_names(child, source, props, out);
    }
}

/// Whether the identifier is the base of a store: the target of an assignment
/// (`x = `, `x(2) = `, `x.f = `, `[x, y] = `) or a `for` iterator.
fn is_store(name: tree_sitter::Node) -> bool {
    if is_for_iterator(name) {
        return true;
    }
    let mut current = name;
    while let Some(parent) = current.parent() {
        match parent.kind() {
            "assignment" => {
                return parent.child_by_field_name("left").is_some_and(|l| {
                    l.start_byte() <= name.start_byte() && name.end_byte() <= l.end_byte()
                })
            }
            "for_statement" => return true,
            "field_expression" | "function_call" => current = parent,
            _ => return false,
        }
    }
    false
}

/// Whether the identifier is the iterator of a `for` loop (before its body).
fn is_for_iterator(name: tree_sitter::Node) -> bool {
    let mut current = name;
    while let Some(parent) = current.parent() {
        if parent.kind() == "for_statement" {
            return find_child_of_kind(parent, "block")
                .is_none_or(|body| name.start_byte() < body.start_byte());
        }
        if matches!(parent.kind(), "block" | "function_definition") {
            return false;
        }
        current = parent;
    }
    false
}

/// The method's `function_definition` node for its byte range.
fn function_node(tree: &tree_sitter::Tree, range: Range<usize>) -> Option<tree_sitter::Node<'_>> {
    let mut node = tree
        .root_node()
        .descendant_for_byte_range(range.start, range.end)?;
    while node.kind() != "function_definition" {
        node = node.parent()?;
    }
    Some(node)
}

/// The PROP diagnostic for one bare use of a property name.
fn prop_diagnostic(name: &str, usage: &NameUse) -> Diagnostic {
    Diagnostic {
        rule_id: "PROP",
        message: format!(
            "'{name}' is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename this variable to improve readability."
        ),
        severity: Severity::Warning,
        byte_range: usage.byte_range.clone(),
        line: usage.line,
        column: usage.column,
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
    fn prop_reports_a_read_before_the_first_store() {
        // Probed: `r = Data;` (7) is PROP although `Data = 2` comes later (8).
        let src = "classdef C\n properties\n  Data\n end\n methods\n  function r = m(obj)\n   r = Data;\n   Data = 2;\n  end\n end\nend\n";
        assert_eq!(prop_lines(src), vec![7, 8]);
    }

    #[test]
    fn prop_reports_a_store_after_a_field_write_of_the_object() {
        // Probed: `obj.Data = 1;` is not a use, but `Data = 2;` and the read after it are.
        let src = "classdef C\n properties\n  Data\n end\n methods\n  function r = m(obj)\n   obj.Data = 1;\n   Data = 2;\n   r = Data;\n  end\n end\nend\n";
        assert_eq!(prop_lines(src), vec![8, 9]);
    }

    #[test]
    fn prop_reports_an_indexed_store_and_a_constant_property() {
        // Probed: `Color(2) = 1;` (7) is PROP; `K.a = 1; y = K;` on a constant (7, 8).
        let src = "classdef C\n properties\n  Color\n end\n methods\n  function m(obj)\n   Color(2) = 1;\n  end\n end\nend\n";
        assert_eq!(prop_lines(src), vec![7]);
        let src = "classdef C\n properties (Constant)\n  K = 1\n end\n methods\n  function y = m(obj)\n   K.a = 1;\n   y = K;\n  end\n end\nend\n";
        assert_eq!(prop_lines(src), vec![7, 8]);
    }

    #[test]
    fn prop_is_silent_in_app_designer_classes() {
        // Probed: App Designer reports ADPROP instead of PROP (ADPROP tests cover it).
        let src = "classdef C < matlab.apps.AppBase\n properties\n  Count = 0\n end\n methods\n  function go(app)\n   Count = 1;\n  end\n end\nend\n";
        assert!(prop_lines(src).is_empty());
    }

    #[test]
    fn prop_ignores_properties_themselves_and_other_names() {
        assert!(!prop("classdef C\n properties\n  a\n end\n methods\n  function m(obj)\n   b = obj.a;\n   disp(b);\n  end\n end\nend\n"));
        assert!(!prop("classdef C\n properties\n  a\n end\nend\n"));
    }
}
