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
//! - calls that read or create variables by name (`eval`, `evalin`, `assignin`,
//!   `load`, `save`, ...),
//! - nested functions, `parfor` and `spmd` blocks,
//! - `break` or `continue` outside a loop.
//!
//! Names that are `global` or `persistent`, or that a nested function or lambda
//! uses, are never reported. Partial stores (`x(i) = v`, `s.f = v`) read the old
//! value, so they are never dead. Constant conditions are not evaluated: every
//! branch counts as possible.
//!
//! A `try` body can be left at any statement, so the variables live in the
//! catch body are live after every statement of the try body.

use std::collections::HashSet;
use std::ops::Range;

use tree_sitter::{Node, Tree};

use super::symbols::{DefKind, Scope, ScopeKind, SymbolTable};

/// The variables that may still be read before they are overwritten.
type Live = HashSet<String>;

/// Calls that read or create variables by name, or end the flow in a way not
/// probed against MATLAB (`error` is a terminator; see `is_error_call`). A scope
/// that uses one is not analysed.
const DYNAMIC_CALLS: &[&str] = &[
    "eval",
    "evalc",
    "evalin",
    "assignin",
    "load",
    "save",
    "clearvars",
    "who",
    "whos",
    "inputname",
    "run",
    "rethrow",
    "throw",
    "throwAsCaller",
    "exit",
    "quit",
];

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

/// Indices into `scope.defs` of the full assignments in `index` whose value is never read.
///
/// Returns `None` when the scope cannot be analysed with confidence. Nothing is
/// then reported for it.
pub fn dead_stores(
    table: &SymbolTable,
    index: usize,
    tree: &Tree,
    source: &str,
) -> Option<Vec<usize>> {
    let scope = table.scopes.get(index)?;
    if !matches!(
        scope.kind,
        ScopeKind::Script | ScopeKind::Function | ScopeKind::LocalFunction | ScopeKind::Method
    ) {
        return None;
    }
    let root = scope_root(tree, scope)?;
    let body = if root.kind() == "source_file" {
        root
    } else {
        match named(root).into_iter().find(|n| n.kind() == "block") {
            Some(block) => block,
            None => return Some(Vec::new()),
        }
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
    for &child in &scope.children {
        if let Some(child) = table.scopes.get(child) {
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
        loops: Vec::new(),
        extra: Live::new(),
        reports: Vec::new(),
        bail: false,
    };
    analysis.block(body, exit, true);
    if analysis.bail {
        return None;
    }
    analysis.reports.sort_unstable();
    analysis.reports.dedup();
    Some(analysis.reports)
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

/// Whether a call under `node` reads or creates variables by name.
///
/// Function definitions are skipped: local functions are separate scopes, and
/// a nested function makes its enclosing scope unanalysable on its own.
fn calls_dynamic(node: Node, source: &str) -> bool {
    if node.kind() == "function_definition" {
        return false;
    }
    if callee_name(node, source).is_some_and(|name| DYNAMIC_CALLS.contains(&name)) {
        return true;
    }
    children(node)
        .into_iter()
        .any(|child| calls_dynamic(child, source))
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

/// The statements of a block, or of a `source_file` without its function definitions.
fn statements(node: Node) -> Vec<Node> {
    named(node)
        .into_iter()
        .filter(|s| !(node.kind() == "source_file" && s.kind() == "function_definition"))
        .collect()
}

fn has_block(node: Node) -> bool {
    node.kind() == "block" || children(node).into_iter().any(has_block)
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
    loops: Vec<LoopTargets>,
    /// Live in the catch body of each enclosing `try`.
    extra: Live,
    /// Indices into `scope.defs` of the dead assignments found.
    reports: Vec<usize>,
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

    fn stmt(&mut self, node: Node, after: Live, report: bool) -> Live {
        let source = self.source;
        let head = &source[node.byte_range()];
        if head.starts_with("parfor")
            || head.starts_with("spmd")
            || node.kind() == "function_definition"
        {
            self.bail = true;
            return after;
        }
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
                    None => {
                        self.bail = true;
                        after
                    }
                }
            }
            "if_statement" => self.if_statement(node, after, report),
            "for_statement" => self.for_statement(node, after, report),
            "while_statement" => self.while_statement(node, after, report),
            "switch_statement" => self.switch_statement(node, after, report),
            "try_statement" => self.try_statement(node, after, report),
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
        for (index, def) in self.scope.defs.iter().enumerate() {
            if def.kind != DefKind::Assignment
                || def.byte_range.start < range.start
                || def.byte_range.end > range.end
            {
                continue;
            }
            if def.partial {
                reads.insert(def.name.clone());
                continue;
            }
            kills.insert(def.name.clone());
            if report
                && !def.multi_output
                && !after.contains(&def.name)
                && !self.excluded.contains(&def.name)
            {
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

    fn if_statement(&mut self, node: Node, after: Live, report: bool) -> Live {
        let mut reads = Live::new();
        let mut out = Live::new();
        let mut has_else = false;
        for child in named(node) {
            match child.kind() {
                _ if is_condition(node, child) => reads.extend(self.uses_in(child.byte_range())),
                "elseif_clause" => {
                    let parts = self.clause_parts(child, &mut reads);
                    out.extend(self.branch(&parts, after.clone(), report));
                }
                "else_clause" => {
                    has_else = true;
                    let parts = self.clause_parts(child, &mut reads);
                    out.extend(self.branch(&parts, after.clone(), report));
                }
                _ => out.extend(self.branch(&[child], after.clone(), report)),
            }
        }
        // Without an else branch the condition may be false: nothing runs.
        if !has_else {
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
                reads.extend(self.uses_in(child.byte_range()));
                var = named(child)
                    .into_iter()
                    .find(|c| c.kind() == "identifier")
                    .map(|c| source[c.byte_range()].to_string());
            } else {
                body.push(child);
            }
        }
        // The range is evaluated once, before the loop.
        let mut top = self.loop_head(&body, &after, after.clone(), var.as_deref(), report);
        top.extend(reads);
        self.finish(top)
    }

    fn while_statement(&mut self, node: Node, after: Live, report: bool) -> Live {
        let mut reads = Live::new();
        let mut body = Vec::new();
        for child in named(node) {
            if is_condition(node, child) {
                reads.extend(self.uses_in(child.byte_range()));
            } else {
                body.push(child);
            }
        }
        // The condition is evaluated before every pass, so it is live at the head.
        let base: Live = after.union(&reads).cloned().collect();
        let top = self.loop_head(&body, &after, base, None, report);
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
