//! Reads that no definition can reach, for the unset-variable checks.
//!
//! A forward "may be defined" walk over a function body. A read is reported
//! when no definition of its name reaches it on any path: that is NODEF, and
//! for a nested function reading a parent variable, SVNODEF (see the
//! unset-variables engine). Branches merge by union, so a definition in one
//! branch makes a later read silent, which MATLAB does too. The rules, from
//! probes of R2026b's `checkcode`:
//!
//! - `if false`, `if 0`, `while false` and the branch of a constant-true `if`
//!   that is not taken are dead: their definitions reach nothing.
//! - Code after `return`, `break`, `continue`, `error(...)` and the other
//!   terminators is unreachable and gives no reads.
//! - A loop body's definitions may reach the top of the next iteration, so the
//!   loop header and the code after the loop see every definition in the body
//!   syntactically. This never makes a read unreachable: it only silences it.
//! - `parfor`: definitions in the body do not reach the code after the loop,
//!   except sliced (indexed or field) assignments. Reads in the body are PFIIN,
//!   not NODEF, so they are not recorded.
//! - `try`: the catch clause starts with everything the try body defines.
//! - `clear x` removes `x`; a plain `clear` is not modelled.

use std::collections::HashSet;

use tree_sitter::Node;

use super::callee::{self, Dotted};
use super::control_flow::{
    bare_command_name, call_argument_nodes, collect_assignment_targets, collect_function_args,
    collect_uses_in_expr, error_call_always_throws, is_endless_loop, literal_condition,
    named_children, node_text, partial_root, PossiblyUnsetVar,
};
use super::liveness::clear_names;

/// The result of [`analyze_may_defined`] for one function.
pub(crate) struct MayDefined {
    /// Reads that no definition reaches, outside `parfor` bodies, in source order.
    pub undefined_reads: Vec<PossiblyUnsetVar>,
    /// Names defined by a statement on a reachable path (inputs and outputs excluded).
    pub defined: HashSet<String>,
    /// Input and output argument names, defined on entry.
    pub arguments: HashSet<String>,
}

/// The names that may be defined at a point; `None` is unreachable code.
type State = Option<HashSet<String>>;

/// Run the may-defined analysis over `func_node`'s body.
pub(crate) fn analyze_may_defined(func_node: Node, source: &str) -> MayDefined {
    let mut arguments = HashSet::new();
    collect_function_args(func_node, source, &mut arguments);
    let mut walker = Walker {
        source,
        undefined: Vec::new(),
        defined: HashSet::new(),
        parfor_depth: 0,
    };
    let mut state: State = Some(arguments.clone());
    if let Some(body) = named_children(func_node)
        .into_iter()
        .find(|c| c.kind() == "block")
    {
        walker.block(body, &mut state);
    }
    let mut undefined_reads = walker.undefined;
    undefined_reads.sort_by_key(|u| u.use_byte_range.start);
    MayDefined {
        undefined_reads,
        defined: walker.defined,
        arguments,
    }
}

struct Walker<'a> {
    source: &'a str,
    undefined: Vec<PossiblyUnsetVar>,
    defined: HashSet<String>,
    /// Depth of `parfor` bodies, where reads are not recorded.
    parfor_depth: usize,
}

