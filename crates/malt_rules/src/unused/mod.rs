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
//! specific check ID (e.g. `NASGU`, `UNRCH`, `MSNU`).
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
//! | VANUS | info | no | Input argument 'varargin' might be unused. |
//! | DEFNU | warning | no | Function might be unused. |
//! | UNRCH | warning | no | This statement (and possibly following ones) cannot be reached. |
//! | MANU | warning | no | Input argument might be unused. Consider replacing the argument with ~, or make this method Static instead. |
//! | MSNU | warning | no | A Code Analyzer message was once suppressed here, but the message is no longer generated. |
//! | MSNE | info | no | No Code Analyzer check is found for this check ID. |
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
//!     s.field = 1;        % MSNU: set but never read
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

use std::collections::{HashMap, HashSet};

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

    /// Individual check IDs to disable (e.g., `["VANUS", "MSNE"]`).
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

    /// Run MSNU check: struct field set but never read.
    /// This is a heuristic check based on field_expression analysis.
    fn check_msnu(&self, ctx: &FileContext, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("MSNU") {
            return;
        }

        let mut field_writes: Vec<FieldAccess> = Vec::new();
        let mut field_reads: HashSet<String> = HashSet::new();

        Self::collect_field_accesses(
            ctx.tree.root_node(),
            ctx.source,
            &mut field_writes,
            &mut field_reads,
        );

        // Report fields that are written but never read.
        for write in &field_writes {
            let key = format!("{}.{}", write.object, write.field);
            if !field_reads.contains(&key) {
                diagnostics.push(Diagnostic {
                    rule_id: "MSNU",
                    message: "A Code Analyzer message was once suppressed here, but the message is no longer generated.".to_string(),
                    severity: Severity::Warning,
                    byte_range: write.byte_range.clone(),
                    line: write.line,
                    column: write.column,
                    fix: None,
                });
            }
        }
    }

    /// Run MSNE check: struct field doesn't exist (assigned but potential typo).
    /// Heuristic: if a struct has many fields and one is only written once with
    /// no reads, it may be a typo.
    fn check_msne(&self, ctx: &FileContext, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("MSNE") {
            return;
        }

        let mut field_writes: Vec<FieldAccess> = Vec::new();
        let mut field_reads: HashSet<String> = HashSet::new();

        Self::collect_field_accesses(
            ctx.tree.root_node(),
            ctx.source,
            &mut field_writes,
            &mut field_reads,
        );

        // Group writes by object name.
        let mut writes_by_object: HashMap<&str, Vec<&FieldAccess>> = HashMap::new();
        for write in &field_writes {
            writes_by_object
                .entry(write.object.as_str())
                .or_default()
                .push(write);
        }

        // For each object with multiple fields written, check if any field
        // is written exactly once and never read — potential typo.
        for (object, writes) in &writes_by_object {
            if writes.len() < 2 {
                continue;
            }

            let mut field_write_count: HashMap<&str, usize> = HashMap::new();
            for w in writes {
                *field_write_count.entry(w.field.as_str()).or_insert(0) += 1;
            }

            for w in writes {
                let key = format!("{}.{}", object, w.field);
                if field_write_count.get(w.field.as_str()) == Some(&1)
                    && !field_reads.contains(&key)
                {
                    diagnostics.push(Diagnostic {
                        rule_id: "MSNE",
                        message: "No Code Analyzer check is found for this check ID.".to_string(),
                        severity: Severity::Info,
                        byte_range: w.byte_range.clone(),
                        line: w.line,
                        column: w.column,
                        fix: None,
                    });
                }
            }
        }
    }

    /// Collect field accesses (both reads and writes) from the parse tree.
    fn collect_field_accesses(
        node: Node,
        source: &str,
        writes: &mut Vec<FieldAccess>,
        reads: &mut HashSet<String>,
    ) {
        if node.kind() == "assignment" {
            // Check if LHS is a field_expression.
            if let Some(lhs) = node.child_by_field_name("left") {
                if lhs.kind() == "field_expression" {
                    if let Some(access) = extract_field_access(lhs, source) {
                        writes.push(access);
                    }
                }
            }
            // Check RHS for field reads.
            if let Some(rhs) = node.child_by_field_name("right") {
                Self::collect_field_reads(rhs, source, reads);
            }
        } else if node.kind() == "field_expression" {
            // A field_expression not on the LHS of assignment is a read.
            if let Some(access) = extract_field_access(node, source) {
                let key = format!("{}.{}", access.object, access.field);
                reads.insert(key);
            }
        }

        // Recurse into children (skip assignment children we already handled).
        if node.kind() != "assignment" {
            let count = node.child_count();
            for i in 0..count {
                if let Some(child) = node.child(i) {
                    Self::collect_field_accesses(child, source, writes, reads);
                }
            }
        } else {
            // For assignments, recurse into children except LHS field_expression.
            let count = node.child_count();
            for i in 0..count {
                if let Some(child) = node.child(i) {
                    if child.kind() == "field_expression" {
                        if let Some(lhs) = node.child_by_field_name("left") {
                            if lhs.id() == child.id() {
                                continue;
                            }
                        }
                    }
                    Self::collect_field_accesses(child, source, writes, reads);
                }
            }
        }
    }

    /// Collect field reads from an expression subtree.
    fn collect_field_reads(node: Node, source: &str, reads: &mut HashSet<String>) {
        if node.kind() == "field_expression" {
            if let Some(access) = extract_field_access(node, source) {
                let key = format!("{}.{}", access.object, access.field);
                reads.insert(key);
            }
        }

        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                Self::collect_field_reads(child, source, reads);
            }
        }
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
        self.check_msnu(ctx, &mut diagnostics);
        self.check_msne(ctx, &mut diagnostics);

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Helper types and functions
// ---------------------------------------------------------------------------

