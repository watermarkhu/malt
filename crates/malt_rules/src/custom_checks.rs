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
//! Configurable code-complexity and style checks from MATLAB's Code Analyzer
//! "Custom Checks" group, handled by a single file-level engine
//! (`CustomChecksEngine`).
//!
//! * Metric checks (`FCNIL`, `FCNOL`, `FCNLL`, `LLMNC`, `MNCSN`, `CYCCOM`,
//!   `MCYCCOM`, `SCYCCOM`, `MSCYCCOM`, `ACYCCOM`, `MACYCCOM`) have a
//!   configurable limit; a limit of `0` disables the check.
//! * The `DAF*` checks and `SYSBANG` flag every use of a construct that the
//!   configuration disallows. They are controlled by boolean `disallow_*`
//!   options. As in MATLAB, all `DAF*` checks are off by default; `SYSBANG`
//!   is on by default.
//!
//! `LLMCC` (line length excluding comments) is not implemented.
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
//!     try                                % DAFTC (with disallow_try_catch = true)
//!         x = 1;
//!     catch
//!     end
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
//! max_function_inputs = 7                 # FCNIL
//! max_function_outputs = 7                # FCNOL
//! max_function_lines = 200                # FCNLL
//! max_line_length = 120                   # LLMNC
//! max_nesting_depth = 5                   # MNCSN
//! max_cyclomatic_complexity = 15          # CYCCOM, MCYCCOM (functions)
//! max_script_cyclomatic_complexity = 20   # SCYCCOM, MSCYCCOM (scripts)
//! max_anonymous_cyclomatic_complexity = 10 # ACYCCOM, MACYCCOM
//! disallow_bang = true                    # SYSBANG
//! disallow_try_catch = false              # DAFTC
//! disallow_persistent = false             # DAFPV
//! disallow_continue = false               # DAFCO
//! disallow_break = false                  # DAFBR
//! disallow_return = false                 # DAFRT
//! disallow_script = false                 # DAFSC
//! disallow_nested_functions = false       # DAFNF
//! disallow_command_syntax = false         # DAFCF
//! disallow_anonymous_functions = false    # DAFAF
//! disallow_char_vectors = false           # DAFCV
//! disallow_cellstr = false                # DAFCVC
//! disallow_varargin = false               # DAFVI
//! disallow_varargout = false              # DAFVO
//! ```

use malt_core::{Category, Config, Diagnostic, FileContext, Rule, Severity};
use serde::Deserialize;
use tree_sitter::Node;

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

const fn default_true() -> bool {
    true
}
const fn d_inputs() -> usize {
    7
}
const fn d_outputs() -> usize {
    7
}
const fn d_lines() -> usize {
    200
}
const fn d_line_length() -> usize {
    120
}
const fn d_nesting() -> usize {
    5
}
const fn d_cyc() -> usize {
    15
}
const fn d_script_cyc() -> usize {
    20
}
const fn d_anon_cyc() -> usize {
    10
}

