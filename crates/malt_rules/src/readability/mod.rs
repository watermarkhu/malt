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
//! The 30 checks cover type-checking simplification (`strcmp(class(x), 'char')`
//! → dedicated predicates), dimension checks (`size(x,dim)==1` →
//! `isrow`/`iscolumn`, `ndims(x)==2` → `ismatrix`), the imaginary unit,
//! unnecessary brackets, `strfind`-based containment tests, `char(10)` and
//! `sprintf('\n')` → `newline`, `repmat` of constants, `error`/`warning`
//! message construction, `display`/`disp` of `sprintf`, flip/rotate, logical
//! helpers, `arguments` attributes, and ambiguous identifiers.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
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
//! | LOGSUM | info | no | Consider using 'nnz' instead of 'sum' for logical vectors to improve readability. |
//! | LOGL | info | yes | Use 'true' or 'false' instead of 'logical(1)' or 'logical(0)'. |
//! | ISCHR | info | yes | Use ISCHAR instead of comparing the class to 'char'. |
//! | ISSTR | info | yes | Use ISSTRUCT instead of comparing the class to 'struct'. |
//! | ISLOG | info | yes | Use ISLOGICAL instead of comparing the class to 'logical'. |
//! | ISCEL | info | yes | Use ISCELL instead of comparing the class to 'cell'. |
//! | IJCL | info | yes | For improved robustness, consider replacing i and j by 1i. |
//! | ISMAT | info | yes | When checking if a variable is a matrix consider using ISMATRIX. |
//! | ISROW | info | yes | When checking if a variable is a row vector consider using ISROW. |
//! | ISCOL | info | yes | When checking if a variable is a column vector consider using ISCOLUMN. |
//! | NBRAK2 | info | yes | Use of brackets [] is unnecessary. |
//! | MFAMB | info | no | Code Analyzer cannot determine whether VAR_NAME is a variable or a function, and assumes it is a function. |
//! | FVINR | info | yes | For readability, add Input attribute to the input arguments block. |
//! | STREMP | info | yes | For readability, use '~contains(str1, str2)' instead of 'isempty(strfind(str1, str2))'. |
//! | STRCL1 | info | yes | For readability, use '~contains(str1, str2)' instead of 'cellfun('isempty', strfind(str1, str2))'. |
//! | STRCLFH | info | yes | For readability, use '~contains(str1, str2)' instead of 'cellfun(@isempty, strfind(str1, str2))'. |
//! | STRIFCND | info | yes | For readability, use 'contains(str1, str2)' instead of 'strfind(str1, str2)'. |
//! | CHARTEN | info | yes | For readability, consider using 'newline' instead of 'char(10)'. |
//! | PSIZE | info | yes | NUMEL(x) is usually faster than PROD(SIZE(x)). |
//! | SPRINTFN | info | yes | For readability, consider using the 'newline' function instead of 'sprintf('\n')'. |
//!
//! ## Fix
//!
//! Rewrites the flagged construct into the clearer equivalent:
//!
//! - `strcmp(class(x), 'char')` → `ischar(x)` (ISCHR, ISSTR, ISLOG, ISCEL).
//! - `size(x, dim) == 1` → `isrow(x)`/`iscolumn(x)` (ISROW, ISCOL);
//!   `ndims(x) == 2` → `ismatrix(x)` (ISMAT).
//! - `repmat(c, x, y)` → `ones`/`zeros`/`true`/`false`/`Inf`/`NaN(x, y)`
//!   (RPMT1, RPMT0, RPMTT, RPMTF, RPMTI, RPMTN).
//! - `char(10)` and `sprintf('\n')` → `newline`; `logical(1)` → `true`.
//! - `flipud(fliplr(x))` → `rot90(x, 2)`; `[scalar]` → `scalar`;
//!   `disp(sprintf(...))` → `fprintf(...)`.
//! - `isempty(strfind(s, p))`, `cellfun(@isempty, strfind(s, p))` →
//!   `~contains(s, p)`; `if strfind(s, p)` → `if contains(s, p)`.
//! - `strcmpi(lower(x), y)` → `strcmpi(x, y)` (STLOW).
//! - `i`/`j` as the imaginary unit → `1i`/`1j` (IJCL).
//! - A trailing comma before a newline in a matrix becomes a semicolon (COMNL).
//! - An `(Input)` attribute is inserted into attribute-less `arguments`
//!   blocks (FVINR).
//!
//! Fixes that can change behaviour (STLOW, STREMP, STRCL1, STRCLFH, STRIFCND,
//! ISROW, ISCOL, IJCL) are unsafe and need `--unsafe-fixes`.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! if strcmp(class(x), 'char')
//! y = flipud(fliplr(m));
//! z = repmat(0, 3, 4);
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! if ischar(x)
//! y = rot90(m, 2);
//! z = zeros(3, 4);
//! ```
//!
//! ### Fixed
//!
//! ```diff
//! - z = repmat(0, 3, 4);
//! + z = zeros(3, 4);
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.READABILITY_ENGINE]
//! severity = "info"
//! disabled_checks = ["IJCL", "NBRAK2"]
//! ```

