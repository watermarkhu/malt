//! Logical-usage checks (BDLGI, BDLOG1, BDLOG2, BDSCI).
//!
//! BDLGI and BDSCI look at a variable used as a condition: the root of an `if`,
//! `elseif` or `while` condition, or an operand of `&&` or `||` (both through
//! parentheses and `~`). BDSCI also reads the variables reached from those
//! through operators (`if d > 0`, `x && d + 1`). Such a variable is reported
//! only when every definition that reaches the use is of the reported kind,
//! which is how MATLAB behaves in the probes: `d = 1:3; if x, d = 2; end;
//! if d` is silent, `if x, d = 1:3; end; if d` is reported. Undefined paths are
//! ignored; a definition the walker cannot classify (an input, a call, an index
//! assignment, a copy) is silent.
//!
//! - BDLGI  — a reaching definition is non-logical: a range, a matrix literal, a
//!   char vector, a cell, a function handle, `*` `/` `^` `\` arithmetic or `-x`.
//! - BDSCI  — a reaching definition is non-scalar: a range, a matrix literal, a
//!   char vector of two or more characters, a cell, or element-wise arithmetic.
//! - BDLOG1 — a transposed condition or `&&`/`||` operand (`if a'`, `x && a'`).
//! - BDLOG2 — an arithmetic operator (`+ - * / \ ^`) at the root of a condition or
//!   `&&`/`||` operand, reported at the operator (`if z + 1`); the elementwise operators
//!   (`.* ./ .^ .\`) give BDLOG1.
//!
//! All diagnostics are `Severity::Warning` and gated on `disabled_checks`.

use std::collections::BTreeMap;

use super::*;

/// Bound on the fixpoint iterations over a loop; the lattice is small, so the
/// bound is only a safety net.
const MAX_LOOP_ITERATIONS: usize = 16;

/// Functions and commands whose effect on variables the walker cannot follow.
const DYNAMIC_CALLS: &[&str] = &[
    "eval",
    "evalc",
    "evalin",
    "assignin",
    "clear",
    "clearvars",
    "load",
    "run",
];

/// What a definition's value is, for BDLGI (not logical), BDSCI (not scalar)
/// and BDLOG1-style reasoning about logical results.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DefClass {
    nonlogical: bool,
    nonscalar: bool,
    logical: bool,
}

/// A value the walker does not classify.
const OTHER: DefClass = DefClass {
    nonlogical: false,
    nonscalar: false,
    logical: false,
};
/// A logical value: never reported.
const LOGICAL: DefClass = DefClass {
    nonlogical: false,
    nonscalar: false,
    logical: true,
};
/// A non-logical value that is a scalar (a matrix product, a char scalar, a handle).
const NONLOGICAL: DefClass = DefClass {
    nonlogical: true,
    nonscalar: false,
    logical: false,
};
/// A non-logical, non-scalar value (a range, a matrix, a cell, element-wise arithmetic).
const ARRAY: DefClass = DefClass {
    nonlogical: true,
    nonscalar: true,
    logical: false,
};

/// The class of a right-hand side, as far as BDLGI and BDSCI need it.
fn classify(node: Node, source: &str) -> DefClass {
    match node.kind() {
        "range" | "matrix" | "cell" => ARRAY,
        "handle_operator" => NONLOGICAL,
        "string" => {
            // Double-quoted strings are not classified (MATLAB is silent on them).
            match node_text(node, source)
                .strip_prefix('\'')
                .and_then(|t| t.strip_suffix('\''))
            {
                Some(inner) if !inner.contains("''") => DefClass {
                    nonlogical: true,
                    nonscalar: inner.chars().count() >= 2,
                    logical: false,
                },
                _ => OTHER,
            }
        }
        "identifier" => match node_text(node, source) {
            "true" | "false" => LOGICAL,
            _ => OTHER,
        },
        "comparison_operator" | "boolean_operator" | "not_operator" => LOGICAL,
        "parenthesis" => first_named_child(node).map_or(OTHER, |c| classify(c, source)),
        // A transpose of a non-logical value is a non-logical array (`b = x'`).
        "postfix_operator" => match node.named_child(0) {
            Some(operand) if !classify(operand, source).logical => ARRAY,
            _ => OTHER,
        },
        "unary_operator" => {
            let Some(operand) = node.named_child(0) else {
                return OTHER;
            };
            let inner = classify(operand, source);
            match node.child(0).map(|op| node_text(op, source)) {
                Some("-") => DefClass {
                    nonlogical: true,
                    nonscalar: inner.nonscalar,
                    logical: false,
                },
                Some("+") => inner,
                _ => OTHER,
            }
        }
        "binary_operator" => {
            // `+` and `-` are silent even around a product (`2*x+1`, `x + [1 2]`).
            match find_operator_text(node, source).as_str() {
                "&" | "|" => LOGICAL,
                "*" | "/" | "^" | "\\" => NONLOGICAL,
                ".*" | "./" | ".^" | ".\\" => ARRAY,
                _ => OTHER,
            }
        }
        _ => OTHER,
    }
}

/// The definitions that may reach a point, per name, as their classes. A name
/// without an entry is undefined on every path there, which is ignored.
#[derive(Clone, Default, Debug, PartialEq)]
struct Flow {
    defs: BTreeMap<String, Vec<DefClass>>,
    /// Set after a construct the walker cannot follow (a dynamic call, a loop
    /// that can `break` or `continue`): every name may then hold any value.
    opaque: bool,
    /// The names whose definitions are kept, see [`boolean_identifiers`]. `None`
    /// keeps every name. A name outside the set is never asked about, so its
    /// definitions are not kept: a loop variable then costs nothing per clone.
    tracked: Option<std::rc::Rc<std::collections::HashSet<String>>>,
}

