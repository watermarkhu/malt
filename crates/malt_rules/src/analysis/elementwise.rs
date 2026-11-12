//! # Operands of elementwise `|` and `&`
//!
//! tree-sitter-matlab binds `|` and `&` tighter than the comparison operators,
//! so `L > 1 | L < 0.1` parses as `L > (1 | L) < 0.1`. MATLAB binds them the
//! other way round, from loosest to tightest: `|`, then `&`, then the
//! comparisons (`<`, `<=`, `>`, `>=`, `==`, `~=`), then the arithmetic. This
//! module flattens a chain of comparisons and elementwise operators back into
//! its leaves in source order, then splits them by MATLAB's precedence, so
//! checks see the operands MATLAB sees.
//!
//! [`elementwise_operands`] returns the two operands of one `|` or `&`.
//! [`is_known_logical`] tells whether an operand is a value MATLAB knows to be
//! logical, which is when the OR2 and AND2 checks stay silent.

use tree_sitter::Node;

use super::reaching::{is_constant, reaching, Binding};

/// Nesting limit for following assignments through other variables.
const MAX_DEPTH: usize = 8;

/// Calls whose result MATLAB knows to be logical (probed: OR2 and AND2 silent).
const LOGICAL_CALLS: &[&str] = &["isempty", "ischar", "strcmp"];

/// Calls whose result MATLAB knows to be numeric (probed: comparisons of their
/// results are silent for OR2 and AND2).
const NUMERIC_CALLS: &[&str] = &["numel", "length", "size", "ndims", "rows", "columns"];

/// One operand of a `|` or `&`: the expressions between the operators that
/// bind no tighter, in source order. A comparison contributes its two sides and
/// its operator is dropped, so `a > 1` has the leaves `a` and `1`.
#[derive(Debug, Clone)]
pub(crate) struct Operand<'a> {
    pub(crate) leaves: Vec<Node<'a>>,
}

impl Operand<'_> {
    /// The source text from the first leaf to the last one.
    pub(crate) fn text<'s>(&self, source: &'s str) -> &'s str {
        match (self.leaves.first(), self.leaves.last()) {
            (Some(first), Some(last)) => &source[first.start_byte()..last.end_byte()],
            _ => "",
        }
    }
}

/// A token of a flattened chain: an expression that is not part of the chain,
/// or an operator with its precedence.
#[derive(Clone, Copy)]
enum Tok<'a> {
    Leaf(Node<'a>),
    Op(Node<'a>, u8),
}

fn text<'s>(node: Node, source: &'s str) -> &'s str {
    &source[node.byte_range()]
}

/// MATLAB's precedence of the operators that take part in a chain: a higher
/// number binds tighter. `None` for every other operator.
fn precedence(op: &str) -> Option<u8> {
    match op {
        "|" => Some(1),
        "&" => Some(2),
        "<" | "<=" | ">" | ">=" | "==" | "~=" => Some(3),
        _ => None,
    }
}

/// The operator token of a `binary_operator` or `comparison_operator`.
fn operator_of<'s>(node: Node, source: &'s str) -> Option<&'s str> {
    operator_token(node).map(|c| text(c, source))
}

/// The operator token (the unnamed child) of a `binary_operator` or
/// `comparison_operator`.
pub(crate) fn operator_token(node: Node) -> Option<Node> {
    let mut cursor = node.walk();
    let token = node.children(&mut cursor).find(|c| !c.is_named());
    token
}

/// Whether a node takes part in a chain: a comparison, or a `|` or `&`.
fn is_chain_node(node: Node, source: &str) -> bool {
    match node.kind() {
        "comparison_operator" => true,
        "binary_operator" => operator_of(node, source).is_some_and(|op| matches!(op, "|" | "&")),
        _ => false,
    }
}

/// The outermost chain node that contains `node` without a non-chain parent
/// between them. `&&` and `||` are not chain nodes, so they end the chain.
fn chain_root<'a>(node: Node<'a>, source: &str) -> Node<'a> {
    let mut root = node;
    while let Some(parent) = root.parent() {
        if !is_chain_node(parent, source) {
            break;
        }
        root = parent;
    }
    root
}

