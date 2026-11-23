//! # Control Flow Analysis — Reachability and Definite-Assignment Tracking
//!
//! This module provides two core analyses for MATLAB source files:
//!
//! 1. **Unreachable code detection** ([`find_unreachable`]) — identifies statements
//!    that follow `return`, `break`, or `continue` within the same block and can
//!    therefore never execute.
//!
//! 2. **Definite-assignment analysis** ([`analyze_definite_assignment`]) — determines
//!    which variables are guaranteed to have been assigned before a given point in
//!    a function, and which variable uses might reference unset variables. This is
//!    the foundation for "unset variables" lint rules.
//!
//! Both analyses operate on tree-sitter parse trees produced by `tree-sitter-matlab`.
//!
//! ## Unreachable code example
//!
//! ```ignore
//! function foo()
//!     return;
//!     x = 1;   % ← unreachable
//!     disp(x); % ← unreachable
//! end
//! ```
//!
//! ## Definite-assignment example
//!
//! ```ignore
//! function y = foo(x)
//!     if x > 0
//!         z = 1;
//!     end
//!     y = z;  % ← z is possibly unset (no else branch)
//! end
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use tree_sitter::Parser;
//! use malt_rules::analysis::control_flow::{find_unreachable, analyze_definite_assignment};
//!
//! let source = "function foo()\n    return;\n    x = 1;\nend\n";
//! let mut parser = Parser::new();
//! parser.set_language(&tree_sitter_matlab::LANGUAGE.into()).unwrap();
//! let tree = parser.parse(source, None).unwrap();
//!
//! let unreachable = find_unreachable(&tree, source);
//! assert_eq!(unreachable.len(), 1);
//!
//! // For definite-assignment, pass the function_definition node:
//! let root = tree.root_node();
//! let func_node = root.child(0).unwrap();
//! let da = analyze_definite_assignment(func_node, source);
//! ```

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

use tree_sitter::{Node, Tree};

use super::reaching::resolved_values;

// ---------------------------------------------------------------------------
// Unreachable code types
// ---------------------------------------------------------------------------

/// A span of code that can never be reached because it follows a
/// `return`, `break`, or `continue` statement in the same block.
#[derive(Debug, Clone)]
pub struct UnreachableSpan {
    /// Byte range of the unreachable code.
    pub byte_range: std::ops::Range<usize>,
    /// 1-indexed line where unreachable code starts.
    pub line: usize,
    /// 1-indexed column where unreachable code starts.
    pub column: usize,
    /// The statement kind that causes unreachability (`"return"`, `"break"`, or `"continue"`).
    pub cause: &'static str,
}

// ---------------------------------------------------------------------------
// Definite-assignment types
// ---------------------------------------------------------------------------

/// Result of definite-assignment analysis for a single function scope.
#[derive(Debug, Clone)]
pub struct DefiniteAssignment {
    /// Variables that are definitely assigned at every path to the end of the function.
    pub definitely_assigned: HashSet<String>,
    /// Variables that are possibly unset (assigned on some paths but not all)
    /// at the point where they are first used.
    pub possibly_unset: Vec<PossiblyUnsetVar>,
}

/// A variable that might not be assigned before its use.
#[derive(Debug, Clone)]
pub struct PossiblyUnsetVar {
    /// Variable name.
    pub name: String,
    /// Byte range of the use site.
    pub use_byte_range: std::ops::Range<usize>,
    /// 1-indexed line of the use.
    pub use_line: usize,
    /// 1-indexed column of the use.
    pub use_column: usize,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the source text covered by a node.
pub(crate) fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// Returns the termination cause string for a terminating statement node kind.
/// The name of a command that is only a name (`q;`), if `node` is one.
pub(crate) fn bare_command_name(node: Node) -> Option<Node> {
    if node.kind() != "command" || node.named_child_count() != 1 {
        return None;
    }
    node.named_child(0)
        .filter(|child| child.kind() == "command_name")
}

fn termination_cause(kind: &str) -> Option<&'static str> {
    match kind {
        "return_statement" => Some("return"),
        "break_statement" => Some("break"),
        "continue_statement" => Some("continue"),
        _ => None,
    }
}

/// Whether a block ends control flow for good: it contains a `return`,
/// `break` or `continue`, or a call that never returns (`error`, `throw`,
/// `rethrow`, `exit`, `quit`).
fn block_never_falls_through(block: Node, source: &str) -> bool {
    let mut cursor = block.walk();
    let found = block.named_children(&mut cursor).any(|stmt| {
        if termination_cause(stmt.kind()).is_some() {
            return true;
        }
        let callee = match stmt.kind() {
            "function_call" => stmt.child_by_field_name("name"),
            "command" => stmt.named_child(0),
            _ => None,
        };
        callee.is_some_and(|c| {
            matches!(
                node_text(c, source),
                "error" | "throw" | "rethrow" | "exit" | "quit"
            )
        })
    });
    found
}

/// Iterate the named children of a node.
pub(crate) fn named_children(node: Node) -> Vec<Node> {
    let mut children = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        children.push(child);
    }
    children
}

// ---------------------------------------------------------------------------
// 1. Unreachable code detection
// ---------------------------------------------------------------------------

/// Find all spans of unreachable code in a file.
///
/// Walks every `block` node in the parse tree via DFS. Within each block,
/// when a `return_statement`, `break_statement`, or `continue_statement` is
/// encountered, all subsequent named sibling statements in that block are
/// collected as unreachable.
pub fn find_unreachable(tree: &Tree, source: &str) -> Vec<UnreachableSpan> {
    let mut spans = Vec::new();
    collect_unreachable_dfs(tree.root_node(), source, &mut spans);
    spans
}

/// DFS helper that visits every node and, for `block` nodes, checks for
/// unreachable code after terminating statements.
fn collect_unreachable_dfs(node: Node, source: &str, spans: &mut Vec<UnreachableSpan>) {
    if node.kind() == "block" || is_flow_root(node) {
        check_block_unreachable(node, source, spans);
    }

    // Recurse into all named children regardless — blocks inside if/for/etc.
    // are children of those compound statements, not of the outer block.
    let children = named_children(node);
    for child in children {
        collect_unreachable_dfs(child, source, spans);
    }
}

/// `true` for a script's body: a file with no function definitions and no
/// syntax errors, whose top-level statements are direct children of the file.
/// Probes: MATLAB flags statements after `return`, `break`, `continue` and `error`
/// at script level (a top-level `break` is a syntax error, BRKFOR, but still ends the flow).
pub fn is_script_body(node: Node) -> bool {
    if node.kind() != "source_file" || node.has_error() {
        return false;
    }
    !named_children(node)
        .iter()
        .any(|c| c.kind() == "function_definition")
}

/// `true` for a file without function definitions, syntax errors allowed: its
/// top-level statements are a flow that a `break` or `return` ends (probed: t__t1,
/// whose statement after a top-level `break` is UNRCH though the file has an error).
fn is_flow_root(node: Node) -> bool {
    node.kind() == "source_file"
        && !named_children(node)
            .iter()
            .any(|c| c.kind() == "function_definition")
}

