//! # READABILITY_ENGINE: Readability Improvements
//!
//! ```malt
//! id = "READABILITY_ENGINE"
//! title = "Readability Improvements"
//! category = "readability"
//! severity = "info"
//! fix = true
//! icon = "lucide/eye"
//! slug = "readability"
//! ```
//!
//! ## Rule
//!
//! A multi-check rule engine that detects readability improvements in MATLAB
//! code. Instead of implementing one struct per check, a single
//! `ReadabilityEngine` dispatches to per-check-ID logic inside its `check()`
//! method (and a file-level pass for MFAMB), emitting diagnostics with the
//! specific check ID (e.g. `ISCHR`, `IJCL`, `RPMT1`).
//!
//! The 35 checks cover type-checking simplification (`isa(x, 'type')` →
//! dedicated predicates), dimension checks (`size(x,dim)==1` → `isrow`/`iscolumn`),
//! variable shadowing, unnecessary brackets, modern string functions,
//! character literals, formatting, control-flow style, error/warning message
//! IDs, input validation, output style, flip/rotate, redundant arithmetic and
//! boolean logic, size and logical helpers, `arguments` attributes, and
//! ambiguous identifiers.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | ASGSL | info | no | Assignment to variable might be unnecessary. |
//! | COMNL | info | yes | Newline following comma acts as a row separator. Replace the comma with a semicolon to make the row separation clearer. Alternatively, use an ellipsis (...) to continue the current row on the next line. |
//! | SPERR | info | no | ERROR takes SPRINTF-like arguments directly. |
//! | SPWRN | info | no | WARNING takes SPRINTF-like arguments directly. |
//! | NCHKE | info | no | Use NARGOUTCHK without ERROR. |
//! | DSPSP | info | yes | 'disp(sprintf(...))' can usually be replaced by 'fprintf(...\n)'. |
//! | DSPSY | info | no | 'display(sprintf(...))' can usually be replaced by 'fprintf(...\n)'. |
//! | STLOW | info | yes | In this comparison the call to UPPER/LOWER is unnecessary. |
//! | FLUDLR | info | yes | For readability, consider using rot90(x,2) instead of flipud(fliplr(x)) or fliplr(flipud(x)). |
//! | RPMT1 | info | yes | For readability, consider using 'ones(x,y)' instead of 'repmat(1,x,y)'. |
//! | RPMT0 | info | yes | For readability, consider using 'zeros(x,y)' instead of 'repmat(0,x,y)'. |
//! | RPMTT | info | yes | For readability, consider using 'true(x,y)' instead of 'repmat(true,x,y)'. |
//! | RPMTF | info | yes | For readability, consider using 'false(x,y)' instead of 'repmat(false,x,y)'. |
//! | RPMTI | info | yes | For readability, consider using 'Inf(x,y)' instead of 'repmat(Inf,x,y)'. |
//! | RPMTN | info | yes | For readability, consider using 'NaN(x,y)' instead of 'repmat(NaN,x,y)'. |
//! | PSIZE | info | yes | NUMEL(x) is usually faster than PROD(SIZE(x)). |
//! | LOGSUM | info | no | Consider using 'nnz' instead of 'sum' for logical vectors to improve readability. |
//! | LOGL | info | no | Use 'true' or 'false' instead of 'logical(1)' or 'logical(0)'. |
//! | ISCHR | info | yes | Use ISCHAR instead of comparing the class to 'char'. |
//! | ISSTR | info | yes | Use ISSTRUCT instead of comparing the class to 'struct'. |
//! | ISLOG | info | yes | Use ISLOGICAL instead of comparing the class to 'logical'. |
//! | ISCEL | info | yes | Use ISCELL instead of comparing the class to 'cell'. |
//! | IJCL | info | no | For improved robustness, consider replacing i and j by 1i. |
//! | ISMAT | info | yes | When checking if a variable is a matrix consider using ISMATRIX. |
//! | ISROW | info | yes | When checking if a variable is a row vector consider using ISROW. |
//! | ISCOL | info | yes | When checking if a variable is a column vector consider using ISCOLUMN. |
//! | NBRAK2 | info | yes | Use of brackets [] is unnecessary. |
//! | MFAMB | info | no | Code Analyzer cannot determine whether VAR_NAME is a variable or a function, and assumes it is a function. |
//! | FVINR | info | yes | For readability, add Input attribute to the input arguments block. |
//! | STREMP | info | yes | For readability, use '~contains(str1, str2)' instead of 'isempty(strfind(str1, str2))'. |
//! | STRCL1 | info | no | For readability, use '~contains(str1, str2)' instead of 'cellfun('isempty', strfind(str1, str2))'. |
//! | STRCLFH | info | no | For readability, use '~contains(str1, str2)' instead of 'cellfun(@isempty, strfind(str1, str2))'. |
//! | STRIFCND | info | no | For readability, use 'contains(str1, str2)' instead of 'strfind(str1, str2)'. |
//! | CHARTEN | info | yes | For readability, consider using 'newline' instead of 'char(10)'. |
//! | SPRINTFN | info | yes | For readability, consider using the 'newline' function instead of 'sprintf('\n')'. |
//!
//! ## Fix
//!
//! Rewrites the flagged construct into the clearer equivalent:
//!
//! - `isa(x, 'type')` → `ischar`/`isstring`/`islogical`/`iscell`/`isnumeric`
//!   (ISCHR, ISSTR, ISLOG, ISCEL, ISMAT).
//! - `size(x, dim) == 1` → `isrow(x)`/`iscolumn(x)` (ISROW, ISCOL).
//! - `x * 1` → `x`, `x * 0` → `zeros(size(x))`, `x + 0` → `x`,
//!   `x - 0` → `x` (RPMT1, RPMT0, RPMTI, RPMTN).
//! - `x | true` → `true` and `x & false` → `false` (RPMTT, RPMTF).
//! - `prod(size(x))` → `numel(x)`, `char(10)` → `newline`,
//!   `sprintf('%d', x)` → `num2str(x)`, `strcmp(s, '')` → `strlength(s)==0`.
//! - `flipud(fliplr(x))` → `rot90(x, 2)`, `[scalar]` → `scalar`,
//!   `disp(sprintf(...))` → `fprintf(...)`.
//! - A trailing comma before a newline in a matrix becomes a semicolon (COMNL).
//! - An `(Input)` attribute is inserted into attribute-less `arguments`
//!   blocks (FVINR).
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! if isa(x, 'char')
//! if a || true
//! y = flipud(fliplr(m));
//! n = prod(size(a));
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! if ischar(x)
//! if a
//! y = rot90(m, 2);
//! n = numel(a);
//! ```
//!
//! ### Fixed
//!
//! ```diff
//! - n = prod(size(a));
//! + n = numel(a);
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.READABILITY_ENGINE]
//! severity = "info"
//! disabled_checks = ["IJCL", "NBRAK2"]
//! ```

