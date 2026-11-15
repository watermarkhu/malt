//! # Reaching bindings and constant values
//!
//! A conservative, textual approximation of what a name holds at a point in
//! the file: the right-hand side of the last whole assignment written before
//! that point in the same function (or in the file, for scripts). Used by the
//! growth checks (AGROW/SAGROW), the constant-operand checks (OR2/AND2) and
//! FVAL. Anything the approximation cannot follow is reported as
//! [`Binding::Unknown`], so a rule that needs a known value stays silent.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use tree_sitter::Node;

use crate::analysis::symbols::FileKey;

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

/// The binding of `name` at the position of `at`.
pub(crate) fn reaching<'a>(at: Node<'a>, source: &str, name: &str) -> Reaching<'a> {
    let scope = enclosing_function(at).unwrap_or_else(|| root_of(at));
    let before = at.start_byte();
    let index = scope_index(scope, source);
    // The binding written last before `at`; among bindings at the same offset,
    // the last in the index order wins.
    let mut best: Option<&Event> = None;
    let mut last_indexed: Option<usize> = None;
    for event in index.events.get(name).into_iter().flatten() {
        if event.start >= before {
            continue;
        }
        match event.kind {
            EventKind::Indexed => {
                last_indexed = Some(last_indexed.map_or(event.start, |s| s.max(event.start)));
            }
            EventKind::Whole | EventKind::Unknown => {
                if best.is_none_or(|b| event.start >= b.start) {
                    best = Some(event);
                }
            }
        }
    }
    let best_start = best.map(|e| e.start);
    let indexed_since = match (last_indexed, best_start) {
        (Some(i), Some(b)) => i > b,
        (Some(_), None) => true,
        _ => false,
    };
    // A bare `clear` after the last binding leaves the name with no known value.
    if index
        .clear_all
        .iter()
        .any(|&s| s < before && best_start.is_none_or(|b| s > b))
    {
        return Reaching {
            binding: Binding::Unknown,
            indexed_since: true,
        };
    }
    let binding = match best {
        Some(event) => match event.kind {
            EventKind::Whole => match index
                .node_at(scope, event.node)
                .and_then(|assignment| assignment.child_by_field_name("right"))
            {
                Some(right) => Binding::Value(right),
                None => Binding::Unknown,
            },
            _ => Binding::Unknown,
        },
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

/// Every value that can reach the point `at` for `name`: the right-hand sides of the
/// whole assignments that may define it there. `None` when the approximation cannot
/// be sure: an indexed write, a loop variable or a `clear` of the name before the
/// point, a binding inside a loop that holds the point (it reaches the point on the
/// next pass), a bare `clear`, or no assignment at all (an input or undefined name).
/// An assignment is dropped when a later assignment in a block that holds the point
/// overwrites it (`x = 1; x = 2; f(x)` gives `2` only), and the assignment that
/// contains the point never reaches it (`t = t` reads the earlier `t`).
pub(crate) fn reaching_values<'a>(at: Node<'a>, source: &str, name: &str) -> Option<Vec<Node<'a>>> {
    let scope = enclosing_function(at).unwrap_or_else(|| root_of(at));
    let before = at.start_byte();
    let index = scope_index(scope, source);
    if index.clear_all.iter().any(|&s| s < before) {
        return None;
    }
    let mut loops = Vec::new();
    let mut current = at.parent();
    while let Some(p) = current {
        if p.id() == scope.id() {
            break;
        }
        if matches!(p.kind(), "for_statement" | "while_statement") {
            loops.push((p.start_byte(), p.end_byte()));
        }
        current = p.parent();
    }
    let mut defs = Vec::new();
    // Assignments after the point inside a loop that holds it reach it on the next pass.
    let mut next_pass = Vec::new();
    for event in index.events.get(name).into_iter().flatten() {
        if event.start >= before {
            if loops
                .iter()
                .any(|&(s, e)| s <= event.start && event.start < e)
            {
                match event.kind {
                    EventKind::Whole => {
                        let assignment = index.node_at(scope, event.node)?;
                        next_pass.push(assignment.child_by_field_name("right")?);
                    }
                    EventKind::Indexed | EventKind::Unknown => return None,
                }
            }
            continue;
        }
        match event.kind {
            EventKind::Whole => {
                let assignment = index.node_at(scope, event.node)?;
                if assignment.end_byte() > before {
                    continue;
                }
                defs.push((event.start, assignment));
            }
            EventKind::Indexed | EventKind::Unknown => return None,
        }
    }
    let mut values = Vec::new();
    for &(start, assignment) in &defs {
        let overwritten = defs.iter().any(|&(later, other)| {
            later > start
                && other.parent().is_some_and(|p| {
                    matches!(p.kind(), "source_file" | "block")
                        && p.start_byte() <= before
                        && before < p.end_byte()
                })
        });
        if !overwritten {
            values.push(assignment.child_by_field_name("right")?);
        }
    }
    values.extend(next_pass);
    (!values.is_empty()).then_some(values)
}