/// `true` for a statement-level unary `+x` or `-x` (not `++x`/`--x`) that
/// follows a completed statement. MATLAB reads the line break before it as the
/// end of the previous statement, which is then reported by NOPRC.
pub fn is_split_statement_continuation(node: Node, source: &str) -> bool {
    // `-x + 2` is a binary operation whose left operand is the unary: the grammar
    // groups it that way, MATLAB starts the statement at the sign.
    let unary = match node.kind() {
        "unary_operator" => node,
        "binary_operator" => match node.child_by_field_name("left") {
            Some(left)
                if left.kind() == "unary_operator" && left.start_byte() == node.start_byte() =>
            {
                left
            }
            _ => return false,
        },
        _ => return false,
    };
    if !node
        .parent()
        .is_some_and(|p| matches!(p.kind(), "source_file" | "block"))
    {
        return false;
    }
    let op = node_text(unary, source).trim();
    let is_incr_decr = unary
        .child_by_field_name("operand")
        .and_then(|o| {
            (o.kind() == "unary_operator")
                .then(|| o.child_by_field_name("operand"))
                .flatten()
        })
        .is_some_and(|o| o.kind() == "identifier");
    // A comment that trails the previous statement (`x = 1 %c`) does not end the line:
    // the line break after it still ends that statement. A comment line does.
    let follows_statement = match node.prev_named_sibling() {
        Some(p) if p.kind() == "comment" => p.prev_named_sibling().is_some_and(|q| {
            q.kind() != "comment" && q.end_position().row == p.start_position().row
        }),
        Some(_) => true,
        None => false,
    };
    (op.starts_with('+') || op.starts_with('-')) && !is_incr_decr && follows_statement
}

/// `true` for `while true ... end` without a `break` that leaves that loop:
/// the statements after it can never run.
pub(crate) fn is_endless_loop(node: Node, source: &str) -> bool {
    node.kind() == "while_statement"
        && node
            .child_by_field_name("condition")
            .is_some_and(|c| node_text(c, source) == "true")
        && !contains_loop_break(node)
}

/// `true` when `node` holds a `break` that belongs to it (nested loops own their breaks).
fn contains_loop_break(node: Node) -> bool {
    let mut cursor = node.walk();
    let children: Vec<Node> = node.named_children(&mut cursor).collect();
    children.into_iter().any(|child| match child.kind() {
        "break_statement" => true,
        "for_statement" | "while_statement" => false,
        _ => contains_loop_break(child),
    })
}

/// `true` when the statement holding `node` follows a `return`, `break`,
/// `continue`, `error(...)` or an endless `while true` loop in the same block.
/// MATLAB's style checks skip such statements (only UNRCH reports them).
///
/// Every check calls this for its nodes, so the first terminating statement of
/// each block is memoised (see [`first_terminator`]). A block of n statements
/// costs one scan, not one scan per statement.
pub fn in_unreachable_code(node: Node, source: &str) -> bool {
    if in_dead_constant_region(node, source) {
        return true;
    }
    let mut statement = node;
    while let Some(parent) = statement.parent() {
        if matches!(parent.kind(), "block" | "source_file") {
            break;
        }
        statement = parent;
    }
    let Some(block) = statement.parent() else {
        return false;
    };
    // A statement is unreachable when a terminating statement comes before it.
    first_terminator(block, source)
        .is_some_and(|at| at < statement.start_byte() - block.start_byte())
}

/// Whether `node` lies in a dead clause of a constant condition (`c = true; if c ...
/// else ... end` makes the `else` dead). MATLAB does not analyse such code, so the
/// checks that use [`in_unreachable_code`] stay silent there.
fn in_dead_constant_region(node: Node, source: &str) -> bool {
    let mut root = node;
    while let Some(parent) = root.parent() {
        root = parent;
    }
    let epoch = malt_core::parse_epoch();
    let stale = DEAD_REGIONS.with(|memo| memo.borrow().epoch != Some(epoch));
    if stale {
        let mut regions = crate::unused::dead_constant_regions(root, source);
        regions.sort_by_key(|r| r.start);
        let mut reach = 0;
        let max_end = regions
            .iter()
            .map(|r| {
                reach = reach.max(r.end);
                reach
            })
            .collect();
        DEAD_REGIONS.with(|memo| {
            *memo.borrow_mut() = DeadRegions {
                epoch: Some(epoch),
                regions,
                max_end,
            }
        });
    }
    let offset = node.start_byte();
    DEAD_REGIONS.with(|memo| {
        let memo = memo.borrow();
        let found = memo.regions.partition_point(|r| r.start <= offset);
        found > 0 && memo.max_end[found - 1] > offset
    })
}

thread_local! {
    /// Dead clauses of the current parse, see [`in_dead_constant_region`].
    static DEAD_REGIONS: RefCell<DeadRegions> = RefCell::new(DeadRegions::default());
}

/// The dead clauses of one parse, sorted by start, with the furthest end among those
/// that start at or before each one (so a containing region is found by search).
#[derive(Default)]
struct DeadRegions {
    epoch: Option<u64>,
    regions: Vec<Range<usize>>,
    max_end: Vec<usize>,
}

thread_local! {
    /// Memo of [`first_terminator`] for the current parse epoch.
    static TERMINATORS: RefCell<TerminatorMemo> = RefCell::new(TerminatorMemo::default());
}

/// [`first_terminator`]'s memo: valid for one parse epoch (see
/// `malt_core::parse_epoch`), because node ids are addresses that a later parse
/// can reuse.
#[derive(Default)]
struct TerminatorMemo {
    epoch: u64,
    /// Block id to the offset, from the block start, of its first terminating
    /// statement. Offsets are relative to the block, so a subtree that occurs at
    /// two places gives the same answer at both.
    blocks: HashMap<usize, Option<usize>>,
}

/// Offset (from the start of `block`) of the first named statement of `block`
/// that ends control flow for the statements after it, if there is one.
fn first_terminator(block: Node, source: &str) -> Option<usize> {
    TERMINATORS.with(|memo| {
        let mut memo = memo.borrow_mut();
        let epoch = malt_core::parse_epoch();
        if memo.epoch != epoch {
            memo.blocks.clear();
            memo.epoch = epoch;
        }
        if let Some(found) = memo.blocks.get(&block.id()) {
            return *found;
        }
        let mut cursor = block.walk();
        let found = block
            .named_children(&mut cursor)
            .find(|child| child.kind() != "comment" && ends_control_flow(*child, source))
            .map(|child| child.start_byte() - block.start_byte());
        memo.blocks.insert(block.id(), found);
        found
    })
}