impl Flow {
    fn tracks(&self, name: &str) -> bool {
        self.tracked
            .as_ref()
            .is_none_or(|names| names.contains(name))
    }

    /// A whole assignment: the only definition of `name` from here on.
    fn kill(&mut self, name: &str, class: DefClass) {
        if self.tracks(name) {
            self.defs.insert(name.to_string(), vec![class]);
        }
    }

    /// A write that keeps the earlier values (an indexed assignment).
    fn add(&mut self, name: &str, class: DefClass) {
        if !self.tracks(name) {
            return;
        }
        let classes = self.defs.entry(name.to_string()).or_default();
        if !classes.contains(&class) {
            classes.push(class);
        }
    }

    /// Merge the state of another path into this one.
    fn join(&mut self, other: &Flow) {
        for (name, classes) in &other.defs {
            for class in classes {
                self.add(name, *class);
            }
        }
        self.opaque |= other.opaque;
    }

    /// The classes of the definitions that may reach a use of `name`.
    fn reaching(&self, name: &str) -> Vec<DefClass> {
        let mut classes = self.defs.get(name).cloned().unwrap_or_default();
        if self.opaque {
            classes.push(OTHER);
        }
        classes
    }
}

/// Walks a file in textual order with a [`Flow`] per scope and decides, for
/// every boolean use of a variable, whether BDLGI, BDSCI and BDLOG1 fire.
struct FlowWalker<'s> {
    source: &'s str,
    want_lgi: bool,
    want_sci: bool,
    want_log1: bool,
    want_log2: bool,
    /// The decision for each (position, check). A loop body is visited more than
    /// once, so the last visit, made with the converged state, wins.
    decided: BTreeMap<(usize, &'static str), Option<Diagnostic>>,
}