/// The values of an expression: itself, or for a name, the values that reach it
/// (see [`reaching_values`]), each followed through its own assignments as far as
/// it is a name. `None` when a name in the chain cannot be followed.
pub(crate) fn resolved_values<'a>(node: Node<'a>, source: &str) -> Option<Vec<Node<'a>>> {
    resolve_at(node, source, 0)
}

fn resolve_at<'a>(node: Node<'a>, source: &str, depth: usize) -> Option<Vec<Node<'a>>> {
    if node.kind() != "identifier" {
        return Some(vec![node]);
    }
    if depth > MAX_DEPTH {
        return None;
    }
    let values = reaching_values(node, source, text(node, source))?;
    let mut out = Vec::new();
    for value in values {
        out.extend(resolve_at(value, source, depth + 1)?);
    }
    Some(out)
}

/// How a statement binds a name, for [`reaching`].
#[derive(Clone, Copy)]
enum EventKind {
    /// A whole assignment `name = rhs`.
    Whole,
    /// A binding the approximation does not follow: a loop variable or one
    /// target of `[a, b] = f()`.
    Unknown,
    /// An indexed or field write `name(i) = v`, `name{i} = v`, `name.f = v`.
    Indexed,
}

/// One binding of a name in a scope.
struct Event {
    /// Start of the binding statement.
    start: usize,
    kind: EventKind,
    /// The statement, as an entry of [`ScopeIndex::nodes`].
    node: usize,
}

/// The bindings of every name in one scope (a function, or the file), in the
/// order the original stack walk met them, so a query costs the bindings of
/// its name instead of a walk over the scope.
struct ScopeIndex {
    events: HashMap<String, Vec<Event>>,
    /// Starts of the `clear` and `clearvars` calls with no name (they clear every variable).
    clear_all: Vec<usize>,
    /// The nodes of the scope as (entry of the parent, child index in it); the
    /// scope itself is entry 0. Nodes cannot be kept across calls, so a binding
    /// statement is found again by its path of child indices.
    nodes: Vec<(usize, usize)>,
}

impl ScopeIndex {
    fn build(scope: Node, source: &str) -> Self {
        let mut events: HashMap<String, Vec<Event>> = HashMap::new();
        let mut nodes = vec![(0, 0)];
        let mut clear_all = Vec::new();
        let mut push = |name: &str, start: usize, kind: EventKind, node: usize| {
            events
                .entry(name.to_string())
                .or_default()
                .push(Event { start, kind, node });
        };
        // The same stack walk as a direct scan: the last child first.
        let mut stack = vec![(scope, 0usize)];
        while let Some((n, entry)) = stack.pop() {
            match n.kind() {
                "assignment" => {
                    if let Some(left) = n.child_by_field_name("left") {
                        let start = n.start_byte();
                        match left.kind() {
                            "identifier" => {
                                push(text(left, source), start, EventKind::Whole, entry)
                            }
                            "matrix" => {
                                let mut seen = HashSet::new();
                                for target in identifiers_below(left, source) {
                                    if seen.insert(target) {
                                        push(target, start, EventKind::Unknown, entry);
                                    }
                                }
                                let base = base_name(left, source);
                                if !seen.contains(base) {
                                    push(base, start, EventKind::Indexed, entry);
                                }
                            }
                            _ => push(base_name(left, source), start, EventKind::Indexed, entry),
                        }
                    }
                }
                "for_statement" => {
                    let mut seen = HashSet::new();
                    for i in 0..n.named_child_count() {
                        let Some(c) = n.named_child(i) else { continue };
                        let iterator = if c.kind() == "iterator" {
                            c.named_child(0)
                        } else {
                            Some(c)
                        };
                        if let Some(it) = iterator.filter(|it| it.kind() == "identifier") {
                            if seen.insert(text(it, source)) {
                                push(text(it, source), n.start_byte(), EventKind::Unknown, entry);
                            }
                        }
                    }
                }
                "function_call" | "command" => {
                    // `clear t` unbinds the names it lists; `clear all` keeps them (probed);
                    // a bare `clear` or `clearvars` unbinds every name.
                    let callee = if n.kind() == "command" {
                        n.child(0).filter(|c| c.kind() == "command_name")
                    } else {
                        n.child_by_field_name("name")
                    };
                    if callee.is_some_and(|c| matches!(text(c, source), "clear" | "clearvars")) {
                        let rest = &source[callee.map_or(0, |c| c.end_byte())..n.end_byte()];
                        let words: Vec<&str> = rest
                            .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-'))
                            .filter(|w| !w.is_empty())
                            .collect();
                        if words.is_empty() {
                            clear_all.push(n.start_byte());
                        }
                        for word in words {
                            if word != "all" && !word.starts_with('-') {
                                push(word, n.start_byte(), EventKind::Unknown, entry);
                            }
                        }
                    }
                }
                _ => {}
            }
            let mut cursor = n.walk();
            for (i, child) in n.children(&mut cursor).enumerate() {
                nodes.push((entry, i));
                stack.push((child, nodes.len() - 1));
            }
        }
        Self {
            events,
            clear_all,
            nodes,
        }
    }