/// `true` when `statement` ends control flow for the statements after it in
/// its block: a `return`, `break`, `continue`, an `if` whose every branch
/// terminates, an endless `while true` loop, or an `error(...)` call.
fn ends_control_flow(statement: Node, source: &str) -> bool {
    if termination_cause(statement.kind()).is_some()
        || (statement.kind() == "if_statement" && if_terminates(statement, source))
        || (statement.kind() == "switch_statement" && switch_terminates(statement, source))
        || is_endless_loop(statement, source)
    {
        return true;
    }
    let callee = match statement.kind() {
        "function_call" => statement.child_by_field_name("name"),
        "command" => statement.named_child(0),
        _ => None,
    };
    callee.is_some_and(|c| node_text(c, source) == "error")
        && error_call_always_throws(statement, source)
}

/// Whether an `error` call always throws. Its first argument is not a variable, or
/// the call has one argument whose every value is certain text (a non-empty literal or
/// `sprintf`). An unknown message may be empty, and `error` then does nothing (probed:
/// `msg = 'abc'; error(msg)` leaves the next statement unreached, a parameter `msg`
/// does not).
pub fn error_call_always_throws(call: Node, source: &str) -> bool {
    let args = call_argument_nodes(call);
    let Some(first) = args.first() else {
        return true;
    };
    if first.kind() != "identifier" {
        return true;
    }
    args.len() == 1
        && resolved_values(*first, source)
            .is_some_and(|values| values.iter().all(|v| certain_text(*v, source)))
}

/// The argument nodes of a `function_call` or a `command`, without comments.
pub(crate) fn call_argument_nodes(call: Node) -> Vec<Node> {
    if call.kind() == "command" {
        return named_children(call)
            .into_iter()
            .skip(1)
            .filter(|c| c.kind() != "comment")
            .collect();
    }
    let arguments = named_children(call)
        .into_iter()
        .find(|c| c.kind() == "arguments");
    arguments.map(named_children).unwrap_or_default()
}

/// Text that `error` always throws with: a non-empty single-quoted literal or `sprintf`.
fn certain_text(node: Node, source: &str) -> bool {
    match node.kind() {
        "string" => {
            let text = node_text(node, source);
            text.len() > 2 && text.starts_with('\'')
        }
        "function_call" => node
            .child_by_field_name("name")
            .is_some_and(|name| node_text(name, source) == "sprintf"),
        _ => false,
    }
}

/// For a single `block` node, find the first terminating statement and mark
/// all subsequent named siblings as unreachable.
fn check_block_unreachable(block: Node, source: &str, spans: &mut Vec<UnreachableSpan>) {
    let children = named_children(block);
    let mut terminator_found: Option<&'static str> = None;

    for child in &children {
        if let Some(cause) = terminator_found {
            // Skip comment nodes — they are not "code". A nested function is not
            // code of this flow either (probed: `return` then `function g()` in f).
            if matches!(child.kind(), "comment" | "function_definition") {
                continue;
            }
            // MATLAB reports only the first statement of an unreachable run:
            // its message reads "this statement (and possibly following ones)".
            let pos = child.start_position();
            spans.push(UnreachableSpan {
                byte_range: child.start_byte()..child.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                cause,
            });
            break;
        } else if let Some(cause) = termination_cause(child.kind()) {
            terminator_found = Some(cause);
        } else if (child.kind() == "if_statement" && if_terminates(*child, source))
            || (child.kind() == "switch_statement" && switch_terminates(*child, source))
        {
            // Probed (plan 6.2): a `switch` with an `otherwise` whose every branch
            // returns ends the flow like an `if` with an `else`.
            terminator_found = Some("return");
        }
    }

    let _ = source; // used implicitly through node byte ranges
}

// ---------------------------------------------------------------------------
// 2. Block termination check
// ---------------------------------------------------------------------------

/// Check if a `block` node unconditionally terminates (all paths through it
/// reach a `return`, `break`, or `continue` statement).
///
/// A block terminates if:
/// - It directly contains a `return_statement`, `break_statement`, or
///   `continue_statement` at the top level, OR
/// - Its last named statement is an `if_statement` that has an `else_clause`
///   and ALL branches (if, elseif*, else) terminate, OR
/// - Its last named statement is a `switch_statement` that has an
///   `otherwise_clause` and ALL cases plus otherwise terminate.
pub fn block_terminates(block_node: Node, source: &str) -> bool {
    let children = named_children(block_node);

    for child in &children {
        // Direct terminator at top level → block terminates.
        if termination_cause(child.kind()).is_some() {
            return true;
        }
    }

    // Check the last named statement for compound termination.
    if let Some(last) = children.last() {
        match last.kind() {
            "if_statement" => return if_terminates(*last, source),
            "switch_statement" => return switch_terminates(*last, source),
            _ => {}
        }
    }

    false
}

/// Check if an `if_statement` unconditionally terminates.
///
/// Every clause that can run must terminate, and some clause must always run: an
/// `else_clause`, or a clause whose condition is a literal true. Probed: `if 1,
/// return, end` makes the code after it unreachable. A clause whose condition is a
/// literal false never runs.
fn if_terminates(if_node: Node, source: &str) -> bool {
    let mut clauses: Vec<(Option<bool>, Node)> = Vec::new();
    for child in named_children(if_node) {
        match child.kind() {
            "block" => clauses.push((literal_condition(if_node, source), child)),
            "elseif_clause" | "else_clause" => {
                let truth = if child.kind() == "else_clause" {
                    Some(true)
                } else {
                    literal_condition(child, source)
                };
                match named_children(child)
                    .into_iter()
                    .find(|c| c.kind() == "block")
                {
                    Some(body) => clauses.push((truth, body)),
                    None => return false,
                }
            }
            _ => {}
        }
    }
    for (truth, body) in clauses {
        match truth {
            Some(false) => {}
            Some(true) => return block_terminates(body, source),
            None if !block_terminates(body, source) => return false,
            None => {}
        }
    }
    false
}

/// The truth of an `if` or `elseif` clause whose condition is a literal.
pub(crate) fn literal_condition(clause: Node, source: &str) -> Option<bool> {
    clause
        .child_by_field_name("condition")
        .and_then(|condition| literal_text_truth(node_text(condition, source).trim()))
}

/// The truth of a literal: `true`, `false`, a number, or one of those negated with
/// `~`. Variables are not folded here.
fn literal_text_truth(text: &str) -> Option<bool> {
    if let Some(rest) = text.strip_prefix('~') {
        return literal_text_truth(rest.trim_start()).map(|truth| !truth);
    }
    match text {
        "true" => Some(true),
        "false" => Some(false),
        _ if text.starts_with(|c: char| c.is_ascii_digit()) => {
            text.parse::<f64>().ok().map(|value| value != 0.0)
        }
        _ => None,
    }
}

