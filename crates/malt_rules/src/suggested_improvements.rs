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
        let kinds = argument_kinds(*arg, source);
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
fn argument_kinds(arg: tree_sitter::Node, source: &str) -> &'static [&'static str] {
    match arg.kind() {
        "number" => &["number"],
        "cell" => &["cell"],
        "string" if source[arg.byte_range()].starts_with('\'') => &["char", "text"],
        "string" => &["text"],
        _ => &[],
    }
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
}