    /// The node of entry `entry`, found from `scope` by its path.
    fn node_at<'a>(&self, scope: Node<'a>, entry: usize) -> Option<Node<'a>> {
        let mut path = Vec::new();
        let mut current = entry;
        while current != 0 {
            let (parent, i) = self.nodes[current];
            path.push(i);
            current = parent;
        }
        let mut node = scope;
        for &i in path.iter().rev() {
            let mut cursor = node.walk();
            node = node.children(&mut cursor).nth(i)?;
        }
        Some(node)
    }
}

/// The text of every identifier at or below `node`, in tree order.
fn identifiers_below<'s>(node: Node, source: &'s str) -> Vec<&'s str> {
    let mut out = Vec::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if n.kind() == "identifier" {
            out.push(text(n, source));
        }
        let mut cursor = n.walk();
        stack.extend(n.children(&mut cursor));
    }
    out
}

/// The indexes of the current file by scope id. See [`scope_index`].
type IndexMemo = Option<(FileKey, HashMap<usize, Rc<ScopeIndex>>)>;

thread_local! {
    static INDEXES: RefCell<IndexMemo> = const { RefCell::new(None) };
}

/// The [`ScopeIndex`] of `scope` in the current file, built on first use.
fn scope_index(scope: Node, source: &str) -> Rc<ScopeIndex> {
    let key = FileKey::of(source);
    INDEXES.with(|memo| {
        let mut memo = memo.borrow_mut();
        let (memo_key, indexes) = memo.get_or_insert_with(|| (key, HashMap::new()));
        if *memo_key != key {
            *memo_key = key;
            indexes.clear();
        }
        indexes
            .entry(scope.id())
            .or_insert_with(|| Rc::new(ScopeIndex::build(scope, source)))
            .clone()
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::parse;

    fn identifiers<'t>(node: Node<'t>, source: &str, name: &str, out: &mut Vec<Node<'t>>) {
        if node.kind() == "identifier" && &source[node.byte_range()] == name {
            out.push(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            identifiers(child, source, name, out);
        }
    }

    /// The source text of the values that reach the `nth` identifier `name` (the last
    /// one when `nth` is None).
    fn values_at(source: &str, name: &str, nth: Option<usize>) -> Option<Vec<String>> {
        let tree = parse(source);
        let mut found = Vec::new();
        identifiers(tree.root_node(), source, name, &mut found);
        let at = match nth {
            Some(n) => *found.get(n)?,
            None => *found.last()?,
        };
        reaching_values(at, source, name).map(|values| {
            values
                .iter()
                .map(|v| source[v.byte_range()].to_string())
                .collect()
        })
    }

    #[test]
    fn a_straight_line_overwrite_leaves_one_value() {
        assert_eq!(
            values_at("t = 'a';\nt = 'b';\ndisp(t)\n", "t", None),
            Some(vec!["'b'".to_string()])
        );
    }

    #[test]
    fn a_conditional_assignment_adds_a_value() {
        assert_eq!(
            values_at("t = 'a';\nif c\n    t = 'b';\nend\ndisp(t)\n", "t", None).map(|v| v.len()),
            Some(2)
        );
    }

    #[test]
    fn an_indexed_write_a_clear_or_no_assignment_leaves_no_value() {
        assert_eq!(
            values_at("t = 'a';\nt(2) = 'b';\ndisp(t)\n", "t", None),
            None
        );
        assert_eq!(values_at("t = 'a';\nclear t\ndisp(t)\n", "t", None), None);
        assert_eq!(values_at("disp(t)\n", "t", None), None);
    }

    #[test]
    fn a_later_assignment_in_a_loop_reaches_the_point_on_the_next_pass() {
        assert_eq!(
            values_at(
                "t = 'a';\nfor k = 1:2\n    disp(t)\n    t = 'b';\nend\n",
                "t",
                Some(1)
            )
            .map(|v| v.len()),
            Some(2)
        );
    }

    #[test]
    fn an_assignment_does_not_reach_its_own_right_side() {
        assert_eq!(
            values_at("t = 'a';\nt = t;\ndisp(t)\n", "t", Some(3)),
            Some(vec!["t".to_string()])
        );
    }
}
