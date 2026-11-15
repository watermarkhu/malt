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

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

use tree_sitter::{Node, Tree};

use super::callee::{callee_word, Dotted};
use super::control_flow::call_argument_nodes;
use super::reaching::resolved_values;
use super::symbols::{is_qualified_call, DefKind, FileKey, Scope, ScopeKind, SymbolTable};

/// Interned variable names of one analysis: a name has a dense id, its position in `list`.
#[derive(Default)]
struct Names {
    ids: HashMap<String, u32>,
    list: Vec<String>,
}

impl Names {
    /// The id of `name`, interned on first use.
    fn intern(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.ids.get(name) {
            return id;
        }
        let id = self.list.len() as u32;
        self.ids.insert(name.to_string(), id);
        self.list.push(name.to_string());
        id
    }

    /// The id of `name` if it is interned. A name that is not interned is in no set.
    fn get(&self, name: &str) -> Option<u32> {
        self.ids.get(name).copied()
    }
}

/// The variables that may still be read before they are overwritten, as a bit
/// set over the interned names. Copying and merging a set costs one word per 64
/// names, so a scope with many variables and deep nesting stays cheap.
#[derive(Clone, Default)]
struct Live {
    words: Vec<u64>,
}

impl Live {
    fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, id: u32) {
        let word = id as usize / 64;
        if self.words.len() <= word {
            self.words.resize(word + 1, 0);
        }
        self.words[word] |= 1u64 << (id % 64);
    }

    fn remove(&mut self, id: u32) {
        if let Some(word) = self.words.get_mut(id as usize / 64) {
            *word &= !(1u64 << (id % 64));
        }
    }

    fn contains(&self, id: u32) -> bool {
        self.words
            .get(id as usize / 64)
            .is_some_and(|word| word & (1u64 << (id % 64)) != 0)
    }

    /// Adds every name of `other`.
    fn absorb(&mut self, other: &Live) {
        if self.words.len() < other.words.len() {
            self.words.resize(other.words.len(), 0);
        }
        for (word, other) in self.words.iter_mut().zip(&other.words) {
            *word |= other;
        }
    }

    /// The names of both sets.
    fn union(&self, other: &Live) -> Live {
        let mut out = self.clone();
        out.absorb(other);
        out
    }

    /// The ids in the set, in increasing order.
    fn ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.words.iter().enumerate().flat_map(|(i, &word)| {
            (0..64u32)
                .filter(move |&bit| word & (1u64 << bit) != 0)
                .map(move |bit| i as u32 * 64 + bit)
        })
    }
}

impl PartialEq for Live {
    fn eq(&self, other: &Live) -> bool {
        let len = self.words.len().max(other.words.len());
        (0..len).all(|i| {
            self.words.get(i).copied().unwrap_or(0) == other.words.get(i).copied().unwrap_or(0)
        })
    }
}

impl Eq for Live {}

/// Subtree facts, memoised per node id for one analysis, so a loop nested in
/// loops is not rescanned at every level.
#[derive(Default)]
struct Facts {
    memo: HashMap<usize, u8>,
}

/// A `break` under the node.
const FACT_BREAK: u8 = 1;
/// A `break`, `continue` or `return` under the node.
const FACT_JUMP: u8 = 2;
/// A `block` under the node.
const FACT_BLOCK: u8 = 4;

impl Facts {
    fn of(&mut self, node: Node) -> u8 {
        if let Some(&bits) = self.memo.get(&node.id()) {
            return bits;
        }
        let mut bits = match node.kind() {
            "break_statement" => FACT_BREAK | FACT_JUMP,
            "continue_statement" | "return_statement" => FACT_JUMP,
            "block" => FACT_BLOCK,
            _ => 0,
        };
        for child in children(node) {
            bits |= self.of(child);
        }
        self.memo.insert(node.id(), bits);
        bits
    }

    fn has_break(&mut self, node: Node) -> bool {
        self.of(node) & FACT_BREAK != 0
    }

    fn contains_jump(&mut self, node: Node) -> bool {
        self.of(node) & FACT_JUMP != 0
    }

