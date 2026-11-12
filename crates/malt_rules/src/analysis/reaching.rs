//! # Reaching bindings and constant values
//!
//! A conservative, textual approximation of what a name holds at a point in
//! the file: the right-hand side of the last whole assignment written before
//! that point in the same function (or in the file, for scripts). Used by the
//! growth checks (AGROW/SAGROW), the constant-operand checks (OR2/AND2) and
//! FVAL. Anything the approximation cannot follow is reported as
//! [`Binding::Unknown`], so a rule that needs a known value stays silent.

use tree_sitter::Node;

/// What a name is bound to at a point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Binding<'a> {
    /// No definition before the point: the name is undefined (an output
    /// argument does not count as a definition).
    Undefined,
    /// A function input argument: its value and size are not known.
    Param,
    /// The right-hand side of the last whole assignment.
    Value(Node<'a>),
    /// Bound by something the approximation does not follow (a loop variable
    /// or a multi-assignment target).
    Unknown,
}

/// The binding of a name at a point, and whether an indexed assignment
/// (`name(i) = v`, `name{i} = v`) was written after that binding.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Reaching<'a> {
    pub(crate) binding: Binding<'a>,
    pub(crate) indexed_since: bool,
}

/// Names whose value is a constant even when they are not assigned in the file.
const CONSTANT_NAMES: &[&str] = &["true", "false", "pi", "eps", "Inf", "inf", "NaN", "nan"];

/// Functions whose result is a constant when their arguments are.
const CONSTANT_FUNCTIONS: &[&str] = &[
    "zeros", "ones", "eye", "true", "false", "pi", "eps", "Inf", "inf", "NaN", "nan",
];

/// Nesting limit for following assignments through other variables.
const MAX_DEPTH: usize = 8;

fn text<'s>(node: Node, source: &'s str) -> &'s str {
    &source[node.byte_range()]
}

/// The nearest enclosing `function_definition`, if any.
fn enclosing_function(node: Node) -> Option<Node> {
    let mut current = node.parent();
    while let Some(p) = current {
        if p.kind() == "function_definition" {
            return Some(p);
        }
        current = p.parent();
    }
    None
}

/// The outermost node, the file's root.
fn root_of(node: Node) -> Node {
    let mut current = node;
    while let Some(p) = current.parent() {
        current = p;
    }
    current
}

/// Whether `name` is an input argument of `func`: the names between the first
/// parentheses of the header line (`function r = f(n, m)`).
fn is_parameter(func: Node, source: &str, name: &str) -> bool {
    let header = text(func, source).lines().next().unwrap_or("");
    let Some(open) = header.find('(') else {
        return false;
    };
    let Some(close) = header[open..].find(')') else {
        return false;
    };
    header[open + 1..open + close]
        .split(',')
        .any(|p| p.trim() == name)
}

/// The variable an assignment target writes into: `x` for `x(i)`, `x{i}`, `x(i).f`.
fn base_name<'s>(target: Node, source: &'s str) -> &'s str {
    let t = text(target, source);
    t.split(['(', '{', '.']).next().unwrap_or("").trim()
}

/// Whether an identifier named `name` occurs anywhere below `node`.
fn mentions(node: Node, source: &str, name: &str) -> bool {
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if n.kind() == "identifier" && text(n, source) == name {
            return true;
        }
        let mut cursor = n.walk();
        stack.extend(n.children(&mut cursor));
    }
    false
}

