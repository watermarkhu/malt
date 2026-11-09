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
//! Detects likely bugs and logic errors in MATLAB code. All 34 checks share a
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
//! | IFCDUP | error | no | The statements under this elseif condition cannot be reached because it is a duplicate of the if/elseif condition on line N. This indicates a bug in the code. Remove or change the condition. |
//! | CTRUE | error | no | This logical comparison always returns true. Did you mean to use F to evaluate function argument: F(...A<=1...)? A comparison of a logical-valued function such as `any(A)` against a numeric literal that every logical value satisfies. |
//! | CFALSE | error | no | This logical comparison always returns false. Did you mean to use F to evaluate function argument: F(...A>1...)? A comparison of a logical-valued function such as `any(A)` against a numeric literal that no logical value satisfies. |
//! | SHOCIRT | error | no | The && operator is unexpected because isscalar(A && B) always returns true. (`isscalar`, `islogical`, `isreal` applied to `&&` or `||`.) |
//! | SHOCIRF | error | no | The || operator is unexpected because isempty(A || B) always returns false. (`isempty`, `isnan`, `ischar`, `isnumeric`, … applied to `&&` or `||`.) |
//! | DEBUGFUN | error | no | Debug functions (`dbcont`, `dbquit`, `dbup`, `dbdown`, `dbstep`) are intended to be used at the command line. At runtime, they will generate an error. Remove the debug function. |
//! | INCR | error | yes (unsafe) | `++x` does not increment the value of x. To increase the value by 1, use x = x + 1. |
//! | DECR | error | yes (unsafe) | `--x` does not decrement the value of x. To decrease the value by 1, use x = x - 1. |
//! | CMDAND | error | yes (unsafe) | Command-syntax statement `a &b`: use 'A && B' or 'A & B' to test whether A and B are both true in MATLAB. |
//! | CMDOR | error | yes (unsafe) | Command-syntax statement `a |b`: use 'A || B' or 'A | B' to test whether either A or B is true in MATLAB. |
//! | RHSFN | warning | no | The expression cannot be assigned to multiple values. |
//! | FNAN | error | yes (unsafe) | Use ISNAN when comparing values to NaN (`x == NaN`, `x ~= NaN`). |
//! | LOGEMP | error | no | Using 'isempty' on a logical expression creates incorrect results. To determine if all the conditions are false, use '~any(..., "all")' instead. |
//! | STCUL | error | no | The comparison will likely fail due to case mismatch. |
//! | LBODUP | error | no | Since both operands are identical, the second operand has no effect on the && operation (`A && A`, `A | A`, …). This indicates a bug in the code. Change one of the operands or remove the operation. |
//! | FUNFUN | error | no | The first input argument must be a function handle. Did you mean '@name'? (A function name passed as a string to `arrayfun`, `structfun`, `bsxfun`, …; legacy `cellfun('isempty', …)` names are allowed.) |
//! | DEFSIZE | error | no | Do not overload 'size' for fundamental data types. |
//! | VARARG | error | no | Initialize VARARGOUT with a CELL. |
//! | STRCMPCSTR | error | no | 'strcmp' always returns false for string elements of a cell array. Use ["str1", "str2"] instead of {"str1", "str2"}. |
//! | ASSRT | error | no | The first input argument to 'assert' must be a condition. To always throw an error, use 'error(msg)' instead. |
//! | BDSCA2 | error | no | Operands to '\|\|' and '&&' must be scalar values (matrix literal, range or sized `zeros`/`ones`/… operand). Use 'all' or 'any' to convert this value into a scalar value or use the element-wise operators '\|' or '&' instead. |
//! | NOPRC | error | no | A line break terminates the statement so it may be incomplete. Use ellipsis (...) to continue the statement. Or add a semicolon to hide the output. |
//! | MOCUP | error | no | Variable X may be cleared before the cleanup function that references X executes, resulting in an undefined variable error. |
//! | MDUPC | error | no | The case value X is a duplicate of one on line N. |
//! | MNANC | error | no | NaN never compares equal to any value, so this case will never be matched (`case NaN`). |
//! | MULCC | error | no | This case cannot be matched due to a call to UPPER or LOWER on the SWITCH value. |
//! | MEXCEP | error | no | To report an MException as a warning (`warning(E.message)`), use a format specifier to ensure the message is printed correctly. For example, 'warning(E.identifier, "%s", E.message)'. |
//! | PFUIXE | error | no | The index variable X might be used after the PARFOR loop on line N, but it is unavailable after the loop. |
//! | PFBFN | error | no | Use of this function is invalid inside a PARFOR loop because it accesses or modifies the workspace in a non-transparent way. |
//! | PFWHOS | error | no | Using "who" or "whos" without "-file" is invalid inside a PARFOR loop because it accesses the workspace in a non-transparent way. |
//! | PFTUSE | error | no | The temporary variable X is used after the PARFOR loop on line N, but its value is not available after the loop. |
//! | PFRNC | error | no | Parfor reduction variable X must be used in the same position in each assignment statement when using non-commutative reduction operations '*', '[,]', or '[;]'. |
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
//! debug_functions = ["dbcont", "dbquit", "dbup", "dbdown", "dbstep"]
//! higher_order_functions = ["cellfun", "arrayfun", "structfun", "bsxfun", "spfun"]
//! ```