/// Flattens a chain into its leaves and operators, in source order.
fn flatten<'a>(node: Node<'a>, source: &str, out: &mut Vec<Tok<'a>>) {
    if !is_chain_node(node, source) {
        out.push(Tok::Leaf(node));
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "comment" {
            continue;
        }
        if child.is_named() {
            flatten(child, source, out);
        } else if let Some(prec) = precedence(text(child, source)) {
            out.push(Tok::Op(child, prec));
        }
    }
}

/// The operand made of `toks`, or `None` when it has no leaves.
fn operand<'a>(toks: &[Tok<'a>]) -> Option<Operand<'a>> {
    let leaves: Vec<Node<'a>> = toks
        .iter()
        .filter_map(|t| match t {
            Tok::Leaf(n) => Some(*n),
            Tok::Op(..) => None,
        })
        .collect();
    (!leaves.is_empty()).then_some(Operand { leaves })
}

/// The two operands of the `|` or `&` node `op`, as MATLAB parses them.
///
/// Each operand runs from `op` outwards until the nearest operator that binds
/// no tighter than `op`. For `L > 1 | L < 0.1` the left operand is `L > 1` and
/// the right one is `L < 0.1`, although the tree has `1 | L` as one node.
/// Returns `None` if `op` is not an operator of its chain.
pub(crate) fn elementwise_operands<'a>(
    op: Node<'a>,
    source: &str,
) -> Option<(Operand<'a>, Operand<'a>)> {
    let token = operator_token(op)?;
    let mut toks = Vec::new();
    flatten(chain_root(op, source), source, &mut toks);
    let at = toks
        .iter()
        .position(|t| matches!(t, Tok::Op(n, _) if n.start_byte() == token.start_byte()))?;
    let Tok::Op(_, prec) = toks[at] else {
        return None;
    };
    let binds = |t: &Tok| match *t {
        Tok::Leaf(_) => true,
        Tok::Op(_, p) => p > prec,
    };
    let mut lo = at;
    while lo > 0 && binds(&toks[lo - 1]) {
        lo -= 1;
    }
    let mut hi = at + 1;
    while hi < toks.len() && binds(&toks[hi]) {
        hi += 1;
    }
    Some((operand(&toks[lo..at])?, operand(&toks[at + 1..hi])?))
}

/// The name of a call, when the callee is a plain identifier.
pub(crate) fn call_name<'s>(call: Node, source: &'s str) -> Option<&'s str> {
    let name = call.child_by_field_name("name")?;
    (name.kind() == "identifier").then(|| text(name, source))
}

/// Whether an operand is a value MATLAB knows to be logical. A single leaf is
/// logical when it is `true`/`false`, a comparison or `|`/`&` of known values,
/// `isempty`/`ischar`/`strcmp`, or a variable that holds one of these. A chain
/// with several leaves is logical when all its leaves have known classes.
pub(crate) fn is_known_logical(operand: &Operand, source: &str) -> bool {
    match operand.leaves.as_slice() {
        [leaf] => logical_value(*leaf, source, 0),
        leaves => leaves.iter().all(|leaf| class_known(*leaf, source, 0)),
    }
}

fn all_named_known(node: Node, source: &str, depth: usize) -> bool {
    let mut cursor = node.walk();
    let all = node
        .named_children(&mut cursor)
        .all(|c| class_known(c, source, depth + 1));
    all
}

/// Whether `node` is known to be logical (see [`is_known_logical`]).
fn logical_value(node: Node, source: &str, depth: usize) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match node.kind() {
        "parenthesized_expression" => node
            .named_child(0)
            .is_some_and(|inner| logical_value(inner, source, depth + 1)),
        "comparison_operator" => all_named_known(node, source, depth),
        "binary_operator" if is_chain_node(node, source) => all_named_known(node, source, depth),
        "identifier" => {
            let name = text(node, source);
            if matches!(name, "true" | "false") {
                return true;
            }
            let r = reaching(node, source, name);
            !r.indexed_since
                && matches!(r.binding, Binding::Value(rhs) if logical_value(rhs, source, depth + 1))
        }
        "function_call" => call_name(node, source).is_some_and(|n| LOGICAL_CALLS.contains(&n)),
        _ => false,
    }
}

