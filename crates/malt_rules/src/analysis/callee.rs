//! # Callee resolution: what a call, command or handle refers to
//!
//! tree-sitter-matlab parses `f(x)` and `A(i)` to the same `function_call` node,
//! `f x` to a `command`, `@f` to a `handle_operator`, and `pkg.f` to a
//! `field_expression`. A trailing `pkg.f(x)` is a `function_call` whose `name`
//! is `f` and whose parent `field_expression` starts with the object or package.
//!
//! This module answers the question every rule asks of those nodes:
//!
//! - [`callee_path`] gives the name as written: `dsp.FIRFilter` for
//!   `dsp.FIRFilter(x)`, `format` for `format long`, `f` for `@f`.
//! - [`resolve`] also says what the name refers to in the file: a variable
//!   in scope, a local or nested function, a package member, or a plain
//!   function that nothing in the file defines.
//! - [`is_shadowed`] tells whether a variable or local function of that name
//!   takes precedence over the function a rule knows about.
//!
//! Where two rules read a call differently, the difference is an argument
//! ([`Dotted`], [`Visibility`]) and not a second helper.

use super::symbols::{is_qualified_call, DefKind, Scope, ScopeKind, SymbolTable};
use tree_sitter::Node;

/// The three shapes a callee takes in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CalleeKind {
    /// `f(x)`, `s.f(x)`, `pkg.f(x)`: a `function_call` node.
    Call,
    /// `f x`: a `command` node.
    Command,
    /// `@f`, `@pkg.f`: a `handle_operator` node.
    Handle,
}

/// The kind of callee `node` is, if it is one.
pub(crate) fn kind(node: Node) -> Option<CalleeKind> {
    match node.kind() {
        "function_call" => Some(CalleeKind::Call),
        "command" => Some(CalleeKind::Command),
        "handle_operator" => Some(CalleeKind::Handle),
        _ => None,
    }
}

/// How a `function_call` whose name is the trailing member of a dotted
/// expression (`f` in `s.f(x)` or `pkg.f(x)`) is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dotted {
    /// The whole path: `pkg.f` in `pkg.f(x)`, `s.f` in `s.f(x)`. Matches no
    /// package entry when `s` is a variable, because `s.f` is not a function.
    Whole,
    /// The trailing member alone: `f`. Used when the object in front does not
    /// matter, such as a call to a method that shares a function's name.
    Trailing,
    /// No name at all: a call on an object or a package is not a plain call.
    Skip,
}

/// Which variables take precedence over a function of the same name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Visibility {
    /// Only the variables of the scope the node sits in (its function, or its
    /// anonymous function).
    Enclosing,
    /// Also the variables of the file's root scope (a script's variables reach
    /// into anonymous functions, `f = @() isdir(p)`).
    WithScript,
}

/// What a callee's name refers to in the file. See [`resolve`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Callee<'a> {
    /// The name is a variable in scope: `x(2)` indexes `x`.
    Variable(&'a str),
    /// The name is a file-level function (the main function or a local function).
    Local(&'a str),
    /// The name is a function nested in an enclosing function.
    Nested(&'a str),
    /// A dotted path `pkg.f` or `comm.gpu.X` whose head is not a variable.
    Qualified(&'a str),
    /// A plain name that no definition in this file shadows.
    Function(&'a str),
    /// The node has no name we can read.
    Unknown,
}

/// The callee name of `node` as written, for a `function_call`, `command` or
/// `handle_operator`. `dotted` only matters for a call on an object or package.
pub(crate) fn callee_path<'a>(node: Node, source: &'a str, dotted: Dotted) -> Option<&'a str> {
    match kind(node)? {
        CalleeKind::Call => call_path(node, source, dotted),
        CalleeKind::Command => command_path(node, source),
        CalleeKind::Handle => handle_path(node, source),
    }
}

/// The name of a `function_call`, read as `dotted` says.
pub(crate) fn call_path<'a>(node: Node, source: &'a str, dotted: Dotted) -> Option<&'a str> {
    if node.kind() != "function_call" {
        return None;
    }
    let name = node.child_by_field_name("name")?;
    let name_text = &source[name.start_byte()..name.end_byte()];
    if !is_qualified_call(node) {
        return Some(name_text);
    }
    match dotted {
        Dotted::Whole => {
            let object = node.parent()?.named_child(0)?;
            Some(&source[object.start_byte()..name.end_byte()])
        }
        Dotted::Trailing => Some(name_text),
        Dotted::Skip => None,
    }
}