    fn has_block(&mut self, node: Node) -> bool {
        self.of(node) & FACT_BLOCK != 0
    }
}

/// The variables with a known truth value at one point of a scope.
type Known = HashMap<String, bool>;

/// Calls that end the flow or clear variables in a way the analysis cannot
/// follow. A scope that uses one is not analysed. Probed: `eval`, `evalc`,
/// `evalin`, `assignin`, `save`, `who` and `whos` are not in this list, MATLAB
/// reports the stores around them (see the module docs).
const DYNAMIC_CALLS: &[&str] = &["run", "throw", "throwAsCaller", "exit", "quit"];

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
#[derive(Debug, Default, Clone)]
pub struct Dead {
    /// Full assignments (`x = v`, `[a, b] = f()`) whose value is never read.
    pub full: Vec<usize>,
    /// Partial assignments (`x(i) = v`, `s.f = v`) after which the variable is
    /// never read. A partial store is dead only if no later read of the
    /// variable can see it, so `s.a = 1; s.a = 2; disp(s)` reports nothing.
    pub partial: Vec<usize>,
    /// Variables whose value on entry to the scope may be read: live before the
    /// body, plus the excluded names (conservatively read). An input outside this
    /// set is never read with its entry value (INUSD).
    pub entry: HashSet<String>,
}

/// The dead-store results of one file, by scope index, with the file they belong to.
type DeadMemo = Option<(FileKey, HashMap<usize, Option<Dead>>)>;

thread_local! {
    /// The dead-store results of the current file. See [`shared_dead_stores`].
    static DEAD_MEMO: RefCell<DeadMemo> = const { RefCell::new(None) };
}

/// [`dead_stores`] for the table of the current file ([`SymbolTable::shared`]),
/// memoised per scope. NASGU, STRNU and the unused-input check all read the
/// same analysis, so the file pays for it once.
pub fn shared_dead_stores(
    table: &SymbolTable,
    index: usize,
    tree: &Tree,
    source: &str,
) -> Option<Dead> {
    let key = FileKey::of(source);
    DEAD_MEMO.with(|memo| {
        let mut memo = memo.borrow_mut();
        let (memo_key, results) = memo.get_or_insert_with(|| (key, HashMap::new()));
        if *memo_key != key {
            *memo_key = key;
            results.clear();
        }
        results
            .entry(index)
            .or_insert_with(|| dead_stores(table, index, tree, source))
            .clone()
    })
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
    let Some(body) = scope_body(scope, tree.root_node()) else {
        return Some(Dead::default());
    };
    if calls_dynamic(body, source) {
        return None;
    }

    let mut names = Names::default();
    let def_ids: Vec<u32> = scope.defs.iter().map(|d| names.intern(&d.name)).collect();
    let use_ids: Vec<u32> = scope.uses.iter().map(|u| names.intern(&u.name)).collect();
    let mut excluded = Live::new();
    for def in scope
        .defs
        .iter()
        .filter(|d| matches!(d.kind, DefKind::Global | DefKind::Persistent))
    {
        excluded.insert(names.intern(&def.name));
    }
    let mut compound = Vec::new();
    compound_targets(body, source, &mut compound);
    for name in &compound {
        excluded.insert(names.intern(name));
    }
    // A nested function shares the variables of the functions around it.
    let mut outer = match scope.kind {
        ScopeKind::NestedFunction => scope.parent,
        _ => None,
    };
    while let Some(index) = outer.and_then(|i| table.scopes.get(i)) {
        for def in &index.defs {
            excluded.insert(names.intern(&def.name));
        }
        for usage in &index.uses {
            excluded.insert(names.intern(&usage.name));
        }
        outer = index.parent;
    }
    for &child in &scope.children {
        // A lambda captures its variables when it is created. The enclosing scope
        // already records them as reads at that point, so they are not excluded.
        // A local function of a script has its own workspace: only a nested
        // function shares the variables of its parent.
        if let Some(child) = table.scopes.get(child).filter(|c| {
            !matches!(c.kind, ScopeKind::Lambda)
                && (scope.kind != ScopeKind::Script || c.kind == ScopeKind::NestedFunction)
        }) {
            for usage in &child.uses {
                excluded.insert(names.intern(&usage.name));
            }
            for def in &child.defs {
                excluded.insert(names.intern(&def.name));
            }
        }
    }
    // Live at the end: every variable of a script, the outputs of a function.
    let mut exit = Live::new();
    for (def, &id) in scope.defs.iter().zip(&def_ids) {
        if scope.kind == ScopeKind::Script || def.kind == DefKind::OutputArg {
            exit.insert(id);
        }
    }

    let mut def_spans: Vec<(usize, usize, usize)> = scope
        .defs
        .iter()
        .enumerate()
        .map(|(i, d)| (d.byte_range.start, d.byte_range.end, i))
        .collect();
    def_spans.sort_unstable();
    let mut use_spans: Vec<(usize, usize, usize)> = scope
        .uses
        .iter()
        .enumerate()
        .map(|(i, u)| (u.byte_range.start, u.byte_range.end, i))
        .collect();
    use_spans.sort_unstable();
    let mut analysis = Analysis {
        scope,
        source,
        names,
        def_ids,
        use_ids,
        exit: exit.clone(),
        excluded,
        truths: scope_truths(table, index, tree.root_node(), source),
        loops: Vec::new(),
        extra: Live::new(),
        reports: Vec::new(),
        partial_reports: Vec::new(),
        bail: false,
        def_spans,
        use_spans,
        facts: Facts::default(),
    };
    let before_body = analysis.block(body, exit, true);
    if analysis.bail {
        return None;
    }
    let mut live = analysis.finish(before_body);
    live.absorb(&analysis.excluded);
    let entry = live
        .ids()
        .map(|id| analysis.names.list[id as usize].clone())
        .collect();
    analysis.reports.sort_unstable();
    analysis.reports.dedup();
    analysis.partial_reports.sort_unstable();
    analysis.partial_reports.dedup();
    Some(Dead {
        full: analysis.reports,
        partial: analysis.partial_reports,
        entry,
    })
}

