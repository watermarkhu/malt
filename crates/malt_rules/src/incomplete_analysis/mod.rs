//! # INCOMPLETE_ANALYSIS: Linter-Internal Limit Checks
//!
//! ```malt
//! id = "INCOMPLETE_ANALYSIS"
//! title = "Incomplete Analysis"
//! category = "incomplete-analysis"
//! severity = "error"
//! fix = false
//! icon = "lucide/wrench"
//! slug = "incomplete-analysis"
//! ```
//!
//! ## Rule
//!
//! Implements the "Incomplete Analysis" checks from MATLAB's Code Analyzer.
//! These diagnostics report when the analysis itself was limited or could not
//! complete reliably: too many diagnostics, too many parse errors, files that
//! are too large, too deeply nested, or too long. They represent linter
//! internal limits and **cannot be disabled** by the user.
//!
//! A single file-level engine (`IncompleteAnalysisEngine`) collects tree
//! metrics (node count, ERROR count, nesting depths) in one pass, then applies
//! each threshold check against the configured limits. Each diagnostic carries
//! the specific check ID (e.g. `TMMSG`, `MDEEP`).
//!
//! Three related MATLAB checks are not emitted by this engine: `QUIT`
//! (analysis did not complete — the panic guard lives in the linter core),
//! `NOFIL` (file not found), and `RDERR` (unable to read file) are handled by
//! the CLI.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | TMMSG | error | no | More than 10,000 Code Analyzer messages were generated, leading to some being deleted. |
//! | TMSMS | error | no | More than 1,000 parse error messages were generated, leading to some being deleted. |
//! | MXASET | error | no | The file is too complex to analyze. Simplify the code to improve code maintainability. For example, reduce the number of operations in expressions. |
//! | NOSPC | error | no | The file is too complex to analyze. Refactor the code to improve code maintainability. For example, reduce the nesting level of conditions or functions. |
//! | MBIG | error | no | Code analysis did not complete. File VAR_FILE is too large. |
//! | MDOTM | error | no | Unable to run code analysis. VAR_FILE has an invalid file extension. |
//! | MDMCR | error | no | Unable to run code analysis. VAR_FILE is a deployed MATLAB file. |
//! | EOFER | error | no | Code analysis did not complete. File contains too many syntax errors. |
//! | EOFMI | error | no | Invalid syntax at end of file. File is incomplete. |
//! | MDEEP | error | no | Parentheses, brackets, and braces are nested too deeply. |
//! | DEEPC | error | no | Block comments are nested too deeply. |
//! | DEEPN | error | no | Functions are nested too deeply. |
//! | DEEPS | error | no | Statements are nested too deeply. |
//! | TEXTL | error | no | Text is too long for MATLAB to parse. |
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! % EOFMI: file ends mid-block without a matching `end`
//! function f()
//!     if x > 0
//!         y = 1;
//! % MDEEP: parentheses nested deeper than the configured limit
//! z = ((((((((((((((((((((((((((((((((((((((((((1)))))))))))))))))))))))))))))))))))))))))))));
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! function f()
//!     if x > 0
//!         y = 1;
//!     end
//! end
//! z = 1;
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.INCOMPLETE_ANALYSIS]
//! max_diagnostics = 10000
//! max_parse_errors = 1000
//! max_node_count = 100000
//! max_file_size = 1048576
//! max_paren_depth = 32
//! max_function_depth = 20
//! max_statement_depth = 15
//! max_line_length = 4096
//! ```

mod check_block_comments;
mod check_diagnostic_count;
mod check_extension;
mod check_file_size;
mod check_function_depth;
mod check_incomplete_file;
mod check_line_lengths;
mod check_nesting_depth;
mod check_node_count;
mod check_paren_depth;
mod check_parse_errors;
mod check_statement_depth;

use malt_core::{Category, Config, Diagnostic, FileContext, Rule, Severity};
use serde::Deserialize;
use tree_sitter::Node;

// ---------------------------------------------------------------------------
// Default threshold functions
// ---------------------------------------------------------------------------