/// The command name of a `command` node: its `command_name` child.
pub(crate) fn command_path<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    if node.kind() != "command" {
        return None;
    }
    let name = node.child(0)?;
    (name.kind() == "command_name").then(|| &source[name.start_byte()..name.end_byte()])
}

/// The first word of a `command`'s text. Some readers take the command name
/// from the text rather than from the `command_name` child; this is that reading.
pub(crate) fn command_word<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    if node.kind() != "command" {
        return None;
    }
    source[node.start_byte()..node.end_byte()]
        .split_whitespace()
        .next()
}

/// The callee of a `function_call` or `command` as the liveness pass reads it:
/// the name `dotted` gives for a call, the first word of the text for a command.
pub(crate) fn callee_word<'a>(node: Node, source: &'a str, dotted: Dotted) -> Option<&'a str> {
    match kind(node)? {
        CalleeKind::Call => call_path(node, source, dotted),
        CalleeKind::Command => command_word(node, source),
        CalleeKind::Handle => None,
    }
}

/// The text of the `name` field of any node, whatever its kind. Only the
/// behaviour-property checks read it this way: they run on every node the
/// traversal visits, so a `function_definition` named `set` reads as a call to
/// `set` there. [`call_path`] is the reader for calls and does not do this.
pub(crate) fn name_field<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    let name = node.child_by_field_name("name")?;
    Some(&source[name.start_byte()..name.end_byte()])
}

/// The function a `handle_operator` refers to: `f` in `@f`, `pkg.f` in `@pkg.f`.
pub(crate) fn handle_path<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    (node.kind() == "handle_operator")
        .then(|| reference_name(node, source))
        .flatten()
}

/// The name of a function reference that is not called: `f` in `@f` and
/// `pkg.f` in a bare `pkg.f` statement. Anything that is not a plain or dotted
/// identifier (a call, an index) gives `None`.
pub(crate) fn reference_name<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    let text = source.get(node.start_byte()..node.end_byte())?;
    let name = match node.kind() {
        "handle_operator" => text.strip_prefix('@')?.trim_start(),
        _ => text,
    };
    let is_identifier = |part: &str| {
        let mut chars = part.chars();
        chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    };
    (!name.is_empty() && name.split('.').all(is_identifier)).then_some(name)
}

/// Whether `name` is a variable or a local function that takes precedence over
/// the function of that name at `node`. Dotted names are never shadowed.
pub(crate) fn is_shadowed(
    node: Node,
    name: &str,
    table: &SymbolTable,
    visibility: Visibility,
) -> bool {
    if name.contains('.') {
        return false;
    }
    variable_at(node, name, table, visibility) || local_function(table, name)
}

/// Whether `name` is a variable visible at `node` under `visibility`.
fn variable_at(node: Node, name: &str, table: &SymbolTable, visibility: Visibility) -> bool {
    table
        .scope_at(node.start_byte())
        .is_some_and(|scope| scope.is_defined(name))
        || (visibility == Visibility::WithScript && table.root_scope().is_defined(name))
}

/// Whether `name` is a variable (not a nested function name, which the symbol
/// table also records as a definition) at `node`, in the enclosing scope or the
/// script scope.
fn data_variable_at(node: Node, name: &str, table: &SymbolTable) -> bool {
    let is_data = |scope: &Scope| {
        scope
            .defs
            .iter()
            .any(|def| def.name == name && def.kind != DefKind::NestedFunction)
    };
    table.scope_at(node.start_byte()).is_some_and(is_data) || is_data(table.root_scope())
}

/// Whether the file defines a main or local function called `name`.
fn local_function(table: &SymbolTable, name: &str) -> bool {
    table.function_scopes().iter().any(|scope| {
        matches!(scope.kind, ScopeKind::Function | ScopeKind::LocalFunction) && scope.name == name
    })
}

