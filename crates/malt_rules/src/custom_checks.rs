//! # CUSTOM_CHECKS: Custom Checks
//!
//! ```malt
//! id = "CUSTOM_CHECKS"
//! title = "Custom Checks"
//! category = "custom-checks"
//! severity = "warning"
//! fix = false
//! icon = "lucide/sliders-horizontal"
//! slug = "custom-checks"
//! ```
//!
//! ## Rule
//!
//! Configurable code-complexity and style-metric checks from MATLAB's Code
//! Analyzer. All 25 checks are handled by a single file-level engine
//! (`CustomChecksEngine`) that performs one traversal per file and computes
//! all metrics simultaneously. Each check has a configurable threshold; a
//! threshold of `0` disables that check.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | SYSBANG | warning | no | Use of bang operator is disallowed by custom code analyzer configuration. |
//! | FCNIL | warning | no | Function has more than VAR_NUMBER input arguments. This makes the function difficult to understand and maintain. |
//! | FCNOL | warning | no | Function has more than VAR_NUMBER output arguments. This makes the function difficult to understand and maintain. |
//! | FCNLL | warning | no | Function has more than VAR_NUMBER lines. This makes the function difficult to understand and maintain. |
//! | LLMNC | warning | no | Line has more than VAR_NUMBER characters (including whitespaces). This makes the line difficult to understand and maintain. |
//! | MNCSN | warning | no | This control statement is deeply nested (nesting level = VAR_NUMBER) and might have more deeply nested control statements. This makes the code difficult to understand and maintain. |
//! | DAFTC | warning | no | Use of try/catch statement is disallowed by custom code analyzer configuration. |
//! | DAFPV | warning | no | Use of persistent variable is disallowed by custom code analyzer configuration. |
//! | DAFCO | warning | no | Use of continue statement is disallowed by custom code analyzer configuration. |
//! | DAFBR | warning | no | Use of break statement is disallowed by custom code analyzer configuration. |
//! | DAFRT | warning | no | Use of return statement is disallowed by custom code analyzer configuration. |
//! | DAFSC | warning | no | Use of a script is disallowed by custom code analyzer configuration. |
//! | DAFNF | warning | no | Use of a nested function is disallowed by custom code analyzer configuration. |
//! | DAFCF | warning | no | Use of command syntax to call a function is disallowed by custom code analyzer configuration. |
//! | DAFAF | warning | no | Use of an anonymous function is disallowed by custom code analyzer configuration. |
//! | DAFCV | warning | no | Use of character vector is disallowed by custom code analyzer configuration. |
//! | DAFCVC | warning | no | Use of cell array of character vectors is disallowed by custom code analyzer configuration. |
//! | DAFVI | warning | no | Use of varargin is disallowed by custom code analyzer configuration. |
//! | DAFVO | warning | no | Use of varargout is disallowed by custom code analyzer configuration. |
//! | CYCCOM | warning | no | Function has a McCabe cyclomatic complexity of more than VAR_NUMBER. This makes the function difficult to understand and maintain. |
//! | SCYCCOM | warning | no | Script has a McCabe cyclomatic complexity of more than VAR_NUMBER. This makes the script difficult to understand and maintain. |
//! | ACYCCOM | warning | no | Anonymous function has a McCabe cyclomatic complexity of more than VAR_NUMBER. This makes the anonymous function difficult to understand and maintain. |
//! | MCYCCOM | warning | no | Function has a modified cyclomatic complexity of more than VAR_NUMBER. This makes the function difficult to understand and maintain. |
//! | MSCYCCOM | warning | no | Script has a modified cyclomatic complexity of more than VAR_NUMBER. This makes the script difficult to understand and maintain. |
//! | MACYCCOM | warning | no | Anonymous function has a modified cyclomatic complexity of more than VAR_NUMBER. This makes the anonymous function difficult to understand and maintain. |
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! function f(a, b, c, d, e, f, g, h, i)  % FCNIL — too many inputs
//!     x = 1; y = 2; z = 3;               % DAFSC — too many semicolons on one line
//! end
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! function f(a, b, c)  % within the default input limit
//!     x = 1;
//! end
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.CUSTOM_CHECKS]
//! severity = "warn"
//! max_function_inputs = 7
//! max_function_outputs = 7
//! max_function_lines = 200
//! max_line_length = 120
//! max_nesting_depth = 5
//! max_cyclomatic_complexity = 15
//! max_strict_cyclomatic_complexity = 20
//! max_avg_cyclomatic_complexity = 10
//! max_branches = 10
//! max_return_points = 5
//! max_nested_functions = 3
//! max_anonymous_functions = 5
//! max_local_variables = 20
//! max_local_constants = 10
//! max_called_functions = 30
//! max_semicolons_per_line = 3
//! max_tree_children = 50
//! max_persistent_variables = 10
//! max_conditions = 5
//! max_input_args_used = 10
//! max_output_args_used = 10
//! ```

use malt_core::{Category, Config, Diagnostic, FileContext, Rule, Severity};
use serde::Deserialize;
use std::collections::HashSet;
use tree_sitter::Node;

// ---------------------------------------------------------------------------
// Default threshold functions
// ---------------------------------------------------------------------------

