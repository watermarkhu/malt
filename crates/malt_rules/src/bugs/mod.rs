//! # BUGS_ENGINE: Bugs
//!
//! ```malt
//! id = "BUGS_ENGINE"
//! title = "Bugs"
//! category = "bugs"
//! severity = "error"
//! fix = true
//! icon = "lucide/bug"
//! slug = "bugs"
//! ```
//!
//! ## Rule
//!
//! Detects likely bugs and logic errors in MATLAB code. All 35 checks share a
//! single hybrid engine (`BugsEngine`) that performs both node-level checking
//! (dispatched per-node during traversal) and file-level checking (full-tree
//! analysis after traversal). Each diagnostic carries the specific check ID
//! (e.g. `IFBDUP`, `FNAN`).
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | IFBDUP | error | no | This condition has no effect because all blocks in this if statement are identical. This indicates a bug in the code. Remove the condition or change the code blocks. |
//! | IFCDUP | error | no | The statements under this VAR_RESERVED_WORD condition cannot be reached because it is a duplicate of the VAR_RESERVED_WORD condition on line VAR_NUMBER. This indicates a bug in the code. Remove or change the condition. |
//! | CTRUE | error | no | This logical comparison always returns true. Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)? |
//! | CFALSE | error | no | This logical comparison always returns false. Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)? |
//! | SHOCIRT | error | no | The VAR_NAME operator is unexpected because VAR_NAME(A VAR_NAME B) always returns true. |
//! | SHOCIRF | error | no | The VAR_NAME operator is unexpected because VAR_NAME(A VAR_NAME B) always returns false. |
//! | DEBUGFUN | error | no | Debug functions are intended to be used at the command line. At runtime, they will generate an error. Remove the debug function. |
//! | INCR | error | no | ++x operation does not increment the value of x. To increase the value by 1, use x = x + 1. |
//! | DECR | error | no | --x operation does not decrement the value of x. To decrease the value by 1, use x = x - 1. |
//! | CMDAND | error | yes | Use 'A && B' or 'A & B' to test whether A and B are both true in MATLAB. |
//! | CMDOR | error | yes | Use 'A \|\| B' or 'A \| B' to test whether either A or B is true in MATLAB. |
//! | RHSFN | error | no | The expression cannot be assigned to multiple values. |
//! | FNAN | error | yes | Use ISNAN when comparing values to NaN. |
//! | LOGEMP | error | yes | Using 'isempty' on a logical expression creates incorrect results. To determine if all the conditions are false, use '~any(..., "all")' instead. |
//! | STCUL | error | no | The comparison will likely fail due to case mismatch. |
//! | LBODUP | error | no | Since both operands are identical, the second operand has no effect on the VAR_RESERVED_WORD operation. This indicates a bug in the code. Change one of the operands or remove the VAR_RESERVED_WORD operation. |
//! | FUNFUN | error | no | The first input argument must be a function handle. Did you mean '@VAR_NAME'? |
//! | DEFSIZE | error | no | Do not overload 'size' for fundamental data types. |
//! | VARARG | error | no | Initialize VARARGOUT with a CELL. |
//! | STRCMPCSTR | error | no | 'strcmp' always returns false for string elements of a cell array. Use ["str1", "str2"] instead of {"str1", "str2"}. |
//! | ASSRT | error | no | The first input argument to 'assert' must be a condition. To always throw an error, use 'error(msg)' instead. |
//! | BDSCA2 | error | no | Operands to '\|\|' and '&&' must be scalar values. Use 'all' or 'any' to convert this value into a scalar value or use the element-wise operators '\|' or '&' instead. |
//! | NOPRC | error | no | A line break terminates the statement so it may be incomplete. Use ellipsis (...) to continue the statement. Or add a semicolon to hide the output. |
//! | MOCUP | error | no | Variable VAR_NAME may be cleared before the cleanup function that references VAR_NAME executes, resulting in an undefined variable error. |
//! | MDUPC | error | no | The case value VAR_NAME is a duplicate of one on line VAR_NUMBER. |
//! | MNANC | error | yes | NaN never compares equal to any value, so this case will never be matched. |
//! | MULCC | error | no | This case cannot be matched due to a call to UPPER or LOWER on the SWITCH value. |
//! | MEXCEP | error | no | To report an MException as a warning, use a format specifier to ensure the message is printed correctly. For example, 'warning(E.identifier, "%s", E.message)'. |
//! | PFUIXE | error | no | The index variable VAR_NAME might be used after the PARFOR loop on line VAR_NUMBER, but it is unavailable after the loop. |
//! | PFBFN | error | no | Use of this function is invalid inside a PARFOR loop because it accesses or modifies the workspace in a non-transparent way. |
//! | PFWHOS | error | no | Using "who" or "whos" without "-file" is invalid inside a PARFOR loop because it accesses the workspace in a non-transparent way. |
//! | PFTUSE | error | no | The temporary variable VAR_NAME is used after the PARFOR loop on line VAR_NUMBER, but its value is not available after the loop. |
//! | PFRNC | error | no | Parfor reduction variable VAR_NAME must be used in the same position in each assignment statement when using non-commutative reduction operations '*', '[,]', or '[;]'. |
//!
//! ## Fix
//!
//! Rewrites the flagged construct into the safe equivalent. For example:
//!
//! - `&` → `&&` and `|` → `||` in boolean contexts.
//! - `x == NaN` → `isnan(x)` (and `x ~= NaN` → `~isnan(x)`).
//! - `isempty(cond)` on a logical expression → `~any(cond, "all")`.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! if x == NaN
//!     disp('not a number');
//! end
//! while true
//!     % infinite
//! end
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! if isnan(x)
//!     disp('not a number');
//! end
//! while true % intentional
//!     % ...
//! end
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.BUGS_ENGINE]
//! severity = "error"
//! debug_functions = ["keyboard", "dbstop", "dbclear", "dbcont", "dbquit", "dbup", "dbdown"]
//! higher_order_functions = ["cellfun", "arrayfun", "structfun", "bsxfun", "spfun"]
//! ```