/// Whether the file defines a function nested in another function called `name`.
fn nested_function(table: &SymbolTable, name: &str) -> bool {
    table
        .function_scopes()
        .iter()
        .any(|scope| scope.kind == ScopeKind::NestedFunction && scope.name == name)
}

/// What the callee of `node` refers to.
///
/// No engine calls this yet: the migrated helpers need only the readers above.
/// It is kept for rules that need the binding, and is tested here. Variables are
/// checked first, in the enclosing scope and then the script scope (as
/// [`Visibility::WithScript`]), then local functions, nested functions, and
/// dotted paths. A dotted path whose head is a variable is `Variable(head)`:
/// `s.f(x)` indexes `s`.
#[allow(dead_code)]
pub(crate) fn resolve<'a>(node: Node, source: &'a str, table: &SymbolTable) -> Callee<'a> {
    let Some(path) = callee_path(node, source, Dotted::Whole) else {
        return Callee::Unknown;
    };
    let head = path.split('.').next().unwrap_or(path);
    if data_variable_at(node, head, table) {
        return Callee::Variable(head);
    }
    if path.contains('.') {
        return Callee::Qualified(path);
    }
    if local_function(table, path) {
        Callee::Local(path)
    } else if nested_function(table, path) {
        Callee::Nested(path)
    } else {
        Callee::Function(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::symbols::{children_of, SymbolTable};

    fn parse(source: &str) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_matlab::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    /// The first node of `kind` in the tree, depth first.
    fn first_of<'t>(node: Node<'t>, kind_name: &str) -> Option<Node<'t>> {
        if node.kind() == kind_name {
            return Some(node);
        }
        children_of(node)
            .into_iter()
            .find_map(|child| first_of(child, kind_name))
    }

    /// The `function_call` whose callee text is `text`, in source order.
    fn call_with_text<'t>(root: Node<'t>, source: &str, text: &str) -> Node<'t> {
        fn walk<'t>(node: Node<'t>, source: &str, text: &str, out: &mut Vec<Node<'t>>) {
            if kind(node).is_some() && callee_path(node, source, Dotted::Whole) == Some(text) {
                out.push(node);
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                walk(child, source, text, out);
            }
        }
        let mut found = Vec::new();
        walk(root, source, text, &mut found);
        *found.first().unwrap_or_else(|| panic!("no callee {text}"))
    }

    fn resolve_text<'a>(source: &'a str, text: &str) -> Callee<'a> {
        let tree = parse(source);
        let table = SymbolTable::build(&tree, source);
        let node = call_with_text(tree.root_node(), source, text);
        resolve(node, source, &table)
    }

    #[test]
    fn plain_call_path() {
        let src = "y = isdir(p);\n";
        let tree = parse(src);
        let call = first_of(tree.root_node(), "function_call").unwrap();
        assert_eq!(callee_path(call, src, Dotted::Whole), Some("isdir"));
        assert_eq!(callee_path(call, src, Dotted::Trailing), Some("isdir"));
        assert_eq!(callee_path(call, src, Dotted::Skip), Some("isdir"));
    }

    #[test]
    fn dotted_package_call_reads_the_whole_path() {
        let src = "y = comm.gpu.X(1);\n";
        let tree = parse(src);
        let call = first_of(tree.root_node(), "function_call").unwrap();
        assert_eq!(callee_path(call, src, Dotted::Whole), Some("comm.gpu.X"));
        assert_eq!(callee_path(call, src, Dotted::Trailing), Some("X"));
        assert_eq!(callee_path(call, src, Dotted::Skip), None);
    }

    #[test]
    fn variable_in_front_is_not_a_package() {
        let src = "s = struct();\nz = s.psd(x);\n";
        let tree = parse(src);
        let call = first_of(tree.root_node(), "function_call");
        let call = call.unwrap();
        // The first call is `struct()`; the second is the qualified one.
        let mut calls = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if node.kind() == "function_call" {
                calls.push(node);
            }
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }
        let qualified = calls
            .into_iter()
            .find(|n| is_qualified_call(*n))
            .unwrap_or(call);
        assert_eq!(callee_path(qualified, src, Dotted::Whole), Some("s.psd"));
        assert_eq!(callee_path(qualified, src, Dotted::Trailing), Some("psd"));
        assert_eq!(callee_path(qualified, src, Dotted::Skip), None);
    }

    #[test]
    fn command_and_handle_paths() {
        let src = "format long\nf = @dsp.FIRFilter;\n";
        let tree = parse(src);
        let command = first_of(tree.root_node(), "command").unwrap();
        assert_eq!(callee_path(command, src, Dotted::Whole), Some("format"));
        assert_eq!(command_word(command, src), Some("format"));
        let handle = first_of(tree.root_node(), "handle_operator").unwrap();
        assert_eq!(
            callee_path(handle, src, Dotted::Whole),
            Some("dsp.FIRFilter")
        );
        assert_eq!(kind(handle), Some(CalleeKind::Handle));
    }

    #[test]
    fn non_callees_have_no_path() {
        let src = "x = 1;\n";
        let tree = parse(src);
        let number = first_of(tree.root_node(), "number").unwrap();
        assert_eq!(kind(number), None);
        assert_eq!(callee_path(number, src, Dotted::Whole), None);
    }

    #[test]
    fn resolves_unshadowed_function() {
        let src = "y = isdir(p);\n";
        assert_eq!(resolve_text(src, "isdir"), Callee::Function("isdir"));
    }

    #[test]
    fn resolves_variable_index() {
        let src = "isdir = [1 2 3];\ny = isdir(2);\n";
        assert_eq!(resolve_text(src, "isdir"), Callee::Variable("isdir"));
    }

    #[test]
    fn script_variable_reaches_anonymous_function() {
        let src = "isdir = 3;\nf = @() isdir(1);\n";
        assert_eq!(resolve_text(src, "isdir"), Callee::Variable("isdir"));
    }

    #[test]
    fn resolves_local_function() {
        let src =
            "function main()\n  y = helper(1);\nend\n\nfunction r = helper(x)\n  r = x;\nend\n";
        assert_eq!(resolve_text(src, "helper"), Callee::Local("helper"));
    }

    #[test]
    fn resolves_nested_function() {
        let src =
            "function outer()\n  y = inner(1);\n  function r = inner(x)\n    r = x;\n  end\nend\n";
        assert_eq!(resolve_text(src, "inner"), Callee::Nested("inner"));
    }

    #[test]
    fn resolves_dotted_package_and_handle() {
        let src = "y = dsp.FIRFilter(3);\n";
        assert_eq!(
            resolve_text(src, "dsp.FIRFilter"),
            Callee::Qualified("dsp.FIRFilter")
        );
        let src = "f = @comm.gpu.X;\n";
        assert_eq!(
            resolve_text(src, "comm.gpu.X"),
            Callee::Qualified("comm.gpu.X")
        );
    }

    #[test]
    fn variable_in_front_of_method_resolves_to_the_variable() {
        let src = "s = struct();\nz = s.psd(x);\n";
        assert_eq!(resolve_text(src, "s.psd"), Callee::Variable("s"));
    }

    #[test]
    fn shadowing_follows_visibility() {
        let src = "isdir = 3;\nfunction r = g(p)\n  r = isdir(p);\nend\n";
        let tree = parse(src);
        let table = SymbolTable::build(&tree, src);
        let call = call_with_text(tree.root_node(), src, "isdir");
        assert!(!is_shadowed(call, "isdir", &table, Visibility::Enclosing));
        assert!(is_shadowed(call, "isdir", &table, Visibility::WithScript));
    }

    #[test]
    fn dotted_names_are_never_shadowed() {
        let src = "dsp = 1;\ny = dsp.FIRFilter(3);\n";
        let tree = parse(src);
        let table = SymbolTable::build(&tree, src);
        let call = call_with_text(tree.root_node(), src, "dsp.FIRFilter");
        assert!(!is_shadowed(
            call,
            "dsp.FIRFilter",
            &table,
            Visibility::WithScript
        ));
    }

    #[test]
    fn reference_name_accepts_identifiers_and_dotted_handles() {
        let src = "f = @dsp.FIRFilter;\n";
        let tree = parse(src);
        let handle = first_of(tree.root_node(), "handle_operator").unwrap();
        assert_eq!(reference_name(handle, src), Some("dsp.FIRFilter"));
    }
}