/// Check if a `switch_statement` unconditionally terminates.
///
/// Requires an `otherwise_clause`, and every case plus otherwise must terminate.
fn switch_terminates(switch_node: Node, source: &str) -> bool {
    let mut has_otherwise = false;
    let children = named_children(switch_node);

    for child in &children {
        match child.kind() {
            "case_clause" => {
                let case_children = named_children(*child);
                let body = case_children.iter().find(|c| c.kind() == "block");
                match body {
                    Some(b) if block_terminates(*b, source) => {}
                    _ => return false,
                }
            }
            "otherwise_clause" => {
                has_otherwise = true;
                let ow_children = named_children(*child);
                let body = ow_children.iter().find(|c| c.kind() == "block");
                match body {
                    Some(b) if block_terminates(*b, source) => {}
                    _ => return false,
                }
            }
            _ => {}
        }
    }

    has_otherwise
}

// ---------------------------------------------------------------------------
// 3. Definite-assignment analysis
// ---------------------------------------------------------------------------

/// Analyze definite assignment for a single function scope.
///
/// `func_node` should be a `function_definition` node. The analysis:
/// 1. Collects input and output arguments as definitely assigned.
/// 2. Walks the function body, tracking which variables are definitely assigned.
/// 3. For each variable use, checks whether it is in the definitely-assigned set.
///    If not (and it is not a known global/persistent), records it as possibly unset.
pub fn analyze_definite_assignment(func_node: Node, source: &str) -> DefiniteAssignment {
    let mut assigned: HashSet<String> = HashSet::new();
    let mut possibly_unset: Vec<PossiblyUnsetVar> = Vec::new();

    // Collect function input arguments as definitely assigned.
    collect_function_args(func_node, source, &mut assigned);

    // Find the function body block.
    let children = named_children(func_node);
    for child in &children {
        if child.kind() == "block" {
            walk_block(*child, source, &mut assigned, &mut possibly_unset);
            break;
        }
    }

    DefiniteAssignment {
        definitely_assigned: assigned,
        possibly_unset,
    }
}

/// Collect input and output argument names from a `function_definition` node
/// into the `assigned` set.
pub(crate) fn collect_function_args(func_node: Node, source: &str, assigned: &mut HashSet<String>) {
    let mut cursor = func_node.walk();
    for child in func_node.children(&mut cursor) {
        match child.kind() {
            "function_arguments" => {
                let mut inner = child.walk();
                for param in child.children(&mut inner) {
                    if param.kind() == "identifier" {
                        assigned.insert(node_text(param, source).to_string());
                    }
                }
            }
            "function_output" => {
                collect_output_identifiers(child, source, assigned);
            }
            _ => {}
        }
    }
}

/// Recursively collect output identifiers (handles `multioutput_variable`).
fn collect_output_identifiers(node: Node, source: &str, assigned: &mut HashSet<String>) {
    match node.kind() {
        "identifier" => {
            assigned.insert(node_text(node, source).to_string());
        }
        "multioutput_variable" | "function_output" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    assigned.insert(node_text(child, source).to_string());
                } else if child.kind() == "multioutput_variable" {
                    collect_output_identifiers(child, source, assigned);
                }
            }
        }
        _ => {}
    }
}

/// Walk a `block` node, updating the definitely-assigned set and recording
/// possibly-unset variable uses.
fn walk_block(
    block: Node,
    source: &str,
    assigned: &mut HashSet<String>,
    uses: &mut Vec<PossiblyUnsetVar>,
) {
    let children = named_children(block);

    for child in &children {
        walk_statement(*child, source, assigned, uses);
    }
}

/// Process a single statement node within a block.
fn walk_statement(
    node: Node,
    source: &str,
    assigned: &mut HashSet<String>,
    uses: &mut Vec<PossiblyUnsetVar>,
) {
    match node.kind() {
        "assignment" => {
            // First, check the RHS for uses of possibly-unset variables.
            if let Some(rhs) = node.child_by_field_name("right") {
                collect_uses_in_expr(rhs, source, assigned, uses);
            }
            // Then add LHS variables to the assigned set.
            if let Some(lhs) = node.child_by_field_name("left") {
                collect_assignment_targets(lhs, source, assigned, uses);
            }
        }
        "global_operator" | "persistent_operator" => {
            // Global/persistent declarations make the variables definitely assigned.
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    assigned.insert(node_text(child, source).to_string());
                }
            }
        }
        "if_statement" => {
            walk_if_statement(node, source, assigned, uses);
        }
        "switch_statement" => {
            walk_switch_statement(node, source, assigned, uses);
        }
        "for_statement" => {
            walk_for_statement(node, source, assigned, uses);
        }
        "while_statement" => {
            walk_while_statement(node, source, assigned, uses);
        }
        "try_statement" => {
            walk_try_statement(node, source, assigned, uses);
        }
        "return_statement" | "break_statement" | "continue_statement" => {
            // No more reachable code in this block after a terminator.
            // The caller (walk_block) will still iterate remaining children,
            // but that is fine — unreachable code detection is handled
            // separately by find_unreachable(). For definite-assignment we
            // could stop early, but for simplicity we continue (it won't
            // produce false positives since no new uses will appear in
            // unreachable code that matters).
            //
            // However, we should signal the caller to stop processing.
            // We handle this by returning early from walk_block via a
            // special approach — but to keep walk_block simple, we just
            // let it continue. Unreachable statements won't add incorrect
            // "possibly unset" entries because the variables they reference
            // would also be unreachable. This is acceptable for now.
        }
        "command" if super::liveness::clear_names(node_text(node, source)).is_some() => {
            // `clear x` leaves x unset again (probed: `x = 1; clear x; disp(x)`
            // reports NODEF). The name itself is not a use.
            for name in super::liveness::clear_names(node_text(node, source)).unwrap_or_default() {
                assigned.remove(&name);
            }
        }
        "command" if node.named_child_count() == 1 && bare_command_name(node).is_some() => {
            // `q;` parses as a command whose only child is its name: a read of `q`
            // (probed: `q;` before `q = 2` reports NODEF).
            if let Some(name_node) = bare_command_name(node) {
                let name = node_text(name_node, source);
                if !assigned.contains(name) {
                    let pos = name_node.start_position();
                    uses.push(PossiblyUnsetVar {
                        name: name.to_string(),
                        use_byte_range: name_node.start_byte()..name_node.end_byte(),
                        use_line: pos.row + 1,
                        use_column: pos.column + 1,
                    });
                }
            }
        }
        "function_call" | "command" => {
            // Check all arguments for uses of unset variables.
            collect_uses_in_expr(node, source, assigned, uses);
        }
        _ => {
            // For any other statement type, recursively look for variable uses.
            collect_uses_in_expr(node, source, assigned, uses);
        }
    }
}

