//! # SUGGESTED_IMPROVEMENTS: Suggested Improvements
//!
//! ```malt
//! id = "SUGGESTED_IMPROVEMENTS"
//! title = "Suggested Improvements"
//! category = "suggested-improvements"
//! severity = "info"
//! fix = false
//! icon = "lucide/lightbulb"
//! slug = "suggested-improvements"
//! data_file = "suggested_improvements.toml"
//! ```
//!
//! ## Rule
//!
//! A data-driven rule module that handles 243 MATLAB Code Analyzer checks
//! about functions and patterns that are not recommended, pairing each with a
//! modern replacement suggestion. Instead of implementing one struct per
//! check, a single `SuggestedImprovementsEngine` loads
//! `data/suggested_improvements.toml` at compile time, parses it into a
//! function-name lookup table, and emits diagnostics with the specific check
//! ID from the matched entry (e.g. `CSVRD`, not the meta ID
//! `SUGGESTED_IMPROVEMENTS`).
//!
//! On each `function_call` or `command` node the engine extracts the function
//! name and performs an O(1) HashMap lookup, reporting the first matching
//! entry for that name (some names map to several check IDs, e.g.
//! `maketform` → MTFA1, MTFA2, MTFP1, MTFP2, MTFB). Entries with an empty
//! `function_name` are "generic" checks that cannot be matched by name alone
//! (they require AST pattern matching) and are skipped during the
//! lookup-based check.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! data = csvread('data.csv');   % CSVRD — use readmatrix instead
//! if isdir(folder)              % ISDIR — use isfolder instead
//!     disp('folder exists');
//! end
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! data = readmatrix('data.csv');
//! if isfolder(folder)
//!     disp('folder exists');
//! end
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.categories]
//! suggested-improvements = "info"
//!
//! [lint.rules]
//! SUGGESTED_IMPROVEMENTS = "off"
//! ```

use std::collections::HashMap;
use std::sync::LazyLock;

use malt_core::{Category, Config, Diagnostic, NodeContext, Rule, Severity};
use serde::Deserialize;

use crate::analysis::symbols::{is_field_name, ScopeKind, SymbolTable};

// ---------------------------------------------------------------------------
// Data types for TOML deserialization
// ---------------------------------------------------------------------------

/// A single suggested improvement check entry from the data file.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct SuggestedEntry {
    /// Check ID matching MATLAB Code Analyzer (e.g., "CSVRD").
    id: String,
    /// The function/class name to match against. Empty string means generic.
    function_name: String,
    /// Human-readable diagnostic message.
    message: String,
    /// Recommended replacement function or syntax.
    replacement: String,
    /// Non-empty for checks that need class tracking or a toolbox that is not
    /// installed. Such entries are not matched by name.
    #[serde(default)]
    pattern: String,
    /// The usage is only discouraged with at least this many arguments
    /// (`eval(s, catch_s)`). 0 means any call.
    #[serde(default)]
    min_args: usize,
    /// The usage is only discouraged with at most this many arguments
    /// (`format` with none: `max_args = 0`). Unset means no limit.
    #[serde(default)]
    max_args: Option<usize>,
    /// The usage is only discouraged when one argument is one of these literals
    /// (`rand('seed', 1)`, `hold all`), compared case-insensitively. Empty means
    /// any call.
    #[serde(default)]
    literal_args: Vec<String>,
    /// 1-based position of the argument that `arg_kinds`, `not_arg_kinds` and
    /// `arg_text` test. 0 means the first argument.
    #[serde(default)]
    arg_at: usize,
    /// The argument at `arg_at` is of one of these kinds: `number`, `text` (a
    /// quoted string, either quote), `char` (single quotes) or `cell`.
    #[serde(default)]
    arg_kinds: Vec<String>,
    /// The argument at `arg_at` is of none of these kinds (`import('a.b')` is
    /// fine, `import(x)` is not).
    #[serde(default)]
    not_arg_kinds: Vec<String>,
    /// The argument at `arg_at` has exactly this source text (`now`).
    #[serde(default)]
    arg_text: String,
    /// Not when the statement or `if` header that holds the call also calls this
    /// function (`if iscellstr(x) || isstring(x)`).
    #[serde(default)]
    not_with_call: String,
    /// The argument at `arg_at`, when it is a variable, takes the kinds that
    /// every assignment to it in its scope gives (`x = 5; year(x)`). A
    /// parameter or a value that is not a literal leaves the kind unknown.
    #[serde(default)]
    arg_through_variables: bool,
    /// The argument at `arg_at` is a dotted release number at least this one
    /// (`verLessThan('matlab', '9.10')` with `9.9`). Other text, such as
    /// `'R2021a'`, does not match.
    #[serde(default)]
    version_at_least: String,
    /// Only inside a `parfor` loop.
    #[serde(default)]
    within_parfor: bool,
    /// Not when the name is an argument of a call to this function that also has
    /// `not_in_call_literal` among its literal arguments.
    #[serde(default)]
    not_in_call: String,
    /// See `not_in_call`.
    #[serde(default)]
    not_in_call_literal: String,
}

