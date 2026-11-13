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
//! Implements the "Incomplete Analysis" checks from MATLAB's Code Analyzer:
//! the limits at which the analysis stops being reliable. Too many messages
//! (TMMSG, reported by the linter, which sees every rule), too many parse errors
//! (TMSMS), parentheses nested too deeply (MDEEP), functions nested too deeply
//! (DEEPN), block comments nested too deeply (DEEPC), statements nested too
//! deeply (NOSPC), and an expression with too many nested operators (MXASET).
//! The thresholds are MATLAB's, measured with `checkcode` (R2026b) on generated
//! files. They cannot be disabled by the user.
//!
//! Some MATLAB checks of this category were never reported in the probes, so
//! malt does not emit them: MBIG (no report up to 16 MB), TEXTL (no report for
//! lines or string literals of 5 MB), DEEPS (nesting beyond the NOSPC limit gives
//! NOSPC only), EOFER (more than 1,000 syntax errors give TMSMS only), and EOFMI
//! (a file ending in an unfinished statement gives SYNER). MDOTM and MDMCR are
//! kept: `checkcode` raises an error instead of reporting them for a file that is
//! not a `.m` file, so only the message text is taken from MATLAB.
//!
//! malt's own safety limit on the depth of the parse tree is not a MATLAB check.
//! Code nested beyond it gets a `MALT_TOO_DEEP` diagnostic and is not analysed
//! (see `Linter::lint`).
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | TMMSG | error | no | More than 10,000 Code Analyzer messages were generated, leading to some being deleted. |
//! | TMSMS | error | no | More than 1,000 parse error messages were generated, leading to some being deleted. |
//! | MXASET | error | no | The file is too complex to analyze. Simplify the code to improve code maintainability. For example, reduce the number of operations in expressions. |
//! | NOSPC | error | no | The file is too complex to analyze. Refactor the code to improve code maintainability. For example, reduce the nesting level of conditions or functions. |
//! | MDOTM | error | no | Unable to run code analysis. VAR_FILE has an invalid file extension. |
//! | MDMCR | error | no | Unable to run code analysis. VAR_FILE is a deployed MATLAB file. |
//! | MDEEP | error | no | Parentheses, brackets, and braces are nested too deeply. |
//! | DEEPC | error | no | Block comments are nested too deeply. |
//! | DEEPN | error | no | Functions are nested too deeply. |
//! | EOFER | error | no | Code analysis did not complete. File contains too many syntax errors. |
//!
//! EOFER is emitted by the syntax engine, for an `end` after a statement with no
//! separator in a script (`syntax_errors`). This engine never emits it: MATLAB reported
//! TMSMS and no EOFER for more than 1,000 errors.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! % MDEEP: 33 nested parentheses (MATLAB accepts 32)
//! z = ((((((((((((((((((((((((((((((((((1))))))))))))))))))))))))))))))))));
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! z = ((((((((((((((((((((((((((((((((1))))))))))))))))))))))))))))))));
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.INCOMPLETE_ANALYSIS]
//! max_parse_errors = 1000
//! max_paren_depth = 32
//! max_function_depth = 17
//! ```

mod check_block_comments;
mod check_extension;
mod check_function_depth;
mod check_incomplete_file;
mod check_nesting_depth;
mod check_operator_depth;
mod check_paren_depth;
mod check_parse_errors;

use malt_core::{Category, Config, Diagnostic, FileContext, Rule, Severity};
use serde::Deserialize;
use tree_sitter::Node;

// ---------------------------------------------------------------------------
// Default threshold functions
// ---------------------------------------------------------------------------

/// MATLAB reports TMSMS once more than 1,000 parse errors are generated.
const fn default_max_parse_errors() -> usize {
    1000
}
/// MATLAB reports MDEEP from 33 nested parentheses, brackets, braces or call
/// arguments on.
const fn default_max_paren_depth() -> usize {
    32
}
/// MATLAB reports DEEPN from the 18th nested function on.
const fn default_max_function_depth() -> usize {
    17
}

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for incomplete analysis checks. Each threshold is MATLAB's
/// measured limit.
///
/// Deserialized from the `[lint.rules.INCOMPLETE_ANALYSIS]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct IncompleteAnalysisConfig {
    /// Maximum parse error (ERROR node) count before TMSMS fires.
    #[serde(default = "default_max_parse_errors")]
    pub max_parse_errors: usize,

    /// Maximum parentheses/bracket nesting depth before MDEEP fires.
    #[serde(default = "default_max_paren_depth")]
    pub max_paren_depth: usize,

    /// Maximum nested `function_definition` depth before DEEPN fires.
    #[serde(default = "default_max_function_depth")]
    pub max_function_depth: usize,
}

impl Default for IncompleteAnalysisConfig {
    fn default() -> Self {
        Self {
            max_parse_errors: default_max_parse_errors(),
            max_paren_depth: default_max_paren_depth(),
            max_function_depth: default_max_function_depth(),
        }
    }
}