mod check_char_literal;
mod check_class_compare;
mod check_comnl;
mod check_disp;
mod check_display;
mod check_error_warning;
mod check_fludlr;
mod check_fvinr;
mod check_ijcl;
mod check_ismat;
mod check_logical;
mod check_prod_size;
mod check_repmat;
mod check_size_comparison;
mod check_sprintf;
mod check_stlow;
mod check_strfind;
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

/// Checks whose rewrite is not equivalent in every situation, so `--fix` leaves
/// it alone (`--unsafe-fixes` applies it):
///
/// - STLOW: `lower`/`upper` reject non-text input that `strcmpi` accepts;
/// - STREMP / STRCL1 / STRCLFH / STRIFCND: `strfind` and `contains` differ for
///   cell and non-text input, and for empty patterns;
/// - ISROW / ISCOL: `size(x, 1) == 1` is true for an N-d array, `isrow(x)` is not;
/// - IJCL: `i` may come from `load`/`eval` rather than the built-in unit.
const UNSAFE_FIXES: &[&str] = &[
    "STLOW", "STREMP", "STRCL1", "STRCLFH", "STRIFCND", "ISROW", "ISCOL", "IJCL",
];

// ---------------------------------------------------------------------------
// Target node types
// ---------------------------------------------------------------------------

/// Node types this engine subscribes to for dispatch.
pub(crate) const TARGET_NODES: &[&str] = &[
    "function_call",
    "comparison_operator",
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

/// The readability engine — a single rule instance that checks 30 readability
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
    ///
    /// Fixes of the checks in [`UNSAFE_FIXES`] are marked unsafe.
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
            fix: fix.map(|f| {
                if UNSAFE_FIXES.contains(&check_id) {
                    f.unsafe_fix()
                } else {
                    f
                }
            }),
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
            "strcmp" | "isequal" => self.check_class_compare(node, source, results),
            "strcmpi" | "strncmpi" => self.check_stlow(node, source, results),
            "isempty" => self.check_isempty_strfind(node, source, results),
            "cellfun" => self.check_cellfun_strfind(node, source, results),
            "strfind" => self.check_strfind_condition(node, source, results),
            "char" => self.check_char_literal(node, source, results),
            "sprintf" => self.check_sprintf(node, source, results),
            "prod" => self.check_prod_size(node, source, results),
            "error" => self.check_error_warning(node, source, "SPERR", results),
            "warning" => self.check_error_warning(node, source, "SPWRN", results),
            "disp" => self.check_disp(node, source, results),
            "display" => self.check_display(node, source, results),
            "sum" => self.check_sum_logical(node, source, results),
            "logical" => self.check_logical_literal(node, source, results),
            "repmat" => self.check_repmat(node, source, results),
            "flipud" | "fliplr" => self.check_fludlr(node, source, func_name, results),
            _ => {}
        }
    }

    /// Checks dispatched for `comparison_operator` nodes.
    fn check_comparison(&self, ctx: &NodeContext, results: &mut Vec<Diagnostic>) {
        let node = ctx.node;
        let source = ctx.source;

        // size(x,dim)==1 patterns (ISROW, ISCOL)
        self.check_size_comparison(node, source, results);
        // ndims(x)==2 (ISMAT)
        self.check_ismat(node, source, results);
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
        self.check_ijcl(ctx.tree, ctx.source, &mut results);
        results
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();

        match ctx.node.kind() {
            "function_call" => self.check_function_call(ctx, &mut results),
            "comparison_operator" => self.check_comparison(ctx, &mut results),
            "matrix" => self.check_matrix(ctx, &mut results),
            "arguments_statement" => self.check_fvinr(ctx, &mut results),
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

/// The callee name of a `function_call` node, when it is a bare identifier.
pub(crate) fn call_name<'a>(node: tree_sitter::Node, source: &'a str) -> Option<&'a str> {
    if crate::analysis::symbols::is_qualified_call(node) {
        return None;
    }
    let name = node.child_by_field_name("name")?;
    (name.kind() == "identifier").then(|| &source[name.start_byte()..name.end_byte()])
}