impl SuggestedEntry {
    /// Whether the use `node`, with its arguments `args`, is the discouraged
    /// usage, not just a use of the function. Argument kinds and text are only
    /// known for function syntax; command syntax never matches those conditions.
    fn applies_to(
        &self,
        node: tree_sitter::Node,
        args: &[tree_sitter::Node],
        source: &str,
    ) -> bool {
        if args.len() < self.min_args || self.max_args.is_some_and(|max| args.len() > max) {
            return false;
        }
        if self.within_parfor && !inside_parfor(node, source) {
            return false;
        }
        if !self.not_with_call.is_empty() && statement_calls(node, source, &self.not_with_call) {
            return false;
        }
        if !self.not_in_call.is_empty()
            && in_call_with_literal(node, source, &self.not_in_call, &self.not_in_call_literal)
        {
            return false;
        }
        if !self.literal_args.is_empty() {
            let matched = args.iter().any(|a| {
                literal_text(*a, source)
                    .is_some_and(|t| self.literal_args.iter().any(|l| l.eq_ignore_ascii_case(t)))
            });
            if !matched {
                return false;
            }
        }
        if !self.version_at_least.is_empty() {
            if node.kind() == "command" {
                return false;
            }
            let Some(arg) = args.get(self.arg_at.max(1) - 1) else {
                return false;
            };
            let version = literal_text(*arg, source).unwrap_or_default();
            if !release_at_least(version, &self.version_at_least) {
                return false;
            }
        }
        let has_kind_condition = !self.arg_kinds.is_empty()
            || !self.not_arg_kinds.is_empty()
            || !self.arg_text.is_empty();
        if !has_kind_condition {
            return true;
        }
        if node.kind() == "command" {
            return false;
        }
        let Some(arg) = args.get(self.arg_at.max(1) - 1) else {
            return false;
        };
        let kinds = if self.arg_through_variables && arg.kind() == "identifier" {
            variable_kinds(*arg, source)
        } else {
            argument_kinds(*arg, source).to_vec()
        };
        if !self.arg_kinds.is_empty()
            && !kinds.iter().any(|k| self.arg_kinds.iter().any(|w| w == k))
        {
            return false;
        }
        if kinds
            .iter()
            .any(|k| self.not_arg_kinds.iter().any(|w| w == k))
        {
            return false;
        }
        self.arg_text.is_empty() || source[arg.byte_range()].trim() == self.arg_text
    }
}

/// The kinds of a function-syntax argument, for the `arg_kinds` conditions.
pub(crate) fn argument_kinds(arg: tree_sitter::Node, source: &str) -> &'static [&'static str] {
    match arg.kind() {
        "number" => &["number"],
        "cell" if is_char_literal_cell(arg, source) => &["cell", "char_cell"],
        "cell" => &["cell"],
        "function_call" if call_name_is(arg, source, "cell") => &["cell"],
        "string" if source[arg.byte_range()].starts_with('\'') => &["char", "text"],
        "string" => &["text"],
        "range" | "matrix" if is_number_expr(arg) => &["number"],
        _ => &[],
    }
}

/// Whether a cell literal holds one or more single-quoted character literals and
/// nothing else (`{'a', 'b'}`, `{'a'; 'b'}`). A cs-list, a variable, a
/// double-quoted string or a transposed cell is not.
fn is_char_literal_cell(cell: tree_sitter::Node, source: &str) -> bool {
    let mut count = 0usize;
    char_literals_only(cell, source, &mut count) && count > 0
}

/// Counts the single-quoted literals under `node`, the elements of a cell
/// literal, descending into its rows; false at any other element.
fn char_literals_only(node: tree_sitter::Node, source: &str, count: &mut usize) -> bool {
    for i in 0..node.named_child_count() {
        let Some(child) = node.named_child(i) else {
            continue;
        };
        match child.kind() {
            "comment" => {}
            "string" if source[child.byte_range()].starts_with('\'') => *count += 1,
            "row" | "array" => {
                if !char_literals_only(child, source, count) {
                    return false;
                }
            }
            _ => return false,
        }
    }
    true
}

/// Whether the function call `node` calls `name`.
fn call_name_is(node: tree_sitter::Node, source: &str, name: &str) -> bool {
    node.child_by_field_name("name")
        .is_some_and(|callee| &source[callee.byte_range()] == name)
}

/// Whether the statement that holds `node` also calls `name`. For a call in an
/// `if` header (or an `elseif` condition) the header is every condition of that
/// `if`, not its bodies: `if iscellstr(x) || isstring(x)` holds `isstring`.
/// Otherwise the whole top-level statement counts.
fn statement_calls(node: tree_sitter::Node, source: &str, name: &str) -> bool {
    let mut current = node;
    let group = loop {
        let Some(parent) = current.parent() else {
            return false;
        };
        match parent.kind() {
            "block" | "source_file" => break current,
            "if_statement" => break parent,
            _ => current = parent,
        }
    };
    calls_named(group, source, name, group.kind() == "if_statement")
}

/// Whether `node` or a descendant calls `name`; with `skip_blocks`, the bodies
/// (`block` nodes) are not searched.
fn calls_named(node: tree_sitter::Node, source: &str, name: &str, skip_blocks: bool) -> bool {
    if node.kind() == "function_call" && call_name_is(node, source, name) {
        return true;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if !(skip_blocks && child.kind() == "block")
            && calls_named(child, source, name, skip_blocks)
        {
            return true;
        }
    }
    false
}

/// Whether `node` is a range or a matrix of number literals (`1:3`, `[1 2]`).
fn is_number_expr(node: tree_sitter::Node) -> bool {
    match node.kind() {
        "number" => true,
        "range" | "matrix" | "row" => {
            let mut any = false;
            for i in 0..node.named_child_count() {
                let Some(child) = node.named_child(i) else {
                    continue;
                };
                if child.kind() == "comment" {
                    continue;
                }
                if !is_number_expr(child) {
                    return false;
                }
                any = true;
            }
            any
        }
        _ => false,
    }
}

/// The kinds of the values that the variable `ident` is given in its scope, for
/// `arg_through_variables` (`x = 5; year(x)`). Every assignment must be a
/// literal of a kind. A parameter, a loop variable, a multiple assignment or any
/// other value leaves the kind unknown, and then the call is not reported.
pub(crate) fn variable_kinds(ident: tree_sitter::Node, source: &str) -> Vec<&'static str> {
    let name = &source[ident.byte_range()];
    let scope = enclosing_function(ident).unwrap_or_else(|| root_of(ident));
    let mut kinds = Vec::new();
    if collect_assigned_kinds(scope, name, source, &mut kinds) {
        kinds
    } else {
        Vec::new()
    }
}

/// The innermost function definition that contains `node`.
fn enclosing_function(node: tree_sitter::Node) -> Option<tree_sitter::Node> {
    let mut current = node.parent();
    while let Some(ancestor) = current {
        if ancestor.kind() == "function_definition" {
            return Some(ancestor);
        }
        current = ancestor.parent();
    }
    None
}