impl FlowWalker<'_> {
    fn text(&self, node: Node) -> &str {
        node_text(node, self.source)
    }

    /// Visit a statement or expression, updating `flow` with its definitions.
    fn visit(&mut self, node: Node, flow: &mut Flow) {
        match node.kind() {
            "function_definition" => {
                // A function has its own variables; its inputs are unknown on entry.
                let mut inner = Flow {
                    tracked: flow.tracked.clone(),
                    ..Flow::default()
                };
                if let Some(args) = child_of_kind(node, "function_arguments") {
                    for id in identifiers_in(args) {
                        inner.kill(self.text(id), OTHER);
                    }
                }
                self.visit_children(node, &mut inner);
            }
            "lambda" => {}
            "assignment" => {
                let right = node.child_by_field_name("right");
                if let Some(right) = right {
                    self.visit(right, flow);
                }
                let Some(left) = node.child_by_field_name("left") else {
                    return;
                };
                let class = right.map_or(OTHER, |r| classify(r, self.source));
                match left.kind() {
                    "identifier" => flow.kill(self.text(left), class),
                    "matrix" | "multioutput_variable" => {
                        for id in identifiers_in(left) {
                            flow.kill(self.text(id), OTHER);
                        }
                    }
                    _ => {
                        if let Some(base) = base_identifier(left) {
                            flow.add(self.text(base), OTHER);
                        }
                    }
                }
            }
            "global_operator" | "persistent_operator" => {
                for id in identifiers_in(node) {
                    flow.kill(self.text(id), OTHER);
                }
            }
            "catch_clause" => {
                for i in 0..node.named_child_count() {
                    let Some(child) = node.named_child(i) else {
                        continue;
                    };
                    if child.kind() == "identifier" {
                        flow.kill(self.text(child), OTHER);
                    }
                }
                self.visit_children(node, flow);
            }
            "command" | "function_call" => {
                let name = node.named_child(0).map(|n| self.text(n));
                if name.is_some_and(|n| DYNAMIC_CALLS.contains(&n)) {
                    flow.opaque = true;
                }
                self.visit_children(node, flow);
            }
            "boolean_operator" => {
                if matches!(find_operator_text(node, self.source).as_str(), "&&" | "||") {
                    for i in 0..2 {
                        if let Some(operand) = node.named_child(i) {
                            self.boolean_use(operand, flow);
                        }
                    }
                }
                self.visit_children(node, flow);
            }
            "for_statement" => self.visit_for(node, flow),
            "while_statement" => self.visit_while(node, flow),
            "if_statement" => self.visit_if(node, flow),
            "switch_statement" => self.visit_switch(node, flow),
            "try_statement" => {
                let pre = flow.clone();
                let mut result = Flow {
                    tracked: flow.tracked.clone(),
                    ..Flow::default()
                };
                let mut tried = pre.clone();
                for i in 0..node.named_child_count() {
                    let Some(child) = node.named_child(i) else {
                        continue;
                    };
                    match child.kind() {
                        "catch_clause" => {
                            let mut caught = pre.clone();
                            caught.join(&tried);
                            self.visit(child, &mut caught);
                            result.join(&caught);
                        }
                        _ => self.visit(child, &mut tried),
                    }
                }
                result.join(&tried);
                *flow = result;
            }
            _ => self.visit_children(node, flow),
        }
    }

    fn visit_children(&mut self, node: Node, flow: &mut Flow) {
        for i in 0..node.named_child_count() {
            if let Some(child) = node.named_child(i) {
                self.visit(child, flow);
            }
        }
    }

    /// A condition of `if`, `elseif` or `while`: the root is a boolean use.
    fn condition(&mut self, cond: Node, flow: &Flow) {
        self.boolean_use(cond, flow);
        self.visit(cond, &mut flow.clone());
    }

    fn visit_if(&mut self, node: Node, flow: &mut Flow) {
        let pre = flow.clone();
        let mut out = Flow::default();
        let mut has_else = false;
        for i in 0..node.named_child_count() {
            let Some(child) = node.named_child(i) else {
                continue;
            };
            match child.kind() {
                "block" | "comment" => {}
                "elseif_clause" => {
                    if let Some(cond) = child.child_by_field_name("condition") {
                        self.condition(cond, &pre);
                    }
                    let mut branch = pre.clone();
                    self.visit(child, &mut branch);
                    out.join(&branch);
                }
                "else_clause" => {
                    has_else = true;
                    let mut branch = pre.clone();
                    self.visit(child, &mut branch);
                    out.join(&branch);
                }
                _ => self.condition(child, &pre),
            }
        }
        // The `if` body is the first block; visited from the state before the statement.
        if let Some(body) = child_of_kind(node, "block") {
            let mut branch = pre.clone();
            self.visit(body, &mut branch);
            out.join(&branch);
        }
        if !has_else {
            out.join(&pre);
        }
        *flow = out;
    }

    fn visit_switch(&mut self, node: Node, flow: &mut Flow) {
        let pre = flow.clone();
        let mut out = Flow::default();
        let mut has_otherwise = false;
        for i in 0..node.named_child_count() {
            let Some(child) = node.named_child(i) else {
                continue;
            };
            match child.kind() {
                "case_clause" | "otherwise_clause" => {
                    has_otherwise |= child.kind() == "otherwise_clause";
                    let mut branch = pre.clone();
                    self.visit(child, &mut branch);
                    out.join(&branch);
                }
                _ => self.visit(child, &mut pre.clone()),
            }
        }
        if !has_otherwise {
            out.join(&pre);
        }
        *flow = out;
    }

    fn visit_while(&mut self, node: Node, flow: &mut Flow) {
        let pre = flow.clone();
        let cond = condition_of(node);
        let body = child_of_kind(node, "block");
        let mut header = pre.clone();
        for _ in 0..MAX_LOOP_ITERATIONS {
            if let Some(cond) = cond {
                self.condition(cond, &header);
            }
            let mut inner = header.clone();
            if let Some(body) = body {
                self.visit(body, &mut inner);
            }
            let mut next = pre.clone();
            next.join(&inner);
            if next == header {
                break;
            }
            header = next;
        }
        header.opaque |= subtree_has_kind(node, &["break_statement", "continue_statement"]);
        *flow = header;
    }

    fn visit_for(&mut self, node: Node, flow: &mut Flow) {
        if let Some(iterator) = child_of_kind(node, "iterator") {
            self.visit(iterator, flow);
        }
        let var = child_of_kind(node, "iterator")
            .and_then(|it| it.named_child(0))
            .filter(|n| n.kind() == "identifier")
            .map(|n| self.text(n).to_string());
        let body = child_of_kind(node, "block");
        let pre = flow.clone();
        let mut header = pre.clone();
        for _ in 0..MAX_LOOP_ITERATIONS {
            let mut inner = header.clone();
            if let Some(var) = &var {
                inner.kill(var, OTHER);
            }
            if let Some(body) = body {
                self.visit(body, &mut inner);
            }
            let mut next = pre.clone();
            next.join(&inner);
            if next == header {
                break;
            }
            header = next;
        }
        header.opaque |= subtree_has_kind(node, &["break_statement", "continue_statement"]);
        *flow = header;
    }

    /// A boolean use: `operand` is a condition or an operand of `&&`/`||`.
    fn boolean_use(&mut self, operand: Node, flow: &Flow) {
        // BDLOG1: a transposed operand, possibly in parentheses.
        let mut value = operand;
        while value.kind() == "parenthesis" {
            match first_named_child(value) {
                Some(inner) => value = inner,
                None => break,
            }
        }
        if self.want_log1 && value.kind() == "postfix_operator" {
            self.log1(value);
        }
        self.arithmetic_condition(operand);

        // BDLGI: the variable itself, through parentheses and `~`.
        let mut root = operand;
        while matches!(root.kind(), "parenthesis" | "not_operator") {
            let Some(inner) = root.named_child(0) else {
                break;
            };
            root = inner;
        }
        if root.kind() == "identifier" {
            self.site(root, flow, "BDLGI");
        }

        // BDSCI: every variable reached through operators.
        let mut vars = Vec::new();
        collect_operand_vars(operand, &mut vars);
        for var in vars {
            self.site(var, flow, "BDSCI");
        }
    }

    /// Decide one identifier for BDLGI or BDSCI from the definitions reaching it.
    fn site(&mut self, ident: Node, flow: &Flow, check: &'static str) {
        let enabled = match check {
            "BDLGI" => self.want_lgi,
            _ => self.want_sci,
        };
        if !enabled {
            return;
        }
        let classes = flow.reaching(self.text(ident));
        let reported = !classes.is_empty()
            && classes.iter().all(|c| match check {
                "BDLGI" => c.nonlogical,
                _ => c.nonscalar,
            });
        let diag = reported.then(|| {
            let pos = ident.start_position();
            Diagnostic {
                rule_id: check,
                message: match check {
                    "BDLGI" => "Variable might be set by a nonlogical operator.".to_string(),
                    _ => "Variable might be set by a nonscalar operator.".to_string(),
                },
                severity: Severity::Warning,
                byte_range: ident.start_byte()..ident.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }
        });
        self.decided.insert((ident.start_byte(), check), diag);
    }

    /// An arithmetic operator at the root of a condition or `&&`/`||` operand, through
    /// parentheses and the elementwise `&`/`|`. Probed on R2026b: `+ - * / \\ ^` give BDLOG2
    /// at the operator (`if z + 1`, `if 2 * 3`, `if (z + 1) * 2` at the `*`), the elementwise
    /// `.* ./ .^ .\\` give BDLOG1, and a comparison, a call, an index, a literal, a variable
    /// or a unary operator gives neither.
    fn arithmetic_condition(&mut self, operand: Node) {
        let mut value = operand;
        while value.kind() == "parenthesis" {
            match first_named_child(value) {
                Some(inner) => value = inner,
                None => return,
            }
        }
        if value.kind() != "binary_operator" {
            return;
        }
        let Some(op) = operator_child(value) else {
            return;
        };
        let text = self.text(op).to_string();
        match text.as_str() {
            "&" | "|" => {
                for side in ["left", "right"] {
                    if let Some(inner) = value.child_by_field_name(side) {
                        self.arithmetic_condition(inner);
                    }
                }
            }
            "+" | "-" | "*" | "/" | "\\" | "^" if self.want_log2 => {
                self.operator_diagnostic(op, "BDLOG2");
            }
            ".*" | "./" | ".^" | ".\\" if self.want_log1 => {
                self.operator_diagnostic(op, "BDLOG1");
            }
            _ => {}
        }
    }

    fn operator_diagnostic(&mut self, op: Node, id: &'static str) {
        let pos = op.start_position();
        let message = if id == "BDLOG2" {
            "A scalar logical value is expected in the conditional expression. \
             Use 'any' or 'all' to reduce the array to a logical scalar, or \
             compare the scalar value to 0."
        } else {
            "A scalar logical value is expected in the conditional expression. \
             Use 'any' or 'all' to reduce the array to a logical scalar."
        };
        let diag = Diagnostic {
            rule_id: id,
            message: message.to_string(),
            severity: Severity::Warning,
            byte_range: op.start_byte()..op.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        };
        self.decided.insert((op.start_byte(), id), Some(diag));
    }

    /// BDLOG1 at the transpose operator of `value` (`a'`, `a.'`).
    fn log1(&mut self, value: Node) {
        let op_start = value
            .named_child(0)
            .map_or(value.start_byte(), |operand| operand.end_byte());
        let line_start = self.source[..op_start].rfind('\n').map_or(0, |i| i + 1);
        let diag = Diagnostic {
            rule_id: "BDLOG1",
            message: "A scalar logical value is expected in the conditional expression. \
                      Use 'any' or 'all' to reduce the array to a logical scalar."
                .to_string(),
            severity: Severity::Warning,
            byte_range: op_start..value.end_byte(),
            line: self.source[..op_start].matches('\n').count() + 1,
            column: op_start + 1 - line_start,
            fix: None,
        };
        self.decided.insert((op_start, "BDLOG1"), Some(diag));
    }
}