/// Configuration for custom checks.
///
/// Deserialized from the `[lint.rules.CUSTOM_CHECKS]` section in `.malt.toml`.
/// Numeric limits disable their check when set to `0`; the `disallow_*`
/// flags enable the corresponding `DAF*` / `SYSBANG` check.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomChecksConfig {
    /// Maximum number of function input arguments (FCNIL).
    #[serde(default = "d_inputs")]
    pub max_function_inputs: usize,
    /// Maximum number of function output arguments (FCNOL).
    #[serde(default = "d_outputs")]
    pub max_function_outputs: usize,
    /// Maximum number of lines in a function (FCNLL).
    #[serde(default = "d_lines")]
    pub max_function_lines: usize,
    /// Maximum line length in characters (LLMNC).
    #[serde(default = "d_line_length")]
    pub max_line_length: usize,
    /// Maximum control-statement nesting depth (MNCSN).
    #[serde(default = "d_nesting")]
    pub max_nesting_depth: usize,
    /// Maximum cyclomatic complexity of a function (CYCCOM/MCYCCOM).
    #[serde(default = "d_cyc")]
    pub max_cyclomatic_complexity: usize,
    /// Maximum cyclomatic complexity of a script (SCYCCOM/MSCYCCOM).
    #[serde(default = "d_script_cyc", alias = "max_strict_cyclomatic_complexity")]
    pub max_script_cyclomatic_complexity: usize,
    /// Maximum cyclomatic complexity of an anonymous function (ACYCCOM/MACYCCOM).
    #[serde(default = "d_anon_cyc", alias = "max_avg_cyclomatic_complexity")]
    pub max_anonymous_cyclomatic_complexity: usize,
    /// Flag the bang (`!`) operator (SYSBANG).
    #[serde(default = "default_true")]
    pub disallow_bang: bool,
    /// Flag try/catch statements (DAFTC).
    #[serde(default)]
    pub disallow_try_catch: bool,
    /// Flag persistent variables (DAFPV).
    #[serde(default)]
    pub disallow_persistent: bool,
    /// Flag `continue` statements (DAFCO).
    #[serde(default)]
    pub disallow_continue: bool,
    /// Flag `break` statements (DAFBR).
    #[serde(default)]
    pub disallow_break: bool,
    /// Flag `return` statements (DAFRT).
    #[serde(default)]
    pub disallow_return: bool,
    /// Flag script files (DAFSC).
    #[serde(default)]
    pub disallow_script: bool,
    /// Flag nested functions (DAFNF).
    #[serde(default)]
    pub disallow_nested_functions: bool,
    /// Flag command-syntax function calls (DAFCF).
    #[serde(default)]
    pub disallow_command_syntax: bool,
    /// Flag anonymous functions (DAFAF).
    #[serde(default)]
    pub disallow_anonymous_functions: bool,
    /// Flag character vectors (DAFCV).
    #[serde(default)]
    pub disallow_char_vectors: bool,
    /// Flag cell arrays of character vectors (DAFCVC).
    #[serde(default)]
    pub disallow_cellstr: bool,
    /// Flag use of `varargin` (DAFVI).
    #[serde(default)]
    pub disallow_varargin: bool,
    /// Flag use of `varargout` (DAFVO).
    #[serde(default)]
    pub disallow_varargout: bool,
}

impl Default for CustomChecksConfig {
    fn default() -> Self {
        Self {
            max_function_inputs: d_inputs(),
            max_function_outputs: d_outputs(),
            max_function_lines: d_lines(),
            max_line_length: d_line_length(),
            max_nesting_depth: d_nesting(),
            max_cyclomatic_complexity: d_cyc(),
            max_script_cyclomatic_complexity: d_script_cyc(),
            max_anonymous_cyclomatic_complexity: d_anon_cyc(),
            disallow_bang: true,
            disallow_try_catch: false,
            disallow_persistent: false,
            disallow_continue: false,
            disallow_break: false,
            disallow_return: false,
            disallow_script: false,
            disallow_nested_functions: false,
            disallow_command_syntax: false,
            disallow_anonymous_functions: false,
            disallow_char_vectors: false,
            disallow_cellstr: false,
            disallow_varargin: false,
            disallow_varargout: false,
        }
    }
}

/// `true` when `limit` is enabled (non-zero) and `value` exceeds it.
fn over(value: usize, limit: usize) -> bool {
    limit > 0 && value > limit
}

