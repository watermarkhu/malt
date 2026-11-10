//! # Dead stores: assignments whose value is never read
//!
//! Backward liveness over the statements of one scope. A full assignment
//! `x = ...` is dead when `x` is not live after it: no path from it reaches a
//! read of `x` before another full assignment or `clear x` replaces it.
//!
//! A script keeps its variables in the workspace, so every variable is live at
//! its end and only overwritten values are dead. A function keeps only its
//! output arguments, so a value that reaches the end unread is dead too.
//!
//! The analysis is conservative. Control flow is modelled for `if`, `switch`,
//! `for`, `while`, `try`, `break`, `continue` and `return`; the constructs it
//! cannot see through make the whole scope unanalysable (`None`):
//!
//! - calls that clear variables or end the flow (`clearvars`, `run`),
//! - `break` or `continue` outside a loop, and `break` in a `parfor`.
//!
//! `eval`, `evalin`, `assignin`, `save`, `who` and `load` do not stop the
//! analysis: probed, MATLAB still reports the stores around them.
//!
//! A `parfor` body runs once, with no back edge: iterations are independent
//! (probed: `s = s + i` in the body is reported when `s` is not read after the
//! loop). An `spmd` block runs once on each lab, so it is a plain sequence.
//!
//! Names that are `global` or `persistent`, or that a nested function uses, are
//! never reported. A variable that a compound assignment changes (`k += 1`) is
//! never reported in its scope (probed: `k = 1; k = 2; k += 1` reports nothing).
//! An anonymous function captures its variables when it is created, so its uses
//! are reads at that point (probed: `u = 1; u = 2; f = @() u` reports only
//! `u = 1`). A partial store (`x(i) = v`, `s.f = v`) reads
//! the old value, so it keeps earlier stores alive; it is itself dead when the
//! variable is not read after it (see [`Dead::partial`]).
//!
//! Constant conditions are evaluated by [`scope_truths`]: literals (`1`, `0`,
//! `true`, `false`, and `~` of them) and variables with a known truth value at
//! that point (`k = 1; if k`). A branch whose condition is constant false never
//! runs, and a constant true condition makes the branches after it, and the
//! fall-through, unreachable. Other conditions, such as `1 == 1`, are not
//! evaluated (probed: MATLAB reports the store before `if 1 == 1`).
//!
//! A `try` body can be left at any statement, so the variables live in the
//! catch body are live after every statement of the try body.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use tree_sitter::{Node, Tree};

use super::symbols::{is_qualified_call, DefKind, Scope, ScopeKind, SymbolTable};

/// The variables that may still be read before they are overwritten.
type Live = HashSet<String>;

/// The variables with a known truth value at one point of a scope.
type Known = HashMap<String, bool>;

/// Calls that end the flow or clear variables in a way the analysis cannot
/// follow. A scope that uses one is not analysed. Probed: `eval`, `evalc`,
/// `evalin`, `assignin`, `save`, `who` and `whos` are not in this list, MATLAB
/// reports the stores around them (see the module docs).
const DYNAMIC_CALLS: &[&str] = &[
    "clearvars",
    "run",
    "rethrow",
    "throw",
    "throwAsCaller",
    "exit",
    "quit",
];

/// Calls that may set variables by name. They do not stop the analysis, but
/// the truth values known before them are forgotten.
const ASSIGNING_CALLS: &[&str] = &["eval", "load"];

/// Words after `clear` that do not name a single variable.
const CLEAR_KEYWORDS: &[&str] = &[
    "all",
    "functions",
    "classes",
    "global",
    "variables",
    "mex",
    "java",
];

/// The dead assignments of one scope, as indices into `scope.defs`.
#[derive(Debug, Default)]
pub struct Dead {
    /// Full assignments (`x = v`, `[a, b] = f()`) whose value is never read.
    pub full: Vec<usize>,
    /// Partial assignments (`x(i) = v`, `s.f = v`) after which the variable is
    /// never read. A partial store is dead only if no later read of the
    /// variable can see it, so `s.a = 1; s.a = 2; disp(s)` reports nothing.
    pub partial: Vec<usize>,
}

