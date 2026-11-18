//! # COMPAT: Compatibility
//!
//! ```malt
//! id = "COMPAT"
//! title = "Compatibility"
//! category = "compatibility"
//! severity = "warning"
//! fix = false
//! icon = "lucide/archive"
//! slug = "compatibility"
//! data_file = "compatibility.toml"
//! ```
//!
//! ## Rule
//!
//! A data-driven rule module that handles 1,924 MATLAB Code Analyzer checks
//! about deprecated and removed functions and behavior changes. Instead of
//! implementing one struct per check, a single `CompatibilityEngine` loads
//! `data/compatibility.toml` at compile time, parses it into a function-name
//! lookup table, and emits diagnostics with the specific check ID from the
//! matched entry (e.g. `DPSD`, not the meta ID `COMPAT`).
//!
//! On each `function_call` or `command` node, the engine extracts the function
//! name and performs an O(1) HashMap lookup. A function name may map to
//! multiple check IDs (e.g. `tcpip` → TCPC + TCPS); every matching entry is
//! emitted. The entries span three category groups:
//!
//! - **compatibility** — deprecated or removed functions (1,012 entries)
//! - **behavior-changes** — behavior changes between releases (905 entries)
//! - **forward-compatibility** — constructs that will break in a future
//!   release (7 entries)
//!
//! Entries with an empty `function_name` are "generic" checks that cannot be
//! matched by function name alone (they require AST pattern matching) and are
//! skipped during the lookup-based check. These are implemented as file-level
//! AST-pattern checks in the `compatibility/generic/` modules (property and
//! attribute removals, removed options and input arguments,
//! forward-compatibility version gates, and behavior-change figure and axes
//! properties) and dispatched from `check_file`.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! psd(x)            % DPSD — removed; use periodogram or pwelch
//! strread(s)        % STREAD — removed; use textscan
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! periodogram(x)
//! textscan(s)
//! ```
//!
//! ## Configuration
//!
//! The lookup engine itself takes no parameters. Per-check severity can be
//! overridden by rule ID or by category:
//!
//! ```toml
//! [lint.categories]
//! compatibility = "warn"
//!
//! [lint.rules]
//! COMPAT = "error"
//! ```

use std::collections::HashMap;
use std::sync::LazyLock;

use malt_core::{Category, Config, Diagnostic, FileContext, NodeContext, Rule, Severity};

use crate::analysis::callee::{self, Dotted, Visibility};
use crate::analysis::symbols::{ScopeKind, SymbolTable};

/// Generic (AST-pattern) compatibility checks that the name-lookup table cannot
/// match. Each check is implemented in its own `check_*.rs` module and
/// registered here.
mod generic;

// ---------------------------------------------------------------------------
// Data types for TOML deserialization
// ---------------------------------------------------------------------------

/// A single compatibility check entry from the data file.
///
/// The entries are generated at build time from `data/compatibility.toml`
/// (see `build.rs`, which mirrors these fields and their defaults).
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CompatEntry {
    /// Check ID matching MATLAB Code Analyzer (e.g., "DPSD").
    id: &'static str,
    /// The function/class name to match against. Empty string means generic.
    function_name: &'static str,
    /// Severity level: "error" or "warning".
    severity: &'static str,
    /// Human-readable diagnostic message.
    message: &'static str,
    /// Category: "compatibility", "forward-compatibility", or "behavior-changes".
    category: &'static str,
    /// Optional pattern marker. "generic" means this check cannot be matched
    /// by function name alone.
    pattern: &'static str,
    /// MATLAB lists the entry under "(Upcoming) Behavior Changes with Low
    /// Reliability Messages" and does not report it by default.
    low_reliability: bool,
    /// Fewest arguments the call must have.
    min_args: Option<usize>,
    /// Most arguments the call may have.
    max_args: Option<usize>,
    /// The call has one of these string arguments (compared without case).
    args_any: &'static [&'static str],
    /// The call has every one of these string arguments.
    args_all: &'static [&'static str],
    /// The call has none of these string arguments.
    unless_args: &'static [&'static str],
    /// Shape of the second argument: `"string"`, `"not_string"` (present and not
    /// a string literal, so positional rather than name-value syntax) or
    /// `"optimoptions"`.
    second_arg: &'static str,
    /// The second argument is one of these strings, compared with case (a quoted
    /// literal, or a variable every assignment of which is such a literal).
    second_arg_any: &'static [&'static str],
    /// Reported only in a script file.
    script_only: bool,
    /// The first argument is text: a quoted literal, or a variable that every
    /// assignment in its scope gives a literal of (`ismethod(mdl, ...)` with an
    /// object `mdl` is not reported).
    first_arg_text: bool,
    /// The first argument is not text (a number, a matrix or a variable that is
    /// not known to hold text); a call whose first argument is a string is silent.
    first_arg_not_text: bool,
    /// Fewest outputs the call is assigned to: `[a, b] = f(x)` has two.
    min_outputs: Option<usize>,
    /// The first argument is one of these strings (compared without case).
    first_arg_any: &'static [&'static str],
    /// The call's only argument is one of these strings: the entry does not apply.
    unless_sole_arg: &'static [&'static str],
    /// The call does not have every one of these strings: the entry does not apply.
    unless_all_args: &'static [&'static str],
    /// The `@name` handle form is reported (`@nanmean`); other entries are silent on it.
    handle: bool,
    /// The argument at this 1-based position is a number literal (`qammod(x, 4, 0)`,
    /// not `pi/4`); 0 means no such condition.
    number_arg_at: usize,
    /// From this 1-based position on the arguments are name-value pairs (an even
    /// count; the names at odd offsets). `args_any` then names one of those names.
    /// 0 means no pairs.
    pairs_from: usize,
    /// Matches every function whose dotted name starts with `function_name` followed
    /// by a dot (`com.mathworks.util.X`). Not looked up by the exact name.
    function_prefix: bool,
}

// ---------------------------------------------------------------------------
// Static data — generated at build time
// ---------------------------------------------------------------------------

include!(concat!(env!("OUT_DIR"), "/compat_data.rs"));

/// Lookup table: function_name → matching `CompatEntry` list.
///
/// Only entries with a non-empty `function_name` and no pattern are
/// included. A function name may map to multiple check IDs (e.g., "tcpip"
/// maps to both TCPC and TCPS); all matching entries are stored so the engine
/// emits every applicable diagnostic.
static COMPAT_TABLE: LazyLock<HashMap<&'static str, Vec<&'static CompatEntry>>> =
    LazyLock::new(|| {
        let mut map: HashMap<&'static str, Vec<&'static CompatEntry>> =
            HashMap::with_capacity(COMPAT_ENTRIES.len());
        for entry in COMPAT_ENTRIES {
            if !entry.function_name.is_empty() && entry.pattern.is_empty() && !entry.function_prefix
            {
                map.entry(entry.function_name).or_default().push(entry);
            }
        }
        map
    });

