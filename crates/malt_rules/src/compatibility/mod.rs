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
use serde::Deserialize;

/// Generic (AST-pattern) compatibility checks that the name-lookup table cannot
/// match. Each check is implemented in its own `check_*.rs` module and
/// registered here.
mod generic;

// ---------------------------------------------------------------------------
// Data types for TOML deserialization
// ---------------------------------------------------------------------------

/// A single compatibility check entry from the data file.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct CompatEntry {
    /// Check ID matching MATLAB Code Analyzer (e.g., "DPSD").
    id: String,
    /// The function/class name to match against. Empty string means generic.
    function_name: String,
    /// Severity level: "error" or "warning".
    severity: String,
    /// Human-readable diagnostic message.
    message: String,
    /// Category: "compatibility", "forward-compatibility", or "behavior-changes".
    category: String,
    /// Optional pattern marker. "generic" means this check cannot be matched
    /// by function name alone.
    #[serde(default)]
    pattern: String,
    /// MATLAB lists the entry under "(Upcoming) Behavior Changes with Low
    /// Reliability Messages" and does not report it by default.
    #[serde(default)]
    low_reliability: bool,
}

/// Top-level structure of the compatibility.toml data file.
#[derive(Debug, Deserialize)]
struct CompatData {
    checks: Vec<CompatEntry>,
}

// ---------------------------------------------------------------------------
// Static data — parsed once, lives for 'static
// ---------------------------------------------------------------------------

/// Raw TOML source embedded at compile time.
const COMPAT_TOML_SOURCE: &str = include_str!("../data/compatibility.toml");

/// Parsed compatibility data. All strings inside are owned and live for 'static
/// because this is a `LazyLock` static.
static COMPAT_DATA: LazyLock<CompatData> = LazyLock::new(|| {
    toml::from_str(COMPAT_TOML_SOURCE).expect("failed to parse data/compatibility.toml")
});

/// Lookup table: function_name → matching `CompatEntry` list.
///
/// Only entries with a non-empty `function_name` and no "generic" pattern are
/// included. A function name may map to multiple check IDs (e.g., "tcpip"
/// maps to both TCPC and TCPS); all matching entries are stored so the engine
/// emits every applicable diagnostic.
static COMPAT_TABLE: LazyLock<HashMap<&'static str, Vec<&'static CompatEntry>>> =
    LazyLock::new(|| {
        let data = &*COMPAT_DATA;
        let mut map: HashMap<&'static str, Vec<&'static CompatEntry>> =
            HashMap::with_capacity(data.checks.len());
        for entry in &data.checks {
            if !entry.function_name.is_empty() && entry.pattern != "generic" {
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

/// Target node types for the compatibility engine.
const TARGET_NODES: &[&str] = &["function_call", "command"];

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
        diagnostics
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        // Extract the function/command name from the node.
        let func_name = match extract_func_name(ctx.node, ctx.source) {
            Some(name) => name,
            None => return Vec::new(),
        };

        // Lookup in the compatibility table.
        let entries = match COMPAT_TABLE.get(func_name) {
            Some(e) => e,
            None => return Vec::new(),
        };

        // A dotted call is reported at the start of its package path (`comm` in
        // `comm.HelicalInterleaver(...)`), as MATLAB does.
        let anchor = match ctx.node.parent() {
            Some(parent)
                if ctx.node.kind() == "function_call"
                    && crate::analysis::symbols::is_qualified_call(ctx.node) =>
            {
                parent.named_child(0).unwrap_or(ctx.node)
            }
            _ => ctx.node,
        };
        let start = anchor.start_position();
        let mut diagnostics = Vec::with_capacity(entries.len());

        for entry in entries {
            if entry.low_reliability && !self.low_reliability {
                continue;
            }
            let severity = parse_severity(&entry.severity);

            // The entry's `id` field is a &str pointing into the static CompatData.
            // Since COMPAT_DATA is a LazyLock static, the String's backing memory
            // lives for 'static, so we can safely transmute the &str lifetime.
            // SAFETY: COMPAT_DATA is a static LazyLock; its contents are never
            // deallocated, so the &str reference is valid for 'static.
            let rule_id: &'static str = unsafe { &*(entry.id.as_str() as *const str) };

            diagnostics.push(Diagnostic {
                rule_id,
                message: entry.message.clone(),
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

/// Extract the function/command name from a `function_call` or `command` node.
///
/// For `function_call`: extracts the `name` field (handles simple identifiers
/// and dotted names like `dsp.FIRFilter`).
///
/// For `command`: extracts the first child (the command name).
fn extract_func_name<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    match node.kind() {
        "function_call" => {
            let name_node = node.child_by_field_name("name")?;
            // `comm.HelicalInterleaver(x)`: the call is the trailing member of a
            // dotted expression, so match the whole package path. A variable in
            // front (`s.psd(x)`) yields a path that matches no entry.
            let start = match node.parent() {
                Some(parent) if crate::analysis::symbols::is_qualified_call(node) => {
                    parent.named_child(0)?.start_byte()
                }
                _ => name_node.start_byte(),
            };
            Some(&source[start..name_node.end_byte()])
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
    fn fcnchk_fires_dfcnchk() {
        assert_call_fires("fcnchk", "DFCNCHK");
    }

    #[test]
    fn matlabpool_fires_matpool() {
        assert_call_fires("matlabpool", "MATPOOL");
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
    fn command_form_matlabpool_fires_matpool() {
        assert_command_fires("matlabpool", "MATPOOL");
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
        // `tcpip` maps to both TCPC (removed) and TCPS (behavior change); both
        // diagnostics must be emitted now that the lookup table stores a Vec.
        let diags = lint_nodes(&*engine(), "tcpip('host', 80);\n");
        assert!(has_id(&diags, "TCPC"), "expected TCPC; got: {diags:?}");
        assert!(has_id(&diags, "TCPS"), "expected TCPS; got: {diags:?}");
    }

    #[test]
    fn multi_function_name_linprog_emits_all() {
        // `linprog` maps to LINPROGS (option) and LINPROGD/LINPROGA (algorithms).
        let diags = lint_nodes(&*engine(), "x = linprog(f, A, b);\n");
        assert!(has_id(&diags, "LINPROGS"), "got: {diags:?}");
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
                };
                out.extend(all.check(&ctx));
                stack.extend(n.children(&mut cursor));
            }
            out
        };
        assert!(has_id(&diags, "JAPIEXT15"), "got: {diags:?}");
    }
}