/// The truth value of every constant condition in the file, keyed by the start
/// byte of the condition node. See the module docs for what counts as constant.
pub fn constant_truths(table: &SymbolTable, root: Node, source: &str) -> HashMap<usize, bool> {
    let mut truths = HashMap::new();
    for index in 0..table.scopes.len() {
        truths.extend(scope_truths(table, index, root, source));
    }
    truths
}

/// The truth value of the constant conditions of scope `index`, keyed by the
/// start byte of the condition node. Conditions in unreachable code are absent.
pub fn scope_truths(
    table: &SymbolTable,
    index: usize,
    root: Node,
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
    let Some(body) = scope_body(scope, root) else {
        return HashMap::new();
    };
    let mut def_spans: Vec<(usize, usize, usize)> = scope
        .defs
        .iter()
        .enumerate()
        .map(|(i, d)| (d.byte_range.start, d.byte_range.end, i))
        .collect();
    def_spans.sort_unstable();
    let mut pass = ConstPass {
        scope,
        source,
        truths: HashMap::new(),
        def_spans,
        def_names: scope.defs.iter().map(|d| d.name.as_str()).collect(),
        facts: Facts::default(),
    };
    pass.sequence(&named(body), Some(Known::new()));
    pass.truths
}

/// Whether the assignment whose target covers `range` stores into a field
/// (`s.f = v`, `s(2).f = v`, `s.f(3) = v`) rather than only indexing (`x(i) = v`,
/// `c{i} = v`). The target is the left-hand side of the enclosing `assignment`.
pub fn is_field_store(tree: &Tree, range: Range<usize>) -> bool {
    let root = tree.root_node();
    let Some(target) = root.descendant_for_byte_range(range.start, range.end) else {
        return false;
    };
    // The nearest assignment at or above the target.
    let path = crate::analysis::ancestors::path_from(root, target);
    let Some(node) = path.iter().rev().find(|n| n.kind() == "assignment") else {
        return false;
    };
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
fn scope_root<'t>(root: Node<'t>, scope: &Scope) -> Option<Node<'t>> {
    let mut node = root.descendant_for_byte_range(scope.byte_range.start, scope.byte_range.end)?;
    while !matches!(node.kind(), "source_file" | "function_definition") {
        // The root of a broken function header is an `ERROR` node, see
        // `SymbolTable::build`.
        match node.parent() {
            Some(parent) => node = parent,
            None => break,
        }
    }
    Some(node)
}