mod check_boolean_tautology;
mod check_char_literal;
mod check_comnl;
mod check_disp;
mod check_display;
mod check_error_warning;
mod check_find_logical;
mod check_fludlr;
mod check_fvinr;
mod check_ij_shadow;
mod check_inline_assignment;
mod check_isa;
mod check_narginchk;
mod check_prod_size;
mod check_redundant_arithmetic;
mod check_size_comparison;
mod check_sprintf;
mod check_stlow;
mod check_strfind;
mod check_strncmp;
mod check_sum_logical;
mod check_unnecessary_brackets;

use malt_core::{Category, Config, Diagnostic, FileContext, Fix, NodeContext, Rule, Severity};
use serde::Deserialize;

use crate::analysis::symbols::{DefKind, SymbolTable};

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for the readability engine.
///
/// Deserialized from `[lint.rules.READABILITY_ENGINE]` in `.malt.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ReadabilityConfig {
    /// Individual check IDs to disable within this engine.
    #[serde(default)]
    pub disabled_checks: Vec<String>,
}

// ---------------------------------------------------------------------------
// Lookup table: isa(x, 'type') → preferred function
// ---------------------------------------------------------------------------

/// Mapping from `isa` second argument (without quotes) to
/// (check_id, preferred_function, description).
pub(crate) const ISA_REPLACEMENTS: &[(&str, &str, &str, &str)] = &[
    (
        "char",
        "ISCHR",
        "ischar",
        "Use ISCHAR instead of comparing the class to 'char'.",
    ),
    (
        "string",
        "ISSTR",
        "isstring",
        "Use ISSTRUCT instead of comparing the class to 'struct'.",
    ),
    (
        "logical",
        "ISLOG",
        "islogical",
        "Use ISLOGICAL instead of comparing the class to 'logical'.",
    ),
    (
        "cell",
        "ISCEL",
        "iscell",
        "Use ISCELL instead of comparing the class to 'cell'.",
    ),
    (
        "double",
        "ISMAT",
        "isnumeric",
        "When checking if a variable is a matrix consider using ISMATRIX.",
    ),
];

