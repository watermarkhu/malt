//! The named things a file declares, shared by the naming checks.

use std::collections::HashSet;
use std::ops::Range;

use tree_sitter::Node;

/// What a declared name is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A class (`classdef Name`).
    Class,
    /// A function, local function, nested function or method.
    Function,
    /// A variable: an input, output, assignment target, loop variable, `global`,
    /// `persistent` or `catch` name. Reported once per scope, at its first declaration.
    Variable,
    /// A property in a `properties` block.
    Property,
    /// A field of a struct or object that an assignment writes (`s.name = 1`).
    Field,
}

/// One declared name.
#[derive(Debug, Clone)]
pub(crate) struct Decl<'a> {
    pub kind: Kind,
    pub name: &'a str,
    pub range: Range<usize>,
    /// For a function: it is a method of a class.
    pub is_method: bool,
    /// For a property: the class name. For a field: the name before it in the chain
    /// (`s` in `s.name`), when that is a plain name.
    pub parent: Option<&'a str>,
}

/// The text of a node.
pub(crate) fn text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// Every name the file declares, in source order.
pub(crate) fn declarations<'a>(root: Node<'a>, source: &'a str) -> Vec<Decl<'a>> {
    let mut state = State {
        source,
        seen: HashSet::new(),
        out: Vec::new(),
    };
    visit(root, &mut state, root.id(), None, false);
    state.out
}

struct State<'a> {
    source: &'a str,
    seen: HashSet<(usize, &'a str)>,
    out: Vec<Decl<'a>>,
}

impl<'a> State<'a> {
    fn push(&mut self, kind: Kind, node: Node<'a>, is_method: bool, parent: Option<&'a str>) {
        self.out.push(Decl {
            kind,
            name: text(node, self.source),
            range: node.byte_range(),
            is_method,
            parent,
        });
    }

    fn variable(&mut self, node: Node<'a>, scope: usize) {
        let name = text(node, self.source);
        if self.seen.insert((scope, name)) {
            self.push(Kind::Variable, node, false, None);
        }
    }
}

fn visit<'a>(
    node: Node<'a>,
    state: &mut State<'a>,
    scope: usize,
    class: Option<&'a str>,
    in_methods: bool,
) {
    let mut scope = scope;
    let mut class = class;
    let mut in_methods = in_methods;
    match node.kind() {
        "function_definition" => {
            if let Some(name) = node.child_by_field_name("name") {
                let is_constructor = in_methods && class == Some(text(name, state.source));
                if name.kind() == "identifier" && !is_constructor {
                    state.push(Kind::Function, name, in_methods, None);
                }
            }
            scope = node.id();
            in_methods = false;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "function_arguments" | "function_output" => {
                        declare_names(child, state, scope);
                    }
                    _ => visit(child, state, scope, class, in_methods),
                }
            }
            return;
        }
        "class_definition" => {
            if let Some(name) = node.child_by_field_name("name") {
                state.push(Kind::Class, name, false, None);
                class = Some(text(name, state.source));
            }
        }
        "methods" => in_methods = true,
        "properties" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "property" {
                    if let Some(name) = child.child_by_field_name("name") {
                        if name.kind() == "identifier" {
                            state.push(Kind::Property, name, false, class);
                        }
                    }
                }
            }
            return;
        }
        // Its names are the function's inputs, declared with the signature.
        "arguments_statement" | "lambda" => return,
        "assignment" => {
            if let Some(left) = node.child_by_field_name("left") {
                declare_target(left, state, scope);
            }
        }
        "for_statement" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "iterator" {
                    if let Some(variable) = child.named_child(0) {
                        if variable.kind() == "identifier" {
                            state.variable(variable, scope);
                        }
                    }
                }
            }
        }
        "global_operator" | "persistent_operator" | "catch_clause" => {
            declare_names(node, state, scope);
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, state, scope, class, in_methods);
    }
}

/// Declare the identifiers that are direct children of `node`, and those in a
/// multi-output group.
fn declare_names<'a>(node: Node<'a>, state: &mut State<'a>, scope: usize) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => state.variable(child, scope),
            "multioutput_variable" => declare_names(child, state, scope),
            _ => {}
        }
    }
}

/// The name node of a chain component: the identifier itself, or the name of an
/// indexing or call (`a` in `a(2)`).
fn component_name(node: Node) -> Option<Node> {
    match node.kind() {
        "identifier" => Some(node),
        "function_call" => node
            .child_by_field_name("name")
            .filter(|name| name.kind() == "identifier"),
        _ => None,
    }
}

/// Declare what an assignment target writes: a variable, the entries of a
/// multi-output group, or the base variable and fields of `s.a.b`.
fn declare_target<'a>(target: Node<'a>, state: &mut State<'a>, scope: usize) {
    match target.kind() {
        "identifier" => state.variable(target, scope),
        "multioutput_variable" => {
            let mut cursor = target.walk();
            for child in target.named_children(&mut cursor) {
                declare_target(child, state, scope);
            }
        }
        "function_call" => {
            if let Some(name) = component_name(target) {
                state.variable(name, scope);
            }
        }
        "field_expression" => {
            let mut cursor = target.walk();
            let mut parent: Option<&'a str> = None;
            for (index, component) in target.named_children(&mut cursor).enumerate() {
                let name = component_name(component);
                if let Some(name) = name {
                    if index == 0 {
                        state.variable(name, scope);
                    } else {
                        state.push(Kind::Field, name, false, parent);
                    }
                }
                parent = name.map(|n| text(n, state.source));
            }
        }
        _ => {}
    }
}