/// The argument nodes of a `function_call` node.
pub(crate) fn call_args<'t>(node: tree_sitter::Node<'t>) -> Vec<tree_sitter::Node<'t>> {
    find_arguments(node)
        .map(|args| {
            (0..args.named_child_count())
                .filter_map(|i| args.named_child(i))
                .collect()
        })
        .unwrap_or_default()
}

/// Whether a `comparison_operator` node uses `==`. The operator token is
/// inspected, not the node text: that would also find the `==` inside
/// `size(x == 1, 1) ~= 1`.
pub(crate) fn is_equality_operator(node: tree_sitter::Node, source: &str) -> bool {
    let mut cursor = node.walk();
    let found = node
        .children(&mut cursor)
        .any(|c| !c.is_named() && &source[c.start_byte()..c.end_byte()] == "==");
    found
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
    // Ambiguous only when the name becomes a variable later in the scope but
    // is not one yet at this call: `h(1); ... h = 3;`. A name already defined
    // is plain indexing; a name never defined is a function.
    let at = name_node.start_byte();
    if is_variable_defined_before(name, at, table) || !is_variable_in_scope(name, at, table) {
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

/// Whether a variable `name` is already in effect at `byte_offset`, in its scope
/// or an enclosing one.
fn is_variable_defined_before(name: &str, byte_offset: usize, table: &SymbolTable) -> bool {
    let mut current = table.scope_at(byte_offset);
    while let Some(scope) = current {
        if scope.defs.iter().any(|d| {
            d.name == name && d.kind != DefKind::NestedFunction && d.effect_start <= byte_offset
        }) {
            return true;
        }
        current = scope.parent.and_then(|idx| table.scopes.get(idx));
    }
    false
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
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    // -- MFAMB ---------------------------------------------------------------

    #[test]
    fn mfamb_fires_when_name_is_called_before_it_becomes_a_variable() {
        let diags = lint_file(&*engine(), "y = somevar(1);\nsomevar = 5;\n");
        assert!(has_id(&diags, "MFAMB"), "got: {diags:?}");
    }

    #[test]
    fn mfamb_does_not_fire_for_plain_indexing() {
        let diags = lint_file(&*engine(), "somevar = 5;\ny = somevar(1);\n");
        assert!(!has_id(&diags, "MFAMB"), "got: {diags:?}");
        let diags = lint_file(&*engine(), "h = @(x) x;\nz = h(1);\n");
        assert!(!has_id(&diags, "MFAMB"), "got: {diags:?}");
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
        let diags = lint_file(&*rule, "y = somevar(1);\nsomevar = 5;\n");
        assert!(!has_id(&diags, "MFAMB"), "got: {diags:?}");
    }
}