/// The node whose named children are the statements of a scope: the
/// `source_file` of a script, the `block` of a function.
fn scope_body<'t>(scope: &Scope, tree_root: Node<'t>) -> Option<Node<'t>> {
    let root = scope_root(tree_root, scope)?;
    if root.kind() == "source_file" {
        Some(root)
    } else if root.kind() == "function_definition" || scope.kind != ScopeKind::Script {
        named(root).into_iter().find(|n| n.kind() == "block")
    } else {
        None
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
fn compound_targets(node: Node, source: &str, out: &mut Vec<String>) {
    if node.kind() == "function_definition" {
        return;
    }
    if matches!(node.kind(), "command" | "binary_operator" | "ERROR") {
        if let Some(name) = compound_target(&source[node.byte_range()]) {
            out.push(name);
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
    if !qualified
        && callee_word(node, source, Dotted::Trailing).is_some_and(|name| names.contains(&name))
    {
        return true;
    }
    children(node)
        .into_iter()
        .any(|child| calls_any(child, source, names))
}

/// A statement that calls `error` or `rethrow`. MATLAB treats it as the end of
/// the flow: values stored before it are dead and the statements after it cannot
/// run (probed: `rethrow(err)` in a catch block reports the same stores as `error`).
/// `error` ends the flow unless its first argument is a variable or a field of one:
/// with `error(msg)`, `error(ME)` or `error(ME.message)` MATLAB keeps the values
/// live and reports no UNRCH after it (probed; `error('a')`, `error(['a' 'b'])` and
/// `error(sprintf('b'))` end the flow).
fn is_error_call(node: Node, source: &str) -> bool {
    match callee_word(node, source, Dotted::Trailing) {
        Some("rethrow") => true,
        Some("error") => !first_argument_is_name(node, source),
        _ => false,
    }
}

/// Whether the first argument of a call or command is a plain name or a field.
fn first_argument_is_name(node: Node, source: &str) -> bool {
    if node.kind() == "function_call" {
        let mut cursor = node.walk();
        return node
            .children(&mut cursor)
            .filter(|c| c.kind() == "arguments")
            .find_map(|args| args.named_child(0))
            .is_some_and(|first| matches!(first.kind(), "identifier" | "field_expression"));
    }
    source[node.byte_range()]
        .split_whitespace()
        .nth(1)
        .is_some_and(is_identifier)
}

/// The variables named by a plain `clear a b` statement.
///
/// `None` for any other statement, including `clear`, `clear all` and
/// `clear -regexp`. Those are read as ordinary uses, which is conservative.
pub(crate) fn clear_names(text: &str) -> Option<Vec<String>> {
    let mut words = text
        .split(|c: char| c.is_whitespace() || c == ';')
        .filter(|w| !w.is_empty());
    if !matches!(words.next()?, "clear" | "clearvars") {
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

/// Whether an `ERROR` statement holds a `function_definition` and nothing else
/// with a block. Probed (`elseif x, c = 2; end` after a `try` closed the
/// function): MATLAB ends the function at that `end`, and the statements after
/// the error are a script's, so their stores are reported only when overwritten.
fn ends_function_early(node: Node) -> bool {
    node.kind() == "ERROR"
        && children(node)
            .into_iter()
            .any(|child| child.kind() == "function_definition")
        && children(node)
            .into_iter()
            .filter(|child| child.kind() != "function_definition")
            .all(|child| !has_block(child))
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
    // Looked up once: `is_condition` scans the children of `node`, which made a long
    // `elseif` chain quadratic.
    let conditions = fields(node, "condition");
    for child in named(node) {
        if conditions
            .iter()
            .any(|condition| condition.id() == child.id())
        {
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
    /// The interned names of the scope, see [`Names`].
    names: Names,
    /// The name id of each def of `scope.defs`.
    def_ids: Vec<u32>,
    /// The name id of each use of `scope.uses`.
    use_ids: Vec<u32>,
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
    /// `(start, end, index)` of each def of `scope.defs`, sorted by start, so
    /// [`within`] finds the defs of a statement without scanning the scope.
    def_spans: Vec<(usize, usize, usize)>,
    /// `(start, end, index)` of each use of `scope.uses`, sorted by start.
    use_spans: Vec<(usize, usize, usize)>,
    /// Memoised subtree facts (`break`, jumps, blocks).
    facts: Facts,
}

/// The indices of the spans of `spans` (sorted by start) that lie within `range`.
fn within(
    spans: &[(usize, usize, usize)],
    range: Range<usize>,
) -> impl Iterator<Item = usize> + '_ {
    let from = spans.partition_point(|span| span.0 < range.start);
    spans[from..]
        .iter()
        .take_while(move |span| span.0 < range.end)
        .filter(move |span| span.1 <= range.end)
        .map(|span| span.2)
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
    fn is_infinite_loop(&mut self, node: Node) -> bool {
        node.kind() == "while_statement"
            && !self.facts.has_break(node)
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
            // A stray `end` after a broken `elseif` closes the function before it:
            // the function is its own scope, and the rest of the node is a plain
            // statement of this scope (see `ends_function_early`).
            "ERROR" if ends_function_early(node) => self.leaf(node, after, report),
            _ if self.facts.has_block(node) => {
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
        let mut kills = Live::new();
        // A name that is not interned is in no set, so clearing it changes nothing.
        for name in cleared.iter().flatten() {
            if let Some(id) = self.names.get(name) {
                kills.insert(id);
            }
        }
        let mut reads = if cleared.is_some() {
            Live::new()
        } else {
            self.uses_in(range.clone())
        };
        // `after` is not extended with the catch body's reads: an error stops the
        // statement before its store, so a store is dead when nothing after it reads
        // it (probed: `s = 1` as the last statement of a try body, read only in catch).
        // The catch body is still live before each statement (see `finish`).
        // An `onCleanup` object is kept for its destructor, MATLAB does not report it.
        let object = source[range.clone()]
            .split_once('=')
            .is_some_and(|(_, rhs)| rhs.trim_start().starts_with("onCleanup"));
        for index in within(&self.def_spans, range.clone()) {
            let def = &self.scope.defs[index];
            if def.kind != DefKind::Assignment {
                continue;
            }
            let id = self.def_ids[index];
            if object {
                kills.insert(id);
                continue;
            }
            if def.partial {
                if report && !after.contains(id) && !self.excluded.contains(id) {
                    self.partial_reports.push(index);
                }
                reads.insert(id);
                continue;
            }
            kills.insert(id);
            if report && !after.contains(id) && !self.excluded.contains(id) {
                self.reports.push(index);
            }
        }
        // `after` is owned, so it is updated in place: a script's exit set holds
        // every variable, and copying it per statement is quadratic.
        let mut live = after;
        for id in kills.ids() {
            live.remove(id);
        }
        live.absorb(&reads);
        self.finish(live)
    }

    /// The parts of a clause that come after its condition. The condition's
    /// reads go to `reads`.
    fn clause_parts<'t>(&self, clause: Node<'t>, reads: &mut Live) -> Vec<Node<'t>> {
        named(clause)
            .into_iter()
            .filter(|part| {
                if is_condition(clause, *part) {
                    reads.absorb(&self.uses_in(part.byte_range()));
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
                reads.absorb(&self.uses_in(condition.byte_range()));
            }
            match self.clause_truth(&clause) {
                Some(false) => {}
                Some(true) => {
                    out.absorb(&self.branch(&clause.parts, after.clone(), report));
                    reachable = false;
                }
                None => out.absorb(&self.branch(&clause.parts, after.clone(), report)),
            }
        }
        // The condition of the last clause may be false: nothing runs.
        if reachable {
            out.absorb(&after);
        }
        out.absorb(&reads);
        self.finish(out)
    }

    fn for_statement(&mut self, node: Node, after: Live, report: bool) -> Live {
        let source = self.source;
        let mut reads = Live::new();
        let mut var = None;
        let mut body = Vec::new();
        for child in named(node) {
            if child.kind() == "iterator" {
                // The loop variable is written, not read: only the range is. The
                // range may itself be a plain variable (`for p = positions`), so
                // only the first identifier is skipped.
                let parts = named(child);
                var = parts
                    .iter()
                    .find(|c| c.kind() == "identifier")
                    .map(|c| source[c.byte_range()].to_string());
                let mut loop_var_seen = false;
                for part in parts {
                    if !loop_var_seen && part.kind() == "identifier" {
                        loop_var_seen = true;
                        continue;
                    }
                    reads.absorb(&self.uses_in(part.byte_range()));
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
            self.drop_name(&mut top, name);
        }
        top.absorb(&reads);
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
        if body.iter().any(|stmt| self.facts.has_break(*stmt)) {
            self.bail = true;
            return after;
        }
        let mut top = self.branch(body, after.clone(), report);
        if let Some(name) = var {
            self.drop_name(&mut top, name);
        }
        top.absorb(&after);
        top.absorb(&reads);
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
                reads.absorb(&self.uses_in(child.byte_range()));
                condition = Some(child);
            } else {
                body.push(child);
            }
        }
        let truth = condition.and_then(|c| self.truths.get(&c.start_byte()).copied());
        let top = match truth {
            // The body never runs.
            Some(false) => after.union(&reads),
            // The loop is left only by `break`, so `after` is reached only through one.
            Some(true) => self.loop_head(&body, &after, reads, None, report),
            // The condition is evaluated before every pass, so it is live at the head.
            None => {
                let base = after.union(&reads);
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
                self.drop_name(&mut entry, name);
            }
            let mut next = base.clone();
            next.absorb(&entry);
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
                _ if is_condition(node, child) => {
                    reads.absorb(&self.uses_in(child.byte_range()));
                }
                "case_clause" => {
                    let parts = self.clause_parts(child, &mut reads);
                    out.absorb(&self.branch(&parts, after.clone(), report));
                }
                "otherwise_clause" => {
                    has_otherwise = true;
                    let parts = self.clause_parts(child, &mut reads);
                    out.absorb(&self.branch(&parts, after.clone(), report));
                }
                _ => {}
            }
        }
        // Without an otherwise clause no case may match.
        if !has_otherwise {
            out.absorb(&after);
        }
        out.absorb(&reads);
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
        self.extra.absorb(&catch_live);
        let body_live = self.branch(&body, after, report);
        self.extra = saved;

        let mut out = body_live;
        out.absorb(&catch_live);
        self.finish(out)
    }

    /// Variables read by the identifiers inside `range`.
    fn uses_in(&self, range: Range<usize>) -> Live {
        let mut live = Live::new();
        for index in within(&self.use_spans, range) {
            live.insert(self.use_ids[index]);
        }
        live
    }

    /// Removes the variable `name` from `live`, if it is interned.
    fn drop_name(&self, live: &mut Live, name: &str) {
        if let Some(id) = self.names.get(name) {
            live.remove(id);
        }
    }

    /// Adds the live variables of the enclosing `try` bodies.
    fn finish(&self, mut live: Live) -> Live {
        live.absorb(&self.extra);
        live
    }
}

/// The forward pass that finds constant conditions: the variables with a known
/// truth value, merged over the branches of each statement.
struct ConstPass<'a> {
    scope: &'a Scope,
    source: &'a str,
    truths: HashMap<usize, bool>,
    /// `(start, end, index)` of each def of `scope.defs`, sorted by start.
    def_spans: Vec<(usize, usize, usize)>,
    /// The names of the defs of the scope.
    def_names: HashSet<&'a str>,
    facts: Facts,
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
            _ if self.facts.has_block(node) => Some(self.remove_defs(env, node)),
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
        if clear_names(text).is_none()
            && matches!(text.split_whitespace().next(), Some("clear" | "clearvars"))
        {
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
        if body.iter().any(|stmt| self.facts.contains_jump(*stmt)) {
            let head = self.remove_defs(pre, node);
            self.forget_truths(node);
            self.sequence(&body, Some(head.clone()));
            return Some(head);
        }
        let mut head = pre.clone();
        for _ in 0..16 {
            self.forget_truths(node);
            let out = self.sequence(&body, Some(head.clone()));
            let next = join(Some(pre.clone()), out).unwrap_or_default();
            if next == head {
                break;
            }
            head = next;
        }
        Some(head)
    }

    /// Drops the truth values recorded inside `node`: a fixed-point pass over a loop
    /// records them again, and a value from an earlier pass must not stay.
    fn forget_truths(&mut self, node: Node) {
        let (start, end) = (node.start_byte(), node.end_byte());
        self.truths.retain(|at, _| *at < start || *at >= end);
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
        if truth == Some(true) && !self.facts.has_break(node) {
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
        for index in within(&self.def_spans, node.byte_range()) {
            env.remove(&self.scope.defs[index].name);
        }
        env
    }

    /// The truth value of a literal or a known variable, `None` for anything else.
    fn eval(&self, node: Node, env: &Known) -> Option<bool> {
        let text = &self.source[node.byte_range()];
        match node.kind() {
            "number" => text.parse::<f64>().ok().map(|v| v != 0.0),
            "matrix" if node.named_child_count() == 0 => Some(false),
            "identifier" => {
                if self.def_names.contains(text) {
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
            "parenthesis" => node.named_child(0).and_then(|inner| self.eval(inner, env)),
            "function_call" => self.eval_known_call(node),
            _ => None,
        }
    }

    /// `strcmp` and `strcmpi` of two text values, and `ischar` of one, when each value
    /// is known: a single literal that reaches the call (`b = 'x'; strcmp(b, 'X')`).
    /// Probed: these fold like a literal condition (the other branch is dead, and a
    /// copy of the result folds too). `==`, `isempty` and `isnumeric` do not fold.
    fn eval_known_call(&self, node: Node) -> Option<bool> {
        let source = self.source;
        let name = node.child_by_field_name("name")?;
        let args: Vec<Node> = call_argument_nodes(node)
            .into_iter()
            .map(|arg| known_literal(arg, source))
            .collect::<Option<_>>()?;
        match (&source[name.byte_range()], args.as_slice()) {
            ("strcmp", [a, b]) => Some(text_of(*a, source)? == text_of(*b, source)?),
            ("strcmpi", [a, b]) => {
                Some(text_of(*a, source)?.eq_ignore_ascii_case(text_of(*b, source)?))
            }
            ("ischar", [a]) => match a.kind() {
                "number" => Some(false),
                "string" => text_of(*a, source).map(|_| true),
                _ => None,
            },
            _ => None,
        }
    }
}

/// The literal that the expression `node` stands for: itself, or the one value that
/// reaches it when it is a name. `None` when there are several values or none is known.
fn known_literal<'a>(node: Node<'a>, source: &str) -> Option<Node<'a>> {
    let mut values = resolved_values(node, source)?;
    (values.len() == 1).then(|| values.swap_remove(0))
}

/// The text of a single-quoted or double-quoted literal without quotes inside.
fn text_of<'s>(node: Node, source: &'s str) -> Option<&'s str> {
    if node.kind() != "string" {
        return None;
    }
    let text = &source[node.byte_range()];
    let inner = text
        .strip_prefix('\'')
        .and_then(|rest| rest.strip_suffix('\''))
        .or_else(|| {
            text.strip_prefix('"')
                .and_then(|rest| rest.strip_suffix('"'))
        })?;
    (!inner.contains(['\'', '"'])).then_some(inner)
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