mod check_assert_constant;
mod check_constant_condition;
mod check_debug_function;
mod check_duplicate_conditions;
mod check_element_wise_boolean;
mod check_funfun;
mod check_length_empty;
mod check_nan_comparison;
mod check_operator_precedence;
mod check_scalar_array_op;
mod check_self_modify;
mod check_short_circuit;
mod check_size_comparison;
mod check_strcmp_char;
mod check_strcmpi_same_case;

use malt_core::{Category, Config, Diagnostic, FileContext, Fix, NodeContext, Rule, Severity};
use serde::Deserialize;
use std::collections::HashSet;
use tree_sitter::Node;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Node types targeted by node-level checks.
const TARGET_NODES: &[&str] = &[
    "if_statement",
    "while_statement",
    "switch_statement",
    "boolean_operator",
    "binary_operator",
    "comparison_operator",
    "function_call",
    "assignment",
    "try_statement",
];

/// Debug functions that should not appear in production code.
const DEFAULT_DEBUG_FUNCTIONS: &[&str] = &[
    "keyboard", "dbstop", "dbclear", "dbcont", "dbquit", "dbup", "dbdown", "dbstack", "dbstatus",
    "dbtype",
];

/// Higher-order functions that accept function handles.
const DEFAULT_HIGHER_ORDER_FUNCTIONS: &[&str] =
    &["cellfun", "arrayfun", "structfun", "bsxfun", "spfun"];

/// Array-producing functions whose result is non-scalar.
const ARRAY_FUNCTIONS: &[&str] = &[
    "zeros",
    "ones",
    "rand",
    "randn",
    "eye",
    "linspace",
    "logspace",
    "repmat",
    "reshape",
    "cell",
    "struct",
    "fieldnames",
];

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for the Bugs detection engine.
///
/// Deserialized from the `[lint.rules.BUGS_ENGINE]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BugsConfig {
    /// Debug functions to flag (DEBUGFUN).
    #[serde(default)]
    pub debug_functions: Vec<String>,

    /// Higher-order functions that accept function handles (FUNFUN).
    #[serde(default)]
    pub higher_order_functions: Vec<String>,
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Bugs Detection Engine: a hybrid rule that performs both node-level and
/// file-level checks for 35 bug-detection rules from MATLAB's Code Analyzer.
pub struct BugsEngine {
    config: BugsConfig,
}

impl BugsEngine {
    /// Factory constructor. Reads rule-specific params from config.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: BugsConfig = config.rule_params("BUGS_ENGINE");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Get the effective debug function list.
    fn debug_functions(&self) -> Vec<&str> {
        if self.config.debug_functions.is_empty() {
            DEFAULT_DEBUG_FUNCTIONS.to_vec()
        } else {
            self.config
                .debug_functions
                .iter()
                .map(|s| s.as_str())
                .collect()
        }
    }

    /// Get the effective higher-order function list.
    fn higher_order_functions(&self) -> Vec<&str> {
        if self.config.higher_order_functions.is_empty() {
            DEFAULT_HIGHER_ORDER_FUNCTIONS.to_vec()
        } else {
            self.config
                .higher_order_functions
                .iter()
                .map(|s| s.as_str())
                .collect()
        }
    }

    // -----------------------------------------------------------------------
    // File-level checks
    // -----------------------------------------------------------------------

    /// IFBDUP: Duplicate if-branch bodies.
    fn check_if_branch_dup_bodies(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        Self::walk_if_branch_dup_bodies(root, source, &mut diagnostics);
        diagnostics
    }