/// Message for a "disallowed by configuration" check.
fn disallowed(what: &str) -> String {
    format!("Use of {what} is disallowed by custom code analyzer configuration.")
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Custom Checks Engine: a file-level rule handling the Code Analyzer
/// "Custom Checks" group in a single traversal.
pub struct CustomChecksEngine {
    config: CustomChecksConfig,
}

fn diag_at(rule_id: &'static str, message: String, node: Node) -> Diagnostic {
    let pos = node.start_position();
    Diagnostic {
        rule_id,
        message,
        severity: Severity::Warning,
        byte_range: node.start_byte()..node.end_byte(),
        line: pos.row + 1,
        column: pos.column + 1,
        fix: None,
    }
}

fn text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// McCabe cyclomatic complexity of `node`, as `(standard, modified)`.
///
/// Decision points: `if`/`elseif`/`for`/`while`/`catch` and the short-circuit
/// operators `&&` / `||`. Standard complexity adds one per `case`; modified
/// complexity counts a whole `switch` as a single decision. Nested functions
/// and (when `skip_lambdas`) anonymous functions are analysed separately.
fn complexity(node: Node, skip_lambdas: bool) -> (usize, usize) {
    let mut std = 1;
    let mut modified = 1;
    count_decisions(node, true, skip_lambdas, &mut std, &mut modified);
    (std, modified)
}

fn count_decisions(
    node: Node,
    is_root: bool,
    skip_lambdas: bool,
    std: &mut usize,
    modified: &mut usize,
) {
    let kind = node.kind();
    if !is_root && (kind == "function_definition" || (skip_lambdas && kind == "lambda")) {
        return;
    }
    match kind {
        "if_statement" | "elseif_clause" | "for_statement" | "while_statement" | "catch_clause" => {
            *std += 1;
            *modified += 1;
        }
        "switch_statement" => *modified += 1,
        "case_clause" => *std += 1,
        "boolean_operator" => {
            let mut c = node.walk();
            let short = node
                .children(&mut c)
                .any(|ch| ch.kind() == "&&" || ch.kind() == "||");
            if short {
                *std += 1;
                *modified += 1;
            }
        }
        _ => {}
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        count_decisions(child, false, skip_lambdas, std, modified);
    }
}

/// Deepest nesting of control statements below `node` (not entering nested
/// functions).
fn nesting_depth(node: Node, current: usize, is_root: bool) -> usize {
    if !is_root && node.kind() == "function_definition" {
        return current;
    }
    let nests = matches!(
        node.kind(),
        "if_statement" | "for_statement" | "while_statement" | "switch_statement" | "try_statement"
    );
    let depth = if nests { current + 1 } else { current };
    let mut best = depth;
    let mut c = node.walk();
    for child in node.children(&mut c) {
        best = best.max(nesting_depth(child, depth, false));
    }
    best
}

/// Is `node` a single-quoted character vector literal?
fn is_char_vector(node: Node, source: &str) -> bool {
    node.kind() == "string" && text(node, source).starts_with('\'')
}

/// Is `node` a non-empty cell array whose elements are all character vectors?
fn is_cellstr(node: Node, source: &str) -> bool {
    if node.kind() != "cell" {
        return false;
    }
    let mut elems = 0;
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        let mut c = n.walk();
        for child in n.children(&mut c) {
            match child.kind() {
                "row" => stack.push(child),
                "string" => {
                    if !is_char_vector(child, source) {
                        return false;
                    }
                    elems += 1;
                }
                "{" | "}" | "," | ";" | "comment" | "line_continuation" => {}
                _ => return false,
            }
        }
    }
    elems > 0
}

impl CustomChecksEngine {
    /// Factory constructor. Reads rule-specific params from config.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: CustomChecksConfig = config.rule_params("CUSTOM_CHECKS");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Line length (LLMNC).
    fn check_line_metrics(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        if self.config.max_line_length == 0 {
            return diagnostics;
        }
        let mut byte_offset = 0;
        for (line_idx, raw) in source.split_inclusive('\n').enumerate() {
            let line = raw.trim_end_matches(['\n', '\r']);
            if line.chars().count() > self.config.max_line_length {
                diagnostics.push(Diagnostic {
                    rule_id: "LLMNC",
                    message: format!(
                        "Line has more than {} characters (including whitespaces). This makes the line difficult to understand and maintain.",
                        self.config.max_line_length
                    ),
                    severity: Severity::Warning,
                    byte_range: byte_offset..byte_offset + line.len(),
                    line: line_idx + 1,
                    column: 1,
                    fix: None,
                });
            }
            byte_offset += raw.len();
        }
        diagnostics
    }

