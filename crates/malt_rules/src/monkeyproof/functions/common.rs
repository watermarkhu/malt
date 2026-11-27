//! Helpers shared by the checks of the higher level rules.

use std::ops::Range;

use tree_sitter::Node;

/// Where a function sits in its file, which decides the limits that apply to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FnKind {
    /// The first function of a function file.
    Main,
    /// A later function at file level, also in a classdef file.
    Local,
    /// A function in a `methods` block of a class.
    Method,
    /// A function defined inside another function.
    Nested,
}

/// A `function_definition` and its kind.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Func<'t> {
    /// The `function_definition` node.
    pub node: Node<'t>,
    /// Where it sits in the file.
    pub kind: FnKind,
}

#[derive(Clone, Copy)]
struct Walk {
    in_class: bool,
    in_function: bool,
}

/// Every `function_definition` below `root`, in source order, with its kind.
pub(crate) fn functions<'t>(root: Node<'t>) -> Vec<Func<'t>> {
    let mut out = Vec::new();
    let mut main_seen = false;
    walk_functions(
        root,
        Walk {
            in_class: false,
            in_function: false,
        },
        &mut main_seen,
        &mut out,
    );
    out
}

fn walk_functions<'t>(node: Node<'t>, state: Walk, main_seen: &mut bool, out: &mut Vec<Func<'t>>) {
    let mut state = state;
    match node.kind() {
        "class_definition" => state.in_class = true,
        "function_definition" => {
            let kind = if state.in_function {
                FnKind::Nested
            } else if state.in_class && !is_file_level(node) {
                FnKind::Method
            } else if !*main_seen {
                *main_seen = true;
                FnKind::Main
            } else {
                FnKind::Local
            };
            out.push(Func { node, kind });
            state.in_function = true;
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_functions(child, state, main_seen, out);
    }
}

/// Whether `node` sits directly under the file root (a local function, also after a
/// `classdef`).
fn is_file_level(node: Node) -> bool {
    node.parent().is_some_and(|p| p.kind() == "source_file")
}

/// The first child of `node` of the given kind.
pub(crate) fn child_of_kind<'t>(node: Node<'t>, kind: &str) -> Option<Node<'t>> {
    let mut cursor = node.walk();
    let found = node.children(&mut cursor).find(|c| c.kind() == kind);
    found
}

/// The name node of a function, signature or class.
pub(crate) fn name_node(node: Node) -> Option<Node> {
    node.child_by_field_name("name")
}

/// The range a finding about a function points at: its name, else its first line.
pub(crate) fn name_range(node: Node) -> Range<usize> {
    name_node(node)
        .map(|n| n.byte_range())
        .unwrap_or_else(|| node.start_byte()..node.start_byte() + 1)
}

/// The range of the first line of `node` in `source`, without trailing whitespace.
pub(crate) fn head_range(node: Node, source: &str) -> Range<usize> {
    let start = node.start_byte();
    let end = node.end_byte().min(source.len());
    let line_end = source[start..end]
        .find('\n')
        .map_or(end, |offset| start + offset);
    let text = source[start..line_end].trim_end();
    start..start + text.len().max(1)
}

/// The text of `node`.
pub(crate) fn text<'a>(node: Node, source: &'a str) -> &'a str {
    source.get(node.byte_range()).unwrap_or("")
}

/// The input parameters of a function or signature, in order: identifiers, `varargin`
/// and `~`.
pub(crate) fn inputs<'t>(func: Node<'t>) -> Vec<Node<'t>> {
    let Some(list) = child_of_kind(func, "function_arguments") else {
        return Vec::new();
    };
    let mut cursor = list.walk();
    let found = list
        .named_children(&mut cursor)
        .filter(|c| matches!(c.kind(), "identifier" | "ignored_argument"))
        .collect();
    found
}

/// The output parameters of a function or signature, in order.
pub(crate) fn outputs<'t>(func: Node<'t>) -> Vec<Node<'t>> {
    let Some(list) = child_of_kind(func, "function_output") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut cursor = list.walk();
    for child in list.named_children(&mut cursor) {
        match child.kind() {
            "identifier" | "ignored_argument" => out.push(child),
            "multioutput_variable" => {
                let mut inner = child.walk();
                out.extend(
                    child
                        .named_children(&mut inner)
                        .filter(|c| matches!(c.kind(), "identifier" | "ignored_argument")),
                );
            }
            _ => {}
        }
    }
    out
}

/// The name of a function as written.
pub(crate) fn function_name<'a>(func: Node, source: &'a str) -> &'a str {
    name_node(func).map_or("", |n| text(n, source))
}

/// Whether a `methods` block carries the attribute `name` set to true (`Abstract`,
/// `Abstract = true`).
pub(crate) fn has_true_attribute(methods: Node, name: &str, source: &str) -> bool {
    let Some(attributes) = child_of_kind(methods, "attributes") else {
        return false;
    };
    let mut cursor = attributes.walk();
    for attribute in attributes.named_children(&mut cursor) {
        let mut inner = attribute.walk();
        let parts: Vec<Node> = attribute.named_children(&mut inner).collect();
        let Some(first) = parts.first() else { continue };
        if text(*first, source) != name {
            continue;
        }
        return match parts.get(1) {
            None => true,
            Some(value) => text(*value, source) != "false",
        };
    }
    false
}

#[cfg(test)]
pub(crate) mod tests_util {
    use malt_core::{Config, Diagnostic, FileContext};

    /// Run the check `id` of this group over `source`, with `toml` as configuration.
    pub(crate) fn run(id: &str, source: &str, toml: &str) -> Vec<Diagnostic> {
        let mut diagnostics = crate::monkeyproof::lint(super::super::from_config, source, toml);
        diagnostics.retain(|d| d.rule_id == id);
        diagnostics
    }

    /// Where a test class file sits on disk.
    pub(crate) enum Layout {
        /// `dir/C.m`.
        Plain,
        /// `dir/@C/C.m`, next to the other files of the class.
        ClassFolder,
    }

    /// Write `source` as the class file in a fresh folder, with `others` beside it, and
    /// run the check `id` over it.
    pub(crate) fn run_at(
        id: &str,
        layout: Layout,
        source: &str,
        others: &[(&str, &str)],
    ) -> Vec<Diagnostic> {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!("malt_mp_fcns_{}_{n}", std::process::id()));
        let dir = match layout {
            Layout::Plain => base.clone(),
            Layout::ClassFolder => base.join("@C"),
        };
        std::fs::create_dir_all(&dir).unwrap();
        for (name, body) in others {
            std::fs::write(dir.join(name), body).unwrap();
        }
        let path = dir.join("C.m");
        std::fs::write(&path, source).unwrap();
        let tree = crate::test_util::parse(source);
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: &path,
            project: None,
        };
        let engine = super::super::from_config(&Config::from_toml("").unwrap());
        let mut diagnostics = engine.check_file(&ctx);
        diagnostics.retain(|d| d.rule_id == id);
        let _ = std::fs::remove_dir_all(&base);
        diagnostics
    }
}