impl Walker<'_> {
    fn block(&mut self, block: Node, state: &mut State) {
        for statement in named_children(block) {
            self.statement(statement, state);
        }
    }

    fn statement(&mut self, node: Node, state: &mut State) {
        let source = self.source;
        match node.kind() {
            "assignment" => self.assignment(node, state),
            "global_operator" | "persistent_operator" => {
                let names = identifiers(node, source);
                self.define(names, state);
            }
            "if_statement" => self.if_statement(node, state),
            "switch_statement" => self.switch_statement(node, state),
            "for_statement" => self.for_statement(node, state),
            "while_statement" => self.while_statement(node, state),
            "try_statement" => self.try_statement(node, state),
            // An spmd body runs in sequence (probed: `spmd t = 1; end; r = t`).
            "spmd_statement" => {
                for child in named_children(node) {
                    if child.kind() == "block" {
                        self.block(child, state);
                    }
                }
            }
            "return_statement" | "break_statement" | "continue_statement" => *state = None,
            // `syms a b real` and `syms x(t)` declare variables (see the symbol table).
            "command" if node.named_child(0).map(|c| node_text(c, source)) == Some("syms") => {
                let names = syms_names(node, source);
                self.define(names, state);
            }
            "command" if clear_names(node_text(node, source)).is_some() => {
                let names = clear_names(node_text(node, source)).unwrap_or_default();
                if let Some(assigned) = state {
                    for name in &names {
                        assigned.remove(name);
                    }
                }
            }
            // `q;` reads `q`.
            "command" if bare_command_name(node).is_some() => {
                if let Some(name) = bare_command_name(node) {
                    self.read_name(name, state);
                }
            }
            "function_call" | "command" => {
                self.reads(node, state);
                self.saved_names(node, source, state);
                self.dynamic_definitions(node, source, state);
                if terminates_path(node, source) {
                    *state = None;
                }
            }
            _ => self.reads(node, state),
        }
    }

    fn assignment(&mut self, node: Node, state: &mut State) {
        let source = self.source;
        if let Some(rhs) = node.child_by_field_name("right") {
            self.reads(rhs, state);
        }
        let Some(lhs) = node.child_by_field_name("left") else {
            return;
        };
        if let Some(assigned) = state {
            let mut uses = Vec::new();
            collect_assignment_targets(lhs, source, assigned, &mut uses);
            self.record(uses);
            // Also the indexed or field targets inside a `[a, c{2}] = ...` list.
            for name in lhs_names(lhs, source) {
                assigned.insert(name.clone());
                self.defined.insert(name);
            }
        }
    }

    fn if_statement(&mut self, node: Node, state: &mut State) {
        let mut entering = state.clone();
        let mut exit: State = None;
        let mut has_else = false;
        let mut clauses = vec![node];
        for child in named_children(node) {
            match child.kind() {
                "elseif_clause" => clauses.push(child),
                "else_clause" => {
                    has_else = true;
                    clauses.push(child);
                }
                _ => {}
            }
        }
        for clause in clauses {
            let is_else = clause.kind() == "else_clause";
            let truth = if is_else {
                None
            } else {
                literal_condition(clause, self.source)
            };
            if !is_else {
                if let Some(condition) = clause.child_by_field_name("condition") {
                    self.reads(condition, &entering);
                }
            }
            let mut branch = if truth == Some(false) {
                None
            } else {
                entering.clone()
            };
            if let Some(block) = named_children(clause)
                .into_iter()
                .find(|c| c.kind() == "block")
            {
                self.block(block, &mut branch);
            }
            join(&mut exit, branch);
            if truth == Some(true) {
                entering = None;
            }
        }
        if !has_else {
            join(&mut exit, entering);
        }
        *state = exit;
    }

    fn switch_statement(&mut self, node: Node, state: &mut State) {
        if let Some(subject) = node.child_by_field_name("condition") {
            self.reads(subject, state);
        }
        let entering = state.clone();
        let mut exit: State = None;
        let mut has_otherwise = false;
        for child in named_children(node) {
            if !matches!(child.kind(), "case_clause" | "otherwise_clause") {
                continue;
            }
            has_otherwise |= child.kind() == "otherwise_clause";
            let mut branch = entering.clone();
            if let Some(expr) = child.child_by_field_name("condition") {
                self.reads(expr, &branch);
            }
            if let Some(block) = named_children(child)
                .into_iter()
                .find(|c| c.kind() == "block")
            {
                self.block(block, &mut branch);
            }
            join(&mut exit, branch);
        }
        if !has_otherwise {
            join(&mut exit, entering);
        }
        *state = exit;
    }

    fn for_statement(&mut self, node: Node, state: &mut State) {
        let source = self.source;
        let parallel = node.child(0).is_some_and(|c| c.kind() == "parfor");
        let mut loop_var: Option<String> = None;
        for child in named_children(node) {
            if child.kind() != "iterator" {
                continue;
            }
            let mut cursor = child.walk();
            for part in child.children(&mut cursor) {
                if loop_var.is_none() && part.kind() == "identifier" {
                    loop_var = Some(node_text(part, source).to_string());
                } else if part.kind() != "=" {
                    self.reads(part, state);
                }
            }
        }
        let body = named_children(node)
            .into_iter()
            .find(|c| c.kind() == "block");

        if parallel {
            // The body starts from the state before the loop. Its definitions do not
            // reach the code after it, except sliced (indexed or field) ones.
            let mut inside = state.clone();
            if let Some(var) = &loop_var {
                add_to(HashSet::from([var.clone()]), &mut inside);
                self.defined.insert(var.clone());
            }
            if let Some(body) = body {
                self.parfor_depth += 1;
                self.block(body, &mut inside);
                self.parfor_depth -= 1;
                let mut sliced = HashSet::new();
                syntactic_defs(body, source, &mut sliced, true);
                add_to(sliced, state);
            }
            return;
        }

        // The body may run again, so its definitions (and the loop variable) reach
        // the top of the loop and the code after it, which may run zero times.
        let mut header = HashSet::new();
        if let Some(body) = body {
            syntactic_defs(body, source, &mut header, false);
        }
        if let Some(var) = &loop_var {
            header.insert(var.clone());
        }
        add_to(header, state);
        let mut inside = state.clone();
        if let Some(body) = body {
            self.block(body, &mut inside);
        }
        if let Some(var) = loop_var {
            if state.is_some() {
                self.defined.insert(var);
            }
        }
    }

    fn while_statement(&mut self, node: Node, state: &mut State) {
        let source = self.source;
        let body = named_children(node)
            .into_iter()
            .find(|c| c.kind() == "block");
        let mut body_defs = HashSet::new();
        if let Some(body) = body {
            syntactic_defs(body, source, &mut body_defs, false);
        }
        if literal_condition(node, source) == Some(false) {
            // The body never runs. Its definitions reach nothing, but they are still
            // defined code (a nested function reading them is silent, probed).
            self.defined.extend(body_defs);
            let mut dead: State = None;
            if let Some(body) = body {
                self.block(body, &mut dead);
            }
            return;
        }
        add_to(body_defs, state);
        let header = state.clone();
        if let Some(condition) = node.child_by_field_name("condition") {
            self.reads(condition, &header);
        }
        let mut inside = header;
        if let Some(body) = body {
            self.block(body, &mut inside);
        }
        if is_endless_loop(node, source) {
            *state = None;
        }
    }

    fn try_statement(&mut self, node: Node, state: &mut State) {
        let source = self.source;
        let entering = state.clone();
        let body = named_children(node)
            .into_iter()
            .find(|c| c.kind() == "block");
        let mut body_defs = HashSet::new();
        if let Some(body) = body {
            syntactic_defs(body, source, &mut body_defs, false);
        }
        let mut exit = entering.clone();
        if let Some(body) = body {
            self.block(body, &mut exit);
        }
        for catch in named_children(node)
            .into_iter()
            .filter(|c| c.kind() == "catch_clause")
        {
            let mut catch_state = entering.clone();
            add_to(body_defs.clone(), &mut catch_state);
            for part in named_children(catch) {
                if part.kind() == "identifier" {
                    self.define(vec![node_text(part, source).to_string()], &mut catch_state);
                }
            }
            if let Some(block) = named_children(catch)
                .into_iter()
                .find(|c| c.kind() == "block")
            {
                self.block(block, &mut catch_state);
            }
            join(&mut exit, catch_state);
        }
        *state = exit;
    }

    /// `save(file, 'x')` and `save file x` read the variables they name, by text: MATLAB
    /// reports NODEF at the name. The first argument is the file (probed: `save('q')`
    /// alone is silent).
    fn saved_names(&mut self, call: Node, source: &str, state: &State) {
        if callee_name(call, source) != Some("save") {
            return;
        }
        let Some(assigned) = state else {
            return;
        };
        for arg in call_argument_nodes(call).into_iter().skip(1) {
            let Some(name) = identifier_text(node_text(arg, source)) else {
                continue;
            };
            if !assigned.contains(&name) {
                let pos = arg.start_position();
                self.record(vec![PossiblyUnsetVar {
                    name,
                    use_byte_range: arg.start_byte()..arg.end_byte(),
                    use_line: pos.row + 1,
                    use_column: pos.column + 1,
                }]);
            }
        }
    }

    /// `load(file, 'x')` and `load file x` define the names they list (probed).
    /// `eval`, `evalin`, `assignin` and an unnamed `load` define nothing: MATLAB
    /// still reports the reads after them (probed).
    fn dynamic_definitions(&mut self, call: Node, source: &str, state: &mut State) {
        if callee_name(call, source) != Some("load") {
            return;
        }
        let names: Vec<String> = call_argument_nodes(call)
            .into_iter()
            .skip(1)
            .filter_map(|arg| identifier_text(node_text(arg, source)))
            .collect();
        self.define(names, state);
    }

    /// Record the reads in `node` that no definition in `state` reaches.
    fn reads(&mut self, node: Node, state: &State) {
        let Some(assigned) = state else {
            return;
        };
        let mut uses = Vec::new();
        collect_uses_in_expr(node, self.source, assigned, &mut uses);
        self.record(uses);
    }

    /// Record a read of the bare name `name_node` if no definition reaches it.
    fn read_name(&mut self, name_node: Node, state: &State) {
        let Some(assigned) = state else {
            return;
        };
        let text = node_text(name_node, self.source);
        if !assigned.contains(text) {
            let pos = name_node.start_position();
            self.record(vec![PossiblyUnsetVar {
                name: text.to_string(),
                use_byte_range: name_node.start_byte()..name_node.end_byte(),
                use_line: pos.row + 1,
                use_column: pos.column + 1,
            }]);
        }
    }

    fn record(&mut self, uses: Vec<PossiblyUnsetVar>) {
        if self.parfor_depth == 0 {
            self.undefined.extend(uses);
        }
    }

    /// Add definitions to a reachable state, and to the defined names.
    fn define(&mut self, names: Vec<String>, state: &mut State) {
        if let Some(assigned) = state {
            for name in names {
                assigned.insert(name.clone());
                self.defined.insert(name);
            }
        }
    }
}