const fn default_max_diagnostics() -> usize {
    10000
}
const fn default_max_parse_errors() -> usize {
    1000
}
const fn default_max_node_count() -> usize {
    100000
}
const fn default_max_file_size() -> usize {
    1_048_576 // 1 MB
}
const fn default_max_paren_depth() -> usize {
    32
}
const fn default_max_function_depth() -> usize {
    20
}
const fn default_max_statement_depth() -> usize {
    15
}
const fn default_max_line_length() -> usize {
    4096
}

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for incomplete analysis checks — each threshold is configurable.
///
/// Deserialized from the `[lint.rules.INCOMPLETE_ANALYSIS]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct IncompleteAnalysisConfig {
    /// Maximum number of diagnostics before TMMSG fires.
    #[serde(default = "default_max_diagnostics")]
    pub max_diagnostics: usize,

    /// Maximum parse error (ERROR node) count before TMSMS fires.
    #[serde(default = "default_max_parse_errors")]
    pub max_parse_errors: usize,

    /// Maximum tree node count before MXASET fires.
    #[serde(default = "default_max_node_count")]
    pub max_node_count: usize,

    /// Maximum source file size in bytes before MBIG fires.
    #[serde(default = "default_max_file_size")]
    pub max_file_size: usize,

    /// Maximum parentheses/bracket nesting depth before MDEEP fires.
    #[serde(default = "default_max_paren_depth")]
    pub max_paren_depth: usize,

    /// Maximum nested `function_definition` depth before DEEPN fires.
    #[serde(default = "default_max_function_depth")]
    pub max_function_depth: usize,

    /// Maximum nested statement (if/for/while/switch/try) depth before DEEPS fires.
    #[serde(default = "default_max_statement_depth")]
    pub max_statement_depth: usize,

    /// Maximum line length in characters before TEXTL fires.
    #[serde(default = "default_max_line_length")]
    pub max_line_length: usize,
}

impl Default for IncompleteAnalysisConfig {
    fn default() -> Self {
        Self {
            max_diagnostics: default_max_diagnostics(),
            max_parse_errors: default_max_parse_errors(),
            max_node_count: default_max_node_count(),
            max_file_size: default_max_file_size(),
            max_paren_depth: default_max_paren_depth(),
            max_function_depth: default_max_function_depth(),
            max_statement_depth: default_max_statement_depth(),
            max_line_length: default_max_line_length(),
        }
    }
}

// ---------------------------------------------------------------------------
// Metrics collected during tree traversal
// ---------------------------------------------------------------------------

/// Metrics gathered in a single traversal of the parse tree.
#[derive(Debug, Default)]
pub(crate) struct TreeMetrics {
    /// Total number of nodes in the tree.
    node_count: usize,
    /// Number of ERROR nodes.
    error_count: usize,
    /// Maximum depth of parentheses/bracket nesting (`(`, `[`, `{`).
    max_paren_depth: usize,
    /// Maximum depth of nested `function_definition` nodes.
    max_function_depth: usize,
    /// Maximum depth of nested statement nodes (if/for/while/switch/try).
    max_statement_depth: usize,
    /// Whether the last child of the root is an ERROR or MISSING node.
    last_node_is_error: bool,
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Incomplete Analysis Engine: a file-level rule that handles all 17
/// incomplete-analysis checks in a single pass.
///
/// These checks represent linter-internal limits and cannot be disabled
/// by the user (`can_be_disabled() == false`).
pub struct IncompleteAnalysisEngine {
    config: IncompleteAnalysisConfig,
}

impl IncompleteAnalysisEngine {
    /// Factory constructor. Reads rule-specific params from config.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: IncompleteAnalysisConfig = config.rule_params("INCOMPLETE_ANALYSIS");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Walk the tree and collect all metrics in one pass.
    fn collect_metrics(&self, root: Node) -> TreeMetrics {
        let mut metrics = TreeMetrics::default();

        // Check last child of root for EOFMI.
        if let Some(last) = last_descendant(root) {
            metrics.last_node_is_error = last.is_error() || last.is_missing();
        }

        Self::walk_tree(root, &mut metrics, 0, 0, 0);
        metrics
    }

    /// Recursive DFS traversal that collects node counts, error counts, and
    /// nesting depths in a single pass.
    fn walk_tree(
        node: Node,
        metrics: &mut TreeMetrics,
        paren_depth: usize,
        function_depth: usize,
        statement_depth: usize,
    ) {
        metrics.node_count += 1;

        if node.is_error() {
            metrics.error_count += 1;
        }

        // Track parentheses/bracket nesting depth (MDEEP).
        let new_paren_depth = match node.kind() {
            "parenthesis" | "arguments" | "matrix" | "cell" => {
                let d = paren_depth + 1;
                if d > metrics.max_paren_depth {
                    metrics.max_paren_depth = d;
                }
                d
            }
            _ => paren_depth,
        };

        // Track function nesting depth (DEEPN).
        let new_function_depth = if node.kind() == "function_definition" {
            let d = function_depth + 1;
            if d > metrics.max_function_depth {
                metrics.max_function_depth = d;
            }
            d
        } else {
            function_depth
        };

        // Track statement nesting depth (DEEPS).
        let new_statement_depth = match node.kind() {
            "if_statement" | "for_statement" | "while_statement" | "switch_statement"
            | "try_statement" => {
                let d = statement_depth + 1;
                if d > metrics.max_statement_depth {
                    metrics.max_statement_depth = d;
                }
                d
            }
            _ => statement_depth,
        };

        // Recurse into children.
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_tree(
                child,
                metrics,
                new_paren_depth,
                new_function_depth,
                new_statement_depth,
            );
        }
    }
}

