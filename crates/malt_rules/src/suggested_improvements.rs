//! # SUGGESTED_IMPROVEMENTS: Suggested Improvements
//!
//! ```malt
//! id = "SUGGESTED_IMPROVEMENTS"
//! title = "Suggested Improvements"
//! category = "suggested-improvements"
//! severity = "info"
//! fix = true
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
//! ## Fix
//!
//! The entries whose replacement is a plain rename of the callee
//! (`renames_call = true` in the data file; MATLAB marks these
//! auto-fixable) carry a fix that rewrites the callee token and nothing
//! else: `gop(@plus, x)` becomes `spmdReduce(@plus, x)` (DGOP),
//! `labindex:numlabs:n` becomes `spmdIndex:spmdSize:n` (DLABINDEX,
//! DNUMLABS), `caxis(limits)` becomes `clim(limits)` (CAXIS), `isdir(p)`
//! becomes `isfolder(p)` (ISDIR), and `Simulink.Variant('a')` becomes
//! `Simulink.VariantExpression('a')` (SIMVARIANT), all byte-identical to
//! MATLAB's own fix. The rename is behaviour-preserving in the current
//! release (the new name is the old function's new spelling), the name is
//! already known not to be shadowed, and so `--fix` applies it. The renames of
//! commands and handles carry no fix: MATLAB's fix for them is not probed.
//!
//! Entries whose replacement is a rewrite of an argument or of the call
//! (`rewrite` in the data file) write the text MATLAB's fix writes, on every
//! fixture and probed shape:
//!
//! - `string('abc')` → `"abc"` (STRQUOT, safe: a literal single-quoted
//!   argument only, `''` becomes `'` and `"` is doubled; blanks inside the
//!   parentheses stay) and `format` → `format("default")` (FORMATNOI, safe:
//!   the documented meaning of the bare command).
//! - Unsafe, needing `--unsafe-fixes`: `datetime(now, 'ConvertFrom',
//!   'datenum')` → `datetime("now")` (TNOW2); `hold all` / `hold('all')` →
//!   `hold on` / `hold('on')` (HOLDALL; the two agree in R2026b, not in
//!   releases before R2014b); `-opengl` → `-image` (PRTOG) and `-painters`
//!   → `-vector` (PRTPT), the argument only, in the command and call form
//!   (the renderers are not the same program).
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

use crate::analysis::callee::{self, call_path, reference_name, Dotted, Visibility};
use crate::analysis::reaching::resolved_values;
use crate::analysis::symbols::{is_field_name, SymbolTable};

// ---------------------------------------------------------------------------
// Data types for TOML deserialization
// ---------------------------------------------------------------------------