/// The dead assignments of scope `index`.
///
/// Returns `None` when the scope cannot be analysed with confidence. Nothing is
/// then reported for it.
pub fn dead_stores(table: &SymbolTable, index: usize, tree: &Tree, source: &str) -> Option<Dead> {
    let scope = table.scopes.get(index)?;
    if !matches!(
        scope.kind,
        ScopeKind::Script
            | ScopeKind::Function
            | ScopeKind::LocalFunction
            | ScopeKind::NestedFunction
            | ScopeKind::Method
    ) {
        return None;
    }
    let Some(body) = scope_body(scope, tree) else {
        return Some(Dead::default());
    };
    if calls_dynamic(body, source) {
        return None;
    }

    let mut excluded: Live = scope
        .defs
        .iter()
        .filter(|d| matches!(d.kind, DefKind::Global | DefKind::Persistent))
        .map(|d| d.name.clone())
        .collect();
    compound_targets(body, source, &mut excluded);
    // A nested function shares the variables of the functions around it.
    let mut outer = match scope.kind {
        ScopeKind::NestedFunction => scope.parent,
        _ => None,
    };
    while let Some(index) = outer.and_then(|i| table.scopes.get(i)) {
        excluded.extend(index.defs.iter().map(|d| d.name.clone()));
        excluded.extend(index.uses.iter().map(|u| u.name.clone()));
        outer = index.parent;
    }
    for &child in &scope.children {
        // A lambda captures its variables when it is created. The enclosing scope
        // already records them as reads at that point, so they are not excluded.
        if let Some(child) = table
            .scopes
            .get(child)
            .filter(|c| !matches!(c.kind, ScopeKind::Lambda))
        {
            excluded.extend(child.uses.iter().map(|u| u.name.clone()));
            excluded.extend(child.defs.iter().map(|d| d.name.clone()));
        }
    }
    let exit: Live = match scope.kind {
        ScopeKind::Script => scope.defs.iter().map(|d| d.name.clone()).collect(),
        _ => scope
            .defs
            .iter()
            .filter(|d| d.kind == DefKind::OutputArg)
            .map(|d| d.name.clone())
            .collect(),
    };

    let mut analysis = Analysis {
        scope,
        source,
        exit: exit.clone(),
        excluded,
        truths: scope_truths(table, index, tree, source),
        loops: Vec::new(),
        extra: Live::new(),
        reports: Vec::new(),
        partial_reports: Vec::new(),
        bail: false,
    };
    analysis.block(body, exit, true);
    if analysis.bail {
        return None;
    }
    analysis.reports.sort_unstable();
    analysis.reports.dedup();
    analysis.partial_reports.sort_unstable();
    analysis.partial_reports.dedup();
    Some(Dead {
        full: analysis.reports,
        partial: analysis.partial_reports,
    })
}

/// The truth value of every constant condition in the file, keyed by the start
/// byte of the condition node. See the module docs for what counts as constant.
pub fn constant_truths(table: &SymbolTable, tree: &Tree, source: &str) -> HashMap<usize, bool> {
    let mut truths = HashMap::new();
    for index in 0..table.scopes.len() {
        truths.extend(scope_truths(table, index, tree, source));
    }
    truths
}

/// The truth value of the constant conditions of scope `index`, keyed by the
/// start byte of the condition node. Conditions in unreachable code are absent.
pub fn scope_truths(
    table: &SymbolTable,
    index: usize,
    tree: &Tree,
    source: &str,
) -> HashMap<usize, bool> {
    let Some(scope) = table.scopes.get(index) else {
        return HashMap::new();
    };
    if !matches!(
        scope.kind,
        ScopeKind::Script
            | ScopeKind::Function
            | ScopeKind::LocalFunction
            | ScopeKind::Method
            | ScopeKind::NestedFunction
    ) {
        return HashMap::new();
    }
    let Some(body) = scope_body(scope, tree) else {
        return HashMap::new();
    };
    let mut pass = ConstPass {
        scope,
        source,
        truths: HashMap::new(),
    };
    pass.sequence(&named(body), Some(Known::new()));
    pass.truths
}

/// Whether the assignment whose target covers `range` stores into a field
/// (`s.f = v`, `s(2).f = v`, `s.f(3) = v`) rather than only indexing (`x(i) = v`,
/// `c{i} = v`). The target is the left-hand side of the enclosing `assignment`.
pub fn is_field_store(tree: &Tree, range: Range<usize>) -> bool {
    let Some(mut node) = tree
        .root_node()
        .descendant_for_byte_range(range.start, range.end)
    else {
        return false;
    };
    while node.kind() != "assignment" {
        let Some(parent) = node.parent() else {
            return false;
        };
        node = parent;
    }
    let Some(mut current) = node.child_by_field_name("left") else {
        return false;
    };
    loop {
        if current.kind() == "field_expression" {
            return true;
        }
        match current.named_child(0) {
            Some(next) => current = next,
            None => return false,
        }
    }
}

/// The node holding a scope's statements: the `source_file`, or the `function_definition`.
fn scope_root<'t>(tree: &'t Tree, scope: &Scope) -> Option<Node<'t>> {
    let mut node = tree
        .root_node()
        .descendant_for_byte_range(scope.byte_range.start, scope.byte_range.end)?;
    while !matches!(node.kind(), "source_file" | "function_definition") {
        node = node.parent()?;
    }
    Some(node)
}

/// The node whose named children are the statements of a scope: the
/// `source_file` of a script, the `block` of a function.
fn scope_body<'t>(scope: &Scope, tree: &'t Tree) -> Option<Node<'t>> {
    let root = scope_root(tree, scope)?;
    if root.kind() == "source_file" {
        Some(root)
    } else {
        named(root).into_iter().find(|n| n.kind() == "block")
    }
}