// ---------------------------------------------------------------------------
// Target node types
// ---------------------------------------------------------------------------

/// Node types this engine subscribes to for dispatch.
pub(crate) const TARGET_NODES: &[&str] = &[
    "function_call",
    "comparison_operator",
    "boolean_operator",
    "identifier",
    "binary_operator",
    "assignment",
    "matrix",
    "arguments_statement",
];

/// Built-in function names that are always treated as functions, never as
/// ambiguous variable-or-function callees.
pub(crate) const BUILTIN_DENYLIST: &[&str] = &[
    "length", "size", "numel", "abs", "sum", "max", "min", "mean",
];

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// The readability engine — a single rule instance that checks 36 readability
/// improvement patterns via dispatched per-check logic.
///
/// Registered once with inventory. Each emitted diagnostic carries the specific
/// check ID (e.g., `"ISCHR"`), not the meta-ID `"READABILITY_ENGINE"`.
pub struct ReadabilityEngine {
    config: ReadabilityConfig,
}

impl ReadabilityEngine {
    /// Factory constructor called by the rule registry.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: ReadabilityConfig = config.rule_params("READABILITY_ENGINE");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Check whether a specific sub-check is enabled.
    fn is_check_enabled(&self, check_id: &str) -> bool {
        !self.config.disabled_checks.iter().any(|d| d == check_id)
    }