    fn walk_if_branch_dup_bodies(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "if_statement" {
            let mut branch_bodies: Vec<(String, Node)> = Vec::new();

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                let body = match child.kind() {
                    "block" => Some(child),
                    "elseif_clause" | "else_clause" => find_block_child(child),
                    _ => None,
                };

                if let Some(body_node) = body {
                    let body_text = normalize_whitespace(node_text(body_node, source));
                    if !body_text.is_empty() {
                        branch_bodies.push((body_text, child));
                    }
                }
            }

            // The condition only has no effect when every branch body is
            // identical (including the else branch).
            if branch_bodies.len() >= 2 {
                let first_text = &branch_bodies[0].0;
                let all_identical = branch_bodies.iter().skip(1).all(|(t, _)| t == first_text);
                if all_identical {
                    let target = node
                        .child_by_field_name("condition")
                        .or_else(|| find_condition_child(node))
                        .unwrap_or(node);
                    let pos = target.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "IFBDUP",
                        message: "This condition has no effect because all blocks in this if statement are identical. This indicates a bug in the code. Remove the condition or change the code blocks.".to_string(),
                        severity: Severity::Error,
                        byte_range: target.start_byte()..target.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_if_branch_dup_bodies(child, source, diagnostics);
        }
    }

    /// IFCDUP: Duplicate if-branch conditions.
    fn check_if_condition_dup(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        Self::walk_if_condition_dup(root, source, &mut diagnostics);
        diagnostics
    }

    fn walk_if_condition_dup(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "if_statement" {
            let mut conditions: Vec<(String, Node)> = Vec::new();

            // Collect the main if condition.
            if let Some(cond) = node
                .child_by_field_name("condition")
                .or_else(|| find_condition_child(node))
            {
                let cond_text = normalize_whitespace(node_text(cond, source));
                conditions.push((cond_text, cond));
            }

            // Collect elseif conditions.
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "elseif_clause" {
                    if let Some(cond) = child
                        .child_by_field_name("condition")
                        .or_else(|| find_condition_child(child))
                    {
                        let cond_text = normalize_whitespace(node_text(cond, source));
                        if let Some((_, first_cond)) =
                            conditions.iter().find(|(t, _)| *t == cond_text)
                        {
                            let pos = cond.start_position();
                            let first_line = first_cond.start_position().row + 1;
                            diagnostics.push(Diagnostic {
                                rule_id: "IFCDUP",
                                message: format!(
                                    "The statements under this elseif condition cannot be reached because it is a duplicate of the elseif condition on line {}. This indicates a bug in the code. Remove or change the condition.",
                                    first_line
                                ),
                                severity: Severity::Error,
                                byte_range: cond.start_byte()..cond.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        } else {
                            conditions.push((cond_text, cond));
                        }
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_if_condition_dup(child, source, diagnostics);
        }
    }

    /// LBODUP / MDUPC: Duplicate case values in switch.
    fn check_switch_dup_cases(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        Self::walk_switch_dup_cases(root, source, &mut diagnostics);
        diagnostics
    }

    fn walk_switch_dup_cases(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "switch_statement" {
            let mut case_values: Vec<(String, Node)> = Vec::new();

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "case_clause" {
                    // The case value is the first meaningful child after "case".
                    if let Some(case_val) = find_case_value(child) {
                        let val_text = normalize_whitespace(node_text(case_val, source));
                        if let Some((_, first)) = case_values.iter().find(|(t, _)| *t == val_text) {
                            let pos = case_val.start_position();
                            let first_line = first.start_position().row + 1;
                            let case_text = node_text(case_val, source).trim();
                            // Report as both LBODUP and MDUPC (they cover different aspects).
                            diagnostics.push(Diagnostic {
                                rule_id: "LBODUP",
                                message: "Since both operands are identical, the second operand has no effect on the VAR_RESERVED_WORD operation. This indicates a bug in the code. Change one of the operands or remove the VAR_RESERVED_WORD operation.".to_string(),
                                severity: Severity::Error,
                                byte_range: case_val.start_byte()..case_val.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                            diagnostics.push(Diagnostic {
                                rule_id: "MDUPC",
                                message: format!(
                                    "The case value {} is a duplicate of one on line {}.",
                                    case_text, first_line
                                ),
                                severity: Severity::Error,
                                byte_range: case_val.start_byte()..case_val.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        } else {
                            case_values.push((val_text, case_val));
                        }
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_switch_dup_cases(child, source, diagnostics);
        }
    }

    /// NOPRC: a statement terminated by a line break where continuation (via
    /// `...`) or a semicolon was likely intended.
    ///
    /// When a line that begins with a unary `+`/`-` follows a completed
    /// statement, tree-sitter parses it as a separate `unary_operator` at
    /// statement level — the signature of a split, incomplete statement.
    ///
    /// # Limitations (heuristic)
    ///
    /// This is a syntactic heuristic, not a faithful reproduction of Code
    /// Analyzer's line-break analysis. It only detects the specific
    /// statement-level unary `+`/`-` signature; other split-statement shapes
    /// are not caught.
    fn check_line_break_termination(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        Self::walk_line_break_termination(root, source, &mut diagnostics);
        diagnostics
    }

    fn walk_line_break_termination(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "unary_operator" {
            if let Some(parent) = node.parent() {
                if parent.kind() == "source_file" || parent.kind() == "block" {
                    let op = node_text(node, source).trim();
                    if op.starts_with('+') || op.starts_with('-') {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "NOPRC",
                            message: "A line break terminates the statement so it may be incomplete. Use ellipsis (...) to continue the statement. Or add a semicolon to hide the output.".to_string(),
                            severity: Severity::Error,
                            byte_range: node.start_byte()..node.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_line_break_termination(child, source, diagnostics);
        }
    }

    /// MEXCEP: Catch clause without an exception identifier.
    fn check_catch_without_id(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        Self::walk_catch_without_id(root, source, &mut diagnostics);
        diagnostics
    }

    fn walk_catch_without_id(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "catch_clause" {
            // Check if the catch clause has an identifier child.
            let has_id = {
                let mut cursor = node.walk();
                let result = node.children(&mut cursor).any(|c| c.kind() == "identifier");
                result
            };

            if !has_id {
                // Also check the source text for `catch <id>` pattern.
                let catch_text = node_text(node, source).trim().to_string();
                let has_id_text = catch_text
                    .strip_prefix("catch")
                    .map(|rest| {
                        let rest = rest.trim();
                        !rest.is_empty() && rest.chars().next().is_some_and(|c| c.is_alphabetic())
                    })
                    .unwrap_or(false);

                if !has_id_text {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "MEXCEP",
                        message: "To report an MException as a warning, use a format specifier to ensure the message is printed correctly. For example, 'warning(E.identifier, \"%s\", E.message)'.".to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        // For try_statement, check its catch_clause children.
        if node.kind() == "try_statement" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "catch_clause" {
                    Self::walk_catch_without_id(child, source, diagnostics);
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "catch_clause" {
                Self::walk_catch_without_id(child, source, diagnostics);
            }
        }
    }

    /// RHSFN: an assignment whose left-hand side is a multiple-output
    /// expression but whose right-hand side cannot return multiple values.
    ///
    /// # Limitations (grammar)
    ///
    /// Only a right-hand side that is NOT a `function_call` is flagged. The
    /// tree-sitter MATLAB grammar uses `function_call` for both actual function
    /// calls and array/cell indexing, so `[a, b] = f(...)` (a scalar-returning
    /// `f`) cannot be distinguished from `[a, b] = f(i)` (an index into an
    /// array `f`). As a result this check misses multi-output assignments whose
    /// RHS is a genuinely scalar-returning function call.
    fn check_multioutput_assignment(&self, root: Node) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        Self::walk_multioutput_assignment(root, &mut diagnostics);
        diagnostics
    }

    fn walk_multioutput_assignment(node: Node, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "assignment" {
            let lhs = node.child_by_field_name("left").or_else(|| node.child(0));
            let rhs = node.child_by_field_name("right").or_else(|| node.child(2));
            if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                // Only a function call can return multiple outputs. Any other
                // right-hand side (identifier, literal, operator expression)
                // yields a single value that cannot be assigned to multiple
                // left-hand targets.
                if lhs.kind() == "multioutput_variable" && rhs.kind() != "function_call" {
                    let pos = rhs.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "RHSFN",
                        message: "The expression cannot be assigned to multiple values."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: rhs.start_byte()..rhs.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_multioutput_assignment(child, diagnostics);
        }
    }

    /// VARARG: `varargout` used in a function without being initialized to a
    /// cell array.
    ///
    /// # Limitations (heuristic)
    ///
    /// "Initialized to a CELL" is recognized only from a literal `cell`
    /// literal (`varargout = {}`) or a `cell(...)` call. Initialization via a
    /// variable, a helper, or any other cell-producing expression is treated
    /// as uninitialized and flagged. Nested function definitions are skipped,
    /// so `varargout` used only inside a nested function is not attributed to
    /// the enclosing function.
    fn check_vararg_init(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        Self::walk_vararg_init(root, source, &mut diagnostics);
        diagnostics
    }

    fn walk_vararg_init(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "function_definition" {
            let declared = Self::function_declares_varargout(node, source);
            let mut used = false;
            let mut initialized = false;
            let mut first_use: Option<(usize, usize, usize, usize)> = None;
            Self::scan_varargout(node, source, &mut used, &mut initialized, &mut first_use);

            if used && !declared && !initialized {
                if let Some((start, end, line, column)) = first_use {
                    diagnostics.push(Diagnostic {
                        rule_id: "VARARG",
                        message: "Initialize VARARGOUT with a CELL.".to_string(),
                        severity: Severity::Error,
                        byte_range: start..end,
                        line,
                        column,
                        fix: None,
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_vararg_init(child, source, diagnostics);
        }
    }

    /// Whether a `function_definition` declares `varargout` in its output list.
    fn function_declares_varargout(node: Node, source: &str) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_output" {
                return Self::contains_identifier(child, "varargout", source);
            }
        }
        false
    }

    /// Recursively scan a function body (skipping nested function definitions)
    /// for `varargout` usage and cell-array initialization.
    fn scan_varargout(
        node: Node,
        source: &str,
        used: &mut bool,
        initialized: &mut bool,
        first_use: &mut Option<(usize, usize, usize, usize)>,
    ) {
        match node.kind() {
            "assignment" => {
                let lhs = node.child_by_field_name("left").or_else(|| node.child(0));
                if let Some(lhs) = lhs {
                    if lhs.kind() == "identifier" && node_text(lhs, source).trim() == "varargout" {
                        let rhs = node.child_by_field_name("right").or_else(|| node.child(2));
                        let is_cell = rhs.is_some_and(|r| {
                            r.kind() == "cell"
                                || (r.kind() == "function_call"
                                    && extract_call_name(r, source) == Some("cell"))
                        });
                        if is_cell {
                            *initialized = true;
                        } else {
                            *used = true;
                            if first_use.is_none() {
                                *first_use = Some((
                                    lhs.start_byte(),
                                    lhs.end_byte(),
                                    lhs.start_position().row + 1,
                                    lhs.start_position().column + 1,
                                ));
                            }
                        }
                    }
                }
            }
            "identifier" if node_text(node, source).trim() == "varargout" => {
                *used = true;
                if first_use.is_none() {
                    *first_use = Some((
                        node.start_byte(),
                        node.end_byte(),
                        node.start_position().row + 1,
                        node.start_position().column + 1,
                    ));
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "function_definition" {
                Self::scan_varargout(child, source, used, initialized, first_use);
            }
        }
    }

    /// Whether a subtree contains an identifier with the given name.
    fn contains_identifier(node: Node, name: &str, source: &str) -> bool {
        if node.kind() == "identifier" && node_text(node, source).trim() == name {
            return true;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if Self::contains_identifier(child, name, source) {
                return true;
            }
        }
        false
    }

    /// Parfor-related checks: PFUIXE, PFBFN, PFWHOS, PFTUSE, PFRNC.
    fn check_parfor_issues(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        Self::walk_parfor_issues(root, source, &mut diagnostics);
        diagnostics
    }

    fn walk_parfor_issues(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "for_statement" {
            let stmt_text = node_text(node, source);
            if !stmt_text.starts_with("parfor") {
                // Not a parfor — recurse into children only.
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    Self::walk_parfor_issues(child, source, diagnostics);
                }
                return;
            }

            // This is a parfor. Extract the loop variable.
            let loop_var = extract_for_variable(node, source);

            // Walk the parfor body for issues.
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "block" {
                    Self::check_parfor_body(child, source, loop_var.as_deref(), diagnostics);
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_parfor_issues(child, source, diagnostics);
        }
    }

    fn check_parfor_body(
        node: Node,
        source: &str,
        loop_var: Option<&str>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "function_call" => {
                let func_name = extract_call_name(node, source);
                if let Some(name) = &func_name {
                    // PFUIXE: Loop variable used in eval.
                    if *name == "eval" || *name == "evalin" || *name == "feval" {
                        if let Some(lv) = loop_var {
                            let args = collect_call_args(node, source);
                            if args.iter().any(|a| a.contains(lv)) {
                                let pos = node.start_position();
                                diagnostics.push(Diagnostic {
                                    rule_id: "PFUIXE",
                                    message: format!(
                                        "The index variable {} might be used after the PARFOR loop on line {}, but it is unavailable after the loop.",
                                        lv, pos.row + 1
                                    ),
                                    severity: Severity::Error,
                                    byte_range: node.start_byte()..node.end_byte(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                            }
                        }
                    }

                    // PFWHOS: who/whos without -file in parfor.
                    if *name == "who" || *name == "whos" {
                        let args = collect_call_args(node, source);
                        let has_file_flag = args.iter().any(|a| {
                            let a = a.trim().trim_matches(|c| c == '\'' || c == '"');
                            a == "-file"
                        });
                        if !has_file_flag {
                            let pos = node.start_position();
                            diagnostics.push(Diagnostic {
                                rule_id: "PFWHOS",
                                message: "Using \"who\" or \"whos\" without \"-file\" is invalid inside a PARFOR loop because it accesses the workspace in a non-transparent way.".to_string(),
                                severity: Severity::Error,
                                byte_range: node.start_byte()..node.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    }

                    // PFBFN: Certain builtin functions problematic in parfor.
                    let problematic_builtins = [
                        "assignin",
                        "evalin",
                        "save",
                        "load",
                        "clear",
                        "global",
                        "persistent",
                    ];
                    if problematic_builtins.contains(name) {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "PFBFN",
                            message: "Use of this function is invalid inside a PARFOR loop because it accesses or modifies the workspace in a non-transparent way.".to_string(),
                            severity: Severity::Error,
                            byte_range: node.start_byte()..node.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }
            "command" => {
                // PFWHOS: `who`/`whos` command form (no parentheses) without
                // `-file` in parfor.
                let text = node_text(node, source).trim();
                let first = text.split_whitespace().next().unwrap_or("");
                if (first == "who" || first == "whos") && !text.contains("-file") {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "PFWHOS",
                        message: "Using \"who\" or \"whos\" without \"-file\" is invalid inside a PARFOR loop because it accesses the workspace in a non-transparent way.".to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
            "assignment" => {
                // PFTUSE / PFRNC: Check for temporary variable misuse and
                // inconsistent reduction patterns.
                if let Some(lhs) = node.child_by_field_name("left").or_else(|| node.child(0)) {
                    if lhs.kind() == "identifier" {
                        let var_name = node_text(lhs, source).trim().to_string();
                        if let Some(rhs) =
                            node.child_by_field_name("right").or_else(|| node.child(2))
                        {
                            let rhs_text = node_text(rhs, source);
                            // PFTUSE: Variable used on both sides but is not a reduction.
                            if rhs_text.contains(&var_name)
                                && !is_reduction_pattern(&var_name, rhs, source)
                            {
                                let pos = node.start_position();
                                diagnostics.push(Diagnostic {
                                    rule_id: "PFTUSE",
                                    message: format!(
                                        "The temporary variable {} is used after the PARFOR loop on line {}, but its value is not available after the loop.",
                                        var_name, pos.row + 1
                                    ),
                                    severity: Severity::Error,
                                    byte_range: node.start_byte()..node.end_byte(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                            }

                            // PFRNC: Reduction operator not consistent.
                            if is_reduction_pattern(&var_name, rhs, source) {
                                // Check for mixed reduction operators on same variable.
                                // This is a simplified check — full analysis would track
                                // across the entire parfor body.
                                if has_mixed_reduction_ops(&var_name, rhs, source) {
                                    let pos = node.start_position();
                                    diagnostics.push(Diagnostic {
                                        rule_id: "PFRNC",
                                        message: format!(
                                            "Parfor reduction variable {} must be used in the same position in each assignment statement when using non-commutative reduction operations '*', '[,]', or '[;]'.",
                                            var_name
                                        ),
                                        severity: Severity::Error,
                                        byte_range: node.start_byte()..node.end_byte(),
                                        line: pos.row + 1,
                                        column: pos.column + 1,
                                        fix: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::check_parfor_body(child, source, loop_var, diagnostics);
        }
    }
}

// ---------------------------------------------------------------------------
// Rule trait implementation
// ---------------------------------------------------------------------------

impl Rule for BugsEngine {
    fn id(&self) -> &'static str {
        "BUGS_ENGINE"
    }

    fn description(&self) -> &'static str {
        "Bug detection checks for likely bugs and logic errors"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn category(&self) -> Category {
        Category::Bugs
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn can_be_disabled(&self) -> bool {
        true
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        let node = ctx.node;
        let source = ctx.source;
        let mut diagnostics = Vec::new();

        match node.kind() {
            "if_statement" | "while_statement" => {
                diagnostics.extend(self.check_constant_condition(node, source));
            }
            "boolean_operator" => {
                diagnostics.extend(self.check_short_circuit(node, source));
            }
            "binary_operator" => {
                diagnostics.extend(self.check_element_wise_boolean(node, source));
                diagnostics.extend(self.check_scalar_array_op(node, source));
            }
            "comparison_operator" => {
                diagnostics.extend(self.check_nan_comparison(node, source));
            }
            "switch_statement" => {
                diagnostics.extend(self.check_switch_upper_lower(node, source));
            }
            "function_call" => {
                diagnostics.extend(self.check_debug_function(node, source));
                diagnostics.extend(self.check_strcmp_case_mismatch(node, source));
                diagnostics.extend(self.check_strcmp_cell(node, source));
                diagnostics.extend(self.check_funfun(node, source));
                diagnostics.extend(self.check_assert_condition(node, source));
                diagnostics.extend(self.check_isempty_logical(node, source));
            }
            "assignment" => {
                diagnostics.extend(self.check_self_modify(node, source));
            }
            // try_statement is handled at file level.
            _ => {}
        }

        diagnostics
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let root = ctx.tree.root_node();
        let source = ctx.source;

        // File-level checks.
        diagnostics.extend(self.check_if_branch_dup_bodies(root, source));
        diagnostics.extend(self.check_if_condition_dup(root, source));
        diagnostics.extend(self.check_switch_dup_cases(root, source));
        diagnostics.extend(self.check_line_break_termination(root, source));
        diagnostics.extend(self.check_catch_without_id(root, source));
        diagnostics.extend(self.check_multioutput_assignment(root));
        diagnostics.extend(self.check_vararg_init(root, source));
        diagnostics.extend(self.check_parfor_issues(root, source));
        diagnostics.extend(self.check_mocup(root, source));
        diagnostics.extend(self.check_size_overload(root, source));

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the text of a node from the source.
pub(crate) fn node_text<'a>(node: Node<'a>, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// Normalize whitespace in a string for comparison purposes.
pub(crate) fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Find the condition child of an `if_statement` or `elseif_clause` node.
///
/// In tree-sitter-matlab, the condition is typically the node between the
/// keyword (`if`/`elseif`/`while`) and the body.
pub(crate) fn find_condition_child(node: Node) -> Option<Node> {
    let mut cursor = node.walk();
    let mut found_keyword = false;
    for child in node.children(&mut cursor) {
        if child.kind() == "if"
            || child.kind() == "elseif"
            || child.kind() == "while"
            || child.is_named() && found_keyword
        {
            if found_keyword && child.kind() != "block" && child.kind() != "end" {
                return Some(child);
            }
            if !child.is_named() || child.kind() == "if" || child.kind() == "while" {
                found_keyword = true;
            }
        }
    }

    // Fallback: return the first named child that is not a keyword or block.
    let mut cursor2 = node.walk();
    let result = node.children(&mut cursor2).find(|child| {
        child.is_named()
            && child.kind() != "block"
            && child.kind() != "elseif_clause"
            && child.kind() != "else_clause"
            && child.kind() != "end"
            && child.kind() != "comment"
    });
    result
}

/// Find a `block` child of a node.
pub(crate) fn find_block_child(node: Node) -> Option<Node> {
    let mut cursor = node.walk();
    let result = node
        .children(&mut cursor)
        .find(|child| child.kind() == "block");
    result
}

/// Find the case value node inside a `case_clause`.
pub(crate) fn find_case_value(case_node: Node) -> Option<Node> {
    let mut cursor = case_node.walk();
    let result = case_node
        .children(&mut cursor)
        .find(|child| child.is_named() && child.kind() != "block" && child.kind() != "comment");
    result
}

/// Check if a condition text represents an always-true value.
pub(crate) fn is_always_true(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed == "true" || trimmed == "1" || trimmed == "1.0"
}

/// Check if a condition text represents an always-false value.
pub(crate) fn is_always_false(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed == "false" || trimmed == "0" || trimmed == "0.0"
}

/// Extract the function name from a `function_call` node.
pub(crate) fn extract_call_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    if node.kind() != "function_call" {
        return None;
    }

    // Try the `name` field first.
    if let Some(name_node) = node.child_by_field_name("name") {
        if name_node.kind() == "identifier" {
            return Some(&source[name_node.start_byte()..name_node.end_byte()]);
        }
        // For field_expression (e.g., obj.method), return the full text.
        return Some(&source[name_node.start_byte()..name_node.end_byte()]);
    }

    // Fallback: first identifier child.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return Some(&source[child.start_byte()..child.end_byte()]);
        }
    }

    None
}

/// Collect arguments from a `function_call` node as text strings.
pub(crate) fn collect_call_args<'a>(node: Node<'a>, source: &'a str) -> Vec<&'a str> {
    let mut args = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "arguments" {
            let mut arg_cursor = child.walk();
            for arg in child.children(&mut arg_cursor) {
                if arg.is_named() {
                    args.push(&source[arg.start_byte()..arg.end_byte()]);
                }
            }
            return args;
        }
    }
    args
}

/// Collect the argument nodes of a `function_call`.
pub(crate) fn arg_nodes<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    let mut args = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "arguments" {
            let mut arg_cursor = child.walk();
            for arg in child.children(&mut arg_cursor) {
                if arg.is_named() {
                    args.push(arg);
                }
            }
            break;
        }
    }
    args
}

/// Return the first argument node of a `function_call`.
pub(crate) fn first_arg_node<'a>(node: Node<'a>) -> Option<Node<'a>> {
    arg_nodes(node).into_iter().next()
}

/// Find the operator text in a binary/boolean/comparison operator node.
pub(crate) fn find_operator_text<'a>(node: Node<'a>, source: &'a str) -> String {
    // The operator is typically the second child (index 1), which is an
    // anonymous node containing the operator symbol.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if !child.is_named() {
            let text = &source[child.start_byte()..child.end_byte()];
            let trimmed = text.trim();
            if !trimmed.is_empty() && trimmed != "(" && trimmed != ")" && trimmed != "," {
                return trimmed.to_string();
            }
        }
    }

    // Fallback: try the operator field.
    if let Some(op_node) = node.child_by_field_name("operator") {
        return node_text(op_node, source).trim().to_string();
    }

    // Last resort: parse from the full text.
    let text = node_text(node, source);
    for op in &[
        "&&", "||", "==", "~=", ">=", "<=", ">", "<", ".+", ".-", ".*", "./", ".\\", ".^", "+",
        "-", "*", "/", "\\", "^", "&", "|",
    ] {
        if text.contains(op) {
            return (*op).to_string();
        }
    }

    String::new()
}

/// Check if a node is inside a loop (`for_statement` or `while_statement`).
pub(crate) fn is_inside_loop(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "for_statement" || parent.kind() == "while_statement" {
            return true;
        }
        current = parent.parent();
    }
    false
}

/// Check if a node is in a boolean context (condition of if/while/elseif).
pub(crate) fn is_in_boolean_context(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "if_statement" | "while_statement" | "elseif_clause" => {
                // Check if the node is in the condition part, not the body.
                // The condition comes before the first block child.
                let in_condition = true;
                let mut cursor = parent.walk();
                for child in parent.children(&mut cursor) {
                    if child.kind() == "block" {
                        break;
                    }
                    if child.id() == node.id() || is_ancestor_of(child, node) {
                        return in_condition;
                    }
                }
                return false;
            }
            // Stop at statement boundaries.
            "source_file" | "function_definition" => return false,
            _ => {}
        }
        current = parent.parent();
    }
    false
}

/// Check if `ancestor` is an ancestor of `node`.
pub(crate) fn is_ancestor_of(ancestor: Node, node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.id() == ancestor.id() {
            return true;
        }
        current = parent.parent();
    }
    false
}

/// Check if a node represents an array-producing expression.
pub(crate) fn is_array_expression(node: Node, source: &str) -> bool {
    match node.kind() {
        "matrix" | "cell" => true,
        "function_call" => {
            if let Some(name) = extract_call_name(node, source) {
                ARRAY_FUNCTIONS.contains(&name)
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Check if a node is a scalar literal (number, true, false).
pub(crate) fn is_scalar_literal(node: Node, source: &str) -> bool {
    match node.kind() {
        "number" => true,
        "true" | "false" => true,
        "identifier" => {
            let text = node_text(node, source).trim();
            text == "true" || text == "false"
        }
        _ => false,
    }
}

/// Extract the loop variable name from a `for_statement`.
pub(crate) fn extract_for_variable<'a>(node: Node<'a>, source: &'a str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "iterator" || child.kind() == "assignment" {
            // The loop variable is the first identifier.
            let mut inner_cursor = child.walk();
            for inner in child.children(&mut inner_cursor) {
                if inner.kind() == "identifier" {
                    return Some(node_text(inner, source).trim().to_string());
                }
            }
        }
        if child.kind() == "identifier" {
            return Some(node_text(child, source).trim().to_string());
        }
    }
    None
}

/// Check if a for loop body is simple enough to potentially be a parfor.
///
/// Returns `true` if the loop body contains no `break`, `continue`, `return`,
/// nested loops, or global/persistent variable access.
pub(crate) fn is_reduction_pattern(var_name: &str, rhs: Node, source: &str) -> bool {
    if rhs.kind() != "binary_operator" {
        return false;
    }

    let lhs_child = match rhs.child(0) {
        Some(c) => c,
        None => return false,
    };

    if lhs_child.kind() == "identifier" {
        let name = node_text(lhs_child, source).trim();
        if name == var_name {
            let op = find_operator_text(rhs, source);
            return op == "+" || op == "-" || op == "*" || op == ".*";
        }
    }

    false
}

/// Check for mixed reduction operators on the same variable.
pub(crate) fn has_mixed_reduction_ops(var_name: &str, rhs: Node, source: &str) -> bool {
    // This is a simplified check. In a full implementation, we would track
    // all reduction operations on `var_name` across the parfor body.
    // Here we check if the RHS contains the variable name in multiple
    // different operations.
    if rhs.kind() != "binary_operator" {
        return false;
    }

    let op = find_operator_text(rhs, source);

    // Check if any child also references the variable with a different op.
    let mut found_ops: Vec<String> = vec![op];
    collect_reduction_ops(var_name, rhs, source, &mut found_ops);

    // If we found more than one unique operator, it's mixed.
    let unique: HashSet<&String> = found_ops.iter().collect();
    unique.len() > 1
}

pub(crate) fn collect_reduction_ops(
    var_name: &str,
    node: Node,
    source: &str,
    ops: &mut Vec<String>,
) {
    if node.kind() == "binary_operator" {
        if let Some(lhs) = node.child(0) {
            if lhs.kind() == "identifier" && node_text(lhs, source).trim() == var_name {
                ops.push(find_operator_text(node, source));
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_reduction_ops(var_name, child, source, ops);
    }
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "BUGS_ENGINE",
    BugsEngine::from_config
));

#[cfg(test)]
use crate::test_util::{lint_file, lint_nodes};

#[cfg(test)]
pub(crate) fn engine() -> Box<dyn Rule> {
    BugsEngine::from_config(&Config::default())
}

#[cfg(test)]
pub(crate) fn node_diags(src: &str) -> Vec<Diagnostic> {
    lint_nodes(&*engine(), src)
}

#[cfg(test)]
pub(crate) fn file_diags(src: &str) -> Vec<Diagnostic> {
    lint_file(&*engine(), src)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, parse};

    // -- IFBDUP --------------------------------------------------------------

    #[test]
    fn ifbdup_fires_on_duplicate_branch_bodies() {
        let src = "\
if x
    a = 1;
    b = 2;
elseif y
    a = 1;
    b = 2;
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "IFBDUP"), "got: {diags:?}");
    }

    #[test]
    fn ifbdup_no_fire_on_distinct_branch_bodies() {
        let src = "\
if x
    a = 1;
elseif y
    b = 2;
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "IFBDUP"), "got: {diags:?}");
    }

    #[test]
    fn ifbdup_fires_on_identical_if_else_bodies() {
        let src = "\
if x
    a = 1;
else
    a = 1;
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "IFBDUP"), "got: {diags:?}");
    }

    #[test]
    fn ifbdup_no_fire_when_not_all_bodies_identical() {
        let src = "\
if x
    a = 1;
elseif y
    a = 1;
else
    b = 2;
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "IFBDUP"), "got: {diags:?}");
    }

    // -- IFCDUP --------------------------------------------------------------

    #[test]
    fn ifcdup_fires_on_duplicate_conditions() {
        let src = "\
if x
    a = 1;
elseif x
    b = 2;
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "IFCDUP"), "got: {diags:?}");
    }

    #[test]
    fn ifcdup_no_fire_on_distinct_conditions() {
        let src = "\
if x
    a = 1;
elseif y
    b = 2;
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "IFCDUP"), "got: {diags:?}");
    }

    // -- LBODUP / MDUPC ------------------------------------------------------

    #[test]
    fn lbodup_and_mdupc_fire_on_duplicate_case_values() {
        let src = "\
switch x
    case 1
        a = 1;
    case 1
        b = 2;
    otherwise
        c = 3;
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "LBODUP"), "got: {diags:?}");
        assert!(has_id(&diags, "MDUPC"), "got: {diags:?}");
    }

    #[test]
    fn lbodup_and_mdupc_no_fire_on_distinct_cases() {
        let src = "\
switch x
    case 1
        a = 1;
    case 2
        b = 2;
    otherwise
        c = 3;
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "LBODUP"), "got: {diags:?}");
        assert!(!has_id(&diags, "MDUPC"), "got: {diags:?}");
    }

    // -- NOPRC ---------------------------------------------------------------

    #[test]
    fn noprc_fires_on_line_break_terminated_statement() {
        let src = "\
a = 1 + 2
+ 3;
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "NOPRC"), "got: {diags:?}");
    }

    #[test]
    fn noprc_no_fire_on_ellipsis_continuation() {
        let src = "\
a = 1 + 2 + ...
3;
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "NOPRC"), "got: {diags:?}");
    }

    #[test]
    fn noprc_no_fire_on_switch_without_otherwise() {
        let src = "\
switch x
    case 1
        a = 1;
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "NOPRC"), "got: {diags:?}");
    }

    // -- MEXCEP --------------------------------------------------------------

    #[test]
    fn mexcep_fires_on_catch_without_identifier() {
        let src = "\
try
    x = 1;
catch
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "MEXCEP"), "got: {diags:?}");
    }

    #[test]
    fn mexcep_no_fire_on_catch_with_identifier() {
        let src = "\
try
    x = 1;
catch err
    disp(err.message);
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "MEXCEP"), "got: {diags:?}");
    }

    // -- RHSFN ---------------------------------------------------------------

    #[test]
    fn rhsfn_fires_on_multioutput_with_single_value_rhs() {
        let src = "a = [1 2];\n[a, b] = a;\n";
        let diags = file_diags(src);
        assert!(has_id(&diags, "RHSFN"), "got: {diags:?}");
    }

    #[test]
    fn rhsfn_fires_on_multioutput_with_literal_rhs() {
        let src = "[a, b] = 1 + 2;\n";
        let diags = file_diags(src);
        assert!(has_id(&diags, "RHSFN"), "got: {diags:?}");
    }

    #[test]
    fn rhsfn_no_fire_on_function_call_rhs() {
        let src = "[a, b] = myfunc(x);\n";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "RHSFN"), "got: {diags:?}");
    }