/// Whether a call under `node` reads or creates variables by name, or cannot
/// be followed. Function definitions are skipped: local functions are separate
/// scopes, and a nested function is excluded from the analysis of its parent.
fn calls_dynamic(node: Node, source: &str) -> bool {
    calls_any(node, source, DYNAMIC_CALLS)
}

/// Adds the variables that a compound assignment (`k += 1`) changes under `node`.
/// The grammar has no compound assignment: `k += 1` parses as a command and
/// `x(2) -= 3` as a binary operator with an error. Function definitions are skipped.
fn compound_targets(node: Node, source: &str, out: &mut Live) {
    if node.kind() == "function_definition" {
        return;
    }
    if matches!(node.kind(), "command" | "binary_operator" | "ERROR") {
        if let Some(name) = compound_target(&source[node.byte_range()]) {
            out.insert(name);
        }
    }
    for child in children(node) {
        compound_targets(child, source, out);
    }
}

/// The variable that a statement such as `k += 1`, `k -= 1` or `x(2) *= 3`
/// assigns, from the statement's text.
fn compound_target(text: &str) -> Option<String> {
    let text = text.trim_start();
    let name: String = text
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if !is_identifier(&name) {
        return None;
    }
    let mut rest = text[name.len()..].trim_start();
    if rest.starts_with('(') {
        rest = rest[rest.find(')')? + 1..].trim_start();
    }
    let mut chars = rest.chars();
    if !matches!(chars.next()?, '+' | '-' | '*' | '/' | '^') || chars.next()? != '=' {
        return None;
    }
    Some(name)
}

/// Whether a call under `node` is one of `names`. Function definitions are skipped.
fn calls_any(node: Node, source: &str, names: &[&str]) -> bool {
    if node.kind() == "function_definition" {
        return false;
    }
    // `obj.save(1)` calls a method, not the function `save`.
    let qualified = node.kind() == "function_call" && is_qualified_call(node);
    if !qualified && callee_name(node, source).is_some_and(|name| names.contains(&name)) {
        return true;
    }
    children(node)
        .into_iter()
        .any(|child| calls_any(child, source, names))
}

/// The name of the function a `function_call` or `command` node calls.
fn callee_name<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    match node.kind() {
        "function_call" => node
            .child_by_field_name("name")
            .map(|n| &source[n.byte_range()]),
        "command" => source[node.byte_range()].split_whitespace().next(),
        _ => None,
    }
}

/// A statement that calls `error`. MATLAB treats it as the end of the flow:
/// values stored before it are dead and the statements after it cannot run.
fn is_error_call(node: Node, source: &str) -> bool {
    callee_name(node, source) == Some("error")
}

/// The variables named by a plain `clear a b` statement.
///
/// `None` for any other statement, including `clear`, `clear all` and
/// `clear -regexp`. Those are read as ordinary uses, which is conservative.
fn clear_names(text: &str) -> Option<Vec<String>> {
    let mut words = text
        .split(|c: char| c.is_whitespace() || c == ';')
        .filter(|w| !w.is_empty());
    if words.next()? != "clear" {
        return None;
    }
    let names: Vec<String> = words.map(str::to_string).collect();
    let plain = !names.is_empty()
        && names
            .iter()
            .all(|n| is_identifier(n) && !CLEAR_KEYWORDS.contains(&n.as_str()));
    plain.then_some(names)
}

fn is_identifier(word: &str) -> bool {
    let mut chars = word.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn children(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    node.children(&mut cursor).collect()
}

fn named(node: Node) -> Vec<Node> {
    children(node)
        .into_iter()
        .filter(|child| child.is_named())
        .collect()
}

/// The children of `parent` stored in the field `name`.
fn fields<'t>(parent: Node<'t>, name: &str) -> Vec<Node<'t>> {
    let mut cursor = parent.walk();
    parent.children_by_field_name(name, &mut cursor).collect()
}

/// Whether `child` is the condition of `parent`. The grammar stores a condition
/// under the field `condition`, whatever its node kind is.
fn is_condition(parent: Node, child: Node) -> bool {
    fields(parent, "condition")
        .iter()
        .any(|condition| condition.id() == child.id())
}

/// The statements of a block, or of a `source_file`, without its function
/// definitions. A nested function is a scope of its own.
fn statements(node: Node) -> Vec<Node> {
    named(node)
        .into_iter()
        .filter(|s| s.kind() != "function_definition")
        .collect()
}

fn has_block(node: Node) -> bool {
    node.kind() == "block" || children(node).into_iter().any(has_block)
}

fn has_break(node: Node) -> bool {
    node.kind() == "break_statement" || children(node).into_iter().any(has_break)
}