    /// Is this file a script (top-level code, no leading function / classdef)?
    fn script_node<'a>(root: Node<'a>) -> Option<Node<'a>> {
        let mut c = root.walk();
        let mut first = None;
        for child in root.children(&mut c) {
            if !child.is_named() || child.kind() == "comment" {
                continue;
            }
            if child.kind() == "class_definition" {
                return None;
            }
            if first.is_none() {
                first = Some(child);
            }
        }
        first.filter(|n| n.kind() != "function_definition")
    }

    /// Walk the tree once and emit all node-level diagnostics.
    fn walk(
        &self,
        node: Node,
        source: &str,
        in_function: bool,
        in_cellstr: bool,
        out: &mut Vec<Diagnostic>,
    ) {
        let cfg = &self.config;
        let mut now_in_function = in_function;
        let mut now_in_cellstr = in_cellstr;

        match node.kind() {
            "function_definition" => {
                if in_function && cfg.disallow_nested_functions {
                    out.push(diag_at("DAFNF", disallowed("a nested function"), node));
                }
                now_in_function = true;
                self.function_checks(node, out);
            }
            "try_statement" if cfg.disallow_try_catch => {
                out.push(diag_at("DAFTC", disallowed("try/catch statement"), node));
            }
            "persistent_operator" if cfg.disallow_persistent => {
                out.push(diag_at("DAFPV", disallowed("persistent variable"), node));
            }
            "continue_statement" if cfg.disallow_continue => {
                out.push(diag_at("DAFCO", disallowed("continue statement"), node));
            }
            "break_statement" if cfg.disallow_break => {
                out.push(diag_at("DAFBR", disallowed("break statement"), node));
            }
            "return_statement" if cfg.disallow_return => {
                out.push(diag_at("DAFRT", disallowed("return statement"), node));
            }
            "command" => {
                if text(node, source).starts_with('!') {
                    if cfg.disallow_bang {
                        out.push(diag_at("SYSBANG", disallowed("bang operator"), node));
                    }
                } else if cfg.disallow_command_syntax {
                    out.push(diag_at(
                        "DAFCF",
                        disallowed("command syntax to call a function"),
                        node,
                    ));
                }
            }
            "lambda" => {
                if cfg.disallow_anonymous_functions {
                    out.push(diag_at("DAFAF", disallowed("an anonymous function"), node));
                }
                let (std, modified) = complexity(node, false);
                let limit = cfg.max_anonymous_cyclomatic_complexity;
                if over(std, limit) {
                    out.push(diag_at(
                        "ACYCCOM",
                        format!("Anonymous function has a McCabe cyclomatic complexity of more than {limit}. This makes the anonymous function difficult to understand and maintain."),
                        node,
                    ));
                }
                if over(modified, limit) {
                    out.push(diag_at(
                        "MACYCCOM",
                        format!("Anonymous function has a modified cyclomatic complexity of more than {limit}. This makes the anonymous function difficult to understand and maintain."),
                        node,
                    ));
                }
            }
            "cell" if cfg.disallow_cellstr && !in_cellstr && is_cellstr(node, source) => {
                out.push(diag_at(
                    "DAFCVC",
                    disallowed("cell array of character vectors"),
                    node,
                ));
                now_in_cellstr = true;
            }
            "string"
                if cfg.disallow_char_vectors && !in_cellstr && is_char_vector(node, source) =>
            {
                out.push(diag_at("DAFCV", disallowed("character vector"), node));
            }
            "identifier" => {
                if let Some(parent) = node.parent() {
                    let name = text(node, source);
                    if cfg.disallow_varargin
                        && name == "varargin"
                        && parent.kind() == "function_arguments"
                    {
                        out.push(diag_at("DAFVI", disallowed("varargin"), node));
                    }
                    if cfg.disallow_varargout && name == "varargout" {
                        let in_output = parent.kind() == "function_output"
                            || (parent.kind() == "multioutput_variable"
                                && parent
                                    .parent()
                                    .is_some_and(|g| g.kind() == "function_output"));
                        if in_output {
                            out.push(diag_at("DAFVO", disallowed("varargout"), node));
                        }
                    }
                }
            }
            _ => {}
        }

        let mut c = node.walk();
        for child in node.children(&mut c) {
            self.walk(child, source, now_in_function, now_in_cellstr, out);
        }
    }

    /// Function-level metrics: FCNIL, FCNOL, FCNLL, MNCSN, CYCCOM, MCYCCOM.
    fn function_checks(&self, func: Node, out: &mut Vec<Diagnostic>) {
        let cfg = &self.config;
        let inputs = Self::count_params(func, "function_arguments");
        let outputs = Self::count_params(func, "function_output");
        let lines = func.end_position().row - func.start_position().row + 1;

        if over(inputs, cfg.max_function_inputs) {
            out.push(diag_at(
                "FCNIL",
                format!("Function has more than {} input arguments. This makes the function difficult to understand and maintain.", cfg.max_function_inputs),
                func,
            ));
        }
        if over(outputs, cfg.max_function_outputs) {
            out.push(diag_at(
                "FCNOL",
                format!("Function has more than {} output arguments. This makes the function difficult to understand and maintain.", cfg.max_function_outputs),
                func,
            ));
        }
        if over(lines, cfg.max_function_lines) {
            out.push(diag_at(
                "FCNLL",
                format!("Function has more than {} lines. This makes the function difficult to understand and maintain.", cfg.max_function_lines),
                func,
            ));
        }
        let depth = nesting_depth(func, 0, true);
        if over(depth, cfg.max_nesting_depth) {
            out.push(diag_at(
                "MNCSN",
                format!("This control statement is deeply nested (nesting level = {depth}) and might have more deeply nested control statements. This makes the code difficult to understand and maintain."),
                func,
            ));
        }
        let (std, modified) = complexity(func, true);
        let limit = cfg.max_cyclomatic_complexity;
        if over(std, limit) {
            out.push(diag_at(
                "CYCCOM",
                format!("Function has a McCabe cyclomatic complexity of more than {limit}. This makes the function difficult to understand and maintain."),
                func,
            ));
        }
        if over(modified, limit) {
            out.push(diag_at(
                "MCYCCOM",
                format!("Function has a modified cyclomatic complexity of more than {limit}. This makes the function difficult to understand and maintain."),
                func,
            ));
        }
    }

    /// Count identifiers in a function's `function_arguments` / `function_output`.
    fn count_params(func: Node, kind: &str) -> usize {
        let mut c = func.walk();
        for child in func.children(&mut c) {
            if child.kind() == kind {
                let mut n = 0;
                let mut cc = child.walk();
                for p in child.children(&mut cc) {
                    match p.kind() {
                        "identifier" => n += 1,
                        "multioutput_variable" => {
                            let mut ic = p.walk();
                            n += p
                                .children(&mut ic)
                                .filter(|x| x.kind() == "identifier")
                                .count();
                        }
                        _ => {}
                    }
                }
                return n;
            }
        }
        0
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
        let root = ctx.tree.root_node();
        let mut diagnostics = self.check_line_metrics(ctx.source);

        if let Some(first) = Self::script_node(root) {
            if self.config.disallow_script {
                diagnostics.push(diag_at("DAFSC", disallowed("a script"), first));
            }
            let (std, modified) = complexity(root, true);
            let limit = self.config.max_script_cyclomatic_complexity;
            if over(std, limit) {
                diagnostics.push(diag_at(
                    "SCYCCOM",
                    format!("Script has a McCabe cyclomatic complexity of more than {limit}. This makes the script difficult to understand and maintain."),
                    first,
                ));
            }
            if over(modified, limit) {
                diagnostics.push(diag_at(
                    "MSCYCCOM",
                    format!("Script has a modified cyclomatic complexity of more than {limit}. This makes the script difficult to understand and maintain."),
                    first,
                ));
            }
        }

        self.walk(root, ctx.source, false, false, &mut diagnostics);
        diagnostics
    }
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

    fn engine_with(params: &str) -> Box<dyn Rule> {
        let config = Config::from_toml(&format!("[lint.rules.CUSTOM_CHECKS]\n{params}\n")).unwrap();
        CustomChecksEngine::from_config(&config)
    }

    fn count(diags: &[Diagnostic], id: &str) -> usize {
        diags.iter().filter(|d| d.rule_id == id).count()
    }

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

    #[test]
    fn sysbang_can_be_disabled() {
        let e = engine_with("disallow_bang = false");
        assert!(!has_id(&lint_file(&*e, "!ls\n"), "SYSBANG"));
    }

    #[test]
    fn llmnc_fires_and_message_has_limit() {
        let source = format!("x = {};\n", "1".repeat(120));
        let diags = lint_file(&*engine(), &source);
        let d = diags.iter().find(|d| d.rule_id == "LLMNC").expect("LLMNC");
        assert!(
            d.message.contains("more than 120 characters"),
            "{}",
            d.message
        );
        assert!(!d.message.contains("VAR_"));
    }

    #[test]
    fn llmnc_ok_on_short_line() {
        assert!(!has_id(&lint_file(&*engine(), "x = 1;\n"), "LLMNC"));
    }

    #[test]
    fn llmnc_zero_disables() {
        let e = engine_with("max_line_length = 0");
        let source = format!("x = {};\n", "1".repeat(500));
        assert!(!has_id(&lint_file(&*e, &source), "LLMNC"));
    }

    #[test]
    fn no_invented_semicolon_check() {
        let diags = lint_file(&*engine(), "a = 1; b = 2; c = 3; d = 4; e = 5;\n");
        assert!(!has_id(&diags, "DAFSC"), "got: {diags:?}");
    }

    #[test]
    fn fcnil_and_fcnol() {
        let e = engine_with("max_function_inputs = 2\nmax_function_outputs = 2");
        let diags = lint_file(&*e, "function [a, b, c] = f(x, y, z)\nend\n");
        assert!(has_id(&diags, "FCNIL"), "got: {diags:?}");
        assert!(has_id(&diags, "FCNOL"), "got: {diags:?}");
        let diags = lint_file(&*e, "function [a, b] = f(x, y)\nend\n");
        assert!(!has_id(&diags, "FCNIL") && !has_id(&diags, "FCNOL"));
    }

    #[test]
    fn fcnll() {
        let e = engine_with("max_function_lines = 3");
        let long = "function f()\n    x = 1;\n    y = 2;\n    z = 3;\nend\n";
        assert!(has_id(&lint_file(&*e, long), "FCNLL"));
        assert!(!has_id(&lint_file(&*e, "function f()\nend\n"), "FCNLL"));
    }

    #[test]
    fn mncsn() {
        let e = engine_with("max_nesting_depth = 2");
        let deep = "function f()\n    if a\n        if b\n            if c\n                x = 1;\n            end\n        end\n    end\nend\n";
        let diags = lint_file(&*e, deep);
        let d = diags.iter().find(|d| d.rule_id == "MNCSN").expect("MNCSN");
        assert!(d.message.contains("nesting level = 3"), "{}", d.message);
        let shallow =
            "function f()\n    if a\n        if b\n            x = 1;\n        end\n    end\nend\n";
        assert!(!has_id(&lint_file(&*e, shallow), "MNCSN"));
    }

    // -- DAF* -------------------------------------------------------------

    fn daf(flag: &str, id: &str, bad: &str, good: &str) {
        let off = lint_file(&*engine(), bad);
        assert!(!has_id(&off, id), "{id} must be off by default: {off:?}");
        let e = engine_with(&format!("{flag} = true"));
        let on = lint_file(&*e, bad);
        assert!(has_id(&on, id), "{id} should fire: {on:?}");
        let d = on.iter().find(|d| d.rule_id == id).unwrap();
        assert!(d
            .message
            .contains("disallowed by custom code analyzer configuration"));
        let ok = lint_file(&*e, good);
        assert!(!has_id(&ok, id), "{id} should not fire: {ok:?}");
    }

    #[test]
    fn daftc() {
        daf(
            "disallow_try_catch",
            "DAFTC",
            "try\n x = 1;\ncatch\nend\n",
            "x = 1;\n",
        );
    }

    #[test]
    fn dafpv() {
        daf(
            "disallow_persistent",
            "DAFPV",
            "function f()\n persistent p;\nend\n",
            "function f()\n x = 1;\nend\n",
        );
    }

    #[test]
    fn dafco() {
        daf(
            "disallow_continue",
            "DAFCO",
            "for i=1:3\n continue\nend\n",
            "for i=1:3\n break\nend\n",
        );
    }

    #[test]
    fn dafbr() {
        daf(
            "disallow_break",
            "DAFBR",
            "for i=1:3\n break\nend\n",
            "for i=1:3\n continue\nend\n",
        );
    }

    #[test]
    fn dafrt() {
        daf(
            "disallow_return",
            "DAFRT",
            "function f()\n return\nend\n",
            "function f()\n x = 1;\nend\n",
        );
    }

    #[test]
    fn dafsc() {
        daf(
            "disallow_script",
            "DAFSC",
            "x = 1;\ndisp(x)\n",
            "function f()\n x = 1;\nend\n",
        );
    }

    #[test]
    fn dafnf() {
        daf(
            "disallow_nested_functions",
            "DAFNF",
            "function f()\n function g()\n end\nend\n",
            "function f()\nend\nfunction g()\nend\n",
        );
    }

    #[test]
    fn dafcf() {
        daf(
            "disallow_command_syntax",
            "DAFCF",
            "hold on\n",
            "hold('on');\n",
        );
    }

    #[test]
    fn dafaf() {
        daf(
            "disallow_anonymous_functions",
            "DAFAF",
            "g = @(x) x + 1;\n",
            "g = @sin;\n",
        );
    }

    #[test]
    fn dafcv() {
        daf(
            "disallow_char_vectors",
            "DAFCV",
            "s = 'abc';\n",
            "s = \"abc\";\n",
        );
    }

    #[test]
    fn dafcvc() {
        daf(
            "disallow_cellstr",
            "DAFCVC",
            "c = {'a', 'b'};\n",
            "c = {1, 'b'};\n",
        );
        // Strings inside a flagged cellstr do not also raise DAFCV.
        let e = engine_with("disallow_cellstr = true\ndisallow_char_vectors = true");
        let diags = lint_file(&*e, "c = {'a', 'b'};\n");
        assert_eq!(count(&diags, "DAFCVC"), 1);
        assert_eq!(count(&diags, "DAFCV"), 0);
    }

    #[test]
    fn dafvi_and_dafvo() {
        daf(
            "disallow_varargin",
            "DAFVI",
            "function f(varargin)\nend\n",
            "function f(a)\n y = varargin;\nend\n",
        );
        daf(
            "disallow_varargout",
            "DAFVO",
            "function varargout = f()\nend\n",
            "function a = f()\nend\n",
        );
    }

    // -- complexity -------------------------------------------------------

    #[test]
    fn cyccom_and_mcyccom() {
        let e = engine_with("max_cyclomatic_complexity = 3");
        let ifs =
            "function f()\n    if a, x = 1; end\n    if b, x = 2; end\n    if c, x = 3; end\nend\n";
        let diags = lint_file(&*e, ifs);
        assert!(
            has_id(&diags, "CYCCOM") && has_id(&diags, "MCYCCOM"),
            "{diags:?}"
        );
        let two = "function f()\n    if a, x = 1; end\n    if b, x = 2; end\nend\n";
        let diags = lint_file(&*e, two);
        assert!(!has_id(&diags, "CYCCOM") && !has_id(&diags, "MCYCCOM"));
    }

    #[test]
    fn modified_complexity_counts_switch_once() {
        let e = engine_with("max_cyclomatic_complexity = 3");
        let sw = "function f(x)\n    switch x\n        case 1\n            a = 1;\n        case 2\n            a = 2;\n        case 3\n            a = 3;\n    end\nend\n";
        let diags = lint_file(&*e, sw);
        assert!(has_id(&diags, "CYCCOM"), "{diags:?}");
        assert!(!has_id(&diags, "MCYCCOM"), "{diags:?}");
    }

    #[test]
    fn comparisons_do_not_add_complexity() {
        let e = engine_with("max_cyclomatic_complexity = 3");
        let src = "function f()\n    x = a > 1;\n    y = b < 2;\n    z = c == 3;\nend\n";
        assert!(!has_id(&lint_file(&*e, src), "CYCCOM"));
    }

    #[test]
    fn cyccom_message_has_limit() {
        let e = engine_with("max_cyclomatic_complexity = 1");
        let diags = lint_file(&*e, "function f()\n    if a, x = 1; end\nend\n");
        let d = diags.iter().find(|d| d.rule_id == "CYCCOM").unwrap();
        assert!(d.message.contains("more than 1."), "{}", d.message);
    }

    #[test]
    fn scyccom_applies_to_scripts_only() {
        let e = engine_with("max_script_cyclomatic_complexity = 2");
        let script = "if a, x = 1; end\nif b, x = 2; end\n";
        let diags = lint_file(&*e, script);
        assert!(
            has_id(&diags, "SCYCCOM") && has_id(&diags, "MSCYCCOM"),
            "{diags:?}"
        );
        let func = "function f()\n    if a, x = 1; end\n    if b, x = 2; end\nend\n";
        let diags = lint_file(&*e, func);
        assert!(
            !has_id(&diags, "SCYCCOM") && !has_id(&diags, "MSCYCCOM"),
            "{diags:?}"
        );
    }

    #[test]
    fn acyccom_applies_to_anonymous_functions() {
        let e = engine_with("max_anonymous_cyclomatic_complexity = 2");
        let diags = lint_file(&*e, "g = @(x) x > 0 && x < 5 && x ~= 3;\n");
        assert!(
            has_id(&diags, "ACYCCOM") && has_id(&diags, "MACYCCOM"),
            "{diags:?}"
        );
        let diags = lint_file(&*e, "g = @(x) x > 0 && x < 5;\n");
        assert!(!has_id(&diags, "ACYCCOM"), "{diags:?}");
    }

    #[test]
    fn methods_are_not_special_cased() {
        let e = engine_with("max_cyclomatic_complexity = 2");
        let src = "classdef C\n    methods\n        function out = m(a)\n            if a > 0, out = 1; end\n            if a < 0, out = 2; end\n        end\n    end\nend\n";
        let diags = lint_file(&*e, src);
        assert!(has_id(&diags, "CYCCOM"), "{diags:?}");
    }

    #[test]
    fn ids_unique_in_docstring() {
        let src = include_str!("custom_checks.rs");
        let mut seen = std::collections::HashSet::new();
        for line in src
            .lines()
            .filter(|l| l.starts_with("//! | ") && !l.contains("---"))
        {
            let id = line.trim_start_matches("//! | ").split(' ').next().unwrap();
            if id != "Check" {
                assert!(seen.insert(id.to_string()), "duplicate {id}");
            }
        }
    }
}