    /// Run the file-level MFAMB check: flag bare-identifier callees that are
    /// also defined as variables in scope.
    fn check_mfamb(&self, tree: &tree_sitter::Tree, source: &str, results: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MFAMB") {
            return;
        }
        let table = SymbolTable::build(tree, source);
        walk_for_mfamb(tree.root_node(), source, &table, results);
    }

    /// Emit a diagnostic for a given node with the specified check ID.
    fn diag(
        &self,
        check_id: &'static str,
        message: &str,
        node: tree_sitter::Node,
        fix: Option<Fix>,
    ) -> Diagnostic {
        let start = node.start_position();
        Diagnostic {
            rule_id: check_id,
            message: message.to_string(),
            severity: self.severity(),
            byte_range: node.start_byte()..node.end_byte(),
            line: start.row + 1,
            column: start.column + 1,
            fix,
        }
    }

    // -----------------------------------------------------------------------
    // Per-node-type dispatch
    // -----------------------------------------------------------------------

    /// Checks dispatched for `function_call` nodes.
    fn check_function_call(&self, ctx: &NodeContext, results: &mut Vec<Diagnostic>) {
        let node = ctx.node;
        let source = ctx.source;

        let func_name = match node.child_by_field_name("name") {
            Some(n) => &source[n.start_byte()..n.end_byte()],
            None => return,
        };

        match func_name {
            "isa" => self.check_isa(node, source, results),
            "strcmp" => self.check_strcmp(node, source, ctx, results),
            "strncmp" | "strncmpi" => self.check_strncmp(node, source, func_name, results),
            "strfind" => self.check_strfind(node, source, results),
            "char" => self.check_char_literal(node, source, results),
            "sprintf" => self.check_sprintf(node, source, results),
            "error" => self.check_error_warning(node, source, "SPERR", results),
            "warning" => self.check_error_warning(node, source, "SPWRN", results),
            "disp" => self.check_disp(node, source, results),
            "display" => self.check_display(node, results),
            "nargchk" => self.check_narginchk(node, source, results),
            "prod" => self.check_prod_size(node, source, results),
            "sum" => self.check_sum_logical(node, source, results),
            "find" => self.check_find_logical(node, source, results),
            "flipud" | "fliplr" => self.check_fludlr(node, source, func_name, results),
            _ => {}
        }
    }

    /// Checks dispatched for `comparison_operator` nodes.
    fn check_comparison(&self, ctx: &NodeContext, results: &mut Vec<Diagnostic>) {
        let node = ctx.node;
        let source = ctx.source;

        // Look for size(x,dim)==1 patterns (ISROW, ISCOL)
        self.check_size_comparison(node, source, results);
    }

    /// Checks dispatched for `boolean_operator` nodes.
    fn check_boolean_operator(&self, ctx: &NodeContext, results: &mut Vec<Diagnostic>) {
        let node = ctx.node;
        let source = ctx.source;

        self.check_boolean_tautology(node, source, results);
    }

    /// Checks dispatched for `assignment` nodes.
    fn check_assignment(&self, ctx: &NodeContext, results: &mut Vec<Diagnostic>) {
        let node = ctx.node;
        let source = ctx.source;

        // IJCL: assignment to `i` or `j` shadows complex unit
        self.check_ij_shadow(node, source, results);

        // ASGSL: assignment on same line as control statement
        self.check_inline_assignment(node, results);
    }

    /// Checks dispatched for `binary_operator` nodes.
    fn check_binary_operator(&self, ctx: &NodeContext, results: &mut Vec<Diagnostic>) {
        let node = ctx.node;
        let source = ctx.source;

        self.check_redundant_arithmetic(node, source, results);
    }

    /// Checks dispatched for `matrix` nodes.
    fn check_matrix(&self, ctx: &NodeContext, results: &mut Vec<Diagnostic>) {
        let node = ctx.node;
        let source = ctx.source;

        // NBRAK2: unnecessary brackets [x] for scalar
        self.check_unnecessary_brackets(node, source, results);

        // COMNL: newline after comma acts as a row separator
        self.check_comnl(node, source, results);
    }

    // -----------------------------------------------------------------------
    // Individual check implementations
    // -----------------------------------------------------------------------

    /// STREMP: `strcmp(s, '')` → `strlength(s)==0`
    /// STRIFCND: `strcmp` inside `if` condition → `matches`
    fn check_strcmp(
        &self,
        node: tree_sitter::Node,
        source: &str,
        ctx: &NodeContext,
        results: &mut Vec<Diagnostic>,
    ) {
        let args = match find_arguments(node) {
            Some(a) => a,
            None => return,
        };

        let named_children: Vec<_> = (0..args.named_child_count())
            .filter_map(|i| args.named_child(i))
            .collect();

        if named_children.len() != 2 {
            return;
        }

        let arg1_text = &source[named_children[0].start_byte()..named_children[0].end_byte()];
        let arg2_text = &source[named_children[1].start_byte()..named_children[1].end_byte()];

        // STREMP: strcmp(s, '') or strcmp('', s)
        if self.is_check_enabled("STREMP")
            && (is_empty_string(arg2_text) || is_empty_string(arg1_text))
        {
            let var = if is_empty_string(arg1_text) {
                arg2_text
            } else {
                arg1_text
            };
            results.push(self.diag(
                "STREMP",
                "For readability, use '~contains(str1, str2)' instead of 'isempty(strfind(str1, str2))'.",
                node,
                Some(Fix::new(
                    node.start_byte()..node.end_byte(),
                    format!("strlength({var})==0"),
                )),
            ));
            return;
        }

        // STRIFCND: strcmp inside if condition → suggest matches
        if self.is_check_enabled("STRIFCND") && is_inside_if_condition(ctx.node) {
            results.push(self.diag(
                "STRIFCND",
                "For readability, use 'contains(str1, str2)' instead of 'strfind(str1, str2)'.",
                node,
                None,
            ));
        }

        // STLOW: unnecessary UPPER/LOWER call in a comparison
        self.check_stlow(&named_children, source, results);
    }
}

// ---------------------------------------------------------------------------
// Rule trait implementation
// ---------------------------------------------------------------------------