/// Walk an `if_statement`, handling branch merging.
///
/// A variable is definitely assigned after the if/elseif/else chain only if
/// it is assigned in ALL branches AND there IS an else branch.
fn walk_if_statement(
    node: Node,
    source: &str,
    assigned: &mut HashSet<String>,
    uses: &mut Vec<PossiblyUnsetVar>,
) {
    let children = named_children(node);

    // Check the condition for uses (`condition` is a field, not a node kind).
    if let Some(condition) = node.child_by_field_name("condition") {
        collect_uses_in_expr(condition, source, assigned, uses);
    }

    let mut branch_sets: Vec<HashSet<String>> = Vec::new();
    let mut has_else = false;

    // Walk the main if-body block.
    for child in &children {
        if child.kind() == "block" {
            let mut branch_assigned = assigned.clone();
            walk_block(*child, source, &mut branch_assigned, uses);
            // A branch that never falls through (return, error, ...) does not
            // limit what is assigned after the statement.
            if !block_never_falls_through(*child, source) {
                branch_sets.push(branch_assigned);
            }
            break;
        }
    }

    // Walk elseif clauses.
    for child in &children {
        if child.kind() == "elseif_clause" {
            let elseif_children = named_children(*child);

            // Check elseif condition for uses.
            if let Some(condition) = child.child_by_field_name("condition") {
                collect_uses_in_expr(condition, source, assigned, uses);
            }

            // Walk elseif body.
            for ec in &elseif_children {
                if ec.kind() == "block" {
                    let mut branch_assigned = assigned.clone();
                    walk_block(*ec, source, &mut branch_assigned, uses);
                    // A branch that never falls through (return, error, ...) does not
                    // limit what is assigned after the statement.
                    if !block_never_falls_through(*ec, source) {
                        branch_sets.push(branch_assigned);
                    }
                    break;
                }
            }
        }
    }

    // Walk else clause.
    for child in &children {
        if child.kind() == "else_clause" {
            has_else = true;
            let else_children = named_children(*child);
            for ec in &else_children {
                if ec.kind() == "block" {
                    let mut branch_assigned = assigned.clone();
                    walk_block(*ec, source, &mut branch_assigned, uses);
                    // A branch that never falls through (return, error, ...) does not
                    // limit what is assigned after the statement.
                    if !block_never_falls_through(*ec, source) {
                        branch_sets.push(branch_assigned);
                    }
                    break;
                }
            }
        }
    }

    // Merge: if there is an else clause, intersect all branch sets.
    // Only variables assigned in ALL branches are definitely assigned.
    if has_else && !branch_sets.is_empty() {
        let intersection = intersect_sets(&branch_sets);
        for name in intersection {
            assigned.insert(name);
        }
    }
    // If there is no else clause, assignments inside branches are NOT
    // definitely assigned (the condition might be false).
}