fn contains_jump(node: Node) -> bool {
    matches!(
        node.kind(),
        "break_statement" | "continue_statement" | "return_statement"
    ) || children(node).into_iter().any(contains_jump)
}

/// A clause of an `if` statement: its condition (`None` for `else`) and its statements.
pub(crate) struct Clause<'t> {
    /// The `if_statement` for the first clause, else the `elseif_clause` or `else_clause`.
    pub(crate) node: Node<'t>,
    pub(crate) condition: Option<Node<'t>>,
    pub(crate) is_else: bool,
    pub(crate) parts: Vec<Node<'t>>,
}

/// The clauses of an `if` statement in order. The statements after the first
/// condition, before any `elseif` or `else`, belong to the first clause.
pub(crate) fn if_clauses(node: Node) -> Vec<Clause> {
    let mut clauses: Vec<Clause> = Vec::new();
    for child in named(node) {
        if is_condition(node, child) {
            clauses.push(Clause {
                node,
                condition: Some(child),
                is_else: false,
                parts: Vec::new(),
            });
        } else if matches!(child.kind(), "elseif_clause" | "else_clause") {
            let condition = child.child_by_field_name("condition");
            let parts = clause_body(child, condition);
            clauses.push(Clause {
                node: child,
                condition,
                is_else: child.kind() == "else_clause",
                parts,
            });
        } else {
            match clauses.last_mut() {
                Some(clause) => clause.parts.push(child),
                None => clauses.push(Clause {
                    node,
                    condition: None,
                    is_else: false,
                    parts: vec![child],
                }),
            }
        }
    }
    clauses
}

/// The named children of a clause other than its condition.
fn clause_body<'t>(clause: Node<'t>, condition: Option<Node<'t>>) -> Vec<Node<'t>> {
    named(clause)
        .into_iter()
        .filter(|part| condition.is_none_or(|c| c.id() != part.id()))
        .collect()
}

/// Where `break` and `continue` of the innermost loop go.
struct LoopTargets {
    brk: Live,
    cont: Live,
}

/// The backward pass over one scope.
struct Analysis<'a> {
    scope: &'a Scope,
    source: &'a str,
    /// Live at the end of the scope: the workspace of a script, or the outputs of a function.
    exit: Live,
    /// Names that are never reported.
    excluded: Live,
    /// The truth value of each constant condition, see [`scope_truths`].
    truths: HashMap<usize, bool>,
    loops: Vec<LoopTargets>,
    /// Live in the catch body of each enclosing `try`.
    extra: Live,
    /// Indices into `scope.defs` of the dead full assignments found.
    reports: Vec<usize>,
    /// Indices into `scope.defs` of the dead partial assignments found.
    partial_reports: Vec<usize>,
    /// Set when control flow is not understood. The scope then reports nothing.
    bail: bool,
}