impl Rule for IncompleteAnalysisEngine {
    fn id(&self) -> &'static str {
        "INCOMPLETE_ANALYSIS"
    }

    fn description(&self) -> &'static str {
        "Internal linter limits and analysis integrity checks"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn category(&self) -> Category {
        Category::IncompleteAnalysis
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        // File-level only — no node subscriptions.
        &[]
    }

    fn can_be_disabled(&self) -> bool {
        false
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let root = ctx.tree.root_node();
        let source = ctx.source;

        // -- Checks that don't need tree traversal --

        // MBIG: File too large.
        let file_name = ctx.file_path.display().to_string();
        self.check_file_size(source, &file_name, &mut diagnostics);

        // MDOTM: Invalid file extension (not .m).
        self.check_extension(ctx, source, &mut diagnostics);

        // -- Single-pass tree traversal --

        let metrics = self.collect_metrics(root);

        // MXASET: File too complex to analyze (node count).
        self.check_node_count(&metrics, source, &mut diagnostics);

        // TMSMS: Too many parse errors.
        // EOFER: Too many syntax errors (same as TMSMS but lower threshold).
        // Fires when error count exceeds max_parse_errors (same metric, distinct ID
        // for MATLAB compatibility).
        self.check_parse_errors(&metrics, source, &mut diagnostics);

        // NOSPC: File too complex (nesting) — uses the maximum of all nesting depths.
        self.check_nesting_depth(&metrics, source, &mut diagnostics);

        // MDEEP: Parentheses/brackets nested too deeply.
        self.check_paren_depth(&metrics, source, &mut diagnostics);

        // DEEPN: Functions nested too deeply.
        self.check_function_depth(&metrics, source, &mut diagnostics);

        // DEEPS: Statements nested too deeply.
        self.check_statement_depth(&metrics, source, &mut diagnostics);

        // EOFMI: Incomplete file (last node is ERROR or MISSING).
        self.check_incomplete_file(&metrics, source, &mut diagnostics);

        // -- Line-based checks --

        // TEXTL: Text too long.
        diagnostics.extend(self.check_line_lengths(source));

        // DEEPC: Block comments nested too deeply.
        diagnostics.extend(self.check_nested_block_comments(source));

        // -- Post-hoc diagnostic count check --

        // TMMSG: Too many diagnostics generated.
        // NOTE: This counts diagnostics emitted by THIS rule only. The linter
        // may also enforce this at a higher level across all rules.
        self.check_diagnostic_count(source, &mut diagnostics);

        // -- No-op checks (handled elsewhere, documented here for completeness) --
        // QUIT:  Analysis did not complete — engine panic guard (catch_unwind in
        //        Linter::lint), emitted when an internal analyzer error occurs.
        // NOFIL: File not found — CLI handles file discovery.
        // RDERR: Unable to read file — CLI handles I/O errors.

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find the last (rightmost, deepest) descendant node in a tree.
pub(crate) fn last_descendant(node: Node) -> Option<Node> {
    let child_count = node.child_count();
    if child_count == 0 {
        return Some(node);
    }
    let last_child = node.child(child_count - 1)?;
    last_descendant(last_child)
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "INCOMPLETE_ANALYSIS",
    IncompleteAnalysisEngine::from_config
));

// ---------------------------------------------------------------------------
// Shared test helpers
// ---------------------------------------------------------------------------

/// Build an engine with default configuration for tests.
#[cfg(test)]
pub(crate) fn engine() -> Box<dyn Rule> {
    IncompleteAnalysisEngine::from_config(&Config::default())
}

/// Build an engine with a lowered/raised threshold in the config.
#[cfg(test)]
pub(crate) fn engine_with(params: &str) -> Box<dyn Rule> {
    let config =
        Config::from_toml(&format!("[lint.rules.INCOMPLETE_ANALYSIS]\n{params}\n")).unwrap();
    IncompleteAnalysisEngine::from_config(&config)
}

// ---------------------------------------------------------------------------
// Tests (generic)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};

    // -- QUIT: engine panic guard (must NOT fire for parse-level problems) ----

    #[test]
    fn quit_does_not_fire_on_clean_file() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "QUIT"), "got: {diags:?}");
    }

    #[test]
    fn quit_does_not_fire_on_syntax_error() {
        let diags = lint_file(&*engine(), "x = ;\n");
        assert!(!has_id(&diags, "QUIT"), "got: {diags:?}");
    }

    #[test]
    fn quit_does_not_fire_on_unterminated_if() {
        let diags = lint_file(&*engine(), "if x > 0\n    y = 1;\n");
        assert!(!has_id(&diags, "QUIT"), "got: {diags:?}");
    }

    #[test]
    fn quit_does_not_fire_on_incomplete_tail() {
        let diags = lint_file(&*engine(), "x = 1; 2");
        assert!(!has_id(&diags, "QUIT"), "got: {diags:?}");
    }
}