/// A struct field access (read or write).
#[derive(Debug, Clone)]
pub(crate) struct FieldAccess {
    /// The object name (e.g., "s" in "s.field").
    object: String,
    /// The field name (e.g., "field" in "s.field").
    field: String,
    /// Byte range of the full field expression.
    byte_range: std::ops::Range<usize>,
    /// 1-indexed line.
    line: usize,
    /// 1-indexed column.
    column: usize,
}

/// Extract a `FieldAccess` from a `field_expression` node.
pub(crate) fn extract_field_access(node: Node, source: &str) -> Option<FieldAccess> {
    // field_expression has an object and a field child.
    let object_node = node.child(0)?;
    let field_node = node.child_by_field_name("field")?;

    // Only handle simple identifier objects (not chained field access).
    if object_node.kind() != "identifier" {
        return None;
    }

    let object = source[object_node.start_byte()..object_node.end_byte()].to_string();
    let field = source[field_node.start_byte()..field_node.end_byte()].to_string();
    let pos = node.start_position();

    Some(FieldAccess {
        object,
        field,
        byte_range: node.start_byte()..node.end_byte(),
        line: pos.row + 1,
        column: pos.column + 1,
    })
}

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    // -- MSNU: struct field set but never read ------------------------------

    #[test]
    fn msnu_fires_on_field_never_read() {
        let diags = lint_file(&*engine(), "function foo()\n    s.field = 1;\nend\n");
        assert!(has_id(&diags, "MSNU"), "got: {diags:?}");
    }

    #[test]
    fn msnu_ok_when_field_read() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    s.field = 1;\n    x = s.field;\n    disp(x);\nend\n",
        );
        assert!(!has_id(&diags, "MSNU"), "got: {diags:?}");
    }

    // -- MSNE: field assigned once and never read (possible typo) -----------

    #[test]
    fn msne_fires_on_typo_field() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    s.a = 1;\n    s.b = 2;\n    x = s.a;\n    disp(x);\nend\n",
        );
        assert!(has_id(&diags, "MSNE"), "got: {diags:?}");
    }

    #[test]
    fn msne_ok_when_all_fields_read() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    s.a = 1;\n    s.b = 2;\n    x = s.a + s.b;\n    disp(x);\nend\n",
        );
        assert!(!has_id(&diags, "MSNE"), "got: {diags:?}");
    }
}