/// Walk a `switch_statement`, handling branch merging.
///
/// Similar to if: a variable is definitely assigned after the switch only
/// if it is assigned in ALL case clauses AND the otherwise clause.
fn walk_switch_statement(
    node: Node,
    source: &str,
    assigned: &mut HashSet<String>,
    uses: &mut Vec<PossiblyUnsetVar>,
) {
    let children = named_children(node);

    // Check the switch expression for uses. The grammar names it by the field
    // `condition`, not by a node kind.
    if let Some(subject) = node.child_by_field_name("condition") {
        collect_uses_in_expr(subject, source, assigned, uses);
    }

    let mut branch_sets: Vec<HashSet<String>> = Vec::new();
    let mut has_otherwise = false;

    for child in &children {
        match child.kind() {
            "case_clause" => {
                let case_children = named_children(*child);

                // Check the case expression for uses. The grammar names it by the field
                // `condition`, not by a node kind.
                if let Some(expr) = child.child_by_field_name("condition") {
                    collect_uses_in_expr(expr, source, assigned, uses);
                }

                // Walk case body.
                for cc in &case_children {
                    if cc.kind() == "block" {
                        let mut branch_assigned = assigned.clone();
                        walk_block(*cc, source, &mut branch_assigned, uses);
                        // A branch that never falls through (return, error, ...) does not
                        // limit what is assigned after the statement.
                        if !block_never_falls_through(*cc, source) {
                            branch_sets.push(branch_assigned);
                        }
                        break;
                    }
                }
            }
            "otherwise_clause" => {
                has_otherwise = true;
                let ow_children = named_children(*child);
                for oc in &ow_children {
                    if oc.kind() == "block" {
                        let mut branch_assigned = assigned.clone();
                        walk_block(*oc, source, &mut branch_assigned, uses);
                        // A branch that never falls through (return, error, ...) does not
                        // limit what is assigned after the statement.
                        if !block_never_falls_through(*oc, source) {
                            branch_sets.push(branch_assigned);
                        }
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    if has_otherwise && !branch_sets.is_empty() {
        let intersection = intersect_sets(&branch_sets);
        for name in intersection {
            assigned.insert(name);
        }
    }
}

/// Walk a `for_statement`.
///
/// The loop iterator variable IS definitely assigned inside the loop body.
/// However, assignments inside the loop body are NOT definitely assigned
/// after the loop (the loop might not execute).
fn walk_for_statement(
    node: Node,
    source: &str,
    assigned: &mut HashSet<String>,
    uses: &mut Vec<PossiblyUnsetVar>,
) {
    let children = named_children(node);

    // Find the iterator and extract the loop variable + range uses.
    let mut loop_var: Option<String> = None;
    for child in &children {
        if child.kind() == "iterator" {
            let mut found_var = false;
            let mut iter_cursor = child.walk();
            for ic in child.children(&mut iter_cursor) {
                if !found_var && ic.kind() == "identifier" {
                    loop_var = Some(node_text(ic, source).to_string());
                    found_var = true;
                } else if ic.kind() != "=" {
                    // Range expression — check for uses.
                    collect_uses_in_expr(ic, source, assigned, uses);
                }
            }
            break;
        }
    }

    // Walk the loop body with a clone. The loop variable is added to the
    // cloned set so that it is considered assigned inside the body.
    for child in &children {
        if child.kind() == "block" {
            let mut body_assigned = assigned.clone();
            if let Some(ref var) = loop_var {
                body_assigned.insert(var.clone());
            }
            walk_block(*child, source, &mut body_assigned, uses);
            // Do NOT merge body_assigned back — loop might not execute.
            break;
        }
    }
}

/// Walk a `while_statement`.
///
/// Assignments inside the loop body are NOT definitely assigned after the
/// loop (the loop might not execute). The condition is checked for uses.
fn walk_while_statement(
    node: Node,
    source: &str,
    assigned: &mut HashSet<String>,
    uses: &mut Vec<PossiblyUnsetVar>,
) {
    let children = named_children(node);

    // Check the condition for uses (`condition` is a field, not a node kind).
    if let Some(condition) = node.child_by_field_name("condition") {
        collect_uses_in_expr(condition, source, assigned, uses);
    }

    // Walk the body with a clone — do not merge back.
    for child in &children {
        if child.kind() == "block" {
            let mut body_assigned = assigned.clone();
            walk_block(*child, source, &mut body_assigned, uses);
            break;
        }
    }
}

/// Walk a `try_statement`.
///
/// A variable assigned only in the try block is NOT definitely assigned
/// (the catch path might skip it). Only variables assigned in BOTH the
/// try body and the catch body are definitely assigned.
fn walk_try_statement(
    node: Node,
    source: &str,
    assigned: &mut HashSet<String>,
    uses: &mut Vec<PossiblyUnsetVar>,
) {
    let children = named_children(node);

    let mut branch_sets: Vec<HashSet<String>> = Vec::new();

    // Walk the try body.
    for child in &children {
        if child.kind() == "block" {
            let mut try_assigned = assigned.clone();
            walk_block(*child, source, &mut try_assigned, uses);
            branch_sets.push(try_assigned);
            break;
        }
    }

    // Walk the catch clause.
    for child in &children {
        if child.kind() == "catch_clause" {
            let catch_children = named_children(*child);
            let mut catch_assigned = assigned.clone();

            // The catch identifier (if any) is definitely assigned inside catch.
            for cc in &catch_children {
                if cc.kind() == "identifier" {
                    catch_assigned.insert(node_text(*cc, source).to_string());
                }
            }

            for cc in &catch_children {
                if cc.kind() == "block" {
                    walk_block(*cc, source, &mut catch_assigned, uses);
                    break;
                }
            }

            branch_sets.push(catch_assigned);
        }
    }

    // Intersect: only variables assigned in both try and catch are definite.
    if branch_sets.len() >= 2 {
        let intersection = intersect_sets(&branch_sets);
        for name in intersection {
            assigned.insert(name);
        }
    }
    // If there is no catch clause, try-body assignments are still not
    // definitely assigned (an error could occur at any point).
}

// ---------------------------------------------------------------------------
// Expression-level use collection
// ---------------------------------------------------------------------------

/// Recursively walk an expression tree looking for `identifier` nodes that
/// represent variable uses. For each one, check if it is in the `assigned`
/// set. If not, record it as possibly unset.
///
/// Skips identifiers that appear in non-use positions:
/// - Function name in `function_call` (could be a built-in or an external function).
/// - Field name in `field_expression` (not a standalone variable).
pub(crate) fn collect_uses_in_expr(
    node: Node,
    source: &str,
    assigned: &HashSet<String>,
    uses: &mut Vec<PossiblyUnsetVar>,
) {
    if node.kind() == "identifier" {
        if is_variable_use(node) {
            let name = node_text(node, source);
            if !assigned.contains(name) {
                let pos = node.start_position();
                uses.push(PossiblyUnsetVar {
                    name: name.to_string(),
                    use_byte_range: node.start_byte()..node.end_byte(),
                    use_line: pos.row + 1,
                    use_column: pos.column + 1,
                });
            }
        }
        return;
    }

    // A lambda binds its parameters inside its body.
    if node.kind() == "lambda" {
        let mut inner = assigned.clone();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "arguments" {
                let mut c = child.walk();
                for p in child.children(&mut c) {
                    if p.kind() == "identifier" {
                        inner.insert(node_text(p, source).to_string());
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "arguments" {
                collect_uses_in_expr(child, source, &inner, uses);
            }
        }
        return;
    }

    // Recurse into children.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_uses_in_expr(child, source, assigned, uses);
    }
}

/// Determine if an `identifier` node represents a variable use (as opposed
/// to a function name or field name).
///
/// Returns `false` for:
/// - The `name` child of a `function_call` (treated as a possible function name).
/// - The `field` child of a `field_expression`.
/// - Identifiers inside `function_arguments` or `function_output`.
fn is_variable_use(node: Node) -> bool {
    let Some(parent) = node.parent() else {
        return true;
    };

    match parent.kind() {
        "function_call" => {
            // The first named child (the function name) is not a variable use.
            // Arguments ARE variable uses — those are inside the argument
            // list, so the parent for those identifiers won't be function_call
            // directly (they'll be inside an `arguments` node or similar).
            //
            // Check if this identifier is the direct name of the function_call.
            // In tree-sitter-matlab, the function name is typically the first child
            // or accessible via a field. We check by position: if it's the first
            // named child, it's the function name.
            if let Some(first) = parent.child(0) {
                if first.id() == node.id() {
                    return false;
                }
            }
            true
        }
        "field_expression" => !super::symbols::is_field_name(node),
        "function_arguments" | "function_output" | "function_definition" => false,
        // The name of a `Name = value` call argument is not a read (probed: MATLAB
        // reports no NODEF on `NumClasses` in `struct(NumClasses = NaN)`).
        "arguments" => node.next_sibling().is_none_or(|next| next.kind() != "="),
        _ => true,
    }
}

/// Collect variables defined by an assignment LHS.
///
/// For simple `identifier` LHS, adds the name to `assigned`.
/// For `multioutput_variable`, adds each identifier.
/// For indexed assignment (`function_call` or `field_expression`), checks
/// the expression for uses but does NOT add new variables (the base
/// variable must already exist).
pub(crate) fn collect_assignment_targets(
    lhs: Node,
    source: &str,
    assigned: &mut HashSet<String>,
    uses: &mut Vec<PossiblyUnsetVar>,
) {
    match lhs.kind() {
        "identifier" => {
            assigned.insert(node_text(lhs, source).to_string());
        }
        "multioutput_variable" => {
            let mut cursor = lhs.walk();
            for child in lhs.children(&mut cursor) {
                if child.kind() == "identifier" {
                    assigned.insert(node_text(child, source).to_string());
                }
            }
        }
        "function_call" | "field_expression" | "cell_index" => {
            // Partial assignment: `A(i) = ...`, `s.f = ...`. It reads the
            // old value only when there is one, and it creates the variable
            // otherwise.
            let root = partial_root(lhs);
            let root_name = root.map(|r| node_text(r, source).to_string());
            let existed = root_name.as_ref().is_some_and(|n| assigned.contains(n));
            if existed {
                collect_uses_in_expr(lhs, source, assigned, uses);
            } else {
                // Index expressions are still reads; the root itself is not.
                let before = uses.len();
                collect_uses_in_expr(lhs, source, assigned, uses);
                if let (Some(name), Some(r)) = (&root_name, root) {
                    let at = r.start_position().row + 1;
                    let col = r.start_position().column + 1;
                    let mut i = before;
                    while i < uses.len() {
                        if uses[i].name == *name
                            && uses[i].use_line == at
                            && uses[i].use_column == col
                        {
                            uses.remove(i);
                        } else {
                            i += 1;
                        }
                    }
                }
            }
            if let Some(name) = root_name {
                assigned.insert(name);
            }
        }
        _ => {
            collect_uses_in_expr(lhs, source, assigned, uses);
        }
    }
}

/// The variable a partial assignment target is rooted at (`x` in `x.y(3).z`).
pub(crate) fn partial_root(mut node: Node) -> Option<Node> {
    loop {
        node = match node.kind() {
            "identifier" => return Some(node),
            "function_call" => node.child_by_field_name("name")?,
            "field_expression" => node.child_by_field_name("object")?,
            "cell_index" => node
                .child_by_field_name("name")
                .or_else(|| node.named_child(0))?,
            _ => return None,
        };
    }
}

// ---------------------------------------------------------------------------
// Set utilities
// ---------------------------------------------------------------------------

/// Compute the intersection of multiple `HashSet<String>` collections.
///
/// Returns the set of strings present in ALL input sets.
fn intersect_sets(sets: &[HashSet<String>]) -> HashSet<String> {
    if sets.is_empty() {
        return HashSet::new();
    }
    let mut result = sets[0].clone();
    for set in &sets[1..] {
        result.retain(|name| set.contains(name));
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    /// The assignment statement whose text starts with `prefix`, at the top level of the file.
    fn statement_starting_with<'t>(tree: &'t Tree, source: &str, prefix: &str) -> Node<'t> {
        named_children(tree.root_node())
            .into_iter()
            .find(|n| source[n.byte_range()].starts_with(prefix))
            .expect("statement not found")
    }

    #[test]
    fn in_unreachable_code_memo_matches_scan_on_long_block() {
        // 3,000 statements, then a return: every statement after it is unreachable.
        let body: String = (0..3000).map(|i| format!("x{i} = {i};\n")).collect();
        let source = format!("{body}return;\ny = 1;\nz = 2;\n");
        let tree = crate::test_util::parse(&source);
        assert!(!in_unreachable_code(
            statement_starting_with(&tree, &source, "x0 "),
            &source
        ));
        assert!(!in_unreachable_code(
            statement_starting_with(&tree, &source, "x2999 "),
            &source
        ));
        assert!(in_unreachable_code(
            statement_starting_with(&tree, &source, "y "),
            &source
        ));
        assert!(in_unreachable_code(
            statement_starting_with(&tree, &source, "z "),
            &source
        ));

        // A later parse must not see the memo of this one: same block shape, no return.
        let plain: String = (0..3000).map(|i| format!("x{i} = {i};\n")).collect();
        let source = format!("{plain}y = 1;\n");
        let tree = crate::test_util::parse(&source);
        assert!(!in_unreachable_code(
            statement_starting_with(&tree, &source, "y "),
            &source
        ));
    }

    #[test]
    fn in_unreachable_code_respects_error_and_endless_loop_terminators() {
        // `error(...)` ends the flow; so does an endless `while true` without a break.
        let source = "x = 1;\nerror('boom');\na = 1;\nwhile true\n    x = x + 1;\nend\nb = 2;\n";
        let tree = crate::test_util::parse(source);
        assert!(!in_unreachable_code(
            statement_starting_with(&tree, source, "x = 1"),
            source
        ));
        assert!(!in_unreachable_code(
            statement_starting_with(&tree, source, "error"),
            source
        ));
        assert!(in_unreachable_code(
            statement_starting_with(&tree, source, "a "),
            source
        ));
        assert!(in_unreachable_code(
            statement_starting_with(&tree, source, "b "),
            source
        ));
    }

    /// Parse MATLAB source and return the tree.
    fn parse(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_matlab::LANGUAGE.into())
            .expect("failed to load tree-sitter-matlab");
        malt_core::note_parse();
        parser.parse(source, None).expect("parse failed")
    }

    /// Find the first `function_definition` node in the tree.
    fn first_function(tree: &Tree) -> Node<'_> {
        let root = tree.root_node();
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() == "function_definition" {
                return child;
            }
        }
        panic!("no function_definition found");
    }

    // ======================================================================
    // Unreachable code tests
    // ======================================================================

    #[test]
    fn unreachable_after_switch_whose_branches_all_return() {
        // Probed (plan 6.2): with an `otherwise` the statement after it is UNRCH;
        // without one it is reachable.
        let with_otherwise = "function f(s)\nswitch s\n    case 1\n        return;\n    otherwise\n        return;\nend\ndisp(1);\nend\n";
        let spans = find_unreachable(&parse(with_otherwise), with_otherwise);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].line, 8);
        let without = "function f(s)\nswitch s\n    case 1\n        return;\n    case 2\n        return;\nend\ndisp(1);\nend\n";
        assert!(find_unreachable(&parse(without), without).is_empty());
    }

    #[test]
    fn unreachable_after_return() {
        let source = "\
function foo()
    x = 1;
    return;
    y = 2;
    disp(y);
end
";
        let tree = parse(source);
        let spans = find_unreachable(&tree, source);

        // MATLAB reports only the first statement of an unreachable run (`y = 2;`),
        // not each statement after it (checked against checkcode in R2026b).
        assert_eq!(
            spans.len(),
            1,
            "expected 1 unreachable span, got {:?}",
            spans
        );
        assert_eq!(spans[0].cause, "return");
        assert_eq!(spans[0].line, 4);

        // Verify they point to the right code.
        let first_text = &source[spans[0].byte_range.clone()];
        assert!(
            first_text.contains("y = 2"),
            "expected 'y = 2' but got '{first_text}'"
        );
    }

    #[test]
    fn unreachable_after_break() {
        let source = "\
function foo()
    for i = 1:10
        break;
        x = i;
    end
end
";
        let tree = parse(source);
        let spans = find_unreachable(&tree, source);

        assert_eq!(
            spans.len(),
            1,
            "expected 1 unreachable span, got {:?}",
            spans
        );
        assert_eq!(spans[0].cause, "break");

        let text = &source[spans[0].byte_range.clone()];
        assert!(text.contains("x = i"), "expected 'x = i' but got '{text}'");
    }

    #[test]
    fn unreachable_after_continue() {
        let source = "\
function foo()
    for i = 1:10
        continue;
        disp(i);
    end
end
";
        let tree = parse(source);
        let spans = find_unreachable(&tree, source);

        assert_eq!(
            spans.len(),
            1,
            "expected 1 unreachable span, got {:?}",
            spans
        );
        assert_eq!(spans[0].cause, "continue");
    }

    #[test]
    fn no_unreachable_when_no_terminator() {
        let source = "\
function foo()
    x = 1;
    y = 2;
    z = x + y;
end
";
        let tree = parse(source);
        let spans = find_unreachable(&tree, source);
        assert!(
            spans.is_empty(),
            "expected no unreachable code, got {:?}",
            spans
        );
    }

    #[test]
    fn unreachable_in_nested_blocks() {
        let source = "\
function foo(x)
    if x > 0
        return;
        dead = 1;
    end
    alive = 2;
end
";
        let tree = parse(source);
        let spans = find_unreachable(&tree, source);

        // Only `dead = 1` is unreachable (inside the if body).
        // `alive = 2` is reachable (the if might not execute).
        assert_eq!(
            spans.len(),
            1,
            "expected 1 unreachable span, got {:?}",
            spans
        );
        let text = &source[spans[0].byte_range.clone()];
        assert!(
            text.contains("dead = 1"),
            "expected 'dead = 1' but got '{text}'"
        );
    }

    // ======================================================================
    // block_terminates tests
    // ======================================================================

    #[test]
    fn block_with_return_terminates() {
        let source = "\
function foo()
    return;
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let children = named_children(func);
        let block = children.iter().find(|c| c.kind() == "block").unwrap();
        assert!(block_terminates(*block, source));
    }

    #[test]
    fn block_without_terminator_does_not_terminate() {
        let source = "\
function foo()
    x = 1;
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let children = named_children(func);
        let block = children.iter().find(|c| c.kind() == "block").unwrap();
        assert!(!block_terminates(*block, source));
    }

    #[test]
    fn if_else_all_return_terminates() {
        let source = "\
function foo(x)
    if x > 0
        return;
    else
        return;
    end
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let children = named_children(func);
        let block = children.iter().find(|c| c.kind() == "block").unwrap();
        assert!(block_terminates(*block, source));
    }

    #[test]
    fn if_without_else_does_not_terminate() {
        let source = "\
function foo(x)
    if x > 0
        return;
    end
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let children = named_children(func);
        let block = children.iter().find(|c| c.kind() == "block").unwrap();
        assert!(!block_terminates(*block, source));
    }

    // ======================================================================
    // Definite-assignment tests
    // ======================================================================

    #[test]
    fn input_args_are_definitely_assigned() {
        let source = "\
function y = foo(x)
    y = x + 1;
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(
            da.definitely_assigned.contains("x"),
            "input arg 'x' should be definitely assigned"
        );
        assert!(
            da.definitely_assigned.contains("y"),
            "output arg 'y' should be definitely assigned"
        );
        assert!(
            da.possibly_unset.is_empty(),
            "no possibly-unset vars expected, got {:?}",
            da.possibly_unset
        );
    }

    #[test]
    fn variable_assigned_in_all_branches_is_definite() {
        let source = "\
function y = foo(x)
    if x > 0
        z = 1;
    else
        z = 2;
    end
    y = z;
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(
            da.definitely_assigned.contains("z"),
            "z assigned in all if/else branches should be definite"
        );
        // z should not appear in possibly_unset.
        assert!(
            !da.possibly_unset.iter().any(|v| v.name == "z"),
            "z should not be possibly unset"
        );
    }

    #[test]
    fn variable_assigned_only_in_if_no_else_is_not_definite() {
        let source = "\
function y = foo(x)
    if x > 0
        z = 1;
    end
    y = z;
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        // z is NOT definitely assigned (no else branch).
        assert!(
            !da.definitely_assigned.contains("z"),
            "z should NOT be definitely assigned without else"
        );
        // z should appear in possibly_unset (at the `y = z` use site).
        assert!(
            da.possibly_unset.iter().any(|v| v.name == "z"),
            "z should be possibly unset, got {:?}",
            da.possibly_unset
        );
    }

    #[test]
    fn loop_body_assignments_are_not_definite() {
        let source = "\
function y = foo(x)
    for i = 1:x
        z = i;
    end
    y = z;
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(
            !da.definitely_assigned.contains("z"),
            "z assigned inside for-loop should NOT be definite"
        );
        assert!(
            da.possibly_unset.iter().any(|v| v.name == "z"),
            "z should be possibly unset after for-loop"
        );
    }

    #[test]
    fn for_loop_iterator_is_assigned_in_body() {
        let source = "\
function foo(n)
    for i = 1:n
        disp(i);
    end
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        // `i` should not appear as possibly unset inside the loop body.
        // (It is the loop iterator, so it's definitely assigned inside the body.)
        assert!(
            !da.possibly_unset.iter().any(|v| v.name == "i"),
            "loop var 'i' should not be possibly unset inside body"
        );
    }

    #[test]
    fn try_catch_only_try_assigned_is_not_definite() {
        let source = "\
function foo()
    try
        z = 1;
    catch
        disp('error');
    end
    disp(z);
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(
            !da.definitely_assigned.contains("z"),
            "z assigned only in try should NOT be definite"
        );
        assert!(
            da.possibly_unset.iter().any(|v| v.name == "z"),
            "z should be possibly unset after try/catch"
        );
    }

    #[test]
    fn try_catch_both_assign_is_definite() {
        let source = "\
function foo()
    try
        z = 1;
    catch
        z = 0;
    end
    disp(z);
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(
            da.definitely_assigned.contains("z"),
            "z assigned in both try and catch should be definite"
        );
    }

    #[test]
    fn global_variable_is_definitely_assigned() {
        let source = "\
function foo()
    global g;
    disp(g);
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(
            da.definitely_assigned.contains("g"),
            "global variable should be definitely assigned"
        );
        assert!(
            !da.possibly_unset.iter().any(|v| v.name == "g"),
            "global should not be possibly unset"
        );
    }

    #[test]
    fn persistent_variable_is_definitely_assigned() {
        let source = "\
function foo()
    persistent p;
    disp(p);
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(
            da.definitely_assigned.contains("p"),
            "persistent variable should be definitely assigned"
        );
    }

    #[test]
    fn while_loop_body_not_definite() {
        let source = "\
function foo(x)
    while x > 0
        z = x;
        x = x - 1;
    end
    disp(z);
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(
            !da.definitely_assigned.contains("z"),
            "z assigned in while-loop body should NOT be definite"
        );
    }

    #[test]
    fn switch_with_otherwise_all_assign_is_definite() {
        let source = "\
function foo(x)
    switch x
        case 1
            z = 10;
        case 2
            z = 20;
        otherwise
            z = 0;
    end
    disp(z);
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(
            da.definitely_assigned.contains("z"),
            "z assigned in all switch cases + otherwise should be definite"
        );
    }

    #[test]
    fn switch_without_otherwise_not_definite() {
        let source = "\
function foo(x)
    switch x
        case 1
            z = 10;
        case 2
            z = 20;
    end
    disp(z);
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(
            !da.definitely_assigned.contains("z"),
            "z assigned in switch without otherwise should NOT be definite"
        );
    }

    #[test]
    fn sequential_assignment_is_definite() {
        let source = "\
function foo()
    x = 1;
    y = x + 2;
    z = y * 3;
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(da.definitely_assigned.contains("x"));
        assert!(da.definitely_assigned.contains("y"));
        assert!(da.definitely_assigned.contains("z"));
        assert!(da.possibly_unset.is_empty());
    }

    #[test]
    fn use_before_assignment_is_possibly_unset() {
        let source = "\
function foo()
    y = x + 1;
    x = 5;
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        // x is used before it is assigned.
        assert!(
            da.possibly_unset.iter().any(|v| v.name == "x"),
            "x used before assignment should be possibly unset"
        );
    }

    #[test]
    fn elseif_chain_all_assign_is_definite() {
        let source = "\
function foo(x)
    if x > 0
        z = 1;
    elseif x < 0
        z = -1;
    else
        z = 0;
    end
    disp(z);
end
";
        let tree = parse(source);
        let func = first_function(&tree);
        let da = analyze_definite_assignment(func, source);

        assert!(
            da.definitely_assigned.contains("z"),
            "z assigned in all if/elseif/else branches should be definite"
        );
    }
}