/// The operator token of a binary operator: the anonymous child between its operands.
fn operator_child(node: Node) -> Option<Node> {
    let mut cursor = node.walk();
    let found = node
        .children(&mut cursor)
        .find(|c| !c.is_named() && !c.is_extra());
    found
}

/// The identifiers reached from a condition or operand through operators, in
/// source order. Calls, indexing, fields, matrices and transposes stop the walk.
fn collect_operand_vars<'a>(node: Node<'a>, out: &mut Vec<Node<'a>>) {
    match node.kind() {
        "identifier" => out.push(node),
        "parenthesis"
        | "not_operator"
        | "unary_operator"
        | "binary_operator"
        | "comparison_operator"
        | "boolean_operator" => {
            for i in 0..node.named_child_count() {
                if let Some(child) = node.named_child(i) {
                    collect_operand_vars(child, out);
                }
            }
        }
        _ => {}
    }
}

fn child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    (0..node.named_child_count())
        .filter_map(|i| node.named_child(i))
        .find(|c| c.kind() == kind)
}

/// The condition of an `if`-like statement: its first named child that is not a
/// block, a comment or a clause.
fn condition_of(node: Node) -> Option<Node> {
    (0..node.named_child_count())
        .filter_map(|i| node.named_child(i))
        .find(|c| !matches!(c.kind(), "block" | "comment"))
}

/// The names that a boolean use can ask about: every identifier under the
/// condition of an `if`, `elseif`, `while` or `switch`, and under any `&&` or `||`
/// (a superset of [`FlowWalker::boolean_use`]'s sites). Only their definitions
/// need to be tracked.
fn boolean_identifiers(
    tree: &tree_sitter::Tree,
    source: &str,
) -> std::collections::HashSet<String> {
    let mut roots = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        match node.kind() {
            "if_statement" | "elseif_clause" | "while_statement" | "switch_statement" => {
                roots.extend(condition_of(node));
            }
            "boolean_operator" => roots.push(node),
            _ => {}
        }
        let mut cursor = node.walk();
        stack.extend(node.named_children(&mut cursor));
    }
    roots
        .into_iter()
        .flat_map(identifiers_in)
        .map(|id| source[id.byte_range()].to_string())
        .collect()
}

/// Every identifier below `node`, in source order.
fn identifiers_in(node: Node) -> Vec<Node> {
    let mut out = Vec::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if n.kind() == "identifier" {
            out.push(n);
            continue;
        }
        let mut cursor = n.walk();
        let children: Vec<Node> = n.named_children(&mut cursor).collect();
        stack.extend(children.into_iter().rev());
    }
    out
}