/// Add names to a reachable state without recording them as defined code.
fn add_to(names: HashSet<String>, state: &mut State) {
    if let Some(assigned) = state {
        assigned.extend(names);
    }
}

/// Merge a branch's state into the state after a statement: a name may be
/// defined after it when it may be defined on some branch that falls through.
fn join(exit: &mut State, other: State) {
    let Some(other) = other else {
        return;
    };
    match exit {
        Some(assigned) => assigned.extend(other),
        None => *exit = Some(other),
    }
}

/// The function name of a call or the command word of a command.
fn callee_name<'s>(call: Node, source: &'s str) -> Option<&'s str> {
    match call.kind() {
        "function_call" => callee::call_path(call, source, Dotted::Trailing),
        "command" => callee::command_path(call, source),
        _ => None,
    }
}

/// Whether a call ends control flow: `error` with a message that always throws,
/// or one of the other calls that never return.
fn terminates_path(call: Node, source: &str) -> bool {
    match callee_name(call, source) {
        Some("error") => error_call_always_throws(call, source),
        Some("throw" | "rethrow" | "exit" | "quit") => true,
        _ => false,
    }
}

/// `text` as a variable name: an identifier, optionally quoted (`'sig'`, `"sig"`).
fn identifier_text(text: &str) -> Option<String> {
    let text = text.trim();
    let text = text
        .strip_prefix(['\'', '"'])
        .and_then(|rest| rest.strip_suffix(['\'', '"']))
        .unwrap_or(text);
    let mut chars = text.chars();
    let first = chars.next()?;
    (first.is_alphabetic() && chars.all(|c| c.is_alphanumeric() || c == '_'))
        .then(|| text.to_string())
}