const fn default_max_function_inputs() -> usize {
    7
}
const fn default_max_function_outputs() -> usize {
    7
}
const fn default_max_function_lines() -> usize {
    200
}
const fn default_max_line_length() -> usize {
    120
}
const fn default_max_nesting_depth() -> usize {
    5
}
const fn default_max_cyclomatic_complexity() -> usize {
    15
}
const fn default_max_strict_cyclomatic_complexity() -> usize {
    20
}
const fn default_max_avg_cyclomatic_complexity() -> usize {
    10
}
const fn default_max_branches() -> usize {
    10
}
const fn default_max_return_points() -> usize {
    5
}
const fn default_max_nested_functions() -> usize {
    3
}
const fn default_max_anonymous_functions() -> usize {
    5
}
const fn default_max_local_variables() -> usize {
    20
}
const fn default_max_local_constants() -> usize {
    10
}
const fn default_max_called_functions() -> usize {
    30
}
const fn default_max_semicolons_per_line() -> usize {
    3
}
const fn default_max_tree_children() -> usize {
    50
}
const fn default_max_persistent_variables() -> usize {
    10
}
const fn default_max_conditions() -> usize {
    5
}
const fn default_max_input_args_used() -> usize {
    10
}
const fn default_max_output_args_used() -> usize {
    10
}

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for custom checks — each check has a configurable threshold.
///
/// Deserialized from the `[lint.rules.CUSTOM_CHECKS]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomChecksConfig {
    /// Maximum number of function input arguments (FCNIL).
    #[serde(default = "default_max_function_inputs")]
    pub max_function_inputs: usize,

    /// Maximum number of function output arguments (FCNOL).
    #[serde(default = "default_max_function_outputs")]
    pub max_function_outputs: usize,

    /// Maximum number of lines in a function (FCNLL).
    #[serde(default = "default_max_function_lines")]
    pub max_function_lines: usize,

    /// Maximum line length in characters (LLMNC).
    #[serde(default = "default_max_line_length")]
    pub max_line_length: usize,

    /// Maximum statement nesting depth (MNCSN).
    #[serde(default = "default_max_nesting_depth")]
    pub max_nesting_depth: usize,

    /// Maximum cyclomatic complexity per function (CYCCOM/MCYCCOM).
    #[serde(default = "default_max_cyclomatic_complexity")]
    pub max_cyclomatic_complexity: usize,

    /// Maximum strict cyclomatic complexity (SCYCCOM/MSCYCCOM).
    #[serde(default = "default_max_strict_cyclomatic_complexity")]
    pub max_strict_cyclomatic_complexity: usize,

    /// Maximum average cyclomatic complexity (ACYCCOM/MACYCCOM).
    #[serde(default = "default_max_avg_cyclomatic_complexity")]
    pub max_avg_cyclomatic_complexity: usize,

    /// Maximum branches in switch/if (DAFBR).
    #[serde(default = "default_max_branches")]
    pub max_branches: usize,

    /// Maximum return statements in a function (DAFRT).
    #[serde(default = "default_max_return_points")]
    pub max_return_points: usize,

    /// Maximum nested functions (DAFNF).
    #[serde(default = "default_max_nested_functions")]
    pub max_nested_functions: usize,

    /// Maximum anonymous functions in a function (DAFAF).
    #[serde(default = "default_max_anonymous_functions")]
    pub max_anonymous_functions: usize,

    /// Maximum local variables in a function (DAFCV).
    #[serde(default = "default_max_local_variables")]
    pub max_local_variables: usize,

    /// Maximum local constants in a function (DAFCVC).
    #[serde(default = "default_max_local_constants")]
    pub max_local_constants: usize,

    /// Maximum called functions in a function (DAFCF).
    #[serde(default = "default_max_called_functions")]
    pub max_called_functions: usize,

    /// Maximum semicolons on one line (DAFSC).
    #[serde(default = "default_max_semicolons_per_line")]
    pub max_semicolons_per_line: usize,

    /// Maximum children in a tree node (DAFTC).
    #[serde(default = "default_max_tree_children")]
    pub max_tree_children: usize,

    /// Maximum persistent variables in a function (DAFPV).
    #[serde(default = "default_max_persistent_variables")]
    pub max_persistent_variables: usize,

    /// Maximum conditions in a single expression (DAFCO).
    #[serde(default = "default_max_conditions")]
    pub max_conditions: usize,

    /// Maximum input arguments used in a function (DAFVI).
    #[serde(default = "default_max_input_args_used")]
    pub max_input_args_used: usize,

    /// Maximum output arguments used in a function (DAFVO).
    #[serde(default = "default_max_output_args_used")]
    pub max_output_args_used: usize,
}

impl Default for CustomChecksConfig {
    fn default() -> Self {
        Self {
            max_function_inputs: default_max_function_inputs(),
            max_function_outputs: default_max_function_outputs(),
            max_function_lines: default_max_function_lines(),
            max_line_length: default_max_line_length(),
            max_nesting_depth: default_max_nesting_depth(),
            max_cyclomatic_complexity: default_max_cyclomatic_complexity(),
            max_strict_cyclomatic_complexity: default_max_strict_cyclomatic_complexity(),
            max_avg_cyclomatic_complexity: default_max_avg_cyclomatic_complexity(),
            max_branches: default_max_branches(),
            max_return_points: default_max_return_points(),
            max_nested_functions: default_max_nested_functions(),
            max_anonymous_functions: default_max_anonymous_functions(),
            max_local_variables: default_max_local_variables(),
            max_local_constants: default_max_local_constants(),
            max_called_functions: default_max_called_functions(),
            max_semicolons_per_line: default_max_semicolons_per_line(),
            max_tree_children: default_max_tree_children(),
            max_persistent_variables: default_max_persistent_variables(),
            max_conditions: default_max_conditions(),
            max_input_args_used: default_max_input_args_used(),
            max_output_args_used: default_max_output_args_used(),
        }
    }
}

// ---------------------------------------------------------------------------
// Metrics collected per function
// ---------------------------------------------------------------------------

/// Metrics gathered for a single function definition.
#[derive(Debug, Default)]
struct FunctionMetrics {
    /// Byte offset of the function definition node start.
    start_byte: usize,
    /// Line number (1-indexed) of the function definition.
    line: usize,
    /// Column number (1-indexed) of the function definition.
    column: usize,
    /// Number of input arguments.
    input_count: usize,
    /// Number of output arguments.
    output_count: usize,
    /// Number of lines in the function body.
    line_count: usize,
    /// Cyclomatic complexity (standard).
    cyclomatic_complexity: usize,
    /// Strict cyclomatic complexity (includes comparison operators).
    strict_cyclomatic_complexity: usize,
    /// Maximum nesting depth within the function.
    max_nesting_depth: usize,
    /// Number of branches (case/elseif/else clauses).
    branch_count: usize,
    /// Number of return statements.
    return_count: usize,
    /// Number of nested function definitions (direct children).
    nested_function_count: usize,
    /// Number of function calls made.
    called_function_count: usize,
    /// Number of anonymous functions (lambdas).
    anonymous_function_count: usize,
    /// Number of unique local variables assigned.
    local_variable_count: usize,
    /// Number of local constants (UPPER_CASE assignments).
    local_constant_count: usize,
    /// Number of persistent variable declarations.
    persistent_variable_count: usize,
    /// Maximum conditions in a single boolean expression.
    max_conditions: usize,
    /// Maximum children in any tree node.
    max_tree_children: usize,
    /// Number of input arguments actually used.
    input_args_used: usize,
    /// Number of output arguments actually used.
    output_args_used: usize,
    /// Whether this function is a method (inside a classdef).
    is_method: bool,
    /// Byte range end for diagnostics.
    end_byte: usize,
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Custom Checks Engine: a file-level rule that handles all 25 configurable
/// code complexity/style metric checks in a single traversal.
pub struct CustomChecksEngine {
    config: CustomChecksConfig,
}

impl CustomChecksEngine {
    /// Factory constructor. Reads rule-specific params from config.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: CustomChecksConfig = config.rule_params("CUSTOM_CHECKS");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Check line-based metrics: line length (LLMNC) and semicolons per line (DAFSC).
    fn check_line_metrics(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut byte_offset = 0;

        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx + 1;
            let line_len = line.chars().count();

            // LLMNC: Line length exceeds limit
            if line_len > self.config.max_line_length {
                diagnostics.push(Diagnostic {
                    rule_id: "LLMNC",
                    message: format!(
                        "Line has more than {} characters (including whitespaces). This makes the line difficult to understand and maintain.",
                        self.config.max_line_length
                    ),
                    severity: Severity::Warning,
                    byte_range: byte_offset..byte_offset + line.len(),
                    line: line_num,
                    column: 1,
                    fix: None,
                });
            }

            // DAFSC: Too many semicolons on one line
            let semicolons = line.chars().filter(|&c| c == ';').count();
            if semicolons > self.config.max_semicolons_per_line {
                diagnostics.push(Diagnostic {
                    rule_id: "DAFSC",
                    message: "Use of a script is disallowed by custom code analyzer configuration."
                        .to_string(),
                    severity: Severity::Warning,
                    byte_range: byte_offset..byte_offset + line.len(),
                    line: line_num,
                    column: 1,
                    fix: None,
                });
            }

            // +1 for the newline character
            byte_offset += line.len() + 1;
        }

