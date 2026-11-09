//! # UNUSED_ENGINE: Unused Constructions
//!
//! ```malt
//! id = "UNUSED_ENGINE"
//! title = "Unused Constructions"
//! category = "unused-constructions"
//! severity = "warning"
//! fix = false
//! icon = "lucide/trash-2"
//! slug = "unused"
//! ```
//!
//! ## Rule
//!
//! Detects unused variables, functions, methods, and dead code in MATLAB
//! files. A single file-level engine (`UnusedEngine`) builds a symbol table
//! and runs unreachability analysis, then cross-references every definition
//! against its usages to identify dead code. It covers 17 checks from
//! MATLAB's "Unused Constructions" category; each diagnostic carries the
//! specific check ID (e.g. `NASGU`, `UNRCH`).
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | NASGU | warning | no | Value assigned to variable might be unused. |
//! | NUSED | warning | no | Global or persistent variable might be unused or unset in this function or script. |
//! | NOEFF | warning | no | The operation or expression VAR_OPERATOR has no evident effect. |
//! | EQEFF | warning | no | To assign values to variables, use =. The == operator compares equality of values. |
//! | ASGLU | warning | no | Value assigned to variable might be unused. Consider replacing the variable with ~ instead. |
//! | SETNU | warning | no | Variable is set, but might be unused. |
//! | PUSE | warning | no | Persistent variable might be unused. |
//! | PREALL | warning | no | The preallocated value assigned to variable might be unused. |
//! | INUSA | warning | no | Input argument might be unused after the function arguments block(s). |
//! | INUSD | warning | no | Input argument might be unused. Consider replacing the argument with ~ instead. |
//! | VANUS | warning | no | Input argument 'varargin' might be unused. |
//! | DEFNU | warning | no | Function might be unused. |
//! | UNRCH | warning | no | This statement (and possibly following ones) cannot be reached. |
//! | MANU | warning | no | Input argument might be unused. Consider replacing the argument with ~, or make this method Static instead. |
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! function f()
//!     temp = compute();   % NASGU: assigned but never used
//!     x = 1;
//!     return;
//!     y = x;              % UNRCH: unreachable after return
//! end
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! function f()
//!     x = compute();
//!     disp(x);
//!     s.field = x;
//!     disp(s.field);
//! end
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.UNUSED_ENGINE]
//! severity = "warning"
//! ignore_patterns = ["_*", "unused*"]
//! disabled_checks = ["VANUS"]
//! ```

use std::collections::HashSet;

use malt_core::{Category, Config, Diagnostic, FileContext, Rule, Severity};
use serde::Deserialize;
use tree_sitter::Node;

use crate::analysis::control_flow::find_unreachable;
use crate::analysis::symbols::{DefKind, ScopeKind, SymbolTable};

// ---------------------------------------------------------------------------
// Submodules: one file per independent check
// ---------------------------------------------------------------------------

mod check_asglu;
mod check_defnu;
mod check_manu;
mod check_nasgu;
mod check_noeff_eqeff;
mod check_nused;
mod check_preall;
mod check_puse;
mod check_setnu;
mod check_unrch;
mod check_unused_inputs;
mod check_vanus;

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for UNUSED_ENGINE.
///
/// Deserialized from the `[lint.rules.UNUSED_ENGINE]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UnusedConfig {
    /// Variable name patterns to ignore (e.g., `["_*", "unused*"]`).
    /// Supports simple glob: `*` matches any sequence, `?` matches one char.
    #[serde(default)]
    pub ignore_patterns: Vec<String>,

    /// Individual check IDs to disable (e.g., `["VANUS", "DEFNU"]`).
    #[serde(default)]
    pub disabled_checks: Vec<String>,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Variables that are always considered "used" — MATLAB built-in specials.
pub(crate) const ALWAYS_USED: &[&str] = &["varargin", "varargout", "nargin", "nargout"];

/// Statement-level parent node types.
pub(crate) const STATEMENT_PARENTS: &[&str] = &["source_file", "block"];

