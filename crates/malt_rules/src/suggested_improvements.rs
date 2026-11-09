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
    /// The usage is only discouraged with at least this many arguments
    /// (`eval(s, catch_s)`). 0 means any call.
    #[serde(default)]
    min_args: usize,
    /// The usage is only discouraged when one argument is one of these string
    /// literals (`rand('seed', 1)`), compared case-insensitively. Empty means
    /// any call.
    #[serde(default)]
    literal_args: Vec<String>,
}

impl SuggestedEntry {
    /// Whether the call `node` is the discouraged usage, not just a call to
    /// the function.
    fn applies_to(&self, node: tree_sitter::Node, source: &str) -> bool {
        if self.min_args == 0 && self.literal_args.is_empty() {
            return true;
        }
        let mut cursor = node.walk();
        let args = node.children(&mut cursor).find(|c| c.kind() == "arguments");
        // A command-syntax call carries no parsed arguments to inspect.
        let Some(args) = args else {
            return false;
        };
        if args.named_child_count() < self.min_args {
            return false;
        }
        if self.literal_args.is_empty() {
            return true;
        }
        let mut cursor = args.walk();
        let matched = args.named_children(&mut cursor).any(|a| {
            a.kind() == "string" && {
                let text = source[a.byte_range()].trim_matches(|c| c == '\'' || c == '"');
                self.literal_args
                    .iter()
                    .any(|l| l.eq_ignore_ascii_case(text))
            }
        });
        matched
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

/// Lookup table: function_name → reference to `SuggestedEntry`.
///
/// Only entries with a non-empty `function_name` are included. For function
/// names that map to multiple check IDs (e.g., "maketform" maps to MTFA1,
/// MTFA2, MTFP1, MTFP2, MTFB), only the first entry is stored — the engine
/// will emit the first matching diagnostic.
static SUGGESTED_TABLE: LazyLock<HashMap<&'static str, &'static SuggestedEntry>> =
    LazyLock::new(|| {
        let data = &*SUGGESTED_DATA;
        let mut map = HashMap::with_capacity(data.checks.len());
        for entry in &data.checks {
            if !entry.function_name.is_empty() {
                // Use the first entry for a given function_name (don't overwrite).
                map.entry(entry.function_name.as_str()).or_insert(entry);
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

/// Target node types for the suggested improvements engine.
const TARGET_NODES: &[&str] = &["function_call", "command"];

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
        // Replacement hints are noisy for general runs; opt in explicitly.
        false
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        // Extract the function/command name from the node.
        let func_name = match extract_func_name(ctx.node, ctx.source) {
            Some(name) => name,
            None => return Vec::new(),
        };

        // Lookup in the suggested improvements table.
        let entry = match SUGGESTED_TABLE.get(func_name) {
            Some(e) => e,
            None => return Vec::new(),
        };

        if !entry.applies_to(ctx.node, ctx.source) {
            return Vec::new();
        }

        let start = ctx.node.start_position();

        // The entry's `id` field is a &str pointing into the static
        // SuggestedData. Since SUGGESTED_DATA is a LazyLock static, the
        // String's backing memory lives for 'static, so we can safely
        // transmute the &str lifetime.
        // SAFETY: SUGGESTED_DATA is a static LazyLock; its contents are never
        // deallocated, so the &str reference is valid for 'static.
        let rule_id: &'static str = unsafe { &*(entry.id.as_str() as *const str) };

        vec![Diagnostic {
            rule_id,
            message: entry.message.clone(),
            severity: Severity::Info,
            byte_range: ctx.node.start_byte()..ctx.node.end_byte(),
            line: start.row + 1,
            column: start.column + 1,
            fix: None,
        }]
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
            if crate::analysis::symbols::is_qualified_call(node) {
                return None;
            }
            let name_node = node.child_by_field_name("name")?;
            Some(&source[name_node.start_byte()..name_node.end_byte()])
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
    fn format_command_fires_formatnoi() {
        let diags = lint_nodes(&*engine(), "format short\n");
        assert!(has_id(&diags, "FORMATNOI"), "got: {diags:?}");
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
}