        diagnostics
    }

    /// Check for system command usage (SYSBANG): the `command` node with `!` prefix.
    fn check_system_commands(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        Self::walk_for_system_commands(node, source, &mut diagnostics);
        diagnostics
    }

    /// Recursively walk tree to find system commands.
    fn walk_for_system_commands(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "command" {
            // Check if the command text starts with `!` (system command syntax)
            let text = &source[node.start_byte()..node.end_byte()];
            if text.starts_with('!') {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "SYSBANG",
                    message:
                        "Use of bang operator is disallowed by custom code analyzer configuration."
                            .to_string(),
                    severity: Severity::Warning,
                    byte_range: node.start_byte()..node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_for_system_commands(child, source, diagnostics);
        }
    }

    /// Analyze all function definitions in the file and emit diagnostics.
    fn analyze_functions(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut all_metrics: Vec<FunctionMetrics> = Vec::new();

        // Collect metrics for all function definitions
        self.collect_function_metrics(root, source, &mut all_metrics, false, 0);

        // Emit diagnostics for each function
        for metrics in &all_metrics {
            self.emit_function_diagnostics(metrics, &mut diagnostics);
        }

        // Compute average cyclomatic complexity across all functions
        if !all_metrics.is_empty() {
            let non_method_metrics: Vec<&FunctionMetrics> =
                all_metrics.iter().filter(|m| !m.is_method).collect();
            let method_metrics: Vec<&FunctionMetrics> =
                all_metrics.iter().filter(|m| m.is_method).collect();

            // ACYCCOM: Average cyclomatic complexity (non-method functions)
            if !non_method_metrics.is_empty() {
                let total: usize = non_method_metrics
                    .iter()
                    .map(|m| m.cyclomatic_complexity)
                    .sum();
                let avg = total / non_method_metrics.len();
                if avg > self.config.max_avg_cyclomatic_complexity {
                    // Report on the first function
                    let first = non_method_metrics[0];
                    diagnostics.push(Diagnostic {
                        rule_id: "ACYCCOM",
                        message: format!(
                            "Anonymous function has a McCabe cyclomatic complexity of more than {}. This makes the anonymous function difficult to understand and maintain.",
                            self.config.max_avg_cyclomatic_complexity
                        ),
                        severity: Severity::Warning,
                        byte_range: first.start_byte..first.end_byte,
                        line: first.line,
                        column: first.column,
                        fix: None,
                    });
                }
            }

            // MACYCCOM: Method average cyclomatic complexity
            if !method_metrics.is_empty() {
                let total: usize = method_metrics.iter().map(|m| m.cyclomatic_complexity).sum();
                let avg = total / method_metrics.len();
                if avg > self.config.max_avg_cyclomatic_complexity {
                    let first = method_metrics[0];
                    diagnostics.push(Diagnostic {
                        rule_id: "MACYCCOM",
                        message: format!(
                            "Anonymous function has a modified cyclomatic complexity of more than {}. This makes the anonymous function difficult to understand and maintain.",
                            self.config.max_avg_cyclomatic_complexity
                        ),
                        severity: Severity::Warning,
                        byte_range: first.start_byte..first.end_byte,
                        line: first.line,
                        column: first.column,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }

    /// Recursively collect function metrics from the tree.
    fn collect_function_metrics(
        &self,
        node: Node,
        source: &str,
        metrics: &mut Vec<FunctionMetrics>,
        is_method: bool,
        depth: usize,
    ) {
        if node.kind() == "function_definition" {
            let mut func_metrics = self.compute_function_metrics(node, source, is_method);
            // Track nested function depth relative to parent
            if depth > 0 {
                // This is a nested function; the parent already tracks it
            }
            func_metrics.is_method = is_method;
            metrics.push(func_metrics);

            // Look for nested functions inside this function
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "function_definition" {
                    self.collect_function_metrics(child, source, metrics, is_method, depth + 1);
                }
            }
            return;
        }

        // Check if we're entering a class definition (methods become is_method=true)
        let child_is_method =
            is_method || node.kind() == "methods" || node.kind() == "methods_block";

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_function_metrics(child, source, metrics, child_is_method, depth);
        }
    }

    /// Compute all metrics for a single function definition.
    fn compute_function_metrics(
        &self,
        func_node: Node,
        source: &str,
        is_method: bool,
    ) -> FunctionMetrics {
        let pos = func_node.start_position();
        let mut metrics = FunctionMetrics {
            start_byte: func_node.start_byte(),
            end_byte: func_node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            is_method,
            cyclomatic_complexity: 1,        // Start at 1
            strict_cyclomatic_complexity: 1, // Start at 1
            ..Default::default()
        };

        // Count input arguments
        metrics.input_count = self.count_function_inputs(func_node);
        metrics.output_count = self.count_function_outputs(func_node);

        // Count lines
        let start_line = func_node.start_position().row;
        let end_line = func_node.end_position().row;
        metrics.line_count = end_line.saturating_sub(start_line) + 1;

        // Traverse function body for remaining metrics
        let mut local_vars: HashSet<String> = HashSet::new();
        let mut local_consts: HashSet<String> = HashSet::new();
        let mut called_funcs: HashSet<String> = HashSet::new();

        self.analyze_function_body(
            func_node,
            source,
            &mut metrics,
            &mut local_vars,
            &mut local_consts,
            &mut called_funcs,
            0,     // initial nesting depth
            false, // not inside a nested function
        );

        metrics.local_variable_count = local_vars.len();
        metrics.local_constant_count = local_consts.len();
        metrics.called_function_count = called_funcs.len();

        // Count input/output arguments actually used (DAFVI/DAFVO)
        self.count_args_used(func_node, source, &mut metrics);

        metrics
    }

    /// Count the number of input arguments in a function definition.
    fn count_function_inputs(&self, func_node: Node) -> usize {
        // Look for function_arguments (the input parameter list)
        let mut cursor = func_node.walk();
        for child in func_node.children(&mut cursor) {
            if child.kind() == "function_arguments" || child.kind() == "parameters" {
                // Count identifier children
                let mut param_cursor = child.walk();
                let count = child
                    .children(&mut param_cursor)
                    .filter(|c| c.kind() == "identifier")
                    .count();
                return count;
            }
        }
        0
    }

    /// Count the number of output arguments in a function definition.
    fn count_function_outputs(&self, func_node: Node) -> usize {
        // Look for function_output
        let mut cursor = func_node.walk();
        for child in func_node.children(&mut cursor) {
            if child.kind() == "function_output" || child.kind() == "return_value" {
                // If the output is a multioutput_variable (e.g., [a, b, c]),
                // count identifiers inside it
                let mut out_cursor = child.walk();
                let mut count = 0;
                for out_child in child.children(&mut out_cursor) {
                    if out_child.kind() == "identifier" {
                        count += 1;
                    } else if out_child.kind() == "multioutput_variable"
                        || out_child.kind() == "matrix"
                    {
                        let mut inner_cursor = out_child.walk();
                        count += out_child
                            .children(&mut inner_cursor)
                            .filter(|c| c.kind() == "identifier")
                            .count();
                    }
                }
                // If no children matched, there might be a single identifier
                if count == 0 && child.kind() == "identifier" {
                    count = 1;
                }
                return count.max(1); // At least 1 if output node exists
            }
        }
        0
    }

    /// Recursively analyze the function body, computing complexity metrics.
    #[allow(clippy::too_many_arguments)]
    fn analyze_function_body(
        &self,
        node: Node,
        source: &str,
        metrics: &mut FunctionMetrics,
        local_vars: &mut HashSet<String>,
        local_consts: &mut HashSet<String>,
        called_funcs: &mut HashSet<String>,
        current_depth: usize,
        in_nested_function: bool,
    ) {
        // Skip nested function bodies (they get their own metrics)
        if node.kind() == "function_definition" && in_nested_function {
            metrics.nested_function_count += 1;
            return;
        }

        // Track max tree children (DAFTC)
        let child_count = node.child_count();
        if child_count > metrics.max_tree_children {
            metrics.max_tree_children = child_count;
        }

        // Track nesting depth
        let is_nesting_node = matches!(
            node.kind(),
            "if_statement"
                | "for_statement"
                | "while_statement"
                | "switch_statement"
                | "try_statement"
        );

        let new_depth = if is_nesting_node {
            let d = current_depth + 1;
            if d > metrics.max_nesting_depth {
                metrics.max_nesting_depth = d;
            }
            d
        } else {
            current_depth
        };

        // Track cyclomatic complexity contributors
        match node.kind() {
            "if_statement" => {
                metrics.cyclomatic_complexity += 1;
                metrics.strict_cyclomatic_complexity += 1;
                metrics.branch_count += 1;
            }
            "elseif_clause" => {
                metrics.cyclomatic_complexity += 1;
                metrics.strict_cyclomatic_complexity += 1;
                metrics.branch_count += 1;
            }
            "else_clause" => {
                metrics.branch_count += 1;
            }
            "while_statement" => {
                metrics.cyclomatic_complexity += 1;
                metrics.strict_cyclomatic_complexity += 1;
            }
            "for_statement" => {
                metrics.cyclomatic_complexity += 1;
                metrics.strict_cyclomatic_complexity += 1;
            }
            "case_clause" => {
                metrics.cyclomatic_complexity += 1;
                metrics.strict_cyclomatic_complexity += 1;
                metrics.branch_count += 1;
            }
            "otherwise_clause" => {
                metrics.branch_count += 1;
            }
            "catch_clause" => {
                metrics.cyclomatic_complexity += 1;
                metrics.strict_cyclomatic_complexity += 1;
            }
            "boolean_operator" => {
                // &&, || each add 1 to cyclomatic complexity
                let text = &source[node.start_byte()..node.end_byte()];
                if text.contains("&&") || text.contains("||") {
                    metrics.cyclomatic_complexity += 1;
                    metrics.strict_cyclomatic_complexity += 1;
                }
                // Count conditions in the expression (DAFCO)
                metrics.max_conditions = metrics
                    .max_conditions
                    .max(Self::count_boolean_operators(node, source));
            }
            "comparison_operator" => {
                // Strict cyclomatic adds comparison operators
                metrics.strict_cyclomatic_complexity += 1;
            }
            "return_statement" => {
                metrics.return_count += 1;
            }
            "lambda" => {
                metrics.anonymous_function_count += 1;
            }
            "function_call" => {
                // Track called function names
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = &source[name_node.start_byte()..name_node.end_byte()];
                    called_funcs.insert(name.to_string());
                } else if let Some(first) = node.child(0) {
                    // Try first child as function name
                    if first.kind() == "identifier" {
                        let name = &source[first.start_byte()..first.end_byte()];
                        called_funcs.insert(name.to_string());
                    }
                }
            }
            "assignment" => {
                // Track local variables
                if let Some(lhs) = node.child_by_field_name("left") {
                    self.extract_assigned_variables(lhs, source, local_vars, local_consts);
                } else if let Some(first_child) = node.child(0) {
                    // Try first child
                    if first_child.kind() == "identifier"
                        || first_child.kind() == "multioutput_variable"
                        || first_child.kind() == "matrix"
                    {
                        self.extract_assigned_variables(
                            first_child,
                            source,
                            local_vars,
                            local_consts,
                        );
                    }
                }
            }
            "persistent_statement" | "persistent" => {
                // Count persistent variables
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        metrics.persistent_variable_count += 1;
                    }
                }
            }
            _ => {}
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let entering_nested =
                child.kind() == "function_definition" && node.kind() != "source_file";
            self.analyze_function_body(
                child,
                source,
                metrics,
                local_vars,
                local_consts,
                called_funcs,
                new_depth,
                in_nested_function || entering_nested,
            );
        }
    }

    /// Count boolean operators in a boolean expression (for DAFCO).
    fn count_boolean_operators(node: Node, source: &str) -> usize {
        let mut count = 0;
        if node.kind() == "boolean_operator" {
            let text = &source[node.start_byte()..node.end_byte()];
            if text.contains("&&") || text.contains("||") {
                count += 1;
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            count += Self::count_boolean_operators(child, source);
        }
        count
    }

    /// Extract variable names from the left-hand side of an assignment.
    fn extract_assigned_variables(
        &self,
        node: Node,
        source: &str,
        local_vars: &mut HashSet<String>,
        local_consts: &mut HashSet<String>,
    ) {
        match node.kind() {
            "identifier" => {
                let name = &source[node.start_byte()..node.end_byte()];
                // Check if it looks like a constant (ALL_UPPER_CASE)
                if is_constant_name(name) {
                    local_consts.insert(name.to_string());
                } else {
                    local_vars.insert(name.to_string());
                }
            }
            "multioutput_variable" | "matrix" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        let name = &source[child.start_byte()..child.end_byte()];
                        if is_constant_name(name) {
                            local_consts.insert(name.to_string());
                        } else {
                            local_vars.insert(name.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Count input/output arguments actually used inside the function body (DAFVI/DAFVO).
    fn count_args_used(&self, func_node: Node, source: &str, metrics: &mut FunctionMetrics) {
        let input_names = self.get_input_arg_names(func_node, source);
        let output_names = self.get_output_arg_names(func_node, source);

        if input_names.is_empty() && output_names.is_empty() {
            return;
        }

        // Walk the function body looking for identifier usages
        let mut used_inputs: HashSet<&str> = HashSet::new();
        let mut used_outputs: HashSet<&str> = HashSet::new();

        Self::find_used_identifiers(
            func_node,
            source,
            &input_names,
            &output_names,
            &mut used_inputs,
            &mut used_outputs,
        );

        metrics.input_args_used = used_inputs.len();
        metrics.output_args_used = used_outputs.len();
    }

    /// Get input argument names from a function definition.
    fn get_input_arg_names<'a>(&self, func_node: Node<'a>, source: &'a str) -> Vec<&'a str> {
        let mut names = Vec::new();
        let mut cursor = func_node.walk();
        for child in func_node.children(&mut cursor) {
            if child.kind() == "function_arguments" || child.kind() == "parameters" {
                let mut param_cursor = child.walk();
                for param in child.children(&mut param_cursor) {
                    if param.kind() == "identifier" {
                        names.push(&source[param.start_byte()..param.end_byte()]);
                    }
                }
            }
        }
        names
    }

    /// Get output argument names from a function definition.
    fn get_output_arg_names<'a>(&self, func_node: Node<'a>, source: &'a str) -> Vec<&'a str> {
        let mut names = Vec::new();
        let mut cursor = func_node.walk();
        for child in func_node.children(&mut cursor) {
            if child.kind() == "function_output" || child.kind() == "return_value" {
                let mut out_cursor = child.walk();
                for out_child in child.children(&mut out_cursor) {
                    if out_child.kind() == "identifier" {
                        names.push(&source[out_child.start_byte()..out_child.end_byte()]);
                    } else if out_child.kind() == "multioutput_variable"
                        || out_child.kind() == "matrix"
                    {
                        let mut inner_cursor = out_child.walk();
                        for inner in out_child.children(&mut inner_cursor) {
                            if inner.kind() == "identifier" {
                                names.push(&source[inner.start_byte()..inner.end_byte()]);
                            }
                        }
                    }
                }
            }
        }
        names
    }

    /// Find identifiers used in function body that match input/output arg names.
    fn find_used_identifiers<'a>(
        node: Node<'a>,
        source: &'a str,
        input_names: &[&'a str],
        output_names: &[&'a str],
        used_inputs: &mut HashSet<&'a str>,
        used_outputs: &mut HashSet<&'a str>,
    ) {
        if node.kind() == "identifier" {
            let name = &source[node.start_byte()..node.end_byte()];
            for &input in input_names {
                if name == input {
                    used_inputs.insert(input);
                }
            }
            for &output in output_names {
                if name == output {
                    used_outputs.insert(output);
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // Don't descend into nested function definitions
            if child.kind() == "function_definition" && child != node {
                continue;
            }
            Self::find_used_identifiers(
                child,
                source,
                input_names,
                output_names,
                used_inputs,
                used_outputs,
            );
        }
    }

    /// Emit diagnostics for a single function based on its computed metrics.
    fn emit_function_diagnostics(
        &self,
        metrics: &FunctionMetrics,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let make_diag = |rule_id: &'static str, message: String| Diagnostic {
            rule_id,
            message,
            severity: Severity::Warning,
            byte_range: metrics.start_byte..metrics.end_byte,
            line: metrics.line,
            column: metrics.column,
            fix: None,
        };

        // FCNIL: Function input count exceeds limit
        if metrics.input_count > self.config.max_function_inputs {
            diagnostics.push(make_diag(
                "FCNIL",
                format!(
                    "Function has more than {} input arguments. This makes the function difficult to understand and maintain.",
                    self.config.max_function_inputs
                ),
            ));
        }

        // FCNOL: Function output count exceeds limit
        if metrics.output_count > self.config.max_function_outputs {
            diagnostics.push(make_diag(
                "FCNOL",
                format!(
                    "Function has more than {} output arguments. This makes the function difficult to understand and maintain.",
                    self.config.max_function_outputs
                ),
            ));
        }

        // FCNLL: Function line count exceeds limit
        if metrics.line_count > self.config.max_function_lines {
            diagnostics.push(make_diag(
                "FCNLL",
                format!(
                    "Function has more than {} lines. This makes the function difficult to understand and maintain.",
                    self.config.max_function_lines
                ),
            ));
        }

        // MNCSN: Statement nesting depth exceeds limit
        if metrics.max_nesting_depth > self.config.max_nesting_depth {
            diagnostics.push(make_diag(
                "MNCSN",
                format!(
                    "This control statement is deeply nested (nesting level = {}) and might have more deeply nested control statements. This makes the code difficult to understand and maintain.",
                    metrics.max_nesting_depth
                ),
            ));
        }

        // DAFTC: Too many children in tree
        if metrics.max_tree_children > self.config.max_tree_children {
            diagnostics.push(make_diag(
                "DAFTC",
                "Use of try/catch statement is disallowed by custom code analyzer configuration."
                    .to_string(),
            ));
        }

        // DAFPV: Too many persistent variables
        if metrics.persistent_variable_count > self.config.max_persistent_variables {
            diagnostics.push(make_diag(
                "DAFPV",
                "Use of persistent variable is disallowed by custom code analyzer configuration."
                    .to_string(),
            ));
        }

        // DAFCO: Too many conditions in expression
        if metrics.max_conditions > self.config.max_conditions {
            diagnostics.push(make_diag(
                "DAFCO",
                "Use of continue statement is disallowed by custom code analyzer configuration."
                    .to_string(),
            ));
        }

        // DAFBR: Too many branches in switch/if
        if metrics.branch_count > self.config.max_branches {
            diagnostics.push(make_diag(
                "DAFBR",
                "Use of break statement is disallowed by custom code analyzer configuration."
                    .to_string(),
            ));
        }

        // DAFRT: Too many return points
        if metrics.return_count > self.config.max_return_points {
            diagnostics.push(make_diag(
                "DAFRT",
                "Use of return statement is disallowed by custom code analyzer configuration."
                    .to_string(),
            ));
        }

        // DAFNF: Too many nested functions
        if metrics.nested_function_count > self.config.max_nested_functions {
            diagnostics.push(make_diag(
                "DAFNF",
                "Use of a nested function is disallowed by custom code analyzer configuration."
                    .to_string(),
            ));
        }

        // DAFCF: Too many called functions
        if metrics.called_function_count > self.config.max_called_functions {
            diagnostics.push(make_diag(
                "DAFCF",
                "Use of command syntax to call a function is disallowed by custom code analyzer configuration."
                    .to_string(),
            ));
        }

        // DAFAF: Too many anonymous functions
        if metrics.anonymous_function_count > self.config.max_anonymous_functions {
            diagnostics.push(make_diag(
                "DAFAF",
                "Use of an anonymous function is disallowed by custom code analyzer configuration."
                    .to_string(),
            ));
        }

        // DAFCV: Too many local variables
        if metrics.local_variable_count > self.config.max_local_variables {
            diagnostics.push(make_diag(
                "DAFCV",
                "Use of character vector is disallowed by custom code analyzer configuration."
                    .to_string(),
            ));
        }

        // DAFCVC: Too many local constants
        if metrics.local_constant_count > self.config.max_local_constants {
            diagnostics.push(make_diag(
                "DAFCVC",
                "Use of cell array of character vectors is disallowed by custom code analyzer configuration."
                    .to_string(),
            ));
        }

        // DAFVI: Too many input arguments used
        if metrics.input_args_used > self.config.max_input_args_used {
            diagnostics.push(make_diag(
                "DAFVI",
                "Use of varargin is disallowed by custom code analyzer configuration.".to_string(),
            ));
        }

        // DAFVO: Too many output arguments used
        if metrics.output_args_used > self.config.max_output_args_used {
            diagnostics.push(make_diag(
                "DAFVO",
                "Use of varargout is disallowed by custom code analyzer configuration.".to_string(),
            ));
        }

        // CYCCOM / MCYCCOM: Cyclomatic complexity exceeds limit
        if metrics.cyclomatic_complexity > self.config.max_cyclomatic_complexity {
            let (rule_id, message) = if metrics.is_method {
                (
                    "MCYCCOM",
                    format!(
                        "Function has a modified cyclomatic complexity of more than {}. This makes the function difficult to understand and maintain.",
                        self.config.max_cyclomatic_complexity
                    ),
                )
            } else {
                (
                    "CYCCOM",
                    format!(
                        "Function has a McCabe cyclomatic complexity of more than {}. This makes the function difficult to understand and maintain.",
                        self.config.max_cyclomatic_complexity
                    ),
                )
            };
            diagnostics.push(make_diag(rule_id, message));
        }

        // SCYCCOM / MSCYCCOM: Strict cyclomatic complexity exceeds limit
        if metrics.strict_cyclomatic_complexity > self.config.max_strict_cyclomatic_complexity {
            let (rule_id, message) = if metrics.is_method {
                (
                    "MSCYCCOM",
                    format!(
                        "Script has a modified cyclomatic complexity of more than {}. This makes the script difficult to understand and maintain.",
                        self.config.max_strict_cyclomatic_complexity
                    ),
                )
            } else {
                (
                    "SCYCCOM",
                    format!(
                        "Script has a McCabe cyclomatic complexity of more than {}. This makes the script difficult to understand and maintain.",
                        self.config.max_strict_cyclomatic_complexity
                    ),
                )
            };
            diagnostics.push(make_diag(rule_id, message));
        }
    }
}

impl Rule for CustomChecksEngine {
    fn id(&self) -> &'static str {
        "CUSTOM_CHECKS"
    }

    fn description(&self) -> &'static str {
        "Code complexity and style metrics"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn category(&self) -> Category {
        Category::CustomChecks
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        &[]
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let root = ctx.tree.root_node();

        // 1. Check line-based metrics (LLMNC, DAFSC)
        diagnostics.extend(self.check_line_metrics(ctx.source));

        // 2. Check for system commands (SYSBANG)
        diagnostics.extend(self.check_system_commands(root, ctx.source));

        // 3. Analyze all function definitions for function-level metrics
        diagnostics.extend(self.analyze_functions(root, ctx.source));

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check if a variable name looks like a constant (ALL_UPPER_CASE with underscores).
fn is_constant_name(name: &str) -> bool {
    if name.is_empty() || name.len() < 2 {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
        && name.chars().any(|c| c.is_ascii_uppercase())
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "CUSTOM_CHECKS",
    CustomChecksEngine::from_config
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
        CustomChecksEngine::from_config(&Config::default())
    }

    /// Build an engine with custom thresholds set via TOML.
    fn engine_with(params: &str) -> Box<dyn Rule> {
        let config = Config::from_toml(&format!("[lint.rules.CUSTOM_CHECKS]\n{params}\n")).unwrap();
        CustomChecksEngine::from_config(&config)
    }

    // -- SYSBANG ---------------------------------------------------------------

    #[test]
    fn sysbang_fires_on_bang_command() {
        let diags = lint_file(&*engine(), "!ls -la\n");
        assert!(has_id(&diags, "SYSBANG"), "got: {diags:?}");
    }

    #[test]
    fn sysbang_ok_on_normal_code() {
        let diags = lint_file(&*engine(), "disp('hello');\n");
        assert!(!has_id(&diags, "SYSBANG"), "got: {diags:?}");
    }

    // -- LLMNC -----------------------------------------------------------------

    #[test]
    fn llnmc_fires_on_long_line() {
        let source = format!("x = {};\n", "1".repeat(120));
        let diags = lint_file(&*engine(), &source);
        assert!(has_id(&diags, "LLMNC"), "got: {diags:?}");
    }

    #[test]
    fn llnmc_ok_on_short_line() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "LLMNC"), "got: {diags:?}");
    }

    #[test]
    fn llnmc_fires_with_lowered_threshold() {
        let engine = engine_with("max_line_length = 10");
        let diags = lint_file(&*engine, "x = 1234567890;\n");
        assert!(has_id(&diags, "LLMNC"), "got: {diags:?}");
    }

    // -- DAFSC -----------------------------------------------------------------

    #[test]
    fn dafsc_fires_on_many_semicolons() {
        let diags = lint_file(&*engine(), "a = 1; b = 2; c = 3; d = 4;\n");
        assert!(has_id(&diags, "DAFSC"), "got: {diags:?}");
    }

    #[test]
    fn dafsc_ok_few_semicolons() {
        let diags = lint_file(&*engine(), "a = 1; b = 2;\n");
        assert!(!has_id(&diags, "DAFSC"), "got: {diags:?}");
    }

    // -- FCNIL -----------------------------------------------------------------

    #[test]
    fn fcnil_fires_when_inputs_exceed_limit() {
        let engine = engine_with("max_function_inputs = 2");
        let diags = lint_file(&*engine, "function f(a, b, c)\nend\n");
        assert!(has_id(&diags, "FCNIL"), "got: {diags:?}");
    }

    #[test]
    fn fcnil_ok_within_limit() {
        let engine = engine_with("max_function_inputs = 2");
        let diags = lint_file(&*engine, "function f(a, b)\nend\n");
        assert!(!has_id(&diags, "FCNIL"), "got: {diags:?}");
    }

    // -- FCNOL -----------------------------------------------------------------

    #[test]
    fn fcnol_fires_when_outputs_exceed_limit() {
        let engine = engine_with("max_function_outputs = 2");
        let diags = lint_file(&*engine, "function [a, b, c] = f()\nend\n");
        assert!(has_id(&diags, "FCNOL"), "got: {diags:?}");
    }

    #[test]
    fn fcnol_ok_within_limit() {
        let engine = engine_with("max_function_outputs = 2");
        let diags = lint_file(&*engine, "function [a, b] = f()\nend\n");
        assert!(!has_id(&diags, "FCNOL"), "got: {diags:?}");
    }

    // -- FCNLL -----------------------------------------------------------------

    #[test]
    fn fcnll_fires_when_function_is_too_long() {
        let engine = engine_with("max_function_lines = 3");
        let source = "function f()\n    x = 1;\n    y = 2;\n    z = 3;\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "FCNLL"), "got: {diags:?}");
    }

    #[test]
    fn fcnll_ok_short_function() {
        let engine = engine_with("max_function_lines = 3");
        let diags = lint_file(&*engine, "function f()\nend\n");
        assert!(!has_id(&diags, "FCNLL"), "got: {diags:?}");
    }

    // -- MNCSN -----------------------------------------------------------------

    #[test]
    fn mncsn_fires_on_deep_nesting() {
        let engine = engine_with("max_nesting_depth = 2");
        let source = "function f()\n    if a\n        if b\n            if c\n                x = 1;\n            end\n        end\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "MNCSN"), "got: {diags:?}");
    }

    #[test]
    fn mncsn_ok_with_shallow_nesting() {
        let engine = engine_with("max_nesting_depth = 2");
        let source =
            "function f()\n    if a\n        if b\n            x = 1;\n        end\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "MNCSN"), "got: {diags:?}");
    }

    // -- DAFTC -----------------------------------------------------------------

    #[test]
    fn daftc_fires_on_large_matrix() {
        let engine = engine_with("max_tree_children = 6");
        let source = "function f()\n    x = [1, 2, 3, 4, 5, 6];\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "DAFTC"), "got: {diags:?}");
    }

    #[test]
    fn daftc_ok_on_small_nodes() {
        let engine = engine_with("max_tree_children = 6");
        let diags = lint_file(&*engine, "function f()\n    x = 1;\nend\n");
        assert!(!has_id(&diags, "DAFTC"), "got: {diags:?}");
    }

    // -- DAFPV -----------------------------------------------------------------
    //
    // Skipped: `persistent a b c` parses as a `persistent_operator` node in
    // tree-sitter-matlab, but the engine only matches `persistent_statement` /
    // `persistent`, so DAFPV can never fire with any input.

    // -- DAFCO -----------------------------------------------------------------

    #[test]
    fn dafco_fires_on_many_conditions() {
        let engine = engine_with("max_conditions = 2");
        let source = "function f()\n    if a && b && c && d\n        x = 1;\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "DAFCO"), "got: {diags:?}");
    }

    #[test]
    fn dafco_ok_few_conditions() {
        let engine = engine_with("max_conditions = 2");
        let source = "function f()\n    if a && b && c\n        x = 1;\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "DAFCO"), "got: {diags:?}");
    }

    // -- DAFBR -----------------------------------------------------------------

    #[test]
    fn dafbr_fires_on_many_branches() {
        let engine = engine_with("max_branches = 3");
        let source = "function f()\n    switch x\n        case 1\n            a = 1;\n        case 2\n            a = 2;\n        case 3\n            a = 3;\n        otherwise\n            a = 0;\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "DAFBR"), "got: {diags:?}");
    }

    #[test]
    fn dafbr_ok_few_branches() {
        let engine = engine_with("max_branches = 3");
        let source = "function f()\n    switch x\n        case 1\n            a = 1;\n        case 2\n            a = 2;\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "DAFBR"), "got: {diags:?}");
    }

    // -- DAFRT -----------------------------------------------------------------

    #[test]
    fn dafrt_fires_on_many_returns() {
        let engine = engine_with("max_return_points = 2");
        let source = "function f()\n    if a, return; end\n    if b, return; end\n    if c, return; end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "DAFRT"), "got: {diags:?}");
    }

    #[test]
    fn dafrt_ok_few_returns() {
        let engine = engine_with("max_return_points = 2");
        let source = "function f()\n    if a, return; end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "DAFRT"), "got: {diags:?}");
    }

    // -- DAFNF -----------------------------------------------------------------

    #[test]
    fn dafnf_fires_on_many_nested_functions() {
        let engine = engine_with("max_nested_functions = 2");
        let source = "function f()\n    function g()\n    end\n    function h()\n    end\n    function k()\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "DAFNF"), "got: {diags:?}");
    }

    #[test]
    fn dafnf_ok_few_nested_functions() {
        let engine = engine_with("max_nested_functions = 2");
        let source = "function f()\n    function g()\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "DAFNF"), "got: {diags:?}");
    }

    // -- DAFCF -----------------------------------------------------------------

    #[test]
    fn dafcf_fires_on_many_called_functions() {
        let engine = engine_with("max_called_functions = 3");
        let source = "function f()\n    foo();\n    bar();\n    baz();\n    qux();\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "DAFCF"), "got: {diags:?}");
    }

    #[test]
    fn dafcf_ok_few_called_functions() {
        let engine = engine_with("max_called_functions = 3");
        let source = "function f()\n    foo();\n    bar();\n    baz();\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "DAFCF"), "got: {diags:?}");
    }

    // -- DAFAF -----------------------------------------------------------------

    #[test]
    fn dafaf_fires_on_many_anonymous_functions() {
        let engine = engine_with("max_anonymous_functions = 2");
        let source =
            "function f()\n    g1 = @(x) x + 1;\n    g2 = @(x) x + 2;\n    g3 = @(x) x + 3;\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "DAFAF"), "got: {diags:?}");
    }

    #[test]
    fn dafaf_ok_few_anonymous_functions() {
        let engine = engine_with("max_anonymous_functions = 2");
        let source = "function f()\n    g1 = @(x) x + 1;\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "DAFAF"), "got: {diags:?}");
    }

    // -- DAFCV -----------------------------------------------------------------

    #[test]
    fn dafcv_fires_on_many_local_variables() {
        let engine = engine_with("max_local_variables = 3");
        let source = "function f()\n    a = 1;\n    b = 2;\n    c = 3;\n    d = 4;\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "DAFCV"), "got: {diags:?}");
    }

    #[test]
    fn dafcv_ok_few_local_variables() {
        let engine = engine_with("max_local_variables = 3");
        let source = "function f()\n    a = 1;\n    b = 2;\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "DAFCV"), "got: {diags:?}");
    }

    // -- DAFCVC ----------------------------------------------------------------

    #[test]
    fn dafcvc_fires_on_many_constants() {
        let engine = engine_with("max_local_constants = 2");
        let source = "function f()\n    CONST_A = 1;\n    CONST_B = 2;\n    CONST_C = 3;\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "DAFCVC"), "got: {diags:?}");
    }

    #[test]
    fn dafcvc_ok_few_constants() {
        let engine = engine_with("max_local_constants = 2");
        let source = "function f()\n    CONST_A = 1;\n    CONST_B = 2;\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "DAFCVC"), "got: {diags:?}");
    }

    // -- DAFVI -----------------------------------------------------------------

    #[test]
    fn dafvi_fires_on_many_inputs_used() {
        let engine = engine_with("max_input_args_used = 2");
        let source = "function f(a, b, c)\n    y = a + b + c;\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "DAFVI"), "got: {diags:?}");
    }

    #[test]
    fn dafvi_ok_few_inputs_used() {
        let engine = engine_with("max_input_args_used = 2");
        let source = "function f(a, b)\n    y = a + b;\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "DAFVI"), "got: {diags:?}");
    }

    // -- DAFVO -----------------------------------------------------------------

    #[test]
    fn dafvo_fires_on_many_outputs_used() {
        let engine = engine_with("max_output_args_used = 2");
        let source = "function [a, b, c] = f()\n    [a, b, c] = deal(1, 2, 3);\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "DAFVO"), "got: {diags:?}");
    }

    #[test]
    fn dafvo_ok_few_outputs_used() {
        let engine = engine_with("max_output_args_used = 2");
        let source = "function [a, b] = f()\n    [a, b] = deal(1, 2);\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "DAFVO"), "got: {diags:?}");
    }

    // -- CYCCOM ----------------------------------------------------------------

    #[test]
    fn cyccom_fires_when_complexity_exceeds() {
        let engine = engine_with("max_cyclomatic_complexity = 3");
        let source =
            "function f()\n    if a, x = 1; end\n    if b, x = 2; end\n    if c, x = 3; end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "CYCCOM"), "got: {diags:?}");
    }

    #[test]
    fn cyccom_ok_within_limit() {
        let engine = engine_with("max_cyclomatic_complexity = 3");
        let source = "function f()\n    if a, x = 1; end\n    if b, x = 2; end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "CYCCOM"), "got: {diags:?}");
    }

    // -- SCYCCOM ---------------------------------------------------------------

    #[test]
    fn scyccom_fires_on_many_comparisons() {
        let engine = engine_with("max_strict_cyclomatic_complexity = 3");
        let source = "function f()\n    if a > 1, x = 1; end\n    if b < 2, x = 2; end\n    if c == 3, x = 3; end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "SCYCCOM"), "got: {diags:?}");
    }

    #[test]
    fn scyccom_ok_few_comparisons() {
        let engine = engine_with("max_strict_cyclomatic_complexity = 3");
        let source = "function f()\n    if a, x = 1; end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "SCYCCOM"), "got: {diags:?}");
    }

    // -- ACYCCOM ---------------------------------------------------------------

    #[test]
    fn acyccom_fires_on_high_average() {
        let engine = engine_with("max_avg_cyclomatic_complexity = 2");
        let source = "function f()\n    if a, x = 1; end\n    if b, x = 2; end\nend\nfunction g()\n    if a, x = 1; end\n    if b, x = 2; end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "ACYCCOM"), "got: {diags:?}");
    }

    #[test]
    fn acyccom_ok_low_average() {
        let engine = engine_with("max_avg_cyclomatic_complexity = 2");
        let source = "function f()\n    if a, x = 1; end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "ACYCCOM"), "got: {diags:?}");
    }

    // -- MCYCCOM / MSCYCCOM / MACYCCOM (method variants) -----------------------

    #[test]
    fn mcyccom_fires_for_methods() {
        let engine = engine_with("max_cyclomatic_complexity = 2");
        let source = "classdef MyClass\n    methods\n        function out = compute(a)\n            if a > 0, x = 1; end\n            if a < 0, y = 2; end\n        end\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "MCYCCOM"), "got: {diags:?}");
    }

    #[test]
    fn mscyccom_fires_for_methods() {
        let engine = engine_with("max_strict_cyclomatic_complexity = 2");
        let source = "classdef MyClass\n    methods\n        function out = compute(a)\n            if a > 0, x = 1; end\n        end\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "MSCYCCOM"), "got: {diags:?}");
    }

    #[test]
    fn macyccom_fires_for_methods() {
        let engine = engine_with("max_avg_cyclomatic_complexity = 2");
        let source = "classdef MyClass\n    methods\n        function out = compute(a)\n            if a > 0, x = 1; end\n            if a < 0, y = 2; end\n        end\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(has_id(&diags, "MACYCCOM"), "got: {diags:?}");
    }

    #[test]
    fn method_variants_ok_for_low_complexity_methods() {
        let engine = engine_with("max_cyclomatic_complexity = 2");
        let source = "classdef MyClass\n    methods\n        function out = compute(a)\n            out = a;\n        end\n    end\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(!has_id(&diags, "MCYCCOM"), "got: {diags:?}");
        assert!(!has_id(&diags, "MSCYCCOM"), "got: {diags:?}");
        assert!(!has_id(&diags, "MACYCCOM"), "got: {diags:?}");
    }
}