/// Whether `node` is known to be numeric.
fn numeric_value(node: Node, source: &str, depth: usize) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match node.kind() {
        "number" | "string" => true,
        "parenthesized_expression" => node
            .named_child(0)
            .is_some_and(|inner| class_known(inner, source, depth + 1)),
        "unary_operator" => all_named_known(node, source, depth),
        "binary_operator" if !is_chain_node(node, source) => all_named_known(node, source, depth),
        "identifier" => {
            let name = text(node, source);
            let r = reaching(node, source, name);
            !r.indexed_since
                && matches!(r.binding, Binding::Value(rhs) if class_known(rhs, source, depth + 1))
        }
        "function_call" => call_name(node, source).is_some_and(|n| NUMERIC_CALLS.contains(&n)),
        _ => false,
    }
}

/// Whether `node` is a brace index (`c{1}`): the grammar gives it a
/// `function_call` with an unnamed `{` child. Probed: MATLAB does not know the
/// value of a cell element, even of a constant cell, so it is never known here.
fn is_brace_index(node: Node) -> bool {
    node.kind() == "function_call" && {
        let mut cursor = node.walk();
        let brace = node
            .children(&mut cursor)
            .any(|c| !c.is_named() && c.kind() == "{");
        brace
    }
}

/// Whether the class of `node` is known: its value is constant, or it is
/// numeric or logical in every run.
fn class_known(node: Node, source: &str, depth: usize) -> bool {
    if depth > MAX_DEPTH || is_brace_index(node) {
        return false;
    }
    is_constant(node, source)
        || numeric_value(node, source, depth)
        || logical_value(node, source, depth)
}

#[cfg(test)]
mod tests {
    use crate::test_util::parse;
    use tree_sitter::Node;

    use super::*;