impl Rule for ReadabilityEngine {
    fn id(&self) -> &'static str {
        "READABILITY_ENGINE"
    }

    fn description(&self) -> &'static str {
        "Readability improvement checks for MATLAB code"
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn category(&self) -> Category {
        Category::Readability
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        self.check_mfamb(ctx.tree, ctx.source, &mut results);
        results
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();

        match ctx.node.kind() {
            "function_call" => self.check_function_call(ctx, &mut results),
            "comparison_operator" => self.check_comparison(ctx, &mut results),
            "boolean_operator" => self.check_boolean_operator(ctx, &mut results),
            "assignment" => self.check_assignment(ctx, &mut results),
            "binary_operator" => self.check_binary_operator(ctx, &mut results),
            "matrix" => self.check_matrix(ctx, &mut results),
            "arguments_statement" => self.check_fvinr(ctx, &mut results),
            // `identifier` nodes are handled via `assignment` LHS check (IJCL)
            // rather than standalone identifier dispatch, to avoid false
            // positives on identifier reads.
            _ => {}
        }

        results
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find the `arguments` child of a `function_call` node.
///
/// In tree-sitter-matlab the call arguments are a child node of kind
/// `arguments` rather than a named field, so `child_by_field_name("arguments")`
/// never matches. This helper locates them by child kind instead.
pub(crate) fn find_arguments(node: tree_sitter::Node) -> Option<tree_sitter::Node> {
    node.named_children(&mut node.walk())
        .find(|c| c.kind() == "arguments")
}

/// Check if a string literal represents an empty string (`''` or `""`).
pub(crate) fn is_empty_string(s: &str) -> bool {
    s == "''" || s == "\"\""
}

/// Return true when `s` contains at least one alphabetic character and every
/// alphabetic character is in the case requested by `uppercase`.
pub(crate) fn is_all_case(s: &str, uppercase: bool) -> bool {
    let mut has_alpha = false;
    for c in s.chars() {
        if c.is_alphabetic() {
            has_alpha = true;
            if c.is_lowercase() == uppercase {
                return false;
            }
        }
    }
    has_alpha
}

/// Check if a node is inside an `if_statement` condition.
pub(crate) fn is_inside_if_condition(node: tree_sitter::Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "if_statement" => return true,
            "block" | "source_file" | "function_definition" => return false,
            _ => current = parent.parent(),
        }
    }
    false
}

/// Extract the operator text from a binary/boolean operator node.
///
/// Iterates over unnamed children to find the operator token between operands.
pub(crate) fn extract_operator_text<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> &'a str {
    // Walk all children (including unnamed) looking for the operator
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if !child.is_named() {
            let text = &source[child.start_byte()..child.end_byte()];
            // Operator tokens are things like +, -, *, .*, |, ||, &, &&, ==
            if !text.trim().is_empty() && text != "(" && text != ")" && text != "[" && text != "]" {
                return text;
            }
        }
    }
    ""
}

/// Find the trailing comma of a `row` node, skipping trailing `comment` and
/// `line_continuation` children.
///
/// The comma token is unnamed, so all children (not just named children) must
/// be inspected. Returns `None` when the row does not end in a comma.
pub(crate) fn find_trailing_comma(row: tree_sitter::Node) -> Option<tree_sitter::Node> {
    for j in (0..row.child_count()).rev() {
        let child = row.child(j)?;
        match child.kind() {
            "comment" | "line_continuation" => continue,
            "," => return Some(child),
            _ => return None,
        }
    }
    None
}

// ---------------------------------------------------------------------------
// MFAMB helpers
// ---------------------------------------------------------------------------

/// Recursively walk the tree looking for `function_call` nodes.
pub(crate) fn walk_for_mfamb(
    node: tree_sitter::Node,
    source: &str,
    table: &SymbolTable,
    results: &mut Vec<Diagnostic>,
) {
    if node.kind() == "function_call" {
        check_one_mfamb(node, source, table, results);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_for_mfamb(child, source, table, results);
    }
}

/// MFAMB: flag a `function_call` whose bare-identifier callee is also a
/// defined variable in scope.
pub(crate) fn check_one_mfamb(
    node: tree_sitter::Node,
    source: &str,
    table: &SymbolTable,
    results: &mut Vec<Diagnostic>,
) {
    let name_node = match node.child_by_field_name("name") {
        Some(n) => n,
        None => return,
    };
    // Only bare-identifier callees are ambiguous.
    if name_node.kind() != "identifier" {
        return;
    }
    // Skip indexed assignments: `x(i) = ...` — function_call is the LHS.
    if let Some(parent) = node.parent() {
        if parent.kind() == "assignment" {
            if let Some(left) = parent.child_by_field_name("left") {
                if left.id() == node.id() {
                    return;
                }
            }
        }
    }
    let name = &source[name_node.start_byte()..name_node.end_byte()];
    if BUILTIN_DENYLIST.contains(&name) {
        return;
    }
    // Primary gate: name is a variable (non-function) in this scope or an
    // enclosing scope.
    if !is_variable_in_scope(name, name_node.start_byte(), table) {
        return;
    }
    let pos = name_node.start_position();
    results.push(Diagnostic {
        rule_id: "MFAMB",
        message: format!(
            "Code Analyzer cannot determine whether {name} is a variable or a function, and assumes it is a function."
        ),
        severity: Severity::Info,
        byte_range: name_node.start_byte()..name_node.end_byte(),
        line: pos.row + 1,
        column: pos.column + 1,
        fix: None,
    });
}