/// The entries with `function_prefix`, matched by the start of a dotted name.
static COMPAT_PREFIXES: LazyLock<Vec<&'static CompatEntry>> = LazyLock::new(|| {
    COMPAT_ENTRIES
        .iter()
        .filter(|entry| entry.function_prefix && entry.pattern.is_empty())
        .collect()
});

/// Whether `name` is a member of the package `prefix` (`com.mathworks.x` in
/// `com.mathworks`), not the package itself.
fn in_package(prefix: &str, name: &str) -> bool {
    name.len() > prefix.len() && name.starts_with(prefix) && name.as_bytes()[prefix.len()] == b'.'
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// The compatibility engine — a single rule instance that checks all
/// deprecated/removed function usage via a data-driven lookup table, plus the
/// generic (AST-pattern) compatibility checks that cannot be matched by
/// function name alone.
///
/// This is registered once with inventory. Each emitted diagnostic carries the
/// specific check ID from the data file (e.g., "DPSD"), not a generic ID.
pub struct CompatibilityEngine {
    /// The forward-compatibility version gates (`FC*`) that are switched on.
    /// MATLAB's factory configuration leaves them off, so they are off here
    /// unless the preset, the category or the rule itself turns them on.
    forward_gates: Vec<&'static str>,
    /// Whether IDISVARHIGH is reported. MATLAB's default `checkcode` never emits it
    /// (probed, also with `-allmsg`), so it is off unless the preset or the rule turns it on.
    idisvar_high: bool,
    /// Whether the low-reliability behavior-change entries are reported (off by
    /// default, as in MATLAB; on under `all` or with the behavior-changes category).
    low_reliability: bool,
}

/// The forward-compatibility version-gate checks (MATLAB's "Forward
/// Compatibility Considerations" group).
const FORWARD_GATE_IDS: &[&str] = &[
    "FCLEN", "FCCPV", "FCDQS", "FCFAV", "FCHBL", "FCLFS", "FCNVA",
];

/// Target node types for the compatibility engine: calls and commands, and the
/// bare references to a function (`@f`, and `pkg.f` used as a value).
const TARGET_NODES: &[&str] = &[
    "function_call",
    "command",
    "handle_operator",
    "field_expression",
    "identifier",
];

impl CompatibilityEngine {
    /// Factory constructor called by the rule registry.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        // Force initialization of static data at construction time so that any
        // parse errors surface early rather than on first lint invocation.
        let _ = &*COMPAT_TABLE;
        let group_on = match config.categories.get(&Category::ForwardCompatibility) {
            Some(severity) => severity.is_some(),
            None => config.preset.enables_default_off,
        };
        let forward_gates = FORWARD_GATE_IDS
            .iter()
            .copied()
            .filter(|id| match config.rules.get(*id) {
                Some(rule) => rule.enabled,
                None => group_on,
            })
            .collect();
        let idisvar_high = config.preset.enables_default_off
            || config
                .rules
                .get("IDISVARHIGH")
                .is_some_and(|rule| rule.enabled);
        let low_reliability = match config.categories.get(&Category::BehaviorChanges) {
            Some(severity) => severity.is_some(),
            None => config.preset.enables_default_off,
        };
        Box::new(Self {
            forward_gates,
            idisvar_high,
            low_reliability,
        })
    }

    /// Run the generic (AST-pattern) compatibility checks over a file.
    ///
    /// Dispatched from `check_file`. The per-check implementations live in the
    /// sibling `check_*.rs` modules and are `pub(crate)` methods on
    /// `CompatibilityEngine`; this walker invokes each one.
    fn check_generic(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        // Each generic check registers a subtree of the walk via a visitor;
        // implemented in the per-check modules below.
        let root = tree.root_node();
        let mut cursor = root.walk();
        self.visit_generic_checks(root, source, &mut cursor, &mut diagnostics);
        // File-level checks that need whole-tree scope analysis (e.g. global
        // redeclaration, nested-import inheritance).
        crate::compatibility::generic::collect_file_checks(self, tree, source, &mut diagnostics);
        diagnostics
    }

    /// DFS that hands each node to the generic check methods.
    fn visit_generic_checks(
        &self,
        node: tree_sitter::Node,
        source: &str,
        cursor: &mut tree_sitter::TreeCursor,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Per-group check dispatch is added by the check_*.rs modules via a
        // single `collect_generic_node_checks` free function (defined in
        // `generic.rs`) so this walker stays independent of individual checks.
        crate::compatibility::generic::collect_node_checks(self, node, source, diagnostics);
        if cursor.goto_first_child() {
            loop {
                self.visit_generic_checks(cursor.node(), source, cursor, diagnostics);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }
}

impl Rule for CompatibilityEngine {
    fn id(&self) -> &'static str {
        "COMPAT"
    }

    fn description(&self) -> &'static str {
        "Deprecated or removed function usage"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn category(&self) -> Category {
        Category::Compatibility
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut diagnostics = self.check_generic(ctx.tree, ctx.source);
        diagnostics.retain(|d| {
            !FORWARD_GATE_IDS.contains(&d.rule_id) || self.forward_gates.contains(&d.rule_id)
        });
        diagnostics.retain(|d| d.rule_id != "IDISVARHIGH" || self.idisvar_high);
        diagnostics.retain(|d| self.low_reliability || !is_low_reliability(d.rule_id));
        diagnostics
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        // Extract the function/command name from the node.
        let func_name = match checked_name(ctx.node, ctx.source) {
            Some(name) => name,
            None => return Vec::new(),
        };

        // Lookup in the compatibility table, then the package prefixes.
        let mut entries: Vec<&CompatEntry> = COMPAT_TABLE
            .get(func_name)
            .map(|found| found.to_vec())
            .unwrap_or_default();
        entries.extend(
            COMPAT_PREFIXES
                .iter()
                .copied()
                .filter(|entry| in_package(entry.function_name, func_name)),
        );
        if entries.is_empty() {
            return Vec::new();
        }
        let is_handle = ctx.node.kind() == "handle_operator"
            || ctx
                .node
                .parent()
                .is_some_and(|parent| parent.kind() == "handle_operator");

        // A dotted call is reported at the start of its package path (`comm` in
        // `comm.HelicalInterleaver(...)`), as MATLAB does.
        let anchor = match ctx.node.parent() {
            Some(parent)
                if ctx.node.kind() == "function_call"
                    && crate::analysis::symbols::is_qualified_call(ctx.node) =>
            {
                parent.named_child(0).unwrap_or(ctx.node)
            }
            // A handle is reported at the name after the `@`.
            _ if ctx.node.kind() == "handle_operator" => {
                ctx.node.named_child(0).unwrap_or(ctx.node)
            }
            _ => ctx.node,
        };
        let start = anchor.start_position();
        let mut diagnostics = Vec::with_capacity(entries.len());

        // A variable or a local function of the same name shadows the removed one.
        let table = SymbolTable::shared(|| root_of(ctx.node), ctx.source);
        if callee::is_shadowed(ctx.node, func_name, &table, Visibility::Enclosing) {
            return Vec::new();
        }
        let script = matches!(table.root_scope().kind, ScopeKind::Script);
        let args = call_args(ctx.node);
        let outputs = output_count(ctx.node);

        for entry in entries {
            if entry.low_reliability && !self.low_reliability {
                continue;
            }
            if is_handle && !entry.handle {
                continue;
            }
            if !applies(entry, &args, outputs, ctx.source, script) {
                continue;
            }
            let severity = parse_severity(entry.severity);

            diagnostics.push(Diagnostic {
                rule_id: entry.id,
                message: entry.message.to_string(),
                severity,
                byte_range: ctx.node.start_byte()..ctx.node.end_byte(),
                line: start.row + 1,
                column: start.column + 1,
                fix: None,
            });
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The name a compatibility check looks up for `node`: the callee of a call,
/// command or handle (see [`callee::callee_path`]), or a bare reference in value
/// position (`[h, a] = legend;`, `x = pkg.f;`). Other shapes give `None`.
fn checked_name<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    if callee::kind(node).is_some() {
        return callee::callee_path(node, source, Dotted::Whole);
    }
    match node.kind() {
        "identifier" => {
            // `[h, a] = legend;`: a call without parentheses, with several outputs.
            let parent = node.parent()?;
            let is_value = parent.kind() == "assignment"
                && parent
                    .child_by_field_name("right")
                    .is_some_and(|right| right.id() == node.id());
            let several = parent
                .child_by_field_name("left")
                .is_some_and(|left| left.kind() == "multioutput_variable");
            (is_value && several).then(|| &source[node.start_byte()..node.end_byte()])
        }
        "field_expression" => {
            let parent = node.parent()?;
            let is_value = match parent.kind() {
                "source_file" | "block" => true,
                "assignment" => parent
                    .child_by_field_name("right")
                    .is_some_and(|right| right.id() == node.id()),
                _ => false,
            };
            if is_value {
                callee::reference_name(node, source)
            } else {
                None
            }
        }
        _ => None,
    }
}
/// Whether the data file lists `id` under MATLAB's low-reliability messages,
/// which are off by default. The generic checks honour the same switch.
fn is_low_reliability(id: &str) -> bool {
    COMPAT_ENTRIES
        .iter()
        .any(|entry| entry.low_reliability && entry.id == id)
}

/// The root node of the tree that `node` belongs to.
fn root_of(node: tree_sitter::Node) -> tree_sitter::Node {
    let mut root = node;
    while let Some(parent) = root.parent() {
        root = parent;
    }
    root
}

/// The arguments of a `function_call` or `command` node, without comments.
fn call_args(node: tree_sitter::Node) -> Vec<tree_sitter::Node> {
    match node.kind() {
        "function_call" => (0..node.child_count())
            .filter_map(|i| node.child(i))
            .find(|c| c.kind() == "arguments")
            .map(|args| {
                (0..args.named_child_count())
                    .filter_map(|i| args.named_child(i))
                    .filter(|c| c.kind() != "comment")
                    .collect()
            })
            .unwrap_or_default(),
        "command" => (0..node.child_count())
            .filter_map(|i| node.child(i))
            .filter(|c| c.kind() == "command_argument")
            .collect(),
        _ => Vec::new(),
    }
}

/// The number of outputs a call is assigned to: `[a, ~, c] = f(x)` has three. A
/// call that is not the right-hand side of an assignment has one.
pub(crate) fn output_count(node: tree_sitter::Node) -> usize {
    let Some(parent) = node.parent() else {
        return 1;
    };
    let is_value = parent.kind() == "assignment"
        && parent
            .child_by_field_name("right")
            .is_some_and(|right| right.id() == node.id());
    match parent.child_by_field_name("left") {
        Some(lhs) if is_value && lhs.kind() == "multioutput_variable" => (0..lhs.child_count())
            .filter_map(|i| lhs.child(i))
            .filter(|c| !matches!(c.kind(), "[" | "]" | "," | "comment"))
            .count()
            .max(1),
        _ => 1,
    }
}

/// The value of a string literal argument without its quotes, or the text of a
/// command-syntax word.
fn literal<'a>(arg: tree_sitter::Node, source: &'a str) -> Option<&'a str> {
    let text = &source[arg.start_byte()..arg.end_byte()];
    match arg.kind() {
        "string" => text.get(1..text.len().saturating_sub(1)),
        "command_argument" => Some(text),
        _ => None,
    }
}

/// Whether an argument is text: a quoted literal, a command-syntax word, or a
/// variable that every assignment in its scope gives a text literal (see
/// `variable_kinds`).
fn is_text_argument(arg: tree_sitter::Node, source: &str) -> bool {
    if arg.kind() == "command_argument" {
        return true;
    }
    let kinds = if arg.kind() == "identifier" {
        crate::suggested_improvements::variable_kinds(arg, source)
    } else {
        crate::suggested_improvements::argument_kinds(arg, source).to_vec()
    };
    kinds.contains(&"text")
}

/// Whether an entry's argument conditions hold for a call with these arguments.
fn applies(
    entry: &CompatEntry,
    args: &[tree_sitter::Node],
    outputs: usize,
    source: &str,
    script: bool,
) -> bool {
    if entry.script_only && !script {
        return false;
    }
    if entry.first_arg_text
        && !args
            .first()
            .is_some_and(|arg| is_text_argument(*arg, source))
    {
        return false;
    }
    if entry.first_arg_not_text
        && args
            .first()
            .is_some_and(|arg| is_text_argument(*arg, source))
    {
        return false;
    }
    if entry.min_args.is_some_and(|min| args.len() < min)
        || entry.max_args.is_some_and(|max| args.len() > max)
        || entry.min_outputs.is_some_and(|min| outputs < min)
    {
        return false;
    }
    let words: Vec<&str> = args
        .iter()
        .filter_map(|arg| literal(*arg, source))
        .collect();
    let given = |value: &&str| words.iter().any(|word| word.eq_ignore_ascii_case(value));
    if entry.pairs_from > 0 {
        // Name-value pairs from `pairs_from` on: the names must be one of `args_any`.
        let Some(rest) = args.get(entry.pairs_from - 1..) else {
            return false;
        };
        if rest.len() % 2 != 0 {
            return false;
        }
        let names: Vec<&str> = rest
            .iter()
            .step_by(2)
            .filter_map(|arg| literal(*arg, source))
            .collect();
        if !entry.args_any.is_empty()
            && !entry
                .args_any
                .iter()
                .any(|value| names.iter().any(|name| name.eq_ignore_ascii_case(value)))
        {
            return false;
        }
    } else if !entry.args_any.is_empty() && !entry.args_any.iter().any(given) {
        return false;
    }
    if entry.number_arg_at > 0
        && !args.get(entry.number_arg_at - 1).is_some_and(|arg| {
            crate::suggested_improvements::argument_kinds(*arg, source).contains(&"number")
        })
    {
        return false;
    }
    if !entry.args_all.iter().all(given) || entry.unless_args.iter().any(given) {
        return false;
    }
    if !entry.unless_all_args.is_empty() && entry.unless_all_args.iter().all(given) {
        return false;
    }
    if !entry.first_arg_any.is_empty()
        && !args
            .first()
            .and_then(|arg| literal(*arg, source))
            .is_some_and(|word| {
                entry
                    .first_arg_any
                    .iter()
                    .any(|v| word.eq_ignore_ascii_case(v))
            })
    {
        return false;
    }
    if args.len() == 1
        && words.len() == 1
        && entry
            .unless_sole_arg
            .iter()
            .any(|v| words[0].eq_ignore_ascii_case(v))
    {
        return false;
    }
    if !entry.second_arg_any.is_empty()
        && !args
            .get(1)
            .is_some_and(|arg| second_arg_is_one_of(*arg, entry.second_arg_any, source))
    {
        return false;
    }
    match entry.second_arg {
        "string" => args.get(1).is_some_and(|arg| arg.kind() == "string"),
        "not_string" => args.get(1).is_some_and(|arg| arg.kind() != "string"),
        "optimoptions" => args
            .get(1)
            .is_some_and(|arg| is_optimoptions_value(*arg, source)),
        _ => true,
    }
}

/// Whether an argument is a call of `optimoptions`, or a variable every assignment of
/// which is one.
fn is_optimoptions_value(arg: tree_sitter::Node, source: &str) -> bool {
    let is_call = |node: tree_sitter::Node| {
        node.kind() == "function_call"
            && node
                .child_by_field_name("name")
                .is_some_and(|name| &source[name.start_byte()..name.end_byte()] == "optimoptions")
    };
    if arg.kind() == "identifier" {
        return crate::suggested_improvements::variable_values(arg, source)
            .is_some_and(|values| values.into_iter().all(is_call));
    }
    is_call(arg)
}

/// Whether an argument is a string literal equal (with case) to one of `words`, or a
/// variable every assignment of which is such a literal.
fn second_arg_is_one_of(arg: tree_sitter::Node, words: &[&str], source: &str) -> bool {
    let matches = |node: tree_sitter::Node| {
        node.kind() == "string" && literal(node, source).is_some_and(|text| words.contains(&text))
    };
    if arg.kind() == "identifier" {
        return crate::suggested_improvements::variable_values(arg, source)
            .is_some_and(|values| values.into_iter().all(matches));
    }
    matches(arg)
}

/// Parse a severity string from the TOML data into a `Severity` enum value.
fn parse_severity(s: &str) -> Severity {
    match s {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        "info" => Severity::Info,
        _ => Severity::Warning,
    }
}

/// Determine the category from the entry's category string.
///
/// This is used internally for documentation/reference but the engine always
/// reports `Category::Compatibility` as its own category for registry purposes.
/// Individual diagnostics carry the specific check ID.
#[allow(dead_code)]
fn parse_category(s: &str) -> Category {
    match s {
        "compatibility" => Category::Compatibility,
        "forward-compatibility" => Category::ForwardCompatibility,
        "behavior-changes" => Category::BehaviorChanges,
        _ => Category::Compatibility,
    }
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "COMPAT",
    CompatibilityEngine::from_config
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
        CompatibilityEngine::from_config(&Config::default())
    }

    /// Assert that calling `name` (function-call form) emits a diagnostic with
    /// the expected check ID and nothing else.
    fn assert_call_fires(name: &str, expected_id: &str) {
        let source = format!("{name}(x);\n");
        let diags = lint_nodes(&*engine(), &source);
        assert!(
            has_id(&diags, expected_id),
            "expected {expected_id} for `{name}(x)`; got: {diags:?}"
        );
        assert!(
            !has_id(&diags, "COMPAT"),
            "diagnostic should carry the specific check ID, not COMPAT; got: {diags:?}"
        );
    }

    /// Assert that `name` in command form (`name arg`) emits the expected ID.
    fn assert_command_fires(name: &str, expected_id: &str) {
        let source = format!("{name} arg\n");
        let diags = lint_nodes(&*engine(), &source);
        assert!(
            has_id(&diags, expected_id),
            "expected {expected_id} for `{name} arg`; got: {diags:?}"
        );
        assert!(
            !has_id(&diags, "COMPAT"),
            "diagnostic should carry the specific check ID, not COMPAT; got: {diags:?}"
        );
    }

    // -- function_call form: deprecated signal-processing names ---------------

    #[test]
    fn psd_fires_dpsd() {
        assert_call_fires("psd", "DPSD");
    }

    #[test]
    fn inline_fires_dinln() {
        assert_call_fires("inline", "DINLN");
    }

    #[test]
    fn fcnchk_is_not_reported_by_matlab() {
        // Probed in MATLAB R2026b: `fcnchk` is silent in every form, so DFCNCHK is unverified.
        assert!(!fires("fcnchk(@sin);\n", "DFCNCHK"));
    }

    #[test]
    fn matlabpool_fires_dpool() {
        assert_call_fires("matlabpool", "DPOOL");
        assert!(!has_id(
            &lint_nodes(&*engine(), "matlabpool('open')\n"),
            "MATPOOL"
        ));
        assert!(!has_id(&lint_nodes(&*engine(), "parpool\n"), "DPOOL"));
    }

    #[test]
    fn treedisp_fires_treedisp() {
        assert_call_fires("treedisp", "TREEDISP");
    }

    #[test]
    fn treefit_fires_treefit() {
        assert_call_fires("treefit", "TREEFIT");
    }

    #[test]
    fn bitmax_fires_dbitmax() {
        assert_call_fires("bitmax", "DBITMAX");
    }

    #[test]
    fn colordef_fires_colordef() {
        assert_call_fires("colordef", "COLORDEF");
    }

    #[test]
    fn whitebg_fires_whitebg() {
        assert_call_fires("whitebg", "WHITEBG");
    }

    // -- command form --------------------------------------------------------

    #[test]
    fn command_form_matlabpool_fires_dpool() {
        assert_command_fires("matlabpool", "DPOOL");
    }

    #[test]
    fn command_form_whitebg_fires_whitebg() {
        assert_command_fires("whitebg", "WHITEBG");
    }

    #[test]
    fn command_form_colordef_fires_colordef() {
        assert_command_fires("colordef", "COLORDEF");
    }

    #[test]
    fn command_form_mupad_fires_mupad() {
        assert_command_fires("mupad", "MUPAD");
    }

    // -- multi-function names emit ALL matching check IDs --------------------

    #[test]
    fn multi_function_name_emits_all_matching_ids() {
        // `tcpip` maps to TCPC (client, the default role) and TCPS (server). Both
        // entries stay in the lookup table; each one applies to its own role.
        let diags = lint_nodes(&*engine(), "tcpip('host', 80);\n");
        assert!(has_id(&diags, "TCPC"), "expected TCPC; got: {diags:?}");
        assert!(
            !has_id(&diags, "TCPS"),
            "TCPS needs the server role; got: {diags:?}"
        );
    }

    #[test]
    fn tcpip_server_role_fires_tcps_only() {
        // Probed in MATLAB R2026b: `tcpip(1,'NetworkRole','server')` is TCPS only;
        // `tcpip(1,'server')` (no NetworkRole) is TCPC.
        assert!(fires("tcpip('x', 1, 'NetworkRole', 'server');\n", "TCPS"));
        assert!(!fires("tcpip('x', 1, 'NetworkRole', 'server');\n", "TCPC"));
        assert!(fires("tcpip(1, 'server');\n", "TCPC"));
        assert!(!fires("tcpip(1, 'server');\n", "TCPS"));
    }

    #[test]
    fn opengl_info_alone_fires_opgli_only() {
        // Probed: `opengl('info')` is OPGLI; `opengl(1,'info')` is not (OPGLO, a
        // different entry); the info argument must come first.
        assert!(fires("opengl('info');\n", "OPGLI"));
        assert!(!fires("opengl(1, 'info');\n", "OPGLI"));
        assert!(!fires("opengl('info', 'data');\n", "OPGLI"));
    }

    #[test]
    fn dry_ice_first_argument_fires_dryice() {
        // Probed: `earthSurfacePermittivity('dry-ice')` and `('dry-ice', 1)` are
        // reported under the function name, `(1, 'dry-ice')` is not.
        assert!(fires(
            "x = earthSurfacePermittivity('dry-ice');\n",
            "DRYICE"
        ));
        assert!(fires("earthSurfacePermittivity('dry-ice', 1);\n", "DRYICE"));
        assert!(!fires(
            "earthSurfacePermittivity(1, 'dry-ice');\n",
            "DRYICE"
        ));
    }

    #[test]
    fn feature_eightycolumns_must_be_the_first_argument() {
        assert!(fires("feature('EightyColumns');\n", "EITYCN"));
        assert!(fires("feature('EightyColumns', 1);\n", "EITYCN"));
        assert!(!fires("feature('x', 'EightyColumns');\n", "EITYCN"));
    }

    #[test]
    fn optimoptions_option_needs_its_value() {
        // Probed: `optimoptions('fmincon','CheckGradients')` without a value is silent.
        assert!(fires(
            "optimoptions('fmincon', 'CheckGradients', true);\n",
            "CHKGR"
        ));
        assert!(!fires(
            "optimoptions('fmincon', 'CheckGradients');\n",
            "CHKGR"
        ));
        assert!(fires(
            "optimoptions('linprog', 'Algorithm', 'simplex');\n",
            "LINPROGS"
        ));
        assert!(!fires(
            "optimoptions('linprog', 'Algorithm', 'dual-simplex');\n",
            "LINPROGS"
        ));
    }

    #[test]
    fn web_handle_output_needs_two_outputs() {
        // Probed: `[s, x] = web(1)` is WEBREMOVE; `x = web(1)` is silent.
        assert!(fires("[s, x] = web(1);\n", "WEBREMOVE"));
        assert!(!fires("x = web(1);\n", "WEBREMOVE"));
    }

    #[test]
    fn qammod_phase_argument_needs_three_arguments() {
        // Probed: `qammod(1,4,0)` is QAMDEPM; `qammod(1,4)` is silent.
        assert!(fires("qammod(1, 4, 0);\n", "QAMDEPM"));
        assert!(!fires("qammod(1, 4);\n", "QAMDEPM"));
        assert!(fires("qamdemod(1, 4, 0);\n", "QAMDEPD"));
        assert!(!fires("qamdemod(1, 4);\n", "QAMDEPD"));
    }

    #[test]
    fn ismethod_text_command_word_fires_imclass() {
        // Probed: the command form `ismethod x abc` is IMCLASS (a command word is text).
        assert!(fires("ismethod x abc;\n", "IMCLASS"));
        assert!(fires("ismethod('x', 'abc');\n", "IMCLASS"));
    }

    #[test]
    fn removed_linprog_algorithms_fire_only_in_optimoptions() {
        // Probed in MATLAB R2026b: the removed algorithm is reported on the
        // `optimoptions('linprog', ...)` call, never on the solver call itself.
        assert!(fires(
            "o = optimoptions('linprog', 'Algorithm', 'simplex');\n",
            "LINPROGS"
        ));
        assert!(fires(
            "o = optimoptions('linprog', 'Algorithm', 'active-set');\n",
            "LINPROGA"
        ));
        assert!(fires(
            "o = optimoptions('linprog', 'Algorithm', 'dual-simplex-legacy');\n",
            "LINPROGD"
        ));
        assert!(!fires(
            "o = optimoptions('linprog', 'Algorithm', 'dual-simplex');\n",
            "LINPROGS"
        ));
        assert!(!fires("x = linprog(f, A, b);\n", "LINPROGS"));
    }

    #[test]
    fn checkgradients_option_fires_only_when_given() {
        assert!(fires(
            "o = optimoptions('fmincon', 'CheckGradients', true);\n",
            "CHKGR"
        ));
        assert!(!fires(
            "o = optimoptions('fmincon', 'Algorithm', 'interior-point');\n",
            "CHKGR"
        ));
    }

    #[test]
    fn min_outputs_legend_fires_only_with_several_outputs() {
        // Probed in MATLAB R2026b: two or more outputs, with `~` counted as an output.
        assert!(fires("[a, b] = legend('x', 'y');\n", "LEGMOP"));
        assert!(fires("[~, ~, ~, ~] = legend('a');\n", "LEGMOP"));
        assert!(!fires("h = legend('a', 'b');\n", "LEGMOP"));
        assert!(!fires("legend('a', 'b');\n", "LEGMOP"));
    }

    #[test]
    fn savefig_compact_fires_only_as_a_third_argument() {
        // Probed in MATLAB R2026b: `compact` after the handle and the file name.
        assert!(fires("savefig(gcf, 'x.fig', 'compact');\n", "SVFIGC"));
        assert!(fires("savefig(gcf, 'x.fig', 'compact', 'x');\n", "SVFIGC"));
        assert!(!fires("savefig(gcf, 'x.fig');\n", "SVFIGC"));
        assert!(!fires("savefig('x.fig', 'COMPACT');\n", "SVFIGC"));
    }

    #[test]
    fn axis_state_fires_only_with_the_state_argument() {
        // Probed in MATLAB R2026b: the command and function forms, with an axes
        // handle, and in any case. Other arguments are not removed.
        assert!(fires("axis('state')\n", "AXSTATE"));
        assert!(fires("axis state\n", "AXSTATE"));
        assert!(fires("axis(ax, 'state');\n", "AXSTATE"));
        assert!(fires("axis('STATE');\n", "AXSTATE"));
        assert!(!fires("axis tight\n", "AXSTATE"));
        assert!(!fires("axis xy\n", "AXSTATE"));
        assert!(!fires("axis([0 1 0 1]);\n", "AXSTATE"));
    }

    #[test]
    fn discard_option_of_drawnow_fires_only_with_that_argument() {
        assert!(fires("drawnow discard\n", "DNDLA"));
        assert!(fires("drawnow('discard');\n", "DNDLA"));
        assert!(!fires("drawnow\n", "DNDLA"));
        assert!(!fires("drawnow update\n", "DNDLA"));
        assert!(!fires("drawnow nocallbacks\n", "DNDLA"));
    }

    #[test]
    fn legend_text_changes_are_not_reported() {
        // MATLAB R2026b did not report the empty-text or property-name label
        // behavior changes in any probed form, so their entries are unverified.
        assert!(!fires("legend('', 'b');\n", "LENEMP"));
        assert!(!fires("legend('Location', 'northeast');\n", "LENEMP"));
        assert!(!fires("legend('Location', 'northeast');\n", "LEGPVPAIR"));
        assert!(!fires(
            "legend('data', 'Location', 'northeast');\n",
            "LEGPVPAIR"
        ));
    }

    // -- negative tests: modern replacements do not fire ---------------------

    #[test]
    fn modern_periodogram_not_flagged() {
        let diags = lint_nodes(&*engine(), "periodogram(x);\n");
        assert!(!has_id(&diags, "DPSD"), "got: {diags:?}");
        assert!(diags.is_empty(), "got: {diags:?}");
    }

    #[test]
    fn modern_strcmp_not_flagged() {
        let diags = lint_nodes(&*engine(), "strcmp('a', 'b');\n");
        assert!(diags.is_empty(), "got: {diags:?}");
    }

    #[test]
    fn modern_plot_not_flagged() {
        let diags = lint_nodes(&*engine(), "plot(x, y);\n");
        assert!(diags.is_empty(), "got: {diags:?}");
    }

    #[test]
    fn modern_fitctree_not_flagged() {
        let diags = lint_nodes(&*engine(), "fitctree(x, y);\n");
        assert!(!has_id(&diags, "TREEFIT"), "got: {diags:?}");
    }

    #[test]
    fn modern_command_disp_not_flagged() {
        let diags = lint_nodes(&*engine(), "disp hello\n");
        assert!(diags.is_empty(), "got: {diags:?}");
    }

    #[test]
    fn deprecated_name_as_variable_not_flagged() {
        let diags = lint_nodes(&*engine(), "psd = 5;\n");
        assert!(!has_id(&diags, "DPSD"), "got: {diags:?}");
    }

    #[test]
    fn removed_toolbox_functions_fire_their_ids() {
        // Probed in MATLAB R2026b without the Communications, Wireless or WSDL toolboxes.
        let cases = [
            ("evdoForwardReferenceChannels", "EVDOFRC"),
            ("evdoForwardWaveformGenerator", "EVDOFWG"),
            ("evdoReverseReferenceChannels", "EVDORRC"),
            ("evdoReverseWaveformGenerator", "EVDORWG"),
            ("cdma2000ForwardReferenceChannels", "CDMA2KFRC"),
            ("cdma2000ForwardWaveformGenerator", "CDMA2KFWG"),
            ("cdma2000ReverseReferenceChannels", "CDMA2KRRC"),
            ("cdma2000ReverseWaveformGenerator", "CDMA2KRWG"),
            ("algintrlv", "ALGINTRLV"),
            ("algdeintrlv", "ALGDEINTRLV"),
            ("helscanintrlv", "HELSCANINTRLV"),
            ("helscandeintrlv", "HELSCANDEINTRLV"),
            ("muxintrlv", "MUXINTRLV"),
            ("muxdeintrlv", "MUXDEINTRLV"),
            ("comm.HelicalInterleaver", "COMMSOHELINTRLV"),
            ("comm.HelicalDeinterleaver", "COMMSOHELDEINTRLV"),
            ("comm.MultiplexedInterleaver", "COMMSOMUXINTRLV"),
            ("comm.MultiplexedDeinterleaver", "COMMSOMUXDEINTRLV"),
            ("pre6GPDSCHConfig", "PDSCHCONFIG"),
            ("matlab.wsdl.createWSDLClient", "WSDLCLIENT"),
            ("matlab.wsdl.setWSDLToolPath", "WSDLTOOL"),
        ];
        for (name, id) in cases {
            assert_call_fires(name, id);
        }
    }

    #[test]
    fn low_reliability_entries_follow_the_preset() {
        // MATLAB lists JAPIEXT15 under "Behavior Changes with Low Reliability
        // Messages" and its default checkcode does not report it.
        let source = "x = org.ros.address();\n";
        let default = CompatibilityEngine::from_config(&Config::default());
        assert!(!has_id(
            &crate::test_util::lint_file(default.as_ref(), source),
            "JAPIEXT15"
        ));
        let all = CompatibilityEngine::from_config(
            &Config::from_toml("[lint]\npreset = \"all\"\n").unwrap(),
        );
        let diags: Vec<_> = {
            let tree = crate::test_util::parse(source);
            let mut out = Vec::new();
            let root = tree.root_node();
            let mut cursor = root.walk();
            let mut stack = vec![root];
            while let Some(n) = stack.pop() {
                let ctx = malt_core::NodeContext {
                    node: n,
                    source,
                    file_path: std::path::Path::new("t.m"),
                    ancestors: None,
                };
                out.extend(all.check(&ctx));
                stack.extend(n.children(&mut cursor));
            }
            out
        };
        assert!(has_id(&diags, "JAPIEXT15"), "got: {diags:?}");
    }

    /// Whether linting `source` with the engine reports `id`.
    fn fires(source: &str, id: &str) -> bool {
        has_id(&lint_nodes(&*engine(), source), id)
    }

    #[test]
    fn variable_of_the_same_name_shadows_the_function() {
        // `psd` is a variable on every line, so none of the uses calls the function.
        assert!(!fires("psd = 5;\nx = psd(3);\npsd x\n", "DPSD"));
    }

    #[test]
    fn parameter_shadows_the_function_in_its_function_only() {
        let source = "function r = f(psd)\nr = psd(2);\nend\nfunction r = g(a)\nr = psd(a);\nend\n";
        let diags = lint_nodes(&*engine(), source);
        assert_eq!(
            diags.iter().filter(|d| d.rule_id == "DPSD").count(),
            1,
            "got: {diags:?}"
        );
    }

    #[test]
    fn local_function_shadows_the_function() {
        assert!(!fires(
            "y = inline(2);\nfunction y = inline(a)\ny = a;\nend\n",
            "DINLN"
        ));
    }

    #[test]
    fn min_args_nargoutchk_fires_with_three_arguments() {
        assert!(!fires("nargoutchk(0, 1);\n", "NCHKNO"));
        assert!(fires("nargoutchk(0, 1, 'x');\n", "NCHKNO"));
    }

    #[test]
    fn args_any_print_option_fires_only_for_that_option() {
        assert!(fires("print('-dps', 'x.ps');\n", "PRINTPS1"));
        assert!(fires("print -dps x.ps\n", "PRINTPS1"));
        assert!(!fires("print('-deps', 'x.ps');\n", "PRINTPS1"));
        assert!(!fires("print(1);\n", "PRINTPS1"));
    }

    #[test]
    fn fdtag_needs_one_of_the_exploration_tags() {
        // Plan 6.2 (dogfooding): `findall` alone is not the behavior change.
        let all = CompatibilityEngine::from_config(
            &Config::from_toml("[lint]\npreset = \"all\"\n").unwrap(),
        );
        let ids = |source: &str| {
            crate::test_util::lint_nodes(all.as_ref(), source)
                .iter()
                .filter(|d| d.rule_id == "FDTAG")
                .count()
        };
        assert_eq!(ids("h = findall(groot, 'Type', 'figure');\n"), 0);
        assert_eq!(ids("h = findall(gcf, 'Tag', 'Exploration.Pan');\n"), 1);
    }

    #[test]
    fn args_any_matches_literals_without_case() {
        assert!(fires("t = textscan(s, '%d', 'bufsize', 5);\n", "BUFSIZE"));
        assert!(!fires("t = textscan(s, '%d');\n", "BUFSIZE"));
        assert!(fires("feature eightycolumns\n", "EITYCN"));
        assert!(!fires("feature('version')\n", "EITYCN"));
    }

    #[test]
    fn unless_args_opengl_info_and_data_are_not_software_options() {
        let info = lint_nodes(&*engine(), "opengl('info');\n");
        assert!(
            has_id(&info, "OPGLI") && !has_id(&info, "OPGLO"),
            "got: {info:?}"
        );
        let data = lint_nodes(&*engine(), "opengl data\n");
        assert!(
            has_id(&data, "OPGLD") && !has_id(&data, "OPGLO"),
            "got: {data:?}"
        );
        assert!(fires("opengl software\n", "OPGLO"));
    }

    #[test]
    fn args_all_intlinprog_option_on_optimoptions() {
        assert!(fires(
            "o = optimoptions('intlinprog', 'Algorithm', 'basic');\n",
            "INTLLEG1"
        ));
        assert!(!fires(
            "o = optimoptions('fmincon', 'Algorithm', 'basic');\n",
            "INTLLEG1"
        ));
        assert!(!fires("k = intlinprog(1);\n", "INTLLEG1"));
    }

    #[test]
    fn second_arg_shape_for_the_removed_solve_forms() {
        assert!(fires("x = solve(p, 'linprog');\n", "OPTMSLV"));
        assert!(fires("x = solve(p, optimoptions('linprog'));\n", "OPTMOPT"));
        assert!(fires("x = solve(p, 'linprog', o);\n", "OPTMNVP"));
        // The name-value form and an unknown second argument are not removed.
        assert!(!fires("x = solve(p, 'Options', o);\n", "OPTMSLV"));
        assert!(!fires("x = solve(p, 'Options', o);\n", "OPTMNVP"));
        assert!(!fires("x = solve(p, o);\n", "OPTMOPT"));
        assert!(!fires("x = solve(p);\n", "OPTMSLV"));
    }

    #[test]
    fn solve_forms_follow_matlab_probes() {
        // MATLAB probes (optm1/optm2): OPTMSLV only for the solvers linprog and
        // intlinprog, with case, also through a variable; OPTMOPT for an optimoptions
        // call or a variable assigned one, whatever the solver.
        assert!(fires("x = solve(p, 'linprog');\n", "OPTMSLV"));
        assert!(fires("x = solve(p, 'intlinprog');\n", "OPTMSLV"));
        assert!(fires("x = solve(p, \"linprog\");\n", "OPTMSLV"));
        assert!(fires("s = 'linprog';\nx = solve(p, s);\n", "OPTMSLV"));
        for solver in [
            "quadprog", "fmincon", "fminunc", "lsqlin", "ga", "LINPROG", "bogus",
        ] {
            assert!(
                !fires(&format!("x = solve(p, '{solver}');\n"), "OPTMSLV"),
                "{solver}"
            );
        }
        assert!(fires(
            "o = optimoptions('fmincon');\nx = solve(p, o);\n",
            "OPTMOPT"
        ));
        assert!(fires("x = solve(p, optimoptions('linprog'));\n", "OPTMOPT"));
        assert!(!fires("x = solve(p, o);\n", "OPTMOPT"));
        assert!(!fires("o = 3;\nx = solve(p, o);\n", "OPTMOPT"));
        assert!(fires("x = solve(p, 'linprog', o);\n", "OPTMNVP"));
        assert!(!fires("x = solve(p, 'fmincon', o);\n", "OPTMNVP"));
    }

    #[test]
    fn bare_nargout_fires_in_a_script_only() {
        // MATLAB probes: `nargout(f)` and `nargout('max')` in a script report nothing
        // (only the bare `nargout` of a script does, which malt does not match yet).
        assert!(!fires("nOut = max(0, nargout(task.Fcn));\n", "FINSNO"));
        assert!(!fires("x = nargout('max');\n", "FINSNO"));
        assert!(!fires(
            "function r = f(fcn)\nr = nargout(fcn);\nend\n",
            "FINSNO"
        ));
    }

    #[test]
    fn ismethod_fires_on_a_text_first_argument_only() {
        // MATLAB probes: `ismethod` with a char or string literal, or a variable
        // assigned one, reports IMCLASS; an object, a parameter or `class(x)` does not.
        assert!(fires("c = ismethod('abc', 'x');\n", "IMCLASS"));
        assert!(fires("s = 'abc';\nc = ismethod(s, 'x');\n", "IMCLASS"));
        assert!(fires(
            "str = \"abc\";\nc = ismethod(str, 'x');\n",
            "IMCLASS"
        ));
        assert!(!fires(
            "function f(mdl)\nc = ismethod(mdl, 'x');\nend\n",
            "IMCLASS"
        ));
        assert!(!fires("c = ismethod(class(mdl), 'x');\n", "IMCLASS"));
    }

    #[test]
    fn script_only_inputname_fires_in_a_script_only() {
        assert!(fires("x = inputname(1);\n", "FINSI"));
        assert!(!fires(
            "function r = f(a)\nr = inputname(1);\nend\n",
            "FINSI"
        ));
    }

    #[test]
    fn object_members_are_not_matched_by_the_constructor() {
        let diags = lint_nodes(&*engine(), "t = tcpclient('a', 1);\nv = visadev('x');\n");
        assert!(diags.is_empty(), "got: {diags:?}");
    }

    #[test]
    fn constructors_are_not_matched_by_their_member_entries() {
        // MATLAB R2026b reports none of the removed properties and methods of these
        // classes on the constructor call (probed).
        for source in [
            "s = serialport('COM1', 9600);\n",
            "t = tcpserver(5000);\n",
            "u = udpport('datagram');\n",
            "b = bluetooth('name', 1);\n",
        ] {
            assert!(lint_nodes(&*engine(), source).is_empty(), "{source}");
        }
    }

    /// Probed in MATLAB R2026b: `@psd` gives no message, `@dsp.DCT` does.
    #[test]
    fn handle_form_follows_the_entry_flag() {
        assert!(!has_id(&lint_nodes(&*engine(), "h = @psd;\n"), "DPSD"));
        assert!(has_id(&lint_nodes(&*engine(), "k = @dsp.DCT;\n"), "DSPDCT"));
    }

    /// Probed (c3): a handle is reported at the name after the `@` (column 6 of
    /// `h = @comm.gpu.AWGNChannel;`), not at the `@`.
    #[test]
    fn handle_is_reported_at_the_name_after_the_at_sign() {
        let diags = lint_nodes(&*engine(), "h = @comm.gpu.AWGNChannel;\n");
        let found = diags
            .iter()
            .find(|d| d.rule_id == "COMMGPUAWGN")
            .expect("COMMGPUAWGN");
        assert_eq!((found.line, found.column), (1, 6));
    }

    /// Probed: `cnncodegen('net')` is silent, `cnncodegen(net)` is reported.
    #[test]
    fn cnncodegen_text_first_argument_is_silent() {
        assert!(!has_id(
            &lint_nodes(&*engine(), "cnncodegen('net');\n"),
            "CNNCGD"
        ));
        assert!(has_id(
            &lint_nodes(&*engine(), "cnncodegen(net);\n"),
            "CNNCGD"
        ));
    }

    /// Probed: the phase `pi/4` is silent, a number literal (`0`, `0.7854`) is reported.
    #[test]
    fn qammod_phase_must_be_a_number_literal() {
        let qammod = "y = qammod(x, 4, 0, 'gray');\n";
        assert!(has_id(&lint_nodes(&*engine(), qammod), "QAMDEPM"));
        let pi_phase = "y = qammod(x, 4, pi/4, 'gray');\n";
        assert!(!has_id(&lint_nodes(&*engine(), pi_phase), "QAMDEPM"));
        let demod = "y = qamdemod(x, 4, 0, 'gray');\n";
        assert!(has_id(&lint_nodes(&*engine(), demod), "QAMDEPD"));
        assert!(!has_id(
            &lint_nodes(&*engine(), "y = qamdemod(x, 4, pi/4);\n"),
            "QAMDEPD"
        ));
    }

    /// Probed: `Bufsize` is reported as an option name in a complete pair list only.
    #[test]
    fn textscan_bufsize_must_be_a_name_in_a_complete_pair_list() {
        let fires = |source: &str| has_id(&lint_nodes(&*engine(), source), "BUFSIZE");
        assert!(fires("c = textscan(fid, '%f', 'Bufsize', 100);\n"));
        assert!(fires(
            "c = textscan(fid, '%f', 'bufsize', 100, 'Delimiter', ',');\n"
        ));
        assert!(!fires("c = textscan(fid, '%f', 'Bufsize');\n"));
        assert!(!fires("c = textscan(fid, '%f', 'Delimiter', 'Bufsize');\n"));
        assert!(!fires(
            "c = textscan(fid, '%f', 'Bufsize', 100, 'Delimiter');\n"
        ));
    }

    /// Probed: any member of `com.mathworks` is reported; the package itself and other
    /// packages are not.
    #[test]
    fn com_mathworks_members_fire_the_namespace_check() {
        assert!(has_id(
            &lint_nodes(&*engine(), "x = com.mathworks.util.X;\n"),
            "JAPIMATHWORKS"
        ));
        assert!(!has_id(
            &lint_nodes(&*engine(), "x = com.mathworks;\n"),
            "JAPIMATHWORKS"
        ));
        assert!(!has_id(
            &lint_nodes(&*engine(), "x = com.example.b(1);\n"),
            "JAPIMATHWORKS"
        ));
    }

    #[test]
    fn handles_and_bare_dotted_names_are_references() {
        // MATLAB R2026b reports a removed function used as a handle or as a bare
        // dotted statement (probed).
        for (source, id) in [
            ("k = @dsp.DCT;\n", "DSPDCT"),
            ("dsp.DCT;\n", "DSPDCT"),
            ("h = @comm.gpu.AWGNChannel;\n", "COMMGPUAWGN"),
        ] {
            assert!(has_id(&lint_nodes(&*engine(), source), id), "{source}");
        }
    }

    #[test]
    fn handle_to_a_variable_of_the_same_name_is_silent() {
        let diags = lint_nodes(&*engine(), "psd = 3;\nh = @psd;\n");
        assert!(!has_id(&diags, "DPSD"), "{diags:?}");
    }

    #[test]
    fn positional_optional_arguments_need_a_non_string_second_argument() {
        assert!(has_id(
            &lint_nodes(&*engine(), "y = autocorr(x, 2);\n"),
            "ACORR"
        ));
        assert!(!has_id(
            &lint_nodes(&*engine(), "y = autocorr(x);\n"),
            "ACORR"
        ));
        assert!(!has_id(
            &lint_nodes(&*engine(), "y = autocorr(x, 'NumLags', 2);\n"),
            "ACORR"
        ));
    }

    #[test]
    fn instrhwinfo_is_reported_for_its_literal_only() {
        assert!(has_id(
            &lint_nodes(&*engine(), "h = instrhwinfo('tcpip');\n"),
            "INSTHWT"
        ));
        assert!(!has_id(
            &lint_nodes(&*engine(), "h = instrhwinfo('visa');\n"),
            "INSTHWT"
        ));
        assert!(!has_id(
            &lint_nodes(&*engine(), "h = instrhwinfo('tcpip');\n"),
            "INSTHWV"
        ));
    }

    #[test]
    fn legend_with_several_outputs_is_reported_without_parentheses() {
        assert!(has_id(
            &lint_nodes(&*engine(), "[h, a] = legend;\n"),
            "LEGMOP"
        ));
        assert!(has_id(
            &lint_nodes(&*engine(), "[h, a] = legend('a');\n"),
            "LEGMOP"
        ));
        assert!(!has_id(&lint_nodes(&*engine(), "h = legend;\n"), "LEGMOP"));
    }

    #[test]
    fn mapoutline_with_a_referencing_matrix_has_a_numeric_first_argument() {
        assert!(has_id(
            &lint_nodes(&*engine(), "m = mapoutline(z, R);\n"),
            "MAOUTL"
        ));
        assert!(!has_id(
            &lint_nodes(&*engine(), "m = mapoutline('z', R);\n"),
            "MAOUTL"
        ));
        assert!(!has_id(&lint_nodes(&*engine(), "mapoutline;\n"), "MAOUTL"));
    }

    #[test]
    fn hpfilter_positional_form_has_two_arguments() {
        assert!(has_id(
            &lint_nodes(&*engine(), "y = hpfilter(x, 1600);\n"),
            "HPFLTR"
        ));
        assert!(!has_id(
            &lint_nodes(&*engine(), "y = hpfilter(x, 1600, 2);\n"),
            "HPFLTR"
        ));
    }

    #[test]
    fn cnncodegen_is_reported_for_its_target_library() {
        assert!(has_id(
            &lint_nodes(&*engine(), "cnncodegen(net, 'targetlib', 'cudnn');\n"),
            "CNNCGC"
        ));
        assert!(!has_id(
            &lint_nodes(&*engine(), "cnncodegen(net, 'targetlib', 'tensorrt');\n"),
            "CNNCGC"
        ));
    }
}