    // -- VARARG --------------------------------------------------------------

    #[test]
    fn vararg_fires_on_uninitialized_varargout() {
        let src = "\
function f()
    varargout{1} = 1;
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "VARARG"), "got: {diags:?}");
    }

    #[test]
    fn vararg_fires_on_non_cell_varargout() {
        let src = "\
function f()
    varargout = 5;
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "VARARG"), "got: {diags:?}");
    }

    #[test]
    fn vararg_no_fire_on_initialized_varargout() {
        let src = "\
function f()
    varargout = {};
    varargout{1} = 1;
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "VARARG"), "got: {diags:?}");
    }

    #[test]
    fn vararg_no_fire_on_declared_varargout() {
        let src = "\
function [varargout] = f()
    varargout{1} = 1;
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "VARARG"), "got: {diags:?}");
    }

    #[test]
    fn vararg_no_fire_on_varargin() {
        let src = "\
function f(varargin)
    x = varargin{1};
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "VARARG"), "got: {diags:?}");
    }

    // -- PFUIXE --------------------------------------------------------------

    #[test]
    fn pfuixe_fires_on_eval_with_loop_var() {
        let src = "\
parfor i = 1:10
    eval(sprintf('x%d', i));
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "PFUIXE"), "got: {diags:?}");
    }

    #[test]
    fn pfuixe_no_fire_outside_parfor() {
        let src = "eval(sprintf('x%d', i));\n";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "PFUIXE"), "got: {diags:?}");
    }

    // -- PFBFN ---------------------------------------------------------------

    #[test]
    fn pfbfn_fires_on_save_in_parfor() {
        let src = "\
parfor i = 1:10
    save('data.mat');
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "PFBFN"), "got: {diags:?}");
    }

    #[test]
    fn pfbfn_no_fire_outside_parfor() {
        let src = "save('data.mat');\n";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "PFBFN"), "got: {diags:?}");
    }

    // -- PFWHOS --------------------------------------------------------------

    #[test]
    fn pfwhos_fires_on_who_in_parfor() {
        let src = "\
parfor i = 1:10
    who('x');
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "PFWHOS"), "got: {diags:?}");
    }

    #[test]
    fn pfwhos_no_fire_outside_parfor() {
        let src = "who('x');\n";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "PFWHOS"), "got: {diags:?}");
    }

    #[test]
    fn pfwhos_no_fire_on_whos_with_file_flag() {
        let src = "\
parfor i = 1:10
    whos('-file', 'x.mat');
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "PFWHOS"), "got: {diags:?}");
    }

    #[test]
    fn pfwhos_fires_on_whos_command_form() {
        let src = "\
parfor i = 1:10
    whos
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "PFWHOS"), "got: {diags:?}");
    }

    // -- PFTUSE --------------------------------------------------------------

    #[test]
    fn pftuse_fires_on_self_use_non_reduction() {
        let src = "\
parfor i = 1:10
    x = x(i);
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "PFTUSE"), "got: {diags:?}");
    }

    #[test]
    fn pftuse_no_fire_outside_parfor() {
        let src = "x = x(i);\n";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "PFTUSE"), "got: {diags:?}");
    }

    // -- PFRNC ---------------------------------------------------------------

    #[test]
    fn pfrnc_fires_on_mixed_reduction_operators() {
        let src = "\
parfor i = 1:10
    x = x + x * 2;
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "PFRNC"), "got: {diags:?}");
    }

    #[test]
    fn pfrnc_no_fire_on_single_reduction_operator() {
        let src = "\
parfor i = 1:10
    x = x + 2;
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "PFRNC"), "got: {diags:?}");
    }

    /// Ensure representative test sources parse without syntax errors.
    #[test]
    fn test_sources_parse() {
        let sources = [
            "y = zeros(3) && x;\n",
            "y = size(x) == [1 2];\n",
            "assert(true);\n",
            "cellfun('isempty', x);\n",
            "try\n    x = 1;\ncatch\nend\n",
            "parfor i = 1:10\n    eval(sprintf('x%d', i));\nend\n",
        ];
        for src in sources {
            let tree = parse(src);
            assert!(!tree.root_node().has_error(), "parse error for: {src:?}");
        }
    }
}