/// The binding of `name` at the position of `at`.
pub(crate) fn reaching<'a>(at: Node<'a>, source: &str, name: &str) -> Reaching<'a> {
    let scope = enclosing_function(at).unwrap_or_else(|| root_of(at));
    let before = at.start_byte();
    let mut best: Option<(usize, Binding<'a>)> = None;
    let mut last_indexed: Option<usize> = None;
    let mut pick = |start: usize, binding: Binding<'a>| {
        if best.is_none_or(|(s, _)| start >= s) {
            best = Some((start, binding));
        }
    };
    let mut stack = vec![scope];
    while let Some(n) = stack.pop() {
        // Children start no earlier than their parent: nothing below a node that
        // starts at or after `at` can come before it.
        if n.start_byte() >= before {
            continue;
        }
        match n.kind() {
            "assignment" => {
                if let Some(left) = n.child_by_field_name("left") {
                    match left.kind() {
                        "identifier" if text(left, source) == name => {
                            let binding = match n.child_by_field_name("right") {
                                Some(right) => Binding::Value(right),
                                None => Binding::Unknown,
                            };
                            pick(n.start_byte(), binding);
                        }
                        "matrix" if mentions(left, source, name) => {
                            pick(n.start_byte(), Binding::Unknown);
                        }
                        "identifier" => {}
                        _ if base_name(left, source) == name => {
                            last_indexed = Some(
                                last_indexed.map_or(n.start_byte(), |s| s.max(n.start_byte())),
                            );
                        }
                        _ => {}
                    }
                }
            }
            "for_statement" => {
                for i in 0..n.named_child_count() {
                    let Some(c) = n.named_child(i) else { continue };
                    let iterator = if c.kind() == "iterator" {
                        c.named_child(0)
                    } else {
                        Some(c)
                    };
                    if iterator
                        .is_some_and(|it| it.kind() == "identifier" && text(it, source) == name)
                    {
                        pick(n.start_byte(), Binding::Unknown);
                        break;
                    }
                }
            }
            _ => {}
        }
        let mut cursor = n.walk();
        stack.extend(n.children(&mut cursor));
    }
    let best_start = best.map(|(s, _)| s);
    let indexed_since = match (last_indexed, best_start) {
        (Some(i), Some(b)) => i > b,
        (Some(_), None) => true,
        _ => false,
    };
    let binding = match best {
        Some((_, b)) => b,
        None if enclosing_function(at).is_some_and(|f| is_parameter(f, source, name)) => {
            Binding::Param
        }
        None => Binding::Undefined,
    };
    Reaching {
        binding,
        indexed_since,
    }
}

/// The arguments of a `function_call` node, as expression nodes.
fn call_arguments(call: Node) -> Vec<Node> {
    let mut cursor = call.walk();
    let Some(args) = call.children(&mut cursor).find(|c| c.kind() == "arguments") else {
        return Vec::new();
    };
    (0..args.named_child_count())
        .filter_map(|i| args.named_child(i))
        .collect()
}

/// Whether an expression has the same value every time the file runs: numbers,
/// strings, literal arrays and cells, operators on those, variables holding
/// such values, and `zeros`/`ones`-style calls on them. Anything else
/// (function results, input, loop variables, fields, parameters) is not.
pub(crate) fn is_constant(node: Node, source: &str) -> bool {
    constant_at(node, source, 0)
}

fn constant_at(node: Node, source: &str, depth: usize) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match node.kind() {
        "number" | "string" => true,
        "identifier" => {
            let name = text(node, source);
            if CONSTANT_NAMES.contains(&name) {
                return true;
            }
            let r = reaching(node, source, name);
            !r.indexed_since
                && match r.binding {
                    Binding::Value(rhs) => constant_at(rhs, source, depth + 1),
                    _ => false,
                }
        }
        "function_call" => {
            let Some(name_node) = node.child_by_field_name("name") else {
                return false;
            };
            let name = text(name_node, source);
            let callee_ok = CONSTANT_FUNCTIONS.contains(&name)
                || (name_node.kind() == "identifier"
                    && {
                        let r = reaching(name_node, source, name);
                        !r.indexed_since
                            && matches!(r.binding, Binding::Value(rhs) if constant_at(rhs, source, depth + 1))
                    });
            callee_ok
                && call_arguments(node)
                    .into_iter()
                    .all(|a| constant_at(a, source, depth + 1))
        }
        "binary_operator"
        | "comparison_operator"
        | "boolean_operator"
        | "unary_operator"
        | "postfix_operator"
        | "parenthesis"
        | "matrix"
        | "cell"
        | "row"
        | "range" => {
            let mut cursor = node.walk();
            let all = node
                .named_children(&mut cursor)
                .all(|c| constant_at(c, source, depth + 1));
            all
        }
        _ => false,
    }
}

/// Whether an assignment's whole target is a value that an indexed write then
/// grows: undefined, `[]`, `{}`, or a plain number. Used by AGROW/SAGROW.
pub(crate) fn is_growable_initial_value(binding: Binding, source: &str) -> bool {
    match binding {
        Binding::Undefined => true,
        Binding::Value(rhs) => {
            let compact: String = text(rhs, source)
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            rhs.kind() == "number" || compact == "[]" || compact == "{}"
        }
        Binding::Param | Binding::Unknown => false,
    }
}