/// The variable an indexed or field target writes to (`x` in `x(2).a`).
fn base_identifier(node: Node) -> Option<Node> {
    let mut current = node;
    loop {
        if current.kind() == "identifier" {
            return Some(current);
        }
        current = current.named_child(0)?;
    }
}

fn subtree_has_kind(node: Node, kinds: &[&str]) -> bool {
    if kinds.contains(&node.kind()) {
        return true;
    }
    (0..node.named_child_count())
        .filter_map(|i| node.named_child(i))
        .any(|c| subtree_has_kind(c, kinds))
}

impl GoodPracticesEngine {
    /// BDLGI / BDLOG1 / BDLOG2 / BDSCI: logical-usage checks. BDLGI and BDSCI
    /// and BDLOG1 come from [`FlowWalker`]; BDLOG2 from the type environment.
    pub(crate) fn check_logical_usage(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        let any_enabled = self.is_check_enabled("BDLGI")
            || self.is_check_enabled("BDLOG1")
            || self.is_check_enabled("BDLOG2")
            || self.is_check_enabled("BDSCI");
        if !any_enabled {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();

        // ---- BDLGI, BDSCI, BDLOG1: flow-sensitive walk ----
        let mut walker = FlowWalker {
            source,
            want_lgi: self.is_check_enabled("BDLGI"),
            want_sci: self.is_check_enabled("BDSCI"),
            want_log1: self.is_check_enabled("BDLOG1"),
            want_log2: self.is_check_enabled("BDLOG2"),
            decided: BTreeMap::new(),
        };
        let mut root = Flow {
            tracked: Some(std::rc::Rc::new(boolean_identifiers(tree, source))),
            ..Flow::default()
        };
        walker.visit(tree.root_node(), &mut root);
        diagnostics.extend(walker.decided.into_values().flatten());

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn logical_ids(source: &str, check: &str) -> Vec<&'static str> {
        let tree = parse(source);
        let eng = engine();
        eng.check_logical_usage(&tree, source)
            .iter()
            .filter(|d| d.rule_id == check)
            .map(|d| d.rule_id)
            .collect()
    }

    /// (line, column) of each `check` diagnostic.
    fn sites(source: &str, check: &str) -> Vec<(usize, usize)> {
        let tree = parse(source);
        engine()
            .check_logical_usage(&tree, source)
            .iter()
            .filter(|d| d.rule_id == check)
            .map(|d| (d.line, d.column))
            .collect()
    }

    // -- BDLGI and BDSCI: MATLAB probes (gap_B) -------------------------------

    #[test]
    fn test_bdlgi_and_bdsci_fire_on_range_matrix_char_and_cell_conditions() {
        // Probed: `d = 1:3; if d`, `e = [1 0]; if e`, `e = 'abc'; if e`, `b = {1}; if b`,
        // `b = [] ; if b` all report both BDLGI and BDSCI at the variable.
        for value in ["1:3", "[1 0]", "'abc'", "{1}", "[]", "[true false]", "[1]"] {
            let source = format!("x = rand;\nd = {value};\nif d, disp(1), end\n");
            assert_eq!(sites(&source, "BDLGI"), vec![(3, 4)], "{value}");
            assert_eq!(sites(&source, "BDSCI"), vec![(3, 4)], "{value}");
        }
    }

    #[test]
    fn test_bdlgi_fires_without_bdsci_on_scalar_nonlogical_values() {
        // Probed: `e = 'a'` and `b = x'` (BDSCI only for the transpose of an
        // unknown value: both), `d = @sin` gives BDLGI only.
        let source = "x = rand;\ne = 'a';\nif e, disp(1), end\n";
        assert_eq!(sites(source, "BDLGI"), vec![(3, 4)]);
        assert!(sites(source, "BDSCI").is_empty());
        let source = "x = rand;\nd = @sin;\nif d, disp(1), end\n";
        assert_eq!(sites(source, "BDLGI"), vec![(3, 4)]);
        assert!(sites(source, "BDSCI").is_empty());
    }

    #[test]
    fn test_bdlgi_fires_on_matrix_products_and_element_wise_arithmetic() {
        // Probed: `b = x*2`, `2*x`, `x/2`, `x^2`, `x\2`, `2^x`, `-x`, `2*x+1`, `[x x]*2`
        // are BDLGI only; `x.*2`, `x.^2`, `x./2`, `x.*y` are BDLGI and BDSCI.
        for value in ["x*2", "2*x", "x/2", "x^2", "x\\2", "2^x", "-x", "[x x]*2"] {
            let source = format!("x = rand;\nb = {value};\nif b, disp(1), end\n");
            assert_eq!(sites(&source, "BDLGI"), vec![(3, 4)], "{value}");
            assert!(sites(&source, "BDSCI").is_empty(), "{value}");
        }
        for value in ["x.*2", "x.^2", "x./2"] {
            let source = format!("x = rand;\nb = {value};\nif b, disp(1), end\n");
            assert_eq!(sites(&source, "BDLGI"), vec![(3, 4)], "{value}");
            assert_eq!(sites(&source, "BDSCI"), vec![(3, 4)], "{value}");
        }
    }

    #[test]
    fn test_bdlgi_silent_on_addition_subtraction_and_constants_of_them() {
        // Probed: `b = x+1`, `x+y`, `x-y`, `2+3` are silent in `if b`.
        for value in ["x+1", "x+y", "x-y", "2+3", "x - 1", "2*x+1", "x + [1 2]"] {
            let source = format!("x = rand;\ny = rand;\nb = {value};\nif b, disp(1), end\n");
            assert!(sites(&source, "BDLGI").is_empty(), "{value}");
        }
    }

    #[test]
    fn test_bdlgi_and_bdsci_silent_on_unclassified_and_logical_values() {
        // Probed silent: numbers, logicals, comparisons, `zeros(1,3)`, `ones(2)`,
        // `rand(3)`, `true(2)`, `double(...)`, copies, indexing, parameters,
        // function results, double-quoted strings.
        let values = [
            "1",
            "true",
            "x > 0",
            "x & 1",
            "~[1 0]",
            "zeros(1,3)",
            "ones(2)",
            "rand(3)",
            "true(2)",
            "y",
            "x(1:2)",
            "rand",
            "\"abc\"",
            "int8(1)",
            "struct('a', 1)",
        ];
        for value in values {
            let source = format!("x = rand;\nd = {value};\nif d, disp(1), end\n");
            assert!(sites(&source, "BDLGI").is_empty(), "{value}");
            assert!(sites(&source, "BDSCI").is_empty(), "{value}");
        }
        // Probed: `d = 1:3; disp(d)` and `y = d + 1` are silent (not a condition).
        let source = "x = rand;\nd = 1:3;\ndisp(d);\ny = d + 1;\ny = d == 1;\n";
        assert!(sites(source, "BDLGI").is_empty());
        assert!(sites(source, "BDSCI").is_empty());
    }

    #[test]
    fn test_bdlgi_fires_on_condition_root_through_parentheses_and_not() {
        // Probed: `if (d)`, `if ~d`, `while d`, `elseif d`, `if x && d`, `y = x || d`,
        // `y = (d) && x`, `y = x > 0 && ~d` (BDLGI at the variable each time).
        for (body, column) in [
            ("if (d), disp(1), end", 5),
            ("if ~d, disp(1), end", 5),
            ("while d, break, end", 7),
            ("if x, disp(1), elseif d, disp(2), end", 23),
            ("if x && d, disp(1), end", 9),
            ("y = x || d;", 10),
            ("y = (d) && x;", 6),
            ("y = x > 0 && ~d;", 15),
        ] {
            let source = format!("x = rand;\nd = [1 0];\n{body}\n");
            let lgi = sites(&source, "BDLGI");
            assert_eq!(lgi.len(), 1, "{body}: {lgi:?}");
            assert_eq!(lgi[0].1, column, "{body}");
            assert_eq!(sites(&source, "BDSCI"), lgi, "{body}");
        }
    }

    #[test]
    fn test_bdsci_fires_on_variables_compared_or_summed_in_a_condition() {
        // Probed: `if d == 1`, `if d + 1`, `if d > 0`, `if x > 0 && d > 0` and
        // `elseif d == 2 && x` are BDSCI at the variable only (not BDLGI).
        for (body, column) in [
            ("if d == 1, disp(1), end", 4),
            ("if d + 1, disp(1), end", 4),
            ("if x > 0 && d > 0, disp(1), end", 13),
        ] {
            let source = format!("x = rand;\nd = 1:3;\n{body}\n");
            assert_eq!(sites(&source, "BDSCI"), vec![(3, column)], "{body}");
            assert!(sites(&source, "BDLGI").is_empty(), "{body}");
        }
    }

    #[test]
    fn test_bdlgi_silent_on_indexing_switch_and_function_arguments() {
        // Probed silent: `if d(1)`, `switch d`, `isempty(d)` and `disp(d)` arguments.
        let source = "x = rand;\nd = [1 0];\nif d(1), disp(1), end\n";
        assert!(sites(source, "BDLGI").is_empty());
        let source = "x = rand;\nd = [1 0];\nswitch d\n case 1\n disp(1)\nend\n";
        assert!(sites(source, "BDLGI").is_empty());
        let source = "x = rand;\nd = 1:3;\nif isempty(d) || x, disp(1), end\n";
        assert!(sites(source, "BDSCI").is_empty());
    }

    #[test]
    fn test_bdlgi_silent_for_a_copy_or_an_unknown_definition() {
        // Probed: `c = d; if c` is silent; so is any unclassified definition.
        let source = "x = rand;\nd = 1:3;\nc = d;\nif c, disp(1), end\n";
        assert!(sites(source, "BDLGI").is_empty());
        let source = "x = rand;\nd = 1:3;\nd = x;\nif d, disp(1), end\n";
        assert!(sites(source, "BDLGI").is_empty());
    }

    #[test]
    fn test_bdlgi_silent_when_a_reaching_definition_is_not_bad() {
        // Probed: the value of a name at a use is every definition that reaches it;
        // one good definition on any path keeps the check silent.
        for body in [
            "d = 1:3;\nd = 2;\nif d, disp(1), end",
            "d = 1:3;\nif x, d = 2; end\nif d, disp(1), end",
            "if x, d = 2; else, d = 1:3; end\nif d, disp(1), end",
            "if x, d = 1; else, d = 1:3; end\nif d, disp(1), end",
            "d = 1:3;\nfor k = 1:2\n d = 2;\nend\nif d, disp(1), end",
            "d = 1:3;\nswitch x\n case 1\n  d = 2;\n otherwise\n  d = 3;\nend\nif d, disp(1), end",
            "d = 1:3;\nif x, d = 2; elseif x > 1, d = 3; end\nif d, disp(1), end",
            "d = 1:3;\nd = d + 1;\nif d, disp(1), end",
            "d = 1:3;\nd(2) = 5;\nif d, disp(1), end",
            "d = 2;\nif x, d = [1 0]; end\nif d, disp(1), end",
            "d = 1:3;\nwhile d\n d = 0;\nend",
            "d = 1:3;\ntry\n d = 2;\ncatch\nend\nif d, disp(1), end",
            "d = 2;\nfor k = 1:2\n if d, disp(1), end\n d = 1:3;\nend",
            "d = 1:3;\nfor k = 1:2\n if x, d = 2; end\n if d, disp(1), end\nend",
        ] {
            let source = format!("x = rand;\n{body}\n");
            assert!(sites(&source, "BDLGI").is_empty(), "{body}");
            assert!(sites(&source, "BDSCI").is_empty(), "{body}");
        }
    }

    #[test]
    fn test_bdlgi_fires_on_every_path_definition_and_inside_loops() {
        // Probed: the nonlogical definition is the only one that reaches the use
        // (an `if` without `else` leaves the earlier value on the other path).
        for body in [
            "if x, d = 1:3; end\nif d, disp(1), end",
            "d = 1:3;\nif x, disp(1), end\nif d, disp(1), end",
            "d = 1:3;\nif x, d = 2; disp(1), end\nd = 1:3;\nif d, disp(1), end",
            "d = [1 0];\nwhile x\n if d, break, end\n x = 0;\nend",
            "d = 1:3;\nn = 0;\nwhile n < 2\n if d, disp(1), end\n n = n + 1;\nend",
            "d = [1 0];\nif x\n if d, disp(1), end\nend",
            "d = x * 2;\nd = x * 3;\nif d, disp(1), end",
        ] {
            let source = format!("x = rand;\n{body}\n");
            assert!(!sites(&source, "BDLGI").is_empty(), "{body}");
        }
    }

    #[test]
    fn test_bdlgi_fires_in_a_function_on_a_local_definition_only() {
        // Probed: a parameter is silent; a local `a = 1:3` or `b = a*2` is reported.
        let source = "function r = f(a)\nif a, r = 1; else, r = 0; end\nend\n";
        assert!(sites(source, "BDLGI").is_empty());
        let source = "function r = f(a)\nb = a * 2;\nif b, r = 1; else, r = 0; end\nend\n";
        assert_eq!(sites(source, "BDLGI"), vec![(3, 4)]);
        let source = "function r = f(a)\na = 1:3;\nif a, r = 1; else, r = 0; end\nend\n";
        assert_eq!(sites(source, "BDLGI"), vec![(3, 4)]);
        // Probed: a function result is silent.
        let source = "function r = f(a)\nb = g(a);\nif b, r = 1; else, r = 0; end\nend\n";
        assert!(sites(source, "BDLGI").is_empty());
    }

    #[test]
    fn test_bdlgi_silent_after_a_dynamic_call() {
        // A `clear` or `eval` may change any variable: the walker stays silent.
        let source = "x = rand;\nd = 1:3;\nclear x\nif d, disp(1), end\n";
        assert!(sites(source, "BDLGI").is_empty());
    }

    #[test]
    fn test_bdlgi_silent_for_unknown_var() {
        let source = "if x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDLGI").is_empty());
    }

    // -- BDLOG1 -------------------------------------------------------------

    #[test]
    fn test_bdlog1_silent_for_logical_variable_conditions() {
        // MATLAB probes: `a = [1 2 3] > 1; if a`, `c = [1 2] > 0; elseif c`,
        // `c = v > 0; while c`, `c = true(3); if c`, `c = ~v; if c` and
        // `c = v > 0 && n > 0; if c` report no BDLOG1.
        for source in [
            "a = [1 2 3] > 1;\nif a\n    y = 1;\nend\n",
            "c = [1 2] > 0;\nif n == 1\n    y = 1;\nelseif c\n    y = 2;\nend\n",
            "function f(v)\nc = v > 0;\nwhile c\n    break\nend\nend\n",
            "c = true(3);\nif c\n    y = 1;\nend\n",
            "function f(v)\nc = ~v;\nif c\n    y = 1;\nend\nend\n",
            "function f(v, n)\nc = v > 0 && n > 0;\nif c\n    y = 1;\nend\nend\n",
        ] {
            assert!(logical_ids(source, "BDLOG1").is_empty(), "{source:?}");
        }
    }

    #[test]
    fn test_bdlog1_silent_for_logical_scalar() {
        let source = "x = true;\nif x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDLOG1").is_empty());
    }

    #[test]
    fn test_bdlog1_fires_on_transposed_condition_at_the_operator() {
        // MATLAB probe on `a=1;\nif a', end; y = 2;`: BDLOG1 on line 2, column 5.
        let source = "a=1;\nif a', end; y = 2;\n";
        let diags = engine().check_logical_usage(&parse(source), source);
        let found: Vec<_> = diags.iter().filter(|d| d.rule_id == "BDLOG1").collect();
        assert_eq!(found.len(), 1, "got: {diags:?}");
        assert_eq!((found[0].line, found[0].column), (2, 5));
    }

    #[test]
    fn test_bdlog1_fires_on_transposed_condition_in_parentheses_and_while() {
        // MATLAB probes: `if (a')` gives column 6, `while a'` column 8, `if a.'` column 5.
        for (source, column) in [
            ("a=1;\nif (a')\nend\n", 6),
            ("a=1;\nwhile a'\nend\n", 8),
            ("a=[1 2];\nif a.'\nend\n", 5),
        ] {
            let diags = engine().check_logical_usage(&parse(source), source);
            let found: Vec<_> = diags.iter().filter(|d| d.rule_id == "BDLOG1").collect();
            assert_eq!(found.len(), 1, "{source:?} got: {diags:?}");
            assert_eq!((found[0].line, found[0].column), (2, column), "{source:?}");
        }
    }

    #[test]
    fn test_bdlog1_fires_on_transposed_boolean_operand() {
        // MATLAB probes: `z = x && (1:3)';` gives column 15, `z = x && [1 0]';`
        // and `y = [1 0]' && true;` (column 10) report BDLOG1 at the transpose.
        for (source, column) in [
            ("x = rand;\nz = x && (1:3)';\n", 15),
            ("x = rand;\nz = x && [1 0]';\n", 15),
            ("x = rand;\ny = [1 0]' && true;\n", 10),
        ] {
            let diags = engine().check_logical_usage(&parse(source), source);
            let found: Vec<_> = diags.iter().filter(|d| d.rule_id == "BDLOG1").collect();
            assert_eq!(found.len(), 1, "{source:?} got: {diags:?}");
            assert_eq!((found[0].line, found[0].column), (2, column), "{source:?}");
        }
    }

    #[test]
    fn test_bdlog1_silent_for_transpose_inside_a_comparison() {
        // MATLAB probes: `if a' == 1` and `x = a';` report no BDLOG1.
        for source in ["a=1;\nif a' == 1\nend\n", "a=1;\nx = a';\n"] {
            let diags = engine().check_logical_usage(&parse(source), source);
            assert!(
                !diags.iter().any(|d| d.rule_id == "BDLOG1"),
                "{source:?} {diags:?}"
            );
        }
    }

    #[test]
    fn test_bdlog1_silent_for_numeric_condition() {
        let source = "x = 5;\nif x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDLOG1").is_empty());
    }

    // -- BDLOG2 -------------------------------------------------------------

    #[test]
    fn test_bdlog2_silent_for_numeric_variable_and_literal_conditions() {
        // Probed on R2026b (-allmsg): `x = 5; if x`, `if 1`, `if 5`, `t = true; if t` and
        // `while k` give no BDLOG2.
        for source in [
            "x = 5;\nif x\n    y = 1;\nend\n",
            "if 1\n    y = 1;\nend\n",
            "if 5\n    y = 1;\nend\n",
            "t = true;\nif t\n    y = 1;\nend\n",
        ] {
            assert!(logical_ids(source, "BDLOG2").is_empty(), "{source:?}");
        }
    }

    #[test]
    fn test_bdlog2_fires_on_an_arithmetic_condition_at_the_operator() {
        // Probed: `if z + 1` (column 6), `if z * 2`, `if z / 2`, `if z \\ 2`, `if z ^ 2`,
        // `if 2 * 3`, `if 1 + 1`, `if z + a`, `if (z + 1)` (column 7) and `if (z + 1) * 2`
        // (at the `*`, column 12).
        for (source, column) in [
            ("function f(z, a)\nif z + 1\nend\nend\n", 6),
            ("function f(z, a)\nif z * 2\nend\nend\n", 6),
            ("function f(z, a)\nif z / 2\nend\nend\n", 6),
            ("function f(z, a)\nif z \\ 2\nend\nend\n", 6),
            ("function f(z, a)\nif z ^ 2\nend\nend\n", 6),
            ("function f(z, a)\nif 2 * 3\nend\nend\n", 6),
            ("function f(z, a)\nif z + a\nend\nend\n", 6),
            ("function f(z, a)\nif (z + 1)\nend\nend\n", 7),
            ("function f(z, a)\nif (z + 1) * 2\nend\nend\n", 12),
            ("function f(z, a)\nif 5 * 5 + z\nend\nend\n", 10),
            ("function f(z, a)\nif z + 1 && z\nend\nend\n", 6),
            ("function f(z, a)\nif z + 1 | z\nend\nend\n", 6),
        ] {
            assert_eq!(sites(source, "BDLOG2"), vec![(2, column)], "{source:?}");
        }
    }

    #[test]
    fn test_bdlog2_silent_for_other_condition_shapes() {
        // Probed: unary minus, calls, indexing, comparisons, `~z` and `z & 1` give no BDLOG2.
        for cond in [
            "-z",
            "numel(a)",
            "a(1)",
            "z == 1",
            "z + 1 > 0",
            "abs(z)",
            "~z",
            "z & 1",
        ] {
            let source = format!("function f(z, a)\nif {cond}\nend\nend\n");
            assert!(logical_ids(&source, "BDLOG2").is_empty(), "{cond}");
        }
    }

    #[test]
    fn test_bdlog1_fires_on_an_elementwise_condition_at_the_operator() {
        // Probed: `if z .* 2` and `if z ./ 2` (column 6) and `if z .^ 2` give BDLOG1.
        for cond in ["z .* 2", "z ./ 2", "z .^ 2"] {
            let source = format!("function f(z)\nif {cond}\nend\nend\n");
            assert_eq!(sites(&source, "BDLOG1"), vec![(2, 6)], "{cond}");
        }
    }

    #[test]
    fn test_bdlog2_silent_for_logical_condition() {
        let source = "x = true;\nif x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDLOG2").is_empty());
    }

    #[test]
    fn test_bdlog2_silent_for_non_scalar_condition() {
        let source = "x = [1 2 3];\nif x\n    y = 1;\nend\n";
        assert!(logical_ids(source, "BDLOG2").is_empty());
    }

    // -- config -------------------------------------------------------------

    #[test]
    fn test_logical_usage_checks_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec![
                    "BDLGI".to_string(),
                    "BDLOG1".to_string(),
                    "BDLOG2".to_string(),
                    "BDSCI".to_string(),
                ],
            },
        };
        let source = "x = a + b;\nif x\n    y = 1;\nend\nx = a > b;\nif x\n    y = 1;\nend\nx = 1:10;\nif x\n    y = 1;\nend\n";
        let tree = parse(source);
        assert!(eng.check_logical_usage(&tree, source).is_empty());
    }
}