/// Check whether `name` is defined as a variable (not a nested function) in
/// the scope containing `byte_offset` or any enclosing scope.
pub(crate) fn is_variable_in_scope(name: &str, byte_offset: usize, table: &SymbolTable) -> bool {
    let mut current = table.scope_at(byte_offset);
    while let Some(scope) = current {
        if scope
            .defs
            .iter()
            .any(|d| d.name == name && d.kind != DefKind::NestedFunction)
        {
            return true;
        }
        current = scope.parent.and_then(|idx| table.scopes.get(idx));
    }
    false
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "READABILITY_ENGINE",
    ReadabilityEngine::from_config
));

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    // -- MFAMB ---------------------------------------------------------------

    #[test]
    fn mfamb_fires_when_name_assigned_and_called() {
        let diags = lint_file(&*engine(), "somevar = 5;\ny = somevar(1);\n");
        assert!(has_id(&diags, "MFAMB"), "got: {diags:?}");
    }

    #[test]
    fn mfamb_does_not_fire_for_builtin_call() {
        let diags = lint_file(&*engine(), "y = length(x);\n");
        assert!(!has_id(&diags, "MFAMB"), "got: {diags:?}");
    }

    #[test]
    fn mfamb_does_not_fire_for_field_expression_call() {
        let diags = lint_file(&*engine(), "somevar = 5;\ny = somevar.method();\n");
        assert!(!has_id(&diags, "MFAMB"), "got: {diags:?}");
    }

    #[test]
    fn mfamb_does_not_fire_when_name_not_a_variable() {
        let diags = lint_file(&*engine(), "y = mylocal(x);\n");
        assert!(!has_id(&diags, "MFAMB"), "got: {diags:?}");
    }

    #[test]
    fn mfamb_does_not_fire_for_indexed_assignment_lhs() {
        let diags = lint_file(&*engine(), "x = 1:10;\nx(1) = 5;\n");
        assert!(!has_id(&diags, "MFAMB"), "got: {diags:?}");
    }

    #[test]
    fn mfamb_disabled_in_config_does_not_fire() {
        let config =
            Config::from_toml("[lint.rules.READABILITY_ENGINE]\ndisabled_checks = [\"MFAMB\"]\n")
                .expect("valid config");
        let rule = ReadabilityEngine::from_config(&config);
        let diags = lint_file(&*rule, "somevar = 5;\ny = somevar(1);\n");
        assert!(!has_id(&diags, "MFAMB"), "got: {diags:?}");
    }

    // -- STREMP --------------------------------------------------------------

    #[test]
    fn stremp_strcmp_empty_fires() {
        let diags = lint_nodes(&*engine(), "x = strcmp(s, '');\n");
        assert!(has_id(&diags, "STREMP"), "got: {diags:?}");
    }

    #[test]
    fn stremp_strcmp_nonempty_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = strcmp(s, 'a');\n");
        assert!(!has_id(&diags, "STREMP"), "got: {diags:?}");
    }

    // -- STRIFCND ------------------------------------------------------------

    #[test]
    fn strifcnd_strcmp_in_if_condition_fires() {
        let source = "if strcmp(a, b)\n    x = 1;\nend\n";
        let diags = lint_nodes(&*engine(), source);
        assert!(has_id(&diags, "STRIFCND"), "got: {diags:?}");
    }

    #[test]
    fn strifcnd_strcmp_at_statement_level_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = strcmp(a, b);\n");
        assert!(!has_id(&diags, "STRIFCND"), "got: {diags:?}");
    }
}