/// A single suggested improvement check entry from the data file.
///
/// The entries are generated at build time from `data/suggested_improvements.toml`
/// (see `build.rs`, which mirrors these fields and their defaults).
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SuggestedEntry {
    /// Check ID matching MATLAB Code Analyzer (e.g., "CSVRD").
    id: &'static str,
    /// The function/class name to match against. Empty string means generic.
    function_name: &'static str,
    /// Human-readable diagnostic message.
    message: &'static str,
    /// Recommended replacement function or syntax.
    replacement: &'static str,
    /// The fix renames the callee token exactly to `replacement` (probed:
    /// MATLAB's fix on `gop(@plus, x)` gives `spmdReduce(@plus, x)`); only the
    /// entries whose replacement is a plain rename set this. Entries that
    /// rewrite an argument (`-opengl`, `MasterSolverOptions`) or a call shape
    /// (`hold('all')`, `datetime(now, ...)`) do not, so they carry no fix.
    renames_call: bool,
    /// The fix of an entry that is not a plain rename: `"node"` replaces the
    /// whole matched call or command with `rewrite_text`, `"literal"` replaces the
    /// matched literal argument (inside its quotes when it has them), `"string"`
    /// rewrites `string('abc')` to `"abc"`. Empty: no such fix. Every one is
    /// the text MATLAB's own fix writes (probed per entry in the data file).
    rewrite: &'static str,
    /// The text `rewrite` puts in place.
    rewrite_text: &'static str,
    /// The rewrite can change behaviour, so it needs `--unsafe-fixes`.
    rewrite_unsafe: bool,
    /// Non-empty for checks that need class tracking or a toolbox that is not
    /// installed. Such entries are not matched by name.
    pattern: &'static str,
    /// The usage is only discouraged with at least this many arguments
    /// (`eval(s, catch_s)`). 0 means any call.
    min_args: usize,
    /// The usage is only discouraged with at most this many arguments
    /// (`format` with none: `max_args = 0`). Unset means no limit.
    max_args: Option<usize>,
    /// The call is assigned to at least this many outputs (`[a, b] = sim('m')`).
    /// 0 means any number, including none. Counted as `compatibility` counts them.
    min_outputs: usize,
    /// The usage is only discouraged when one argument is one of these literals
    /// (`rand('seed', 1)`, `hold all`), compared case-insensitively. Empty means
    /// any call.
    literal_args: &'static [&'static str],
    /// The literal in `literal_args` counts only when another argument follows it:
    /// `cdfread(f, 'ConvertEpochToDatenum', x)` is the name-value pair,
    /// `cdfread(f, 'ConvertEpochToDatenum')` is not.
    literal_has_value: bool,
    /// The literal of `literal_args` must be the argument at this 1-based position
    /// (`hough(BW,'ThetaResolution',VAL)`); 0 means any position.
    literal_at: usize,
    /// The literal matches its case exactly (`rand('seed')`, not `rand('SEED')`).
    literal_exact: bool,
    /// Function syntax only: command syntax (`import a.b`) never matches.
    function_only: bool,
    /// Not when the call is the value of an assignment (`y = format;`).
    not_assigned: bool,
    /// Only as a statement: not inside an assignment, a call, an `if` or an anonymous
    /// function (`import(1);` yes, `y = import(1);` no).
    statement_only: bool,
    /// 1-based position of the argument that `arg_kinds`, `not_arg_kinds` and
    /// `arg_text` test. 0 means the first argument.
    arg_at: usize,
    /// The argument at `arg_at` is of one of these kinds: `number`, `text` (a
    /// quoted string, either quote), `char` (single quotes) or `cell`.
    arg_kinds: &'static [&'static str],
    /// The argument at `arg_at` is of none of these kinds (`import('a.b')` is
    /// fine, `import(x)` is not).
    not_arg_kinds: &'static [&'static str],
    /// The argument at `arg_at` has exactly this source text (`now`).
    arg_text: &'static str,
    /// Not when the statement or `if` header that holds the call also calls this
    /// function (`if iscellstr(x) || isstring(x)`).
    not_with_call: &'static str,
    /// The argument at `arg_at`, when it is a variable, takes the kinds that
    /// every assignment to it in its scope gives (`x = 5; year(x)`). A
    /// parameter or a value that is not a literal leaves the kind unknown.
    arg_through_variables: bool,
    /// The argument at `arg_at`, when it is a variable, takes the kind of the one value
    /// that reaches the call (`x = 'abc'; string(x)`). Two values, a parameter or a loop
    /// variable leave the kind unknown (probed for STRQUOT).
    arg_sole_value: bool,
    /// The argument at `arg_at` is a dotted release number at least this one
    /// (`verLessThan('matlab', '9.10')` with `9.9`). Other text, such as
    /// `'R2021a'`, does not match.
    version_at_least: &'static str,
    /// Only inside a `parfor` loop.
    within_parfor: bool,
    /// Not when the name is an argument of a call to this function that also has
    /// `not_in_call_literal` among its literal arguments.
    not_in_call: &'static str,
    /// See `not_in_call`.
    not_in_call_literal: &'static str,
    /// The `@name` handle form is reported (`@nanmean`). MATLAB is silent on it
    /// for some entries (`@now`, `@hour`).
    handle: bool,
    /// Command syntax with arguments is reported although a variable of the name
    /// shadows the function (`display abc`, `corner abc`). Other entries are silent
    /// then (`hour abc` with `hour = 5;`).
    shadowed_command: bool,
    /// Not when every argument is of one of these kinds (`import('a', 'b')` is
    /// silent, `import('a', 1)` and `import(x)` are not). Needs at least one argument.
    not_all_arg_kinds: &'static [&'static str],
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
        if self.function_only && node.kind() == "command" {
            return false;
        }
        if self.not_assigned && is_assigned_value(node) {
            return false;
        }
        if self.statement_only
            && !node
                .parent()
                .is_some_and(|p| matches!(p.kind(), "block" | "source_file"))
        {
            return false;
        }
        if self.min_outputs > 0 && crate::compatibility::output_count(node) < self.min_outputs {
            return false;
        }
        if !self.not_all_arg_kinds.is_empty()
            && !args.is_empty()
            && args.iter().all(|arg| {
                argument_kinds(*arg, source)
                    .iter()
                    .any(|kind| self.not_all_arg_kinds.contains(kind))
            })
        {
            return false;
        }
        if self.within_parfor && !inside_parfor(node, source) {
            return false;
        }
        if !self.not_with_call.is_empty() && statement_calls(node, source, self.not_with_call) {
            return false;
        }
        if !self.not_in_call.is_empty()
            && in_call_with_literal(node, source, self.not_in_call, self.not_in_call_literal)
        {
            return false;
        }
        if !self.literal_args.is_empty() {
            let is_literal = |t: &str| {
                self.literal_args.iter().any(|l| {
                    if self.literal_exact {
                        *l == t
                    } else {
                        l.eq_ignore_ascii_case(t)
                    }
                })
            };
            let matched = if self.literal_at > 0 {
                args.get(self.literal_at - 1)
                    .and_then(|a| literal_text(*a, source))
                    .is_some_and(is_literal)
            } else {
                args.iter().enumerate().any(|(i, a)| {
                    (!self.literal_has_value || i + 1 < args.len())
                        && literal_text(*a, source).is_some_and(is_literal)
                })
            };
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
            // A variable is a version when every value that reaches the call is one
            // (probed: `vv = '9.10'; verLessThan('matlab', vv)` is reported).
            let versions: Vec<&str> = if arg.kind() == "identifier" {
                resolved_values(*arg, source)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|v| literal_text(v, source))
                    .collect::<Option<Vec<_>>>()
                    .unwrap_or_default()
            } else {
                literal_text(*arg, source).into_iter().collect()
            };
            if versions.is_empty()
                || !versions
                    .iter()
                    .all(|v| release_at_least(v, self.version_at_least))
            {
                return false;
            }
        }
        let has_kind_condition = !self.arg_kinds.is_empty()
            || !self.not_arg_kinds.is_empty()
            || !self.arg_text.is_empty();
        if !has_kind_condition {
            return true;
        }
        let Some(arg) = args.get(self.arg_at.max(1) - 1) else {
            return false;
        };
        // Command arguments are text (`hour 1`, `display abc`): MATLAB reports the
        // command for every such argument, whatever its kind would be in function
        // syntax. Command syntax has no argument text to compare with `arg_text`.
        let kinds = if node.kind() == "command" {
            if !self.arg_text.is_empty() {
                return false;
            }
            vec!["text"]
        } else if self.arg_through_variables && arg.kind() == "identifier" {
            variable_kinds(*arg, source)
        } else if self.arg_sole_value && arg.kind() == "identifier" {
            sole_value_kinds(*arg, source)
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

/// Whether `node` is the value of an assignment (`y = format;`).
fn is_assigned_value(node: tree_sitter::Node) -> bool {
    node.parent().is_some_and(|p| {
        p.kind() == "assignment"
            && p.child_by_field_name("right")
                .is_some_and(|right| right.id() == node.id())
    })
}

/// The kinds of a function-syntax argument, for the `arg_kinds` conditions.
pub(crate) fn argument_kinds(arg: tree_sitter::Node, source: &str) -> &'static [&'static str] {
    match arg.kind() {
        "number" => &["number"],
        "cell" if is_char_literal_cell(arg, source) => &["cell", "char_cell"],
        "cell" => &["cell"],
        "function_call" if call_path(arg, source, Dotted::Trailing) == Some("cell") => &["cell"],
        "function_call" if call_path(arg, source, Dotted::Trailing) == Some("struct") => {
            &["struct"]
        }
        "function_handle" | "handle_operator" => &["handle"],
        "string" if source[arg.byte_range()].starts_with('\'') => &["char", "text"],
        "string" => &["text"],
        "range" | "matrix" if is_number_expr(arg) => &["number"],
        "unary_operator" if is_signed_number(arg, source) => &["signed_number"],
        _ => &[],
    }
}