/// Node types representing discarded expressions at statement level.
pub(crate) const NO_EFFECT_EXPR_NODES: &[&str] = &[
    "binary_operator",
    "unary_operator",
    "postfix_operator",
    "number",
    "string",
    "identifier",
    "cell",
    "matrix",
    "range",
    "lambda",
    "field_expression",
    "boolean_operator",
];

/// Node type for comparison operators (result discarded = EQEFF).
pub(crate) const COMPARISON_NODE: &str = "comparison_operator";

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// File-level rule engine for detecting unused constructions.
///
/// This rule performs its analysis in `check_file()` by building a symbol table
/// and running unreachability analysis, then cross-referencing definitions with
/// usages to identify dead code.
pub struct UnusedEngine {
    config: UnusedConfig,
}

impl UnusedEngine {
    /// Factory constructor. Reads rule-specific params from config.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: UnusedConfig = config.rule_params("UNUSED_ENGINE");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Check if a check ID is disabled in config.
    fn is_check_disabled(&self, check_id: &str) -> bool {
        self.config
            .disabled_checks
            .iter()
            .any(|d| d.eq_ignore_ascii_case(check_id))
    }

    /// Check if a variable name should be ignored based on configured patterns.
    fn should_ignore_name(&self, name: &str) -> bool {
        // Always ignore `_` and names starting with `~`.
        if name == "_" || name.starts_with('~') {
            return true;
        }
        // Always ignore MATLAB built-in specials.
        if ALWAYS_USED.contains(&name) {
            return true;
        }
        // Check configured patterns.
        self.config
            .ignore_patterns
            .iter()
            .any(|pat| matches_simple_glob(pat, name))
    }
}

impl Rule for UnusedEngine {
    fn id(&self) -> &'static str {
        "UNUSED_ENGINE"
    }

    fn description(&self) -> &'static str {
        "Detect unused variables, functions, and dead code"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn category(&self) -> Category {
        Category::UnusedConstructions
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        // File-level rule — no per-node targeting.
        &[]
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // 1. Build symbol table.
        let table = SymbolTable::build(ctx.tree, ctx.source);

        // 2. Run symbol-based checks.
        self.check_nasgu(&table, &mut diagnostics);
        self.check_unused_inputs(&table, &mut diagnostics);
        self.check_asglu(&table, &mut diagnostics);
        self.check_nused(&table, &mut diagnostics);
        self.check_puse(&table, &mut diagnostics);
        self.check_setnu(&table, &mut diagnostics);
        self.check_preall(&table, &mut diagnostics);
        self.check_vanus(&table, &mut diagnostics);
        self.check_defnu(&table, &mut diagnostics);
        self.check_manu(&table, &mut diagnostics);

        // 3. Run tree-walk checks.
        self.check_unrch(ctx, &mut diagnostics);
        self.check_noeff_eqeff(ctx, &mut diagnostics);

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Helper types and functions
// ---------------------------------------------------------------------------

/// Simple glob pattern matching supporting `*` (any sequence) and `?` (single char).
pub(crate) fn matches_simple_glob(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    glob_match_recursive(&pat, &txt, 0, 0)
}

/// Recursive glob matching helper.
pub(crate) fn glob_match_recursive(pat: &[char], txt: &[char], pi: usize, ti: usize) -> bool {
    if pi == pat.len() {
        return ti == txt.len();
    }

    match pat[pi] {
        '*' => {
            // `*` matches zero or more characters.
            for i in ti..=txt.len() {
                if glob_match_recursive(pat, txt, pi + 1, i) {
                    return true;
                }
            }
            false
        }
        '?' => {
            // `?` matches exactly one character.
            if ti < txt.len() {
                glob_match_recursive(pat, txt, pi + 1, ti + 1)
            } else {
                false
            }
        }
        c => {
            if ti < txt.len() && txt[ti] == c {
                glob_match_recursive(pat, txt, pi + 1, ti + 1)
            } else {
                false
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "UNUSED_ENGINE",
    UnusedEngine::from_config
));