mod check_assert_constant;
mod check_command_logical;
mod check_debug_function;
mod check_funfun;
mod check_incr_decr;
mod check_isempty_logical;
mod check_logical_comparison;
mod check_mocup;
mod check_nan_comparison;
mod check_nonscalar_operand;
mod check_parfor;
mod check_short_circuit;
mod check_size_overload;
mod check_strcmp_case_mismatch;
mod check_strcmp_cell;
mod check_switch_upper_lower;
mod check_warning_message;

use malt_core::{Category, Config, Diagnostic, FileContext, Fix, NodeContext, Rule, Severity};
use serde::Deserialize;
use tree_sitter::Node;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Node types targeted by node-level checks.
const TARGET_NODES: &[&str] = &[
    "switch_statement",
    "boolean_operator",
    "binary_operator",
    "comparison_operator",
    "function_call",
    "unary_operator",
    "command",
];

/// Debug functions that should not appear in production code.
const DEFAULT_DEBUG_FUNCTIONS: &[&str] = &["dbcont", "dbquit", "dbup", "dbdown", "dbstep"];

/// Higher-order functions that accept function handles.
const DEFAULT_HIGHER_ORDER_FUNCTIONS: &[&str] =
    &["cellfun", "arrayfun", "structfun", "bsxfun", "spfun"];

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
/// file-level checks for 34 bug-detection checks from MATLAB's Code Analyzer.
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
            let mut conditions: Vec<(String, Node, &str)> = Vec::new();

            // Collect the main if condition.
            if let Some(cond) = node
                .child_by_field_name("condition")
                .or_else(|| find_condition_child(node))
            {
                let cond_text = normalize_whitespace(node_text(cond, source));
                conditions.push((cond_text, cond, "if"));
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
                        if let Some((_, first_cond, first_kw)) =
                            conditions.iter().find(|(t, _, _)| *t == cond_text)
                        {
                            let pos = cond.start_position();
                            let first_line = first_cond.start_position().row + 1;
                            diagnostics.push(Diagnostic {
                                rule_id: "IFCDUP",
                                message: format!(
                                    "The statements under this elseif condition cannot be reached because it is a duplicate of the {} condition on line {}. This indicates a bug in the code. Remove or change the condition.",
                                    first_kw, first_line
                                ),
                                severity: Severity::Error,
                                byte_range: cond.start_byte()..cond.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        } else {
                            conditions.push((cond_text, cond, "elseif"));
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

    /// MDUPC: Duplicate case values in switch.
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
                    let is_incr_decr = node
                        .child_by_field_name("operand")
                        .and_then(|o| {
                            (o.kind() == "unary_operator")
                                .then(|| o.child_by_field_name("operand"))
                                .flatten()
                        })
                        .is_some_and(|o| o.kind() == "identifier");
                    let follows_statement = node
                        .prev_named_sibling()
                        .is_some_and(|p| p.kind() != "comment");
                    if (op.starts_with('+') || op.starts_with('-'))
                        && !is_incr_decr
                        && follows_statement
                    {
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

    /// RHSFN: an assignment whose left-hand side is a multiple-output
    /// expression but whose right-hand side cannot return multiple values.
    ///
    /// # Limitations (grammar)
    ///
    /// Only right-hand sides that can never produce several values are flagged
    /// (literals, matrices, cells, ranges and operator expressions). A
    /// `function_call`, a bare identifier or a field expression may be a call to
    /// a multi-output function (the grammar cannot tell calls from indexing), so
    /// those are left alone.
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
                if lhs.kind() == "multioutput_variable"
                    && matches!(
                        rhs.kind(),
                        "number"
                            | "string"
                            | "matrix"
                            | "cell"
                            | "range"
                            | "binary_operator"
                            | "boolean_operator"
                            | "comparison_operator"
                            | "unary_operator"
                    )
                {
                    let pos = rhs.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "RHSFN",
                        message: "The expression cannot be assigned to multiple values."
                            .to_string(),
                        severity: Severity::Warning,
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
            "command" => {
                diagnostics.extend(self.check_command_logical(node, source));
            }
            "boolean_operator" => {
                diagnostics.extend(self.check_identical_operands(node, source));
                diagnostics.extend(self.check_nonscalar_operand(node, source));
            }
            "binary_operator" => {
                diagnostics.extend(self.check_identical_operands(node, source));
            }
            "unary_operator" => {
                diagnostics.extend(self.check_incr_decr(node, source));
            }
            "comparison_operator" => {
                diagnostics.extend(self.check_nan_comparison(node, source));
                diagnostics.extend(self.check_logical_comparison(node, source));
            }
            "switch_statement" => {
                diagnostics.extend(self.check_switch_upper_lower(node, source));
                diagnostics.extend(self.check_switch_nan_case(node, source));
            }
            "function_call" => {
                diagnostics.extend(self.check_debug_function(node, source));
                diagnostics.extend(self.check_strcmp_case_mismatch(node, source));
                diagnostics.extend(self.check_strcmp_cell(node, source));
                diagnostics.extend(self.check_funfun(node, source));
                diagnostics.extend(self.check_assert_condition(node, source));
                diagnostics.extend(self.check_isempty_logical(node, source));
                diagnostics.extend(self.check_short_circuit(node, source));
                diagnostics.extend(self.check_warning_message(node, source));
            }
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

/// Check if a node is a known non-scalar value: a matrix literal with at
/// least two elements, a range, or `zeros`/`ones`/`rand`/`randn`/`eye` called
/// with literal sizes whose product exceeds one.
pub(crate) fn is_array_expression(node: Node, source: &str) -> bool {
    match node.kind() {
        "matrix" => {
            let mut count = 0;
            let mut cursor = node.walk();
            for row in node.children(&mut cursor) {
                let mut inner = row.walk();
                count += row
                    .children(&mut inner)
                    .filter(|c| c.is_named() && c.kind() != "comment")
                    .count();
            }
            count > 1
        }
        "range" => true,
        "function_call" => {
            let Some(name) = extract_call_name(node, source) else {
                return false;
            };
            if !["zeros", "ones", "rand", "randn", "eye"].contains(&name) {
                return false;
            }
            let args = arg_nodes(node);
            if args.is_empty() || args.iter().any(|a| a.kind() != "number") {
                return false;
            }
            let product: f64 = args
                .iter()
                .filter_map(|a| node_text(*a, source).trim().parse::<f64>().ok())
                .product();
            product > 1.0
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

    #[test]
    fn ifcdup_message_names_keywords() {
        let diags = file_diags("if x\n    a = 1;\nelseif x\n    b = 2;\nend\n");
        let d = diags
            .iter()
            .find(|d| d.rule_id == "IFCDUP")
            .expect("IFCDUP");
        assert!(
            d.message
                .contains("duplicate of the if condition on line 1"),
            "got: {}",
            d.message
        );
    }

    #[test]
    fn noprc_no_fire_on_increment_statement() {
        let diags = file_diags("x = 1;\n++x;\n");
        assert!(!has_id(&diags, "NOPRC"), "got: {diags:?}");
    }

    // -- MDUPC ------------------------------------------------------

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

    // -- RHSFN ---------------------------------------------------------------

    #[test]
    fn rhsfn_fires_on_multioutput_with_single_value_rhs() {
        let src = "[a, b] = [1 2];\n";
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
    fn rhsfn_no_fire_on_identifier_or_field_rhs() {
        let diags = file_diags("[a, b] = obj.method;\n[c, d] = f;\n");
        assert!(!has_id(&diags, "RHSFN"), "got: {diags:?}");
    }

    #[test]
    fn rhsfn_is_a_warning() {
        let diags = file_diags("[m, n] = [1, 2];\n");
        assert_eq!(diags[0].severity, Severity::Warning);
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

    /// Ensure representative test sources parse without syntax errors.
    #[test]
    fn test_sources_parse() {
        let sources = [
            "y = zeros(3) && x;\n",
            "y = size(x) == [1 2];\n",
            "assert(true);\n",
            "arrayfun('f', x);\n",
            "parfor i = 1:10\n    eval(sprintf('x%d', i));\nend\n",
        ];
        for src in sources {
            let tree = parse(src);
            assert!(!tree.root_node().has_error(), "parse error for: {src:?}");
        }
    }
}