impl Analysis<'_> {
    /// Live variables before the statements of a block, given those live after them.
    ///
    /// `report` is false while a loop is iterated to a fixed point, so each
    /// assignment is reported once, from the final pass.
    fn block(&mut self, node: Node, after: Live, report: bool) -> Live {
        let stmts = statements(node);
        self.sequence(&stmts, after, report)
    }

    /// Live variables before a clause body. The body is a block, or statements
    /// written after the clause header on the same line (`if k, a = 1; end`).
    fn branch(&mut self, parts: &[Node], after: Live, report: bool) -> Live {
        let stmts: Vec<Node> = parts
            .iter()
            .flat_map(|part| {
                if part.kind() == "block" {
                    named(*part)
                } else {
                    vec![*part]
                }
            })
            .collect();
        self.sequence(&stmts, after, report)
    }

    fn sequence(&mut self, stmts: &[Node], after: Live, report: bool) -> Live {
        // Statements after a return, break or continue cannot run: they are
        // not reported.
        let mut reachable = Vec::with_capacity(stmts.len());
        let mut flows = true;
        for stmt in stmts {
            reachable.push(flows);
            if matches!(
                stmt.kind(),
                "return_statement" | "break_statement" | "continue_statement"
            ) || is_error_call(*stmt, self.source)
                || self.is_infinite_loop(*stmt)
            {
                flows = false;
            }
        }
        let mut live = after;
        for i in (0..stmts.len()).rev() {
            live = self.stmt(stmts[i], live, report && reachable[i]);
            if self.bail {
                break;
            }
        }
        live
    }

    /// A `while` whose condition is constant true and that has no `break`: the
    /// statements after it cannot run.
    fn is_infinite_loop(&self, node: Node) -> bool {
        node.kind() == "while_statement"
            && !has_break(node)
            && named(node)
                .into_iter()
                .filter(|c| is_condition(node, *c))
                .any(|c| self.truths.get(&c.start_byte()) == Some(&true))
    }

    fn stmt(&mut self, node: Node, after: Live, report: bool) -> Live {
        let source = self.source;
        match node.kind() {
            "return_statement" => self.finish(self.exit.clone()),
            "break_statement" | "continue_statement" => {
                let is_break = node.kind() == "break_statement";
                match self.loops.last() {
                    Some(target) => {
                        let live = if is_break {
                            target.brk.clone()
                        } else {
                            target.cont.clone()
                        };
                        self.finish(live)
                    }
                    // Outside a loop the flow ends (probed: the statements after it are
                    // unreachable). Nothing stays live, not even a script's workspace
                    // (`x = 1; break` reports `x = 1`), but a function keeps its outputs.
                    None => match self.scope.kind {
                        ScopeKind::Script => self.finish(Live::new()),
                        _ => self.finish(self.exit.clone()),
                    },
                }
            }
            "if_statement" => self.if_statement(node, after, report),
            "for_statement" => self.for_statement(node, after, report),
            "while_statement" => self.while_statement(node, after, report),
            "switch_statement" => self.switch_statement(node, after, report),
            "try_statement" => self.try_statement(node, after, report),
            "spmd_statement" => self.spmd_statement(node, after, report),
            // An `arguments` block only declares and checks the inputs.
            "arguments_statement" => self.leaf(node, after, report),
            _ if has_block(node) => {
                self.bail = true;
                after
            }
            // Nothing after the call runs, so nothing is live after it: only
            // its arguments are read, and a `try` catches the error.
            _ if is_error_call(node, source) => {
                let reads = self.uses_in(node.byte_range());
                self.finish(reads)
            }
            _ => self.leaf(node, after, report),
        }
    }

    /// An assignment or any other statement without nested blocks.
    fn leaf(&mut self, node: Node, after: Live, report: bool) -> Live {
        let range = node.byte_range();
        let source = self.source;
        let cleared = clear_names(&source[range.clone()]);
        let mut kills: Live = cleared.clone().unwrap_or_default().into_iter().collect();
        let mut reads = if cleared.is_some() {
            Live::new()
        } else {
            self.uses_in(range.clone())
        };
        let after = self.finish(after);
        // An `onCleanup` object is kept for its destructor, MATLAB does not report it.
        let object = source[range.clone()]
            .split_once('=')
            .is_some_and(|(_, rhs)| rhs.trim_start().starts_with("onCleanup"));
        for (index, def) in self.scope.defs.iter().enumerate() {
            if def.kind != DefKind::Assignment
                || def.byte_range.start < range.start
                || def.byte_range.end > range.end
            {
                continue;
            }
            if object {
                kills.insert(def.name.clone());
                continue;
            }
            if def.partial {
                if report && !after.contains(&def.name) && !self.excluded.contains(&def.name) {
                    self.partial_reports.push(index);
                }
                reads.insert(def.name.clone());
                continue;
            }
            kills.insert(def.name.clone());
            if report && !after.contains(&def.name) && !self.excluded.contains(&def.name) {
                self.reports.push(index);
            }
        }
        let mut live: Live = after
            .into_iter()
            .filter(|name| !kills.contains(name))
            .collect();
        live.extend(reads);
        self.finish(live)
    }

    /// The parts of a clause that come after its condition. The condition's
    /// reads go to `reads`.
    fn clause_parts<'t>(&self, clause: Node<'t>, reads: &mut Live) -> Vec<Node<'t>> {
        named(clause)
            .into_iter()
            .filter(|part| {
                if is_condition(clause, *part) {
                    reads.extend(self.uses_in(part.byte_range()));
                    false
                } else {
                    true
                }
            })
            .collect()
    }

    /// The constant truth value of a clause. `else` always runs when reached.
    fn clause_truth(&self, clause: &Clause) -> Option<bool> {
        if clause.is_else {
            return Some(true);
        }
        clause
            .condition
            .and_then(|c| self.truths.get(&c.start_byte()).copied())
    }

    /// Only the clauses that can run contribute. A constant true clause ends
    /// the chain: the clauses after it and the fall-through are unreachable.
    fn if_statement(&mut self, node: Node, after: Live, report: bool) -> Live {
        let mut reads = Live::new();
        let mut out = Live::new();
        let mut reachable = true;
        for clause in if_clauses(node) {
            if !reachable {
                break;
            }
            if let Some(condition) = clause.condition {
                reads.extend(self.uses_in(condition.byte_range()));
            }
            match self.clause_truth(&clause) {
                Some(false) => {}
                Some(true) => {
                    out.extend(self.branch(&clause.parts, after.clone(), report));
                    reachable = false;
                }
                None => out.extend(self.branch(&clause.parts, after.clone(), report)),
            }
        }
        // The condition of the last clause may be false: nothing runs.
        if reachable {
            out.extend(after);
        }
        out.extend(reads);
        self.finish(out)
    }

    fn for_statement(&mut self, node: Node, after: Live, report: bool) -> Live {
        let source = self.source;
        let mut reads = Live::new();
        let mut var = None;
        let mut body = Vec::new();
        for child in named(node) {
            if child.kind() == "iterator" {
                // The loop variable is written, not read: only the range is.
                let parts = named(child);
                var = parts
                    .iter()
                    .find(|c| c.kind() == "identifier")
                    .map(|c| source[c.byte_range()].to_string());
                for part in parts.into_iter().filter(|c| c.kind() != "identifier") {
                    reads.extend(self.uses_in(part.byte_range()));
                }
            } else {
                body.push(child);
            }
        }
        if source[node.byte_range()].starts_with("parfor") {
            return self.parfor_statement(&body, var.as_deref(), reads, after, report);
        }
        // The range is evaluated once, before the loop.
        // The loop variable is assigned even when the range is empty (probed:
        // `i = 5; for i = 1:3, end` reports `i = 5`), so no value of it reaches the loop.
        let mut top = self.loop_head(&body, &after, after.clone(), var.as_deref(), report);
        if let Some(name) = var.as_deref() {
            top.remove(name);
        }
        top.extend(reads);
        self.finish(top)
    }

    /// A `parfor` loop. Iterations run on independent workers, so no value
    /// travels from one iteration to the next: the body runs once, with the
    /// loop's exit as its live-out, and has no back edge (probed: MATLAB reports
    /// `s = s + i` in a body whose `s` is not read after the loop). The loop may
    /// run zero times, so the exit stays live before it. `break` is not allowed
    /// in a `parfor`, so the analysis gives up on it.
    fn parfor_statement(
        &mut self,
        body: &[Node],
        var: Option<&str>,
        reads: Live,
        after: Live,
        report: bool,
    ) -> Live {
        if body.iter().any(|stmt| has_break(*stmt)) {
            self.bail = true;
            return after;
        }
        let mut top = self.branch(body, after.clone(), report);
        if let Some(name) = var {
            top.remove(name);
        }
        top.extend(after);
        top.extend(reads);
        self.finish(top)
    }

    /// An `spmd` block runs once on every lab: a plain sequence of statements.
    fn spmd_statement(&mut self, node: Node, after: Live, report: bool) -> Live {
        let parts = named(node);
        let live = self.branch(&parts, after, report);
        self.finish(live)
    }

    fn while_statement(&mut self, node: Node, after: Live, report: bool) -> Live {
        let mut reads = Live::new();
        let mut body = Vec::new();
        let mut condition = None;
        for child in named(node) {
            if is_condition(node, child) {
                reads.extend(self.uses_in(child.byte_range()));
                condition = Some(child);
            } else {
                body.push(child);
            }
        }
        let truth = condition.and_then(|c| self.truths.get(&c.start_byte()).copied());
        let top = match truth {
            // The body never runs.
            Some(false) => after.union(&reads).cloned().collect(),
            // The loop is left only by `break`, so `after` is reached only through one.
            Some(true) => self.loop_head(&body, &after, reads, None, report),
            // The condition is evaluated before every pass, so it is live at the head.
            None => {
                let base: Live = after.union(&reads).cloned().collect();
                self.loop_head(&body, &after, base, None, report)
            }
        };
        self.finish(top)
    }

    /// Live at the head of a loop: the loop either leaves to `after` or runs
    /// `body` again. `base` holds what is live on every pass, and `var` is the
    /// loop variable, stored at the start of each pass.
    fn loop_head(
        &mut self,
        body: &[Node],
        after: &Live,
        base: Live,
        var: Option<&str>,
        report: bool,
    ) -> Live {
        let mut top = base.clone();
        loop {
            self.loops.push(LoopTargets {
                brk: after.clone(),
                cont: top.clone(),
            });
            let mut entry = self.branch(body, top.clone(), false);
            self.loops.pop();
            if let Some(name) = var {
                entry.remove(name);
            }
            let mut next = base.clone();
            next.extend(entry);
            if next == top {
                break;
            }
            top = next;
        }
        if report {
            self.loops.push(LoopTargets {
                brk: after.clone(),
                cont: top.clone(),
            });
            self.branch(body, top.clone(), true);
            self.loops.pop();
        }
        top
    }

    fn switch_statement(&mut self, node: Node, after: Live, report: bool) -> Live {
        let mut reads = Live::new();
        let mut out = Live::new();
        let mut has_otherwise = false;
        for child in named(node) {
            match child.kind() {
                _ if is_condition(node, child) => reads.extend(self.uses_in(child.byte_range())),
                "case_clause" => {
                    let parts = self.clause_parts(child, &mut reads);
                    out.extend(self.branch(&parts, after.clone(), report));
                }
                "otherwise_clause" => {
                    has_otherwise = true;
                    let parts = self.clause_parts(child, &mut reads);
                    out.extend(self.branch(&parts, after.clone(), report));
                }
                _ => {}
            }
        }
        // Without an otherwise clause no case may match.
        if !has_otherwise {
            out.extend(after);
        }
        out.extend(reads);
        self.finish(out)
    }

    fn try_statement(&mut self, node: Node, after: Live, report: bool) -> Live {
        let mut body = Vec::new();
        let mut catch_live = None;
        for child in named(node) {
            if child.kind() == "catch_clause" {
                // `catch err` names the error on the same line; the rest is the body.
                let mut parts = named(child);
                if parts.first().is_some_and(|p| {
                    p.kind() == "identifier" && p.start_position().row == child.start_position().row
                }) {
                    parts.remove(0);
                }
                catch_live = Some(self.branch(&parts, after.clone(), report));
            } else {
                body.push(child);
            }
        }
        // Without a catch clause the error leaves the scope, so everything
        // the scope keeps is live.
        let catch_live = catch_live.unwrap_or_else(|| self.exit.clone());

        // An error can leave the try body at any statement.
        let saved = self.extra.clone();
        self.extra.extend(catch_live.iter().cloned());
        let body_live = self.branch(&body, after, report);
        self.extra = saved;

        let mut out = body_live;
        out.extend(catch_live);
        self.finish(out)
    }

    /// Variables read by the identifiers inside `range`.
    fn uses_in(&self, range: Range<usize>) -> Live {
        self.scope
            .uses
            .iter()
            .filter(|u| range.start <= u.byte_range.start && u.byte_range.end <= range.end)
            .map(|u| u.name.clone())
            .collect()
    }

    /// Adds the live variables of the enclosing `try` bodies.
    fn finish(&self, mut live: Live) -> Live {
        live.extend(self.extra.iter().cloned());
        live
    }
}