/// Adds to `kinds` the kinds of every assignment to `name` under `scope`, not
/// counting nested functions. Returns false when an assignment is not a literal
/// of a kind, or when `name` is bound another way (a loop, `[a, name] = f()`).
fn collect_assigned_kinds(
    scope: tree_sitter::Node,
    name: &str,
    source: &str,
    kinds: &mut Vec<&'static str>,
) -> bool {
    for i in 0..scope.child_count() {
        let Some(child) = scope.child(i) else {
            continue;
        };
        match child.kind() {
            "function_definition" => continue,
            "assignment" => {
                let (Some(left), Some(right)) = (
                    child.child_by_field_name("left"),
                    child.child_by_field_name("right"),
                ) else {
                    return false;
                };
                if left.kind() == "identifier" && &source[left.byte_range()] == name {
                    let assigned = argument_kinds(right, source);
                    if assigned.is_empty() {
                        return false;
                    }
                    for &kind in assigned {
                        if !kinds.contains(&kind) {
                            kinds.push(kind);
                        }
                    }
                } else if mentions(left, name, source) {
                    return false;
                }
            }
            "for_statement" => {
                let var = (0..child.named_child_count())
                    .filter_map(|j| child.named_child(j))
                    .find(|c| c.kind() == "identifier");
                if var.is_some_and(|v| &source[v.byte_range()] == name) {
                    return false;
                }
            }
            _ => {}
        }
        if !collect_assigned_kinds(child, name, source, kinds) {
            return false;
        }
    }
    true
}

/// Whether the identifier `name` occurs anywhere in `node`.
fn mentions(node: tree_sitter::Node, name: &str, source: &str) -> bool {
    if node.kind() == "identifier" && &source[node.byte_range()] == name {
        return true;
    }
    (0..node.named_child_count())
        .filter_map(|i| node.named_child(i))
        .any(|c| mentions(c, name, source))
}

/// The text of a literal argument, without its quotes. Command-syntax arguments
/// are literal text too.
fn literal_text<'a>(arg: tree_sitter::Node, source: &'a str) -> Option<&'a str> {
    match arg.kind() {
        "string" | "command_argument" => {
            Some(source[arg.byte_range()].trim_matches(|c| c == '\'' || c == '"'))
        }
        _ => None,
    }
}