    /// The first `binary_operator` in `node` whose operator is `op`.
    fn find_op<'a>(node: Node<'a>, source: &str, op: &str) -> Option<Node<'a>> {
        if node.kind() == "binary_operator" && operator_of(node, source) == Some(op) {
            return Some(node);
        }
        let mut cursor = node.walk();
        let found = node
            .children(&mut cursor)
            .find_map(|c| find_op(c, source, op));
        found
    }

    /// The leaf texts of the two operands of the first `op` in `src`.
    fn operand_texts(src: &str, op: &str) -> (Vec<String>, Vec<String>) {
        let tree = parse(src);
        let node = find_op(tree.root_node(), src, op).expect("operator");
        let (left, right) = elementwise_operands(node, src).expect("operands");
        let texts = |o: &Operand| o.leaves.iter().map(|n| text(*n, src).to_string()).collect();
        (texts(&left), texts(&right))
    }

    fn strs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn or_takes_comparisons_as_operands_not_the_mis_parsed_leaves() {
        // The tree has `1 | L`; MATLAB has `(L > 1) | (L < 0.1)`.
        let (left, right) = operand_texts("if L > 1 | L < 0.1\nend\n", "|");
        assert_eq!(left, strs(&["L", "1"]));
        assert_eq!(right, strs(&["L", "0.1"]));
    }

    #[test]
    fn and_takes_comparisons_as_operands() {
        let (left, right) = operand_texts("if running == 1 & isempty(items) == 0\nend\n", "&");
        assert_eq!(left, strs(&["running", "1"]));
        assert_eq!(right, strs(&["isempty(items)", "0"]));
    }

    #[test]
    fn and_binds_tighter_than_or() {
        // `a > 1 & b < 2 | c`: the `|` has `a > 1 & b < 2` on the left.
        let (left, right) = operand_texts("if a > 1 & b < 2 | c\nend\n", "|");
        assert_eq!(left, strs(&["a", "1", "b", "2"]));
        assert_eq!(right, strs(&["c"]));
        let (left, right) = operand_texts("if a > 1 & b < 2 | c\nend\n", "&");
        assert_eq!(left, strs(&["a", "1"]));
        assert_eq!(right, strs(&["b", "2"]));
    }

    #[test]
    fn or_on_the_left_of_a_comparison_takes_the_comparison_on_its_right() {
        // `a | b > 1`: MATLAB has `a | (b > 1)`.
        let (left, right) = operand_texts("if a | b > 1\nend\n", "|");
        assert_eq!(left, strs(&["a"]));
        assert_eq!(right, strs(&["b", "1"]));
    }

    #[test]
    fn arithmetic_stays_inside_its_operand() {
        let (left, right) = operand_texts("if x + 1 > 2 | y\nend\n", "|");
        assert_eq!(left, strs(&["x + 1", "2"]));
        assert_eq!(right, strs(&["y"]));
    }

    #[test]
    fn parenthesised_operand_is_one_leaf() {
        let (left, right) = operand_texts("if (a > 1) | b\nend\n", "|");
        assert_eq!(left, strs(&["(a > 1)"]));
        assert_eq!(right, strs(&["b"]));
    }

    #[test]
    fn short_circuit_ends_the_chain() {
        // `a > 1 | b || c`: the `|` operands stop at `||`.
        let (left, right) = operand_texts("if a > 1 | b || c\nend\n", "|");
        assert_eq!(left, strs(&["a", "1"]));
        assert_eq!(right, strs(&["b"]));
    }

    /// Whether the left (`true`) or the right operand of the first `op` in
    /// `src` is known to be logical.
    fn known(src: &str, op: &str) -> (bool, bool) {
        let tree = parse(src);
        let node = find_op(tree.root_node(), src, op).expect("operator");
        let (left, right) = elementwise_operands(node, src).expect("operands");
        (is_known_logical(&left, src), is_known_logical(&right, src))
    }

    #[test]
    fn constant_comparison_is_known_logical() {
        // Probed: `L = 0.75` makes `L > 1` a known logical value.
        assert_eq!(
            known("L = 0.75;\nif L > 1 | L < 0.1\nend\n", "|"),
            (true, true)
        );
    }

    #[test]
    fn comparison_of_input_is_not_known() {
        assert_eq!(
            known("x = input('x');\nif x > 1 | x < 0\nend\n", "|"),
            (false, false)
        );
    }

    #[test]
    fn isempty_and_numel_comparisons_are_known_logical() {
        let src = "x = input('x');\nif x == 1 & isempty(items) == 0\nend\n";
        assert_eq!(known(src, "&"), (false, true));
        let src = "x = input('x');\nn = numel(x);\nif n > 3 | x > 0\nend\n";
        assert_eq!(known(src, "|"), (true, false));
    }

    #[test]
    fn logical_variables_and_literals() {
        assert_eq!(
            known("flag = true;\nif flag & x > 1\nend\n", "&"),
            (true, false)
        );
        assert_eq!(
            known("x = input('x');\nif x | 0\nend\n", "|"),
            (false, false)
        );
    }

    #[test]
    fn indexed_or_field_values_are_not_known() {
        // Probed: an indexed array that grows, a struct field and a cell
        // element all stay unknown, so OR2 fires on them.
        let src = "x = zeros(1, 3);\nx(2) = 5;\nif x(2) > 1 | x(2) < 0\nend\n";
        assert_eq!(known(src, "|"), (false, false));
        let src = "s.d = 0;\nif s.d == 0 | isnan(s.d)\nend\n";
        assert_eq!(known(src, "|"), (false, false));
        let src = "c = {1, 2};\nif c{1} > 1 & c{2} < 2\nend\n";
        assert_eq!(known(src, "&"), (false, false));
    }

    #[test]
    fn operator_outside_a_chain_has_no_operands_of_its_own() {
        // `x` is not an operand of `|` in `f(a | b)`: the `|` is its own chain.
        let src = "if f(a | b)\nend\n";
        let tree = parse(src);
        let node = find_op(tree.root_node(), src, "|").expect("operator");
        let (left, right) = elementwise_operands(node, src).expect("operands");
        assert_eq!(left.leaves.len(), 1);
        assert_eq!(right.leaves.len(), 1);
    }
}