// ---------------------------------------------------------------------------
// Metrics collected during tree traversal
// ---------------------------------------------------------------------------

/// Metrics gathered in a single traversal of the parse tree.
#[derive(Debug, Default)]
pub(crate) struct TreeMetrics {
    /// Number of ERROR nodes.
    error_count: usize,
    /// Maximum depth of parentheses/bracket nesting (`(`, `[`, `{`).
    max_paren_depth: usize,
    /// Maximum depth of nested `function_definition` nodes.
    max_function_depth: usize,
    /// Line of the function at `max_function_depth` (where DEEPN is reported).
    max_function_line: usize,
    /// Largest weighted nesting of statements on one path (NOSPC).
    max_statement_weight: usize,
    /// Largest number of binary operators nested in one another (MXASET).
    max_operator_depth: usize,
    /// Whether the last child of the root is an ERROR or MISSING node.
    last_node_is_error: bool,
}

/// The nesting of the node being visited, counted from the root.
#[derive(Debug, Clone, Copy, Default)]
struct Depths {
    /// Parentheses, brackets, braces and call arguments.
    paren: usize,
    /// Nested `function_definition`s.
    function: usize,
    /// Weighted nesting of statements (see `statement_weight`).
    statement: usize,
    /// Nested binary operators.
    operator: usize,
}

/// MATLAB's NOSPC weight of a statement, in quarter levels. Measured on
/// generated files: a path of 248 `if`s (weight 992) is fine and 249 (996) is
/// reported; 124 `switch`es (992) are fine and 125 (1000) are reported; 246 `if`s
/// with 3 `for`s (993) are fine and 247 `if`s with 2 `for`s (994) are reported.
fn statement_weight(kind: &str) -> usize {
    match kind {
        "if_statement" => 4,
        "for_statement" | "while_statement" => 3,
        "try_statement" => 2,
        "switch_statement" => 8,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Incomplete Analysis Engine: a file-level rule that handles the
/// incomplete-analysis checks in a single pass.
///
/// These checks represent MATLAB's own analysis limits and cannot be disabled
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

        Self::walk_tree(root, &mut metrics, Depths::default());
        metrics
    }

    /// Recursive DFS traversal that collects error counts and the nesting
    /// measures of each check in a single pass.
    fn walk_tree(node: Node, metrics: &mut TreeMetrics, parent: Depths) {
        if node.is_error() {
            metrics.error_count += 1;
        }

        let line = node.start_position().row + 1;
        let mut depths = parent;
        match node.kind() {
            "parenthesis" | "arguments" | "matrix" | "cell" => {
                depths.paren += 1;
                metrics.max_paren_depth = metrics.max_paren_depth.max(depths.paren);
            }
            "function_definition" => {
                depths.function += 1;
                if depths.function > metrics.max_function_depth {
                    metrics.max_function_depth = depths.function;
                    metrics.max_function_line = line;
                }
            }
            "if_statement" | "for_statement" | "while_statement" | "switch_statement"
            | "try_statement" => {
                depths.statement += statement_weight(node.kind());
                metrics.max_statement_weight = metrics.max_statement_weight.max(depths.statement);
            }
            "binary_operator" => {
                depths.operator += 1;
                metrics.max_operator_depth = metrics.max_operator_depth.max(depths.operator);
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_tree(child, metrics, depths);
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

        // -- Single-pass tree traversal --

        let metrics = self.collect_metrics(root);

        // MXASET: an expression with too many nested binary operators.
        self.check_operator_depth(&metrics, source, &mut diagnostics);

        // TMSMS: Too many parse errors.
        self.check_parse_errors(&metrics, source, &mut diagnostics);

        // NOSPC: statements nested too deeply (weighted by statement kind).
        self.check_nesting_depth(&metrics, source, &mut diagnostics);

        // MDEEP: Parentheses/brackets nested too deeply.
        self.check_paren_depth(&metrics, source, &mut diagnostics);

        // DEEPN: Functions nested too deeply.
        self.check_function_depth(&metrics, source, &mut diagnostics);

        // EOFMI: Incomplete file (last node is ERROR or MISSING).
        self.check_incomplete_file(&metrics, source, &mut diagnostics);

        // -- Line-based checks --

        // DEEPC: Block comments nested too deeply.
        diagnostics.extend(self.check_nested_block_comments(source));

        // MDOTM / MDMCR: invalid or deployed file extension.
        self.check_extension(ctx, source, &mut diagnostics);

        // -- No-op checks (handled elsewhere, documented here for completeness) --
        // TMMSG: Too many messages. The linter counts every rule's diagnostics.
        // QUIT:  Analysis did not complete. The engine panic guard lives in the linter core.
        // NOFIL: File not found. The CLI handles file discovery.
        // RDERR: Unable to read file. The CLI handles I/O errors.

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