/// Whether `text` is a dotted release number (`9.10`, `24.1`) at least `min`
/// (`9.9`), compared part by part with missing parts as 0. Other text, such as
/// `R2021a`, is not a release number and does not match.
fn release_at_least(text: &str, min: &str) -> bool {
    let number = |s: &str| -> Option<Vec<u64>> {
        s.split('.')
            .map(|p| {
                (!p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
                    .then(|| p.parse::<u64>().ok())
                    .flatten()
            })
            .collect()
    };
    let (Some(v), Some(m)) = (number(text), number(min)) else {
        return false;
    };
    let len = v.len().max(m.len());
    let padded = |xs: &[u64]| {
        (0..len)
            .map(|i| xs.get(i).copied().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    padded(&v) >= padded(&m)
}

/// Whether `node` lies inside a `parfor` loop.
fn inside_parfor(node: tree_sitter::Node, source: &str) -> bool {
    let mut current = node.parent();
    while let Some(ancestor) = current {
        if ancestor.kind() == "for_statement" && source[ancestor.byte_range()].starts_with("parfor")
        {
            return true;
        }
        current = ancestor.parent();
    }
    false
}

/// Whether `node` is an argument of a call to `name` that also has `literal`
/// among its literal arguments.
fn in_call_with_literal(node: tree_sitter::Node, source: &str, name: &str, literal: &str) -> bool {
    let Some(arguments) = node.parent().filter(|p| p.kind() == "arguments") else {
        return false;
    };
    let Some(call) = arguments.parent().filter(|p| p.kind() == "function_call") else {
        return false;
    };
    let Some(call_name) = call.child_by_field_name("name") else {
        return false;
    };
    source[call_name.byte_range()] == *name
        && call_args(call)
            .iter()
            .any(|a| literal_text(*a, source).is_some_and(|t| t.eq_ignore_ascii_case(literal)))
}

/// The arguments of a `function_call` or `command` node, without comments.
fn call_args(node: tree_sitter::Node) -> Vec<tree_sitter::Node> {
    let children: Vec<_> = (0..node.child_count())
        .filter_map(|i| node.child(i))
        .collect();
    match node.kind() {
        "function_call" => children
            .iter()
            .find(|c| c.kind() == "arguments")
            .map(|args| {
                (0..args.named_child_count())
                    .filter_map(|i| args.named_child(i))
                    .filter(|c| c.kind() != "comment")
                    .collect()
            })
            .unwrap_or_default(),
        "command" => children
            .into_iter()
            .filter(|c| c.kind() == "command_argument")
            .collect(),
        _ => Vec::new(),
    }
}

/// Top-level structure of the suggested_improvements.toml data file.
#[derive(Debug, Deserialize)]
struct SuggestedData {
    checks: Vec<SuggestedEntry>,
}

// ---------------------------------------------------------------------------
// Static data — parsed once, lives for 'static
// ---------------------------------------------------------------------------

/// Raw TOML source embedded at compile time.
const SUGGESTED_TOML_SOURCE: &str = include_str!("data/suggested_improvements.toml");

/// Parsed suggested improvements data. All strings inside are owned and live
/// for 'static because this is a `LazyLock` static.
static SUGGESTED_DATA: LazyLock<SuggestedData> = LazyLock::new(|| {
    toml::from_str(SUGGESTED_TOML_SOURCE).expect("failed to parse data/suggested_improvements.toml")
});

/// Lookup table: function_name → the entries for that name.
///
/// Only entries with a non-empty `function_name` and no `pattern` are included.
/// A name can map to several check IDs (e.g., "maketform" maps to MTFA1, MTFA2,
/// MTFP1, MTFP2, MTFB, and "get" to GETFSP and VIDREAD); each entry whose
/// conditions hold is reported.
static SUGGESTED_TABLE: LazyLock<HashMap<&'static str, Vec<&'static SuggestedEntry>>> =
    LazyLock::new(|| {
        let data = &*SUGGESTED_DATA;
        let mut map: HashMap<&'static str, Vec<&'static SuggestedEntry>> = HashMap::new();
        for entry in &data.checks {
            if !entry.function_name.is_empty() && entry.pattern.is_empty() {
                map.entry(entry.function_name.as_str())
                    .or_default()
                    .push(entry);
            }
        }
        map
    });

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// The suggested improvements engine — a single rule instance that checks all
/// not-recommended function usage via a data-driven lookup table.
///
/// This is registered once with inventory. Each emitted diagnostic carries the
/// specific check ID from the data file (e.g., "CSVRD"), not a generic ID.
pub struct SuggestedImprovementsEngine;

/// Target node types for the suggested improvements engine. A bare name
/// (`clock`, `format`, `@nanmean`) is a use with no arguments.
const TARGET_NODES: &[&str] = &["function_call", "command", "identifier"];

impl SuggestedImprovementsEngine {
    /// Factory constructor called by the rule registry.
    pub fn from_config(_config: &Config) -> Box<dyn Rule> {
        // Force initialization of static data at construction time so that any
        // parse errors surface early rather than on first lint invocation.
        let _ = &*SUGGESTED_TABLE;
        Box::new(Self)
    }
}

impl Rule for SuggestedImprovementsEngine {
    fn id(&self) -> &'static str {
        "SUGGESTED_IMPROVEMENTS"
    }

    fn description(&self) -> &'static str {
        "Not-recommended function or pattern with suggested replacement"
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn category(&self) -> Category {
        Category::SuggestedImprovements
    }

    fn enabled_by_default(&self) -> bool {
        // MATLAB's Code Analyzer reports these by default; the `recommended`
        // preset turns the whole category off and `all` keeps it on.
        true
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        let node = ctx.node;
        // A name used as a call (`f(x)`) or field (`s.clock`) is handled by its
        // function_call node, not as a bare name.
        if node.kind() == "identifier"
            && (node.parent().is_some_and(|p| p.kind() == "function_call") || is_field_name(node))
        {
            return Vec::new();
        }

        let func_name = match extract_func_name(node, ctx.source) {
            Some(name) => name,
            None => return Vec::new(),
        };
        let Some(entries) = SUGGESTED_TABLE.get(func_name) else {
            return Vec::new();
        };

        let args = call_args(node);
        let applicable: Vec<&SuggestedEntry> = entries
            .iter()
            .copied()
            .filter(|entry| entry.applies_to(node, &args, ctx.source))
            .collect();
        if applicable.is_empty() || is_shadowed(node, func_name, ctx.source) {
            return Vec::new();
        }

        let start = node.start_position();
        applicable
            .into_iter()
            .map(|entry| Diagnostic {
                // The entry's `id` is a &'static str from SUGGESTED_DATA, a
                // LazyLock static that is never deallocated.
                rule_id: entry.id.as_str(),
                message: entry.message.clone(),
                severity: Severity::Info,
                byte_range: node.start_byte()..node.end_byte(),
                line: start.row + 1,
                column: start.column + 1,
                fix: None,
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the function/command name from a `function_call` or `command` node.
///
/// For `function_call`: extracts the `name` field (handles simple identifiers
/// and dotted names like `sigwin.hamming`).
///
/// For `command`: extracts the first child (the command name).
fn extract_func_name<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    match node.kind() {
        "function_call" => {
            let name_node = node.child_by_field_name("name")?;
            // `sigwin.hamming(n)`: the call is the trailing member of a dotted
            // expression, so match the whole package path. A variable in front
            // (`s.isdir(x)`) yields a path that matches no entry.
            let start = match node.parent() {
                Some(parent) if crate::analysis::symbols::is_qualified_call(node) => {
                    parent.named_child(0)?.start_byte()
                }
                _ => name_node.start_byte(),
            };
            Some(&source[start..name_node.end_byte()])
        }
        "identifier" => Some(&source[node.start_byte()..node.end_byte()]),
        "command" => {
            let name_node = node.child(0)?;
            if name_node.kind() == "command_name" {
                Some(&source[name_node.start_byte()..name_node.end_byte()])
            } else {
                None
            }
        }
        _ => None,
    }
}

/// The root node of the tree that `node` belongs to.
fn root_of(node: tree_sitter::Node) -> tree_sitter::Node {
    let mut root = node;
    while let Some(parent) = root.parent() {
        root = parent;
    }
    root
}

/// Whether a name is shadowed where `node` uses it: a variable of that name in
/// its scope, or a local function. MATLAB then uses that, not the function.
/// Dotted names are never shadowed.
fn is_shadowed(node: tree_sitter::Node, name: &str, source: &str) -> bool {
    if name.contains('.') {
        return false;
    }
    let table = SymbolTable::build_from_root(root_of(node), source);
    // A script variable also shadows the function inside an anonymous function
    // (`f = @() isdir(p)`), which has a scope of its own.
    table.root_scope().is_defined(name)
        || table
            .scope_at(node.start_byte())
            .is_some_and(|scope| scope.is_defined(name))
        || table.function_scopes().iter().any(|scope| {
            matches!(scope.kind, ScopeKind::Function | ScopeKind::LocalFunction)
                && scope.name == name
        })
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "SUGGESTED_IMPROVEMENTS",
    SuggestedImprovementsEngine::from_config
));

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        SuggestedImprovementsEngine::from_config(&Config::default())
    }

    // -- CSV-style file IO ---------------------------------------------------

    #[test]
    fn csvread_fires_csvrd() {
        let diags = lint_nodes(&*engine(), "data = csvread('data.csv');\n");
        assert!(has_id(&diags, "CSVRD"), "got: {diags:?}");
    }

    #[test]
    fn csvwrite_fires_csvwt() {
        let diags = lint_nodes(&*engine(), "csvwrite('out.csv', data);\n");
        assert!(has_id(&diags, "CSVWT"), "got: {diags:?}");
    }

    #[test]
    fn readmatrix_does_not_fire() {
        let diags = lint_nodes(&*engine(), "data = readmatrix('data.csv');\n");
        assert!(!has_id(&diags, "CSVRD"), "got: {diags:?}");
        assert!(!has_id(&diags, "DLMRD"), "got: {diags:?}");
    }

    // -- Excel IO ------------------------------------------------------------

    #[test]
    fn xlsread_fires_xlsrd() {
        let diags = lint_nodes(&*engine(), "data = xlsread('book.xlsx');\n");
        assert!(has_id(&diags, "XLSRD"), "got: {diags:?}");
    }

    #[test]
    fn xlswrite_fires_xlswt() {
        let diags = lint_nodes(&*engine(), "xlswrite('out.xlsx', data);\n");
        assert!(has_id(&diags, "XLSWT"), "got: {diags:?}");
    }

    #[test]
    fn readtable_does_not_fire() {
        let diags = lint_nodes(&*engine(), "data = readtable('book.xlsx');\n");
        assert!(!has_id(&diags, "XLSRD"), "got: {diags:?}");
    }

    // -- dlmread / dlmwrite --------------------------------------------------

    #[test]
    fn dlmread_fires_dlmrd() {
        let diags = lint_nodes(&*engine(), "data = dlmread('data.txt');\n");
        assert!(has_id(&diags, "DLMRD"), "got: {diags:?}");
    }

    #[test]
    fn dlmwrite_fires_dlmwt() {
        let diags = lint_nodes(&*engine(), "dlmwrite('out.txt', data);\n");
        assert!(has_id(&diags, "DLMWT"), "got: {diags:?}");
    }

    #[test]
    fn writematrix_does_not_fire() {
        let diags = lint_nodes(&*engine(), "writematrix(data, 'out.txt');\n");
        assert!(!has_id(&diags, "DLMWT"), "got: {diags:?}");
    }

    // -- Statistics replacements ----------------------------------------------

    #[test]
    fn hist_fires_hist() {
        let diags = lint_nodes(&*engine(), "hist(x, 20);\n");
        assert!(has_id(&diags, "HIST"), "got: {diags:?}");
    }

    #[test]
    fn histc_fires_histc() {
        let diags = lint_nodes(&*engine(), "n = histc(x, edges);\n");
        assert!(has_id(&diags, "HISTC"), "got: {diags:?}");
    }

    #[test]
    fn histogram_does_not_fire() {
        let diags = lint_nodes(&*engine(), "histogram(x, 20);\n");
        assert!(!has_id(&diags, "HIST"), "got: {diags:?}");
    }

    #[test]
    fn histcounts_does_not_fire() {
        let diags = lint_nodes(&*engine(), "n = histcounts(x, edges);\n");
        assert!(!has_id(&diags, "HISTC"), "got: {diags:?}");
    }

    // -- NaN-aware statistics --------------------------------------------------

    #[test]
    fn nanmean_fires_nanmean() {
        let diags = lint_nodes(&*engine(), "m = nanmean(x);\n");
        assert!(has_id(&diags, "NANMEAN"), "got: {diags:?}");
    }

    #[test]
    fn nansum_fires_nansum() {
        let diags = lint_nodes(&*engine(), "s = nansum(x);\n");
        assert!(has_id(&diags, "NANSUM"), "got: {diags:?}");
    }

    #[test]
    fn plain_mean_does_not_fire() {
        let diags = lint_nodes(&*engine(), "m = mean(x);\n");
        assert!(!has_id(&diags, "NANMEAN"), "got: {diags:?}");
    }

    // -- Filesystem -----------------------------------------------------------

    #[test]
    fn isdir_fires_isdir() {
        let diags = lint_nodes(&*engine(), "if isdir(p), disp('yes'); end\n");
        assert!(has_id(&diags, "ISDIR"), "got: {diags:?}");
    }

    #[test]
    fn isfolder_does_not_fire() {
        let diags = lint_nodes(&*engine(), "if isfolder(p), disp('yes'); end\n");
        assert!(!has_id(&diags, "ISDIR"), "got: {diags:?}");
    }

    // -- String functions -----------------------------------------------------

    #[test]
    fn strmatch_fires_match2() {
        let diags = lint_nodes(&*engine(), "i = strmatch(s, strs);\n");
        assert!(has_id(&diags, "MATCH2"), "got: {diags:?}");
    }

    #[test]
    fn findstr_fires_fstr() {
        let diags = lint_nodes(&*engine(), "k = findstr(a, b);\n");
        assert!(has_id(&diags, "FSTR"), "got: {diags:?}");
    }

    #[test]
    fn strncmp_does_not_fire() {
        let diags = lint_nodes(&*engine(), "i = strncmp(s, strs, 3);\n");
        assert!(!has_id(&diags, "MATCH2"), "got: {diags:?}");
    }

    // -- Numeric integration ---------------------------------------------------

    #[test]
    fn quad_fires_dquad() {
        let diags = lint_nodes(&*engine(), "q = quad(f, 0, 1);\n");
        assert!(has_id(&diags, "DQUAD"), "got: {diags:?}");
    }

    #[test]
    fn dblquad_fires_ddblqd() {
        let diags = lint_nodes(&*engine(), "q = dblquad(f, 0, 1, 0, 1);\n");
        assert!(has_id(&diags, "DDBLQD"), "got: {diags:?}");
    }

    #[test]
    fn integral_does_not_fire() {
        let diags = lint_nodes(&*engine(), "q = integral(f, 0, 1);\n");
        assert!(!has_id(&diags, "DQUAD"), "got: {diags:?}");
    }

    // -- Plotting ---------------------------------------------------------------

    #[test]
    fn plotyy_fires_plotyy() {
        let diags = lint_nodes(&*engine(), "plotyy(x, y1, x, y2);\n");
        assert!(has_id(&diags, "PLOTYY"), "got: {diags:?}");
    }

    #[test]
    fn polar_fires_polar() {
        let diags = lint_nodes(&*engine(), "polar(theta, rho);\n");
        assert!(has_id(&diags, "POLAR"), "got: {diags:?}");
    }

    #[test]
    fn yyaxis_does_not_fire() {
        let diags = lint_nodes(&*engine(), "yyaxis left\nplot(x, y1);\n");
        assert!(!has_id(&diags, "PLOTYY"), "got: {diags:?}");
    }

    // -- Date/time -------------------------------------------------------------

    #[test]
    fn datestr_fires_datst() {
        let diags = lint_nodes(&*engine(), "s = datestr(t);\n");
        assert!(has_id(&diags, "DATST"), "got: {diags:?}");
    }

    #[test]
    fn datenum_fires_datnm() {
        let diags = lint_nodes(&*engine(), "n = datenum(t);\n");
        assert!(has_id(&diags, "DATNM"), "got: {diags:?}");
    }

    #[test]
    fn clock_fires_clock() {
        let diags = lint_nodes(&*engine(), "c = clock();\n");
        assert!(has_id(&diags, "CLOCK"), "got: {diags:?}");
    }

    // -- Other not-recommended functions ---------------------------------------

    #[test]
    fn eval_with_two_arguments_fires_ev2in() {
        let diags = lint_nodes(&*engine(), "eval('x = 1', 'x = 0');\n");
        assert!(has_id(&diags, "EV2IN"), "got: {diags:?}");
        let diags = lint_nodes(&*engine(), "eval('x = 1');\n");
        assert!(!has_id(&diags, "EV2IN"), "got: {diags:?}");
    }

    #[test]
    fn lasterr_fires_lerr() {
        let diags = lint_nodes(&*engine(), "s = lasterr();\n");
        assert!(has_id(&diags, "LERR"), "got: {diags:?}");
    }

    #[test]
    fn isequalwithequalnans_fires_diseqn() {
        let diags = lint_nodes(&*engine(), "eq = isequalwithequalnans(a, b);\n");
        assert!(has_id(&diags, "DISEQN"), "got: {diags:?}");
    }

    #[test]
    fn mlint_fires_mlnt() {
        let diags = lint_nodes(&*engine(), "r = mlint('file.m');\n");
        assert!(has_id(&diags, "MLNT"), "got: {diags:?}");
    }

    // -- Command-syntax dispatch ------------------------------------------------

    #[test]
    fn format_without_arguments_fires_formatnoi() {
        let diags = lint_nodes(&*engine(), "format\n");
        assert!(has_id(&diags, "FORMATNOI"), "got: {diags:?}");
        let diags = lint_nodes(&*engine(), "format long\n");
        assert!(!has_id(&diags, "FORMATNOI"), "got: {diags:?}");
    }

    #[test]
    fn help_command_does_not_fire() {
        let diags = lint_nodes(&*engine(), "help csvread\n");
        assert!(!has_id(&diags, "FORMATNOI"), "got: {diags:?}");
    }

    #[test]
    fn argument_conditions_narrow_the_call() {
        use crate::test_util::lint_nodes;
        let engine = SuggestedImprovementsEngine::from_config(&Config::default());
        let ids = |src: &str| -> Vec<&'static str> {
            lint_nodes(&*engine, src)
                .iter()
                .map(|d| d.rule_id)
                .collect()
        };
        assert!(ids("x = rand(3, 2);\n").is_empty());
        assert_eq!(ids("rand('seed', 1);\n"), vec!["RAND"]);
        assert_eq!(ids("randn('state', 1);\n"), vec!["RAND"]);
        assert!(ids("eval('a = 1;');\n").is_empty());
        assert_eq!(ids("eval('a = 1;', 'a = 0;');\n"), vec!["EV2IN"]);
        assert!(ids("y = coder.inline('always');\n").is_empty());
    }

    #[test]
    fn blfread_numeric_channel_fires_blchid() {
        let diags = lint_nodes(&*engine(), "x = blfread('a.blf', 1);\n");
        assert!(has_id(&diags, "BLCHID"), "got: {diags:?}");
    }

    #[test]
    fn blfread_name_value_channel_does_not_fire_blchid() {
        let diags = lint_nodes(&*engine(), "x = blfread('a.blf', 'ChannelID', 1);\n");
        assert!(!has_id(&diags, "BLCHID"), "got: {diags:?}");
        let diags = lint_nodes(&*engine(), "x = blfread('a.blf');\n");
        assert!(!has_id(&diags, "BLCHID"), "got: {diags:?}");
    }

    // -- Conditions and dispatch --------------------------------------------------

    /// The check IDs reported for `src`, in order.
    fn rule_ids(src: &str) -> Vec<&'static str> {
        lint_nodes(&*engine(), src)
            .iter()
            .map(|d| d.rule_id)
            .collect()
    }

    #[test]
    fn engine_is_on_by_default() {
        assert!(engine().enabled_by_default());
    }

    #[test]
    fn bare_names_are_uses_without_arguments() {
        assert_eq!(rule_ids("c = clock;\n"), vec!["CLOCK"]);
        assert_eq!(rule_ids("t = now;\n"), vec!["TNOW1"]);
        assert_eq!(rule_ids("n = numlabs;\n"), vec!["DNUMLABS"]);
        assert_eq!(rule_ids("h = @nanmean;\n"), vec!["NANMEAN"]);
        // A `hold` or `ver` with no argument matches no entry.
        assert!(rule_ids("hold;\n").is_empty());
        assert!(rule_ids("ver;\n").is_empty());
    }

    #[test]
    fn command_syntax_arguments_match_literals() {
        assert_eq!(rule_ids("print -opengl f.png\n"), vec!["PRTOG"]);
        assert_eq!(rule_ids("print -painters f.png\n"), vec!["PRTPT"]);
        assert!(rule_ids("print -image f.png\n").is_empty());
        assert_eq!(rule_ids("hold all\n"), vec!["HOLDALL"]);
        assert!(rule_ids("hold on\n").is_empty());
    }

    #[test]
    fn verlessthan_matlab_fires_from_release_9_9() {
        // MATLAB reports verLessThan('matlab', V) only when V is a dotted release of 9.9 or later
        for version in [
            "'9.9'", "'9.9.0'", "'9.10'", "'9.10.0'", "'9.100'", "'9.15'", "'24.1'",
        ] {
            assert_eq!(
                rule_ids(&format!("x = verLessThan('matlab', {version});\n")),
                vec!["VERLESSMATLAB"],
                "{version}"
            );
        }
        for version in [
            "'9.8'", "'9.8.1'", "'9.2.0'", "'9.1'", "'9.0'", "'8.5'", "'R2020a'", "'R2021a'",
        ] {
            assert!(
                rule_ids(&format!("x = verLessThan('matlab', {version});\n")).is_empty(),
                "{version}"
            );
        }
        // the product must be MATLAB (case-insensitive); a variable or matrix version is not matched
        assert!(rule_ids("x = verLessThan('Simulink', '9.10');\n").is_empty());
        assert_eq!(
            rule_ids("x = verLessThan('MATLAB', \"9.10\");\n"),
            vec!["VERLESSMATLAB"]
        );
        assert!(rule_ids("x = verLessThan('matlab', [9 10]);\n").is_empty());
    }

    #[test]
    fn verlessthan_matlab_fires_in_any_position() {
        // a bare statement, an if condition, an argument of another call, a struct field value
        assert_eq!(
            rule_ids("verLessThan('matlab', '9.10');\n"),
            vec!["VERLESSMATLAB"]
        );
        assert_eq!(
            rule_ids("if verLessThan('matlab', '9.10')\nend\n"),
            vec!["VERLESSMATLAB"]
        );
        assert_eq!(
            rule_ids("s = struct('release', v.Release, 'isOld', verLessThan('matlab', '9.10'));\n"),
            vec!["VERLESSMATLAB"]
        );
        assert_eq!(
            rule_ids("x = ~verLessThan('matlab', '9.10');\n"),
            vec!["VERLESSMATLAB"]
        );
    }

    #[test]
    fn release_at_least_compares_dotted_numbers() {
        assert!(release_at_least("9.10", "9.9"));
        assert!(release_at_least("9.9.0", "9.9"));
        assert!(!release_at_least("9.8.9", "9.9"));
        assert!(release_at_least("10", "9.9"));
        assert!(!release_at_least("R2021a", "9.9"));
        assert!(!release_at_least("", "9.9"));
    }

    #[test]
    fn literal_args_pick_the_check_when_one_name_has_several() {
        assert_eq!(rule_ids("v = get(0, 'FormatSpacing');\n"), vec!["GETFSP"]);
        assert_eq!(rule_ids("v = get(0, 'Format');\n"), vec!["GETFMT"]);
        assert_eq!(rule_ids("v = get(v, 'NumberOfFrames');\n"), vec!["VIDREAD"]);
        assert!(rule_ids("v = get(h, 'Color');\n").is_empty());
        assert_eq!(
            rule_ids("set(0, 'FormatSpacing', 'loose');\n"),
            vec!["SETFSP"]
        );
        assert_eq!(rule_ids("set(0, 'Format', 'short');\n"), vec!["SETFMT"]);
        assert!(rule_ids("set(h, 'Color', 'r');\n").is_empty());
        assert_eq!(rule_ids("y = interp1(x, v, 'pp');\n"), vec!["INTRPP"]);
        assert!(rule_ids("y = interp1(x, v, xi);\n").is_empty());
        assert_eq!(rule_ids("v = ver('matlab');\n"), vec!["VERMATLAB"]);
        assert!(rule_ids("v = ver('x');\n").is_empty());
        assert_eq!(rule_ids("p = feature('getpid');\n"), vec!["FEATGPID"]);
    }

    #[test]
    fn import_in_function_syntax_fires_unless_a_char_literal() {
        assert_eq!(rule_ids("import(x)\n"), vec!["IMPORTDYN"]);
        assert_eq!(rule_ids("import(\"a.b\")\n"), vec!["IMPORTDYN"]);
        assert!(rule_ids("import('a.b')\n").is_empty());
        assert!(rule_ids("import java.util.*\n").is_empty());
    }

    #[test]
    fn string_by_argument_kind() {
        assert_eq!(rule_ids("s = string('abc');\n"), vec!["STRQUOT"]);
        assert!(rule_ids("s = string(x);\n").is_empty());
        assert_eq!(rule_ids("s = string({'a', 'b'});\n"), vec!["STRCLQT"]);
        assert!(rule_ids("s = string({'a'}, 1);\n").is_empty());
    }

    #[test]
    fn string_of_a_cell_needs_only_single_quoted_literals() {
        // MATLAB probes: `string({'a'; 'b'})`, `string({'a', ''})` and `string({'a'})`
        // report STRCLQT; a cs-list, a variable, a double-quoted element, a
        // transposed cell and a second argument do not.
        assert_eq!(rule_ids("s = string({'a'; 'b'});\n"), vec!["STRCLQT"]);
        assert_eq!(rule_ids("s = string({'a', ''});\n"), vec!["STRCLQT"]);
        assert_eq!(rule_ids("s = string({'a'});\n"), vec!["STRCLQT"]);
        assert!(rule_ids("s = string({s.f});\n").is_empty());
        assert!(rule_ids("s = string({PanelPerFold{1}.Family});\n").is_empty());
        assert!(rule_ids("s = string({'a', v});\n").is_empty());
        assert!(rule_ids("s = string({\"a\", \"b\"});\n").is_empty());
        assert!(rule_ids("s = string({'a', 'b'}');\n").is_empty());
    }

    #[test]
    fn iscellstr_silent_beside_isstring_or_on_a_known_cell() {
        // MATLAB probes: ISCLSTR is reported for `if iscellstr(v)`, `x = iscellstr(v);`,
        // `if n > 0 || iscellstr(v)` and an `elseif` after an `if n > 0`; it is silent
        // with `isstring` in the same expression, statement or `if` chain, and for a
        // cell literal, a `cell(...)` value or a variable assigned a cell.
        assert_eq!(rule_ids("if iscellstr(v)\nend\n"), vec!["ISCLSTR"]);
        assert_eq!(rule_ids("x = iscellstr(v);\n"), vec!["ISCLSTR"]);
        assert_eq!(rule_ids("if n > 0 || iscellstr(v)\nend\n"), vec!["ISCLSTR"]);
        assert_eq!(
            rule_ids("if n > 0\nelseif iscellstr(v)\nend\n"),
            vec!["ISCLSTR"]
        );
        assert!(rule_ids("if isstring(v) || iscellstr(v)\nend\n").is_empty());
        assert!(rule_ids("x = iscellstr(v) || isstring(v);\n").is_empty());
        assert!(rule_ids("if isstring(v)\nelseif iscellstr(v)\nend\n").is_empty());
        assert!(rule_ids("if iscellstr(v)\nelseif isstring(v)\nend\n").is_empty());
        assert!(rule_ids("if iscellstr({'a', 'b'})\nend\n").is_empty());
        assert!(rule_ids("c = cell(2, 1);\nif iscellstr(c)\nend\n").is_empty());
        assert!(rule_ids("x1 = {'a', 'b'};\nif iscellstr(x1)\nend\n").is_empty());
    }

    #[test]
    fn day_with_a_literal_argument() {
        assert_eq!(rule_ids("d = day(7);\n"), vec!["TNDAY"]);
        assert_eq!(rule_ids("d = day('01-Jan-2020');\n"), vec!["TNDAY"]);
        assert!(rule_ids("d = day(x);\n").is_empty());
        // `day(now)` is a use of `now`, not a literal date.
        assert_eq!(rule_ids("d = day(now);\n"), vec!["TNOW1"]);
    }

    #[test]
    fn datetime_from_datenum_replaces_the_bare_name() {
        assert_eq!(
            rule_ids("t = datetime(now, 'ConvertFrom', 'datenum');\n"),
            vec!["TNOW2"]
        );
        assert_eq!(
            rule_ids("t = datetime(today, 'ConvertFrom', 'datenum');\n"),
            vec!["TTDAY2"]
        );
        assert_eq!(rule_ids("t = datetime(now);\n"), vec!["TNOW1"]);
        assert_eq!(rule_ids("t = datetime(today);\n"), vec!["TTDAY1"]);
        assert!(rule_ids("t = datetime(x, 'ConvertFrom', 'datenum');\n").is_empty());
        assert!(rule_ids("t = datetime('now');\n").is_empty());
    }

    #[test]
    fn sim_fires_only_in_a_parfor_loop() {
        assert!(rule_ids("sim('m');\n").is_empty());
        assert_eq!(
            rule_ids("parfor i = 1:3\n    sim('m');\nend\n"),
            vec!["SIM"]
        );
    }

    #[test]
    fn variables_and_local_functions_shadow_the_name() {
        assert!(rule_ids("isdir = 3;\ny = isdir(1);\n").is_empty());
        assert!(rule_ids("isdir = 3;\nf = @() isdir(p);\n").is_empty());
        assert!(rule_ids("clock = 1;\nc = clock;\n").is_empty());
        assert!(rule_ids("s.clock = 1;\nc = s.clock;\n").is_empty());
        assert!(rule_ids("q = quad(1);\nfunction r = quad(a)\nr = a;\nend\n").is_empty());
        assert_eq!(rule_ids("y = isdir(1);\n"), vec!["ISDIR"]);
    }

    #[test]
    fn dotted_names_match_the_whole_package_path() {
        assert_eq!(rule_ids("m = sigwin.hamming(4);\n"), vec!["DHMMNG"]);
        assert!(rule_ids("m = s.hamming(4);\n").is_empty());
    }

    #[test]
    fn date_part_functions_fire_on_serial_or_text_values() {
        let family = [
            ("year", "TYEAR"),
            ("month", "TMNTH"),
            ("day", "TNDAY"),
            ("hour", "THOUR"),
            ("minute", "TMNUT"),
            ("second", "TSCND"),
            ("quarter", "TQURT"),
            ("datevec", "TDTVEC"),
        ];
        for (name, id) in family {
            // Literal serial numbers, text and cells.
            assert_eq!(rule_ids(&format!("y = {name}(7);\n")), vec![id]);
            assert_eq!(rule_ids(&format!("y = {name}('01-Jan-2020');\n")), vec![id]);
            assert_eq!(rule_ids(&format!("y = {name}({{'a'}});\n")), vec![id]);
            // A variable holding a literal of such a kind.
            assert_eq!(rule_ids(&format!("x = 5;\ny = {name}(x);\n")), vec![id]);
            assert_eq!(rule_ids(&format!("x = 'abc';\ny = {name}(x);\n")), vec![id]);
            assert_eq!(rule_ids(&format!("x = 1:3;\ny = {name}(x);\n")), vec![id]);
            assert_eq!(
                rule_ids(&format!("x = [2020 1 1];\ny = {name}(x);\n")),
                vec![id]
            );
            // A datetime, a datenum, a function result, a parameter or an
            // unknown name is not a serial number or text to MATLAB.
            assert!(
                !rule_ids(&format!("t = datetime(2020, 1, 1);\ny = {name}(t);\n")).contains(&id)
            );
            assert!(
                !rule_ids(&format!("d = datenum(2020, 1, 1);\ny = {name}(d);\n")).contains(&id)
            );
            assert!(!rule_ids(&format!("y = {name}(foo(x));\n")).contains(&id));
            assert!(!rule_ids(&format!("y = {name}(x);\n")).contains(&id));
            assert!(rule_ids(&format!(
                "x = 5;\nx = datetime(2020, 1, 1);\ny = {name}(x);\n"
            ))
            .is_empty());
            assert!(!rule_ids(&format!("x = [a 1];\ny = {name}(x);\n")).contains(&id));
            assert!(!rule_ids(&format!("function y = f(x)\ny = {name}(x);\nend\n")).contains(&id));
        }
    }

    #[test]
    fn display_fires_on_a_known_literal_value_only() {
        assert_eq!(rule_ids("dd = display(3);\n"), vec!["DISPLAYPROG"]);
        assert_eq!(rule_ids("display('abc');\n"), vec!["DISPLAYPROG"]);
        assert_eq!(rule_ids("q = 3;\ndisplay(q);\n"), vec!["DISPLAYPROG"]);
        assert_eq!(rule_ids("display({1});\n"), vec!["DISPLAYPROG"]);
        assert!(rule_ids("display(x);\n").is_empty());
        assert!(rule_ids("t = datetime(2020, 1, 1);\ndisplay(t);\n").is_empty());
        assert!(rule_ids("display(-3);\n").is_empty());
        assert!(rule_ids("display(3, 2);\n").is_empty());
        assert!(rule_ids("display\n").is_empty());
        assert!(rule_ids("display();\n").is_empty());
    }

    #[test]
    fn display_definitions_and_methods_are_not_calls() {
        // A method named display (`function display(obj)`) is not a call, and a
        // call to display on a parameter has no known value.
        let source = "classdef C\n    methods\n        function display(obj)\n            fprintf('%s\\n', inputname(1));\n            disp(obj);\n        end\n    end\nend\n";
        assert!(rule_ids(source).is_empty());
        assert!(rule_ids("function display(obj)\nfprintf('a');\nend\n").is_empty());
    }
}