/// The forward pass that finds constant conditions: the variables with a known
/// truth value, merged over the branches of each statement.
struct ConstPass<'a> {
    scope: &'a Scope,
    source: &'a str,
    truths: HashMap<usize, bool>,
}

impl ConstPass<'_> {
    /// The known truth values after `parts` (blocks are flattened), given those
    /// before. `None` when the end of `parts` cannot be reached.
    fn sequence(&mut self, parts: &[Node], env: Option<Known>) -> Option<Known> {
        let mut env = env;
        for part in parts {
            let stmts = if part.kind() == "block" {
                named(*part)
            } else {
                vec![*part]
            };
            for stmt in stmts {
                env = self.stmt(stmt, env?);
            }
        }
        env
    }

    fn stmt(&mut self, node: Node, env: Known) -> Option<Known> {
        match node.kind() {
            "return_statement" | "break_statement" | "continue_statement" => None,
            _ if is_error_call(node, self.source) => None,
            "if_statement" => self.if_statement(node, env),
            "for_statement" => self.for_statement(node, env),
            "while_statement" => self.while_statement(node, env),
            "switch_statement" => self.switch_statement(node, env),
            "try_statement" => self.try_statement(node, env),
            _ if has_block(node) => Some(self.remove_defs(env, node)),
            _ => Some(self.leaf(node, env)),
        }
    }

    /// A statement without nested blocks: an assignment of a known value, or
    /// anything else, which makes every variable it defines unknown.
    fn leaf(&mut self, node: Node, mut env: Known) -> Known {
        let source = self.source;
        let text = &source[node.byte_range()];
        let assigned = (node.kind() == "assignment")
            .then(|| {
                let left = node.child_by_field_name("left")?;
                let right = node.child_by_field_name("right")?;
                (left.kind() == "identifier").then(|| {
                    (
                        source[left.byte_range()].to_string(),
                        self.eval(right, &env),
                    )
                })
            })
            .flatten();
        if calls_any(node, source, ASSIGNING_CALLS) {
            env.clear();
        }
        if clear_names(text).is_none() && text.split_whitespace().next() == Some("clear") {
            env.clear();
        }
        if let Some(names) = clear_names(text) {
            for name in names {
                env.remove(&name);
            }
        }
        env = self.remove_defs(env, node);
        if let Some((name, Some(truth))) = assigned {
            env.insert(name, truth);
        }
        env
    }

    /// A `for` loop without `break`, `continue` or `return` in its body: the
    /// values known at the head are the values before the loop joined with those
    /// at the end of the body, and the values after the loop are those at the
    /// head (probed: `x = 0; for i = 1:3, x = 1; end; if x` does not fold). A
    /// jump makes the loop exit from the middle of the body, so nothing the body
    /// defines stays known.
    fn for_statement(&mut self, node: Node, env: Known) -> Option<Known> {
        let body: Vec<Node> = named(node)
            .into_iter()
            .filter(|c| c.kind() != "iterator")
            .collect();
        let pre = match named(node).into_iter().find(|c| c.kind() == "iterator") {
            Some(iterator) => self.remove_defs(env.clone(), iterator),
            None => env,
        };
        if body.iter().any(|stmt| contains_jump(*stmt)) {
            let head = self.remove_defs(pre, node);
            self.sequence(&body, Some(head.clone()));
            return Some(head);
        }
        let mut head = pre.clone();
        for _ in 0..16 {
            let out = self.sequence(&body, Some(head.clone()));
            let next = join(Some(pre.clone()), out).unwrap_or_default();
            if next == head {
                break;
            }
            head = next;
        }
        Some(head)
    }

    fn if_statement(&mut self, node: Node, env: Known) -> Option<Known> {
        let mut out = None;
        let mut reachable = true;
        for clause in if_clauses(node) {
            if !reachable {
                break;
            }
            match self.clause_truth(&clause, &env) {
                Some(false) => {}
                Some(true) => {
                    out = join(out, self.sequence(&clause.parts, Some(env.clone())));
                    reachable = false;
                }
                None => out = join(out, self.sequence(&clause.parts, Some(env.clone()))),
            }
        }
        if reachable {
            out = join(out, Some(env));
        }
        out
    }

    fn clause_truth(&mut self, clause: &Clause, env: &Known) -> Option<bool> {
        if clause.is_else {
            return Some(true);
        }
        let condition = clause.condition?;
        let truth = self.eval(condition, env)?;
        self.truths.insert(condition.start_byte(), truth);
        Some(truth)
    }

    fn while_statement(&mut self, node: Node, env: Known) -> Option<Known> {
        let head = self.remove_defs(env, node);
        let mut condition = None;
        let mut body = Vec::new();
        for child in named(node) {
            if is_condition(node, child) {
                condition = Some(child);
            } else {
                body.push(child);
            }
        }
        let truth = condition.and_then(|c| self.eval(c, &head));
        if let (Some(c), Some(t)) = (condition, truth) {
            self.truths.insert(c.start_byte(), t);
        }
        if truth != Some(false) {
            self.sequence(&body, Some(head.clone()));
        }
        // Only a `break` leaves a loop whose condition is always true.
        if truth == Some(true) && !has_break(node) {
            return None;
        }
        Some(head)
    }

    fn switch_statement(&mut self, node: Node, env: Known) -> Option<Known> {
        let mut out = None;
        let mut has_otherwise = false;
        for child in named(node) {
            if matches!(child.kind(), "case_clause" | "otherwise_clause") {
                has_otherwise |= child.kind() == "otherwise_clause";
                let parts = clause_body(child, child.child_by_field_name("condition"));
                out = join(out, self.sequence(&parts, Some(env.clone())));
            }
        }
        if !has_otherwise {
            out = join(out, Some(env));
        }
        out
    }

    fn try_statement(&mut self, node: Node, env: Known) -> Option<Known> {
        let mut body = Vec::new();
        let mut catch_parts = None;
        for child in named(node) {
            if child.kind() == "catch_clause" {
                catch_parts = Some(named(child));
            } else {
                body.push(child);
            }
        }
        // An error can leave the body at any statement, so the catch body
        // starts with what the body defines unknown.
        let mut catch_env = env.clone();
        for part in &body {
            catch_env = self.remove_defs(catch_env, *part);
        }
        let body_out = self.sequence(&body, Some(env));
        let catch_out = catch_parts.and_then(|parts| self.sequence(&parts, Some(catch_env)));
        join(body_out, catch_out)
    }

    /// `env` without the variables that the statements under `node` define.
    fn remove_defs(&self, mut env: Known, node: Node) -> Known {
        let range = node.byte_range();
        for def in &self.scope.defs {
            if range.start <= def.byte_range.start && def.byte_range.end <= range.end {
                env.remove(&def.name);
            }
        }
        env
    }

    /// The truth value of a literal or a known variable, `None` for anything else.
    fn eval(&self, node: Node, env: &Known) -> Option<bool> {
        let text = &self.source[node.byte_range()];
        match node.kind() {
            "number" => text.parse::<f64>().ok().map(|v| v != 0.0),
            "identifier" => {
                if self.scope.defs.iter().any(|d| d.name == text) {
                    env.get(text).copied()
                } else {
                    match text {
                        "true" => Some(true),
                        "false" => Some(false),
                        _ => None,
                    }
                }
            }
            "not_operator" | "unary_operator" if text.starts_with(['~', '!']) => node
                .named_child(0)
                .and_then(|operand| self.eval(operand, env))
                .map(|v| !v),
            "parenthesized_expression" => {
                node.named_child(0).and_then(|inner| self.eval(inner, env))
            }
            _ => None,
        }
    }
}

/// The known values of two paths merged: a value survives only where both agree.
/// `None` is a path that does not reach the point, and is dropped.
fn join(a: Option<Known>, b: Option<Known>) -> Option<Known> {
    match (a, b) {
        (None, other) | (other, None) => other,
        (Some(a), Some(b)) => Some(
            a.into_iter()
                .filter(|(name, value)| b.get(name) == Some(value))
                .collect(),
        ),
    }
}