/// Whether `node` is a unary `+` or `-` of a number, or of a variable that holds only
/// numbers (`year(-5)`, `year(-x)`). Probed: MATLAB reports TYEAR on these, but not
/// DISPLAYPROG on `display(-3)` or `display(-x)`, so the kind is its own.
fn is_signed_number(node: tree_sitter::Node, source: &str) -> bool {
    if !source[node.byte_range()].starts_with(['-', '+']) {
        return false;
    }
    let Some(operand) = node.named_child(0) else {
        return false;
    };
    match operand.kind() {
        "number" => true,
        "identifier" => {
            let kinds = variable_kinds(operand, source);
            !kinds.is_empty() && kinds.iter().all(|k| *k == "number")
        }
        _ => false,
    }
}

/// The kinds of the one value that reaches the variable `ident` in a function
/// (`x = 'abc'; string(x)`). Probed: STRQUOT is silent in a script (`x = 'abc';
/// string(x)`), when the value is one of two (`if c, x = 'def'; end`), and for a
/// parameter or a number.
fn sole_value_kinds(ident: tree_sitter::Node, source: &str) -> Vec<&'static str> {
    let mut current = ident.parent();
    let in_function = loop {
        match current {
            Some(node) if node.kind() == "function_definition" => break true,
            Some(node) => current = node.parent(),
            None => break false,
        }
    };
    if !in_function {
        return Vec::new();
    }
    match resolved_values(ident, source) {
        Some(values) if values.len() == 1 => argument_kinds(values[0], source).to_vec(),
        _ => Vec::new(),
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
    if node.kind() == "function_call" && call_path(node, source, Dotted::Trailing) == Some(name) {
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

/// The right-hand side of every assignment to the variable `ident` in its scope, or
/// `None` when the variable is bound another way (a loop, `[a, name] = f()`, a
/// parameter) or never assigned. Like `variable_kinds`, but gives the values.
pub(crate) fn variable_values<'a>(
    ident: tree_sitter::Node<'a>,
    source: &str,
) -> Option<Vec<tree_sitter::Node<'a>>> {
    let name = &source[ident.byte_range()];
    let scope = enclosing_function(ident).unwrap_or_else(|| root_of(ident));
    let mut values = Vec::new();
    if collect_assigned_values(scope, name, source, &mut values) && !values.is_empty() {
        Some(values)
    } else {
        None
    }
}