/// The direct identifier children of `node`.
fn identifiers(node: Node, source: &str) -> Vec<String> {
    named_children(node)
        .into_iter()
        .filter(|c| c.kind() == "identifier")
        .map(|c| node_text(c, source).to_string())
        .collect()
}

/// The names a `syms` command declares: the leading identifier of each argument.
fn syms_names(command: Node, source: &str) -> Vec<String> {
    named_children(command)
        .into_iter()
        .skip(1)
        .filter_map(|arg| {
            let text = node_text(arg, source).trim();
            let name: String = text
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            name.starts_with(char::is_alphabetic).then_some(name)
        })
        .collect()
}

/// The names an assignment target defines: the variable itself, each output of
/// a `[a, b] = ...` list, or the root of an indexed or field target.
fn lhs_names(lhs: Node, source: &str) -> Vec<String> {
    match lhs.kind() {
        "identifier" => vec![node_text(lhs, source).to_string()],
        "multioutput_variable" => {
            let mut names = Vec::new();
            for child in named_children(lhs) {
                match child.kind() {
                    "identifier" => names.push(node_text(child, source).to_string()),
                    "function_call" | "field_expression" | "cell_index" => {
                        names.extend(lhs_names(child, source));
                    }
                    _ => {}
                }
            }
            names
        }
        "function_call" | "field_expression" | "cell_index" => partial_root(lhs)
            .map(|root| vec![node_text(root, source).to_string()])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// Every name defined inside `node` (not inside nested functions). With
/// `partial_only`, only indexed or field assignments are collected.
fn syntactic_defs(node: Node, source: &str, out: &mut HashSet<String>, partial_only: bool) {
    // A `parfor` body keeps only its sliced definitions (see `for_statement`).
    let partial_only = partial_only
        || (node.kind() == "for_statement" && node.child(0).is_some_and(|c| c.kind() == "parfor"));
    match node.kind() {
        "function_definition" => return,
        "assignment" => {
            if let Some(lhs) = node.child_by_field_name("left") {
                let partial = matches!(
                    lhs.kind(),
                    "function_call" | "field_expression" | "cell_index"
                );
                if !partial_only || partial {
                    out.extend(lhs_names(lhs, source));
                }
            }
        }
        "global_operator" | "persistent_operator" if !partial_only => {
            out.extend(identifiers(node, source));
        }
        // The loop variable is the first identifier; the rest is the range.
        "iterator" if !partial_only => {
            out.extend(
                named_children(node)
                    .into_iter()
                    .find(|c| c.kind() == "identifier")
                    .map(|c| node_text(c, source).to_string()),
            );
        }
        "catch_clause" if !partial_only => {
            out.extend(identifiers(node, source));
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        syntactic_defs(child, source, out, partial_only);
    }
}