fn collect_assigned_values<'a>(
    scope: tree_sitter::Node<'a>,
    name: &str,
    source: &str,
    values: &mut Vec<tree_sitter::Node<'a>>,
) -> bool {
    for child in crate::analysis::symbols::children_of(scope) {
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
                    values.push(right);
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
        if !collect_assigned_values(child, name, source, values) {
            return false;
        }
    }
    true
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
    for child in crate::analysis::symbols::children_of(scope) {
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

// ---------------------------------------------------------------------------
// Static data — generated at build time
// ---------------------------------------------------------------------------

include!(concat!(env!("OUT_DIR"), "/suggested_data.rs"));

/// Lookup table: function_name → the entries for that name.
///
/// Only entries with a non-empty `function_name` and no `pattern` are included.
/// A name can map to several check IDs (e.g., "maketform" maps to MTFA1, MTFA2,
/// MTFP1, MTFP2, MTFB, and "get" to GETFSP and VIDREAD); each entry whose
/// conditions hold is reported.
static SUGGESTED_TABLE: LazyLock<HashMap<&'static str, Vec<&'static SuggestedEntry>>> =
    LazyLock::new(|| {
        let mut map: HashMap<&'static str, Vec<&'static SuggestedEntry>> = HashMap::new();
        for entry in SUGGESTED_ENTRIES {
            if !entry.function_name.is_empty() && entry.pattern.is_empty() {
                map.entry(entry.function_name).or_default().push(entry);
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
/// (`clock`, `format`, `@nanmean`) is a use with no arguments, and a bare dotted
/// name (`sigwin.hamming;`, `@sigwin.hamming`) is a reference to the function.
const TARGET_NODES: &[&str] = &[
    "function_call",
    "command",
    "identifier",
    "field_expression",
    "handle_operator",
];

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
        // function_call or field_expression node, not as a bare name.
        if node.kind() == "identifier"
            && (node.parent().is_some_and(|p| p.kind() == "function_call") || is_field_name(node))
        {
            return Vec::new();
        }

        // Probed (plan 6.2): MATLAB reports `hgsetget` as a superclass (and nowhere
        // else: 49 probes of calls and bare names gave nothing), at the name.
        if node.kind() == "identifier"
            && node
                .parent()
                .and_then(|p| p.parent())
                .is_some_and(|p| p.kind() == "superclasses")
            && &ctx.source[node.start_byte()..node.end_byte()] == "hgsetget"
        {
            let pos = node.start_position();
            return vec![Diagnostic {
                rule_id: "HGSTGT",
                message: "hgsetget is not recommended. Use matlab.mixin.SetGet or matlab.mixin.SetGetExactNames instead.".to_string(),
                severity: Severity::Info,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }];
        }

        let func_name = match checked_name(node, ctx.source) {
            Some(name) => name,
            None => return Vec::new(),
        };
        let Some(entries) = SUGGESTED_TABLE.get(func_name) else {
            return Vec::new();
        };

        let args = call_args(node);
        // `@name` is a handle, reported only by the entries that MATLAB reports on it.
        let is_handle = node.kind() == "handle_operator"
            || node
                .parent()
                .is_some_and(|parent| parent.kind() == "handle_operator");
        let mut applicable: Vec<&SuggestedEntry> = entries
            .iter()
            .copied()
            .filter(|entry| {
                (entry.handle || !is_handle) && entry.applies_to(node, &args, ctx.source)
            })
            .collect();
        // A variable of the name shadows the function, except in command syntax with
        // arguments, where only the entries marked `shadowed_command` still report.
        // A bare name parses as a command too, so it needs an argument.
        if !applicable.is_empty()
            && callee::is_shadowed(
                node,
                func_name,
                &SymbolTable::shared(|| root_of(node), ctx.source),
                Visibility::WithScript,
            )
        {
            let command_call = node.kind() == "command" && !args.is_empty();
            applicable.retain(|entry| command_call && entry.shadowed_command);
        }
        if applicable.is_empty() {
            return Vec::new();
        }

        let start = node.start_position();
        applicable
            .into_iter()
            .map(|entry| Diagnostic {
                rule_id: entry.id,
                message: entry.message.to_string(),
                severity: Severity::Info,
                byte_range: node.start_byte()..node.end_byte(),
                line: start.row + 1,
                column: start.column + 1,
                fix: entry
                    .renames_call
                    .then(|| callee_rename_fix(node, entry.replacement))
                    .flatten()
                    .or_else(|| rewrite_fix(entry, node, &args, ctx.source)),
            })
            .collect()
    }
}

/// The fix of an entry with a `rewrite` (see [`SuggestedEntry::rewrite`]).
fn rewrite_fix(
    entry: &SuggestedEntry,
    node: tree_sitter::Node,
    args: &[tree_sitter::Node],
    source: &str,
) -> Option<malt_core::Fix> {
    let fix = match entry.rewrite {
        "node" => malt_core::Fix::new(node.byte_range(), entry.rewrite_text),
        "literal" => {
            let arg = args.iter().find(|a| {
                literal_text(**a, source).is_some_and(|t| {
                    entry.literal_args.iter().any(|l| {
                        if entry.literal_exact {
                            *l == t
                        } else {
                            l.eq_ignore_ascii_case(t)
                        }
                    })
                })
            })?;
            let range = if arg.kind() == "string" {
                arg.start_byte() + 1..arg.end_byte() - 1
            } else {
                arg.byte_range()
            };
            malt_core::Fix::new(range, entry.rewrite_text)
        }
        "string" => {
            // `string('abc')` is `"abc"`: the call and its parentheses go, the
            // blanks inside them stay (probed), a `'` pair becomes `''` -> `'`
            // and a `"` is doubled. Only a literal single-quoted argument.
            let [arg] = args else { return None };
            let text = &source[arg.byte_range()];
            if arg.kind() != "string" || !text.starts_with('\'') || !text.ends_with('\'') {
                return None;
            }
            let inner = text[1..text.len() - 1]
                .replace("''", "'")
                .replace('"', "\"\"");
            let call = node.byte_range();
            let open = source[call.start..arg.start_byte()].to_string();
            let close = source[arg.end_byte()..call.end].to_string();
            let before = open
                .trim_start_matches(|c: char| c.is_alphanumeric())
                .strip_prefix('(')?;
            let after = close.strip_suffix(')')?;
            malt_core::Fix::new(call, format!("{before}\"{inner}\"{after}"))
        }
        _ => return None,
    };
    Some(if entry.rewrite_unsafe {
        fix.unsafe_fix()
    } else {
        fix
    })
}

/// The rename fix for a `renames_call` entry: replace the callee token with the
/// replacement, keeping the call, its arguments and the surrounding text as they
/// are (probed: MATLAB's fix on `gop(@plus, x)` gives `spmdReduce(@plus, x)`,
/// and on `Simulink.Variant('a')` the whole dotted path becomes
/// `Simulink.VariantExpression`).
fn callee_rename_fix(node: tree_sitter::Node, replacement: &'static str) -> Option<malt_core::Fix> {
    if node.kind() == "function_call" {
        let name = node
            .child_by_field_name("name")
            .filter(|n| n.kind() == "identifier")?;
        // A dotted callee parses with the call at the tail of a field chain
        // (`Simulink.Variant('a')`): the edit spans the chain up to the name,
        // not just the tail identifier.
        let mut outer = node;
        while let Some(parent) = outer.parent().filter(|p| p.kind() == "field_expression") {
            outer = parent;
        }
        let range = if outer.id() == node.id() {
            name.byte_range()
        } else {
            outer.start_byte()..name.end_byte()
        };
        return Some(malt_core::Fix::new(range, replacement));
    }
    // A bare name (`labindex:numlabs:n`) or a dotted reference as a value
    // (`pkg.f`): the node is the token to rewrite. Commands and handles are
    // left alone: MATLAB's fix for them is not probed, and their node covers
    // more than the name.
    matches!(node.kind(), "identifier" | "field_expression")
        .then(|| malt_core::Fix::new(node.byte_range(), replacement))
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
fn checked_name<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    match node.kind() {
        // `sigwin.hamming(n)` matches the whole package path; a variable in front
        // (`s.isdir(x)`) yields a path that matches no entry.
        "function_call" => call_path(node, source, Dotted::Whole),
        "identifier" => Some(&source[node.start_byte()..node.end_byte()]),
        // `pkg.f` as a statement or the value of an assignment, or `@pkg.f`: a
        // reference to the function, not a call (`pkg.f(x)` is a function_call).
        // `@f` of a plain name is an identifier (below); only the dotted form is
        // a handle of its own, so the two do not both report.
        "handle_operator" => reference_name(node, source).filter(|name| name.contains('.')),
        "field_expression" => {
            let parent = node.parent()?;
            let is_value = match parent.kind() {
                // `a.b.c` names the inner part, not a package function.
                "field_expression" | "multioutput_variable" => false,
                "assignment" => parent
                    .child_by_field_name("right")
                    .is_some_and(|right| right.id() == node.id()),
                // `pkg.f 1` on a statement is command syntax, which MATLAB rejects.
                "source_file" | "block" => ends_statement(node, source),
                _ => true,
            };
            if is_value {
                reference_name(node, source)
            } else {
                None
            }
        }
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

/// Whether only a separator or a comment follows `node` on its line. A dotted
/// name followed by an argument (`pkg.f 1`, `pkg.f 'a'`) is not a bare reference:
/// MATLAB is silent on those (probes p29_C).
fn ends_statement(node: tree_sitter::Node, source: &str) -> bool {
    let rest = source[node.end_byte()..]
        .split('\n')
        .next()
        .unwrap_or("")
        .trim();
    rest.is_empty() || rest.starts_with([';', ',', '%'])
}

/// The root node of the tree that `node` belongs to.
fn root_of(node: tree_sitter::Node) -> tree_sitter::Node {
    let mut root = node;
    while let Some(parent) = root.parent() {
        root = parent;
    }
    root
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

    /// Probed in MATLAB R2026b (`/tmp/probe/c_A`): `@nanmean` is reported, `@now` is not.
    #[test]
    fn handle_form_is_reported_only_for_entries_matlab_reports_on_it() {
        assert_eq!(rule_ids("h = @nanmean;\n"), vec!["NANMEAN"]);
        assert!(rule_ids("h = @now;\n").is_empty());
    }

    /// Probed: `corner abc` and `display abc` are reported with a variable of the same
    /// name, `hour abc` is not.
    #[test]
    fn command_syntax_with_a_shadowing_variable_is_reported_only_for_flagged_entries() {
        assert_eq!(rule_ids("corner = 5;\ncorner abc;\n"), vec!["CRNR"]);
        assert_eq!(
            rule_ids("display = 5;\ndisplay abc;\n"),
            vec!["DISPLAYPROG"]
        );
        assert!(rule_ids("hour = 5;\nhour abc;\n").is_empty());
    }

    /// Probed: `import('abc', 1)`, `import(x)` are reported; `import('abc', 'def')` and
    /// `import('abc')` are not.
    #[test]
    fn import_fires_unless_every_argument_is_a_single_quoted_literal() {
        assert_eq!(rule_ids("import('abc', 1);\n"), vec!["IMPORTDYN"]);
        assert_eq!(rule_ids("import(x);\n"), vec!["IMPORTDYN"]);
        assert!(rule_ids("import('abc', 'def');\n").is_empty());
        assert!(rule_ids("import('abc');\n").is_empty());
        assert!(rule_ids("import abc 1;\n").is_empty());
    }

    /// Probed: only a numeric third argument is reported (`medfilt1(x, 3, 1)`).
    #[test]
    fn medfilt1_fires_only_for_a_numeric_third_argument() {
        assert_eq!(rule_ids("y = medfilt1(x, 3, 1);\n"), vec!["MDFLT1"]);
        assert!(rule_ids("y = medfilt1(x, 'a', 'b');\n").is_empty());
        assert!(rule_ids("y = medfilt1(x, 3, []);\n").is_empty());
        assert!(rule_ids("y = medfilt1(x, 3, 'omitnan');\n").is_empty());
    }

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
    fn bare_dotted_names_and_handles_are_references() {
        // MATLAB R2026b reports a dotted name used as a statement, a value or a
        // handle (probes p29_C: `sigwin.hamming;`, `@sigwin.hamming`).
        assert_eq!(rule_ids("sigwin.hamming;\n"), vec!["DHMMNG"]);
        assert_eq!(rule_ids("w = sigwin.hamming;\n"), vec!["DHMMNG"]);
        assert_eq!(rule_ids("h = @sigwin.hamming;\n"), vec!["DHMMNG"]);
        // Command syntax on a dotted name is silent in MATLAB (probe `sigwin.hamming seed`).
        assert!(rule_ids("sigwin.hamming seed\n").is_empty());
        assert!(rule_ids("sigwin.hamming 1\n").is_empty());
        assert!(rule_ids("sigwin.hamming 'a'\n").is_empty());
        // A call of the dotted name is one use, not a call and a reference.
        assert_eq!(rule_ids("w = sigwin.hamming(3);\n"), vec!["DHMMNG"]);
        // A value in any position: an argument, an operand, an `if` condition, a
        // matrix element (probes p29_C/D0, D1).
        assert_eq!(rule_ids("disp(sigwin.hamming + 1);\n"), vec!["DHMMNG"]);
        assert_eq!(rule_ids("if sigwin.hamming == 1, end\n"), vec!["DHMMNG"]);
        assert_eq!(rule_ids("x = [sigwin.hamming 1];\n"), vec!["DHMMNG"]);
        // A chain names a member of the function, and an assignment target is
        // not a use (probes D0 line 6, E0).
        assert!(rule_ids("z = sigwin.hamming.x;\n").is_empty());
        assert!(rule_ids("sigwin.hamming = 5;\n").is_empty());
        // A field of a variable is not the package function.
        assert!(rule_ids("s.hamming;\n").is_empty());
    }

    #[test]
    fn blfread_channel_id_through_a_numeric_variable() {
        // MATLAB reports a numeric variable as the channel ID (probe `x = [1 2]`).
        assert_eq!(
            rule_ids("x = [1 2];\nblfread('f.blf', x);\n"),
            vec!["BLCHID"]
        );
        // A name-value channel is not the positional one (probe `blfread(1, 'ChannelID', 2)`).
        assert!(rule_ids("blfread(1, 'ChannelID', 2);\n").is_empty());
    }

    // Probed: a variable holding a dotted release from 9.9 on is VERLESSMATLAB when every
    // value that reaches the call is one (a copy counts); a built version and '9.8' are not.
    #[test]
    fn verlessthan_follows_a_variable_holding_a_release() {
        let fires = |src: &str| {
            lint_nodes(&*engine(), src)
                .iter()
                .any(|d| d.rule_id == "VERLESSMATLAB")
        };
        assert!(fires("vv = '9.10';\nok = verLessThan('matlab', vv);\n"));
        assert!(fires(
            "w = '9.10';\nvv = w;\nok = verLessThan('matlab', vv);\n"
        ));
        assert!(fires(
            "vv = '9.10';\nvv = '9.11';\nok = verLessThan('matlab', vv);\n"
        ));
        assert!(!fires(
            "vv = '9.10';\nvv = '9.8';\nok = verLessThan('matlab', vv);\n"
        ));
        assert!(!fires(
            "vv = ['9.' '10'];\nok = verLessThan('matlab', vv);\n"
        ));
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
        // get/set with the numeric-format and spacing options, and interp1 'pp', are
        // unverified: MATLAB reported none of them in its probes.
        assert!(rule_ids("v = get(0, 'FormatSpacing');\n").is_empty());
        assert!(rule_ids("v = get(0, 'Format');\n").is_empty());
        assert!(rule_ids("v = get(v, 'NumberOfFrames');\n").is_empty());
        assert!(rule_ids("v = get(h, 'Color');\n").is_empty());
        assert!(rule_ids("set(0, 'FormatSpacing', 'loose');\n").is_empty());
        assert!(rule_ids("set(0, 'Format', 'short');\n").is_empty());
        assert!(rule_ids("set(h, 'Color', 'r');\n").is_empty());
        assert!(rule_ids("y = interp1(x, v, 'pp');\n").is_empty());
        assert!(rule_ids("y = interp1(x, v, xi);\n").is_empty());
        assert_eq!(rule_ids("v = ver('matlab');\n"), vec!["VERMATLAB"]);
        assert!(rule_ids("v = ver('x');\n").is_empty());
        assert_eq!(rule_ids("p = feature('getpid');\n"), vec!["FEATGPID"]);
    }

    // Probes p29_A (MATLAB R2026b checkcode): each test is one shape MATLAB reports
    // or stays silent on.

    #[test]
    fn command_syntax_is_reported_for_every_argument_kind() {
        assert_eq!(rule_ids("hour 1\n"), vec!["THOUR"]);
        assert_eq!(rule_ids("year abc\n"), vec!["TYEAR"]);
        assert_eq!(rule_ids("hour(1)\n"), vec!["THOUR"]);
        assert!(rule_ids("hour\n").is_empty());
        assert!(rule_ids("hour()\n").is_empty());
        assert!(rule_ids("hour(@sin)\n").is_empty());
    }

    #[test]
    fn strmatch_three_arguments_are_strcmp_only() {
        assert_eq!(rule_ids("y = strmatch(a);\n"), vec!["MATCH2"]);
        assert_eq!(rule_ids("y = strmatch(a, b);\n"), vec!["MATCH2"]);
        assert_eq!(rule_ids("y = strmatch(a, b, c, d);\n"), vec!["MATCH2"]);
        assert_eq!(rule_ids("y = strmatch(a, b, c);\n"), vec!["MATCH3"]);
    }

    #[test]
    fn eval_family_takes_the_argument_counts_of_the_message() {
        assert_eq!(rule_ids("eval(a, b);\n"), vec!["EV2IN"]);
        assert!(rule_ids("eval(a, b, c);\n").is_empty());
        assert_eq!(rule_ids("evalin('base', a, b);\n"), vec!["EV3IN"]);
        assert!(rule_ids("evalin('base', a);\n").is_empty());
        assert!(rule_ids("evalin('base', a, b, c);\n").is_empty());
        assert_eq!(rule_ids("evalc(a, b);\n"), vec!["EVLC"]);
        assert!(rule_ids("evalc(a, b, c);\n").is_empty());
    }

    #[test]
    fn rand_seed_option_is_the_first_argument_in_lowercase() {
        assert_eq!(rule_ids("rand('seed', 1);\n"), vec!["RAND"]);
        assert_eq!(rule_ids("rand seed\n"), vec!["RAND"]);
        assert!(rule_ids("rand(3, 'seed');\n").is_empty());
        assert!(rule_ids("rand('SEED');\n").is_empty());
        assert_eq!(rule_ids("randn('state', 1);\n"), vec!["RAND"]);
    }

    #[test]
    fn hough_theta_resolution_is_the_second_of_three_arguments() {
        assert_eq!(
            rule_ids("h = hough(bw, 'ThetaResolution', 5);\n"),
            vec!["HOUGH"]
        );
        assert!(rule_ids("h = hough(bw, 'ThetaResolution');\n").is_empty());
        assert!(rule_ids("h = hough(bw, 'Theta', -90:5:85);\n").is_empty());
    }

    #[test]
    fn datetime_from_datenum_takes_exactly_three_arguments() {
        assert_eq!(
            rule_ids("t = datetime(now, 'ConvertFrom', 'datenum');\n"),
            vec!["TNOW2"]
        );
        assert_eq!(
            rule_ids("t = datetime(today, 'ConvertFrom', 'datenum');\n"),
            vec!["TTDAY2"]
        );
    }

    #[test]
    fn image_input_layer_augmentation_needs_a_value() {
        assert_eq!(
            rule_ids("l = imageInputLayer([28 28 1], 'DataAugmentation', 'none');\n"),
            vec!["IMGDT"]
        );
        assert_eq!(
            rule_ids("l = imageInputLayer('DataAugmentation', x);\n"),
            vec!["IMGDT"]
        );
        assert!(rule_ids("l = imageInputLayer(1, 'DataAugmentation');\n").is_empty());
        assert!(rule_ids("l = imageInputLayer('DataAugmentation');\n").is_empty());
    }

    #[test]
    fn maketform_type_and_argument_count() {
        assert_eq!(rule_ids("t = maketform('affine', a);\n"), vec!["MTFA1"]);
        assert_eq!(rule_ids("t = maketform('AFFINE', u, x);\n"), vec!["MTFA2"]);
        assert_eq!(rule_ids("t = maketform('projective', a);\n"), vec!["MTFP1"]);
        assert_eq!(
            rule_ids("t = maketform('projective', u, x);\n"),
            vec!["MTFP2"]
        );
        assert_eq!(rule_ids("t = maketform('box', 1, 2, 3);\n"), vec!["MTFB"]);
        assert!(rule_ids("t = maketform('custom', a, b, c, d, e);\n").is_empty());
    }

    #[test]
    fn pmtm_needs_three_output_arguments() {
        assert_eq!(rule_ids("[p, f, c] = pmtm(x);\n"), vec!["PMTMCONF"]);
        assert!(rule_ids("[p, f] = pmtm(x);\n").is_empty());
        assert!(rule_ids("p = pmtm(x);\n").is_empty());
    }

    #[test]
    fn iscellstr_is_silent_for_a_known_value() {
        assert_eq!(rule_ids("b = iscellstr(zz);\n"), vec!["ISCLSTR"]);
        assert_eq!(rule_ids("b = iscellstr(@sin);\n"), vec!["ISCLSTR"]);
        assert!(rule_ids("b = iscellstr(1);\n").is_empty());
        assert!(rule_ids("b = iscellstr({'a'});\n").is_empty());
        assert!(rule_ids("b = iscellstr(zz, 1);\n").is_empty());
        assert!(rule_ids("b = iscellstr(zz) || isstring(zz);\n").is_empty());
        assert!(rule_ids("s = struct('a', 1);\nb = iscellstr(s);\n").is_empty());
        assert!(rule_ids("c = 'abc';\nb = iscellstr(c);\n").is_empty());
    }

    #[test]
    fn display_is_silent_for_a_handle_or_an_unknown_argument() {
        assert_eq!(rule_ids("x = 5;\ndisplay(x);\n"), vec!["DISPLAYPROG"]);
        assert_eq!(rule_ids("display abc\n"), vec!["DISPLAYPROG"]);
        assert!(rule_ids("fh = @sin;\ndisplay(fh);\n").is_empty());
        assert!(rule_ids("display(zz);\n").is_empty());
        assert!(rule_ids("display(1, 2);\n").is_empty());
    }

    #[test]
    fn import_two_arguments_and_statements_only() {
        assert_eq!(rule_ids("import(1, 2);\n"), vec!["IMPORTDYN"]);
        assert!(rule_ids("import('abc', 'def');\n").is_empty());
        assert!(rule_ids("y = import(1);\n").is_empty());
        assert!(rule_ids("disp(import(1));\n").is_empty());
    }

    #[test]
    fn nargchk_geotiffread_medfilt1_and_format_arities() {
        assert_eq!(rule_ids("n = nargchk(1, 2, 3);\n"), vec!["NCHKI"]);
        assert!(rule_ids("y = nargchk;\n").is_empty());
        assert_eq!(rule_ids("img = geotiffread(f);\n"), vec!["GTRED"]);
        assert!(rule_ids("img = geotiffread(f, 1);\n").is_empty());
        assert_eq!(rule_ids("y = medfilt1(x, 3, 1);\n"), vec!["MDFLT1"]);
        assert!(rule_ids("y = medfilt1(x, 3);\n").is_empty());
        assert_eq!(rule_ids("format;\n"), vec!["FORMATNOI"]);
        assert!(rule_ids("y = format;\n").is_empty());
    }

    #[test]
    fn string_of_one_text_argument() {
        assert_eq!(rule_ids("s = string('abc');\n"), vec!["STRQUOT"]);
        assert!(rule_ids("s = string('abc', 'def');\n").is_empty());
    }

    #[test]
    fn string_of_variable_with_one_text_value_in_a_function() {
        // Probed: `x = 'abc'; y = string(x)` is reported in a function, also after a
        // reassignment; silent in a script, after a conditional write, for a parameter
        // and for a number.
        let in_fn = |body: &str| format!("function y = f(p)\n{body}\nend\n");
        assert_eq!(
            rule_ids(&in_fn("x = 'abc';\ny = string(x);")),
            vec!["STRQUOT"]
        );
        assert_eq!(
            rule_ids(&in_fn("x = 'abc';\nx = 'def';\ny = string(x);")),
            vec!["STRQUOT"]
        );
        assert!(rule_ids("x = 'abc';\ns = string(x);\n").is_empty());
        assert!(rule_ids(&in_fn(
            "x = 'abc';\nif p\n    x = 'def';\nend\ny = string(x);"
        ))
        .is_empty());
        assert!(rule_ids(&in_fn("y = string(p);")).is_empty());
        assert!(rule_ids(&in_fn("x = 3;\ny = string(x);")).is_empty());
    }

    #[test]
    fn signed_numbers_for_the_date_functions_but_not_display() {
        // Probed: TYEAR and TMNUT on a signed number or a variable holding a number
        // (`year(-5)`, `year(- 5)`, `year(+x)`, `minute(-5)`); DISPLAYPROG is silent on
        // `display(-3)`, `display(+3)` and `display(-x)`; a parameter is not known.
        assert_eq!(rule_ids("y = year(-5);\n"), vec!["TYEAR"]);
        assert_eq!(rule_ids("y = year(- 5);\n"), vec!["TYEAR"]);
        assert_eq!(rule_ids("y = year(-5.5);\n"), vec!["TYEAR"]);
        assert_eq!(rule_ids("x = 5;\ny = year(+x);\n"), vec!["TYEAR"]);
        assert_eq!(rule_ids("x = -5;\ny = year(x);\n"), vec!["TYEAR"]);
        assert_eq!(rule_ids("y = minute(-5);\n"), vec!["TMNUT"]);
        assert!(rule_ids("display(-3);\n").is_empty());
        assert!(rule_ids("display(+3);\n").is_empty());
        assert!(rule_ids("x = 3;\ndisplay(-x);\n").is_empty());
        assert!(rule_ids("function y = f(p)\ny = year(-p);\nend\n").is_empty());
        assert!(rule_ids("y = weekday(-5);\n").is_empty());
    }

    #[test]
    fn command_syntax_is_a_call_despite_a_variable() {
        assert_eq!(rule_ids("corner = 3;\ncorner abc;\n"), vec!["CRNR"]);
        assert!(rule_ids("corner = 3;\ncorner;\n").is_empty());
    }

    #[test]
    fn hgsetget_is_reported_as_a_superclass_only() {
        // Probed (plan 6.2): `classdef C < hgsetget` and `< handle & hgsetget`.
        let src = "classdef C < hgsetget\nend\n";
        assert_eq!(rule_ids(src), vec!["HGSTGT"]);
        assert_eq!(
            rule_ids("classdef C < handle & hgsetget\nend\n"),
            vec!["HGSTGT"]
        );
        assert!(rule_ids("classdef C < handle\nend\n").is_empty());
        assert!(rule_ids("x = hgsetget;\n").is_empty());
    }

    #[test]
    fn unverified_entries_are_silent() {
        assert!(rule_ids("eml.extrinsic('sin');\n").is_empty());
        assert!(rule_ids("p = addrule('x');\n").is_empty());
        assert!(rule_ids("t = addParamValue('x', 1);\n").is_empty());
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

    #[test]
    fn dotted_package_name_fires_bare_as_handle_and_called() {
        // Probed: a package function is a use without arguments, in an
        // assignment, as a handle, and with a call; a field of a struct is not.
        assert_eq!(rule_ids("comm.PAMModulator;\n"), vec!["COMMPAMM"]);
        assert_eq!(rule_ids("y = comm.PAMModulator;\n"), vec!["COMMPAMM"]);
        assert_eq!(rule_ids("h = @comm.PAMModulator;\n"), vec!["COMMPAMM"]);
        assert_eq!(rule_ids("comm.PAMModulator(4);\n"), vec!["COMMPAMM"]);
        assert_eq!(rule_ids("H5.close;\n"), vec!["H5CLS"]);
        assert_eq!(rule_ids("h = @H5.close;\n"), vec!["H5CLS"]);
        assert_eq!(rule_ids("y = H5.open;\n"), vec!["H5OPN"]);
        assert!(rule_ids("s.comm.PAMModulator = 1;\n").is_empty());
    }

    #[test]
    fn instrhwinfo_fires_for_its_own_interface_only() {
        assert_eq!(rule_ids("x = instrhwinfo('ivi');\n"), vec!["INSTHWI"]);
        assert_eq!(rule_ids("x = instrhwinfo(\"IVI\");\n"), vec!["INSTHWI"]);
        assert_eq!(rule_ids("instrhwinfo ivi\n"), vec!["INSTHWI"]);
        assert_eq!(rule_ids("x = instrhwinfo('vxipnp');\n"), vec!["INWVX"]);
        assert!(!rule_ids("x = instrhwinfo();\n").contains(&"INSTHWI"));
        assert!(!rule_ids("x = instrhwinfo('vxipnp');\n").contains(&"INSTHWI"));
    }

    #[test]
    fn cdfread_epoch_option_needs_a_value_after_its_name() {
        assert_eq!(
            rule_ids("x = cdfread(f, 'ConvertEpochToDatenum', true);\n"),
            vec!["CDFEPOCH2DATE"]
        );
        assert_eq!(
            rule_ids("x = cdfread('ConvertEpochToDatenum', true);\n"),
            vec!["CDFEPOCH2DATE"]
        );
        assert_eq!(
            rule_ids("x = cdfread(f, \"convertepochtodatenum\", 1);\n"),
            vec!["CDFEPOCH2DATE"]
        );
        assert!(rule_ids("x = cdfread(f, 'ConvertEpochToDatenum');\n").is_empty());
        assert!(rule_ids("x = cdfread(f);\n").is_empty());
        assert!(rule_ids("x = cdfread(f, 'DatetimeType', 'datetime');\n").is_empty());
    }

    #[test]
    fn sim_with_several_outputs_and_a_single_quoted_model_fires_oldsim() {
        // Probed: two or more outputs, and the model is a single-quoted literal.
        assert_eq!(rule_ids("[a, b] = sim('m');\n"), vec!["OLDSIM"]);
        assert_eq!(rule_ids("[a, ~] = sim('m');\n"), vec!["OLDSIM"]);
        assert!(rule_ids("a = sim('m');\n").is_empty());
        assert!(rule_ids("[a, b] = sim(\"m\");\n").is_empty());
        assert!(rule_ids("[a, b] = sim(m);\n").is_empty());
        assert!(rule_ids("[a, b] = sim;\n").is_empty());
    }
}

#[cfg(test)]
mod rename_fix_tests {
    use crate::test_util::lint_nodes;
    use malt_core::{apply_fixes_with, FixMode};

    pub(crate) fn engine() -> Box<dyn crate::Rule> {
        super::SuggestedImprovementsEngine::from_config(&malt_core::Config::default())
    }

    #[test]
    fn renames_call_fix_rewrites_only_the_callee() {
        // Probed: MATLAB's fix on `gop(@plus, x)` gives `spmdReduce(@plus, x)`,
        // and the bare names of a range (`labindex:numlabs:n`) rename in place.
        let (src, fixed) = ("y = gop(@plus, x);\n", "y = spmdReduce(@plus, x);\n");
        let diags = lint_nodes(&*engine(), src);
        let out = apply_fixes_with(src, &diags, FixMode::Safe).0;
        assert_eq!(out, fixed, "the rename is safe: {src}");

        let (src, fixed) = (
            "chunk = labindex:numlabs:nSamples;\n",
            "chunk = spmdIndex:spmdSize:nSamples;\n",
        );
        let diags = lint_nodes(&*engine(), src);
        let out = apply_fixes_with(src, &diags, FixMode::Safe).0;
        assert_eq!(out, fixed, "{src}: {out:?}");

        // A dotted callee renames whole; commands and handles carry no fix
        // (MATLAB's fix for them is not probed).
        let (src, fixed) = (
            "q = Simulink.Variant('a');\n",
            "q = Simulink.VariantExpression('a');\n",
        );
        let diags = lint_nodes(&*engine(), src);
        let out = apply_fixes_with(src, &diags, FixMode::Safe).0;
        assert_eq!(out, fixed, "{out:?}");
        let diags = lint_nodes(&*engine(), "mlint('f.m');\n");
        let fix = diags
            .iter()
            .find(|d| d.rule_id == "MLNT")
            .and_then(|d| d.fix.as_ref())
            .expect("the plain-call form carries the rename");
        assert_eq!(fix.replacement, "checkcode");
    }
}
