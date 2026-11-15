//! # PERFORMANCE_ENGINE: Performance Improvement Checks
//!
//! ```malt
//! id = "PERFORMANCE_ENGINE"
//! title = "Performance Improvement Checks"
//! category = "performance"
//! severity = "info"
//! fix = true
//! icon = "lucide/zap"
//! slug = "performance"
//! ```
//!
//! ## Rule
//!
//! A hybrid rule module implementing 40 performance-related MATLAB lint
//! checks from MATLAB's Code Analyzer. Most checks operate at the node level
//! (function calls, operators, commands), while AGROW, SAGROW, SPRIX, and PFBNS
//! require file-level traversal to detect array-growth patterns inside loops.
//! Each diagnostic carries the specific check ID (e.g. `AGROW`, `MINV`).
//!
//! A single `PerformanceEngine` struct handles all 40 checks. Node-level
//! checks are dispatched by node kind (`function_call`, `command`,
//! `binary_operator`, `boolean_operator`, `comparison_operator`, `cell`); file-level checks traverse the full tree looking for
//! growth patterns inside `for_statement`/`while_statement`. The node-level
//! `check_*` methods and their tests live in sibling `check_*.rs` modules;
//! this module keeps the engine, the dispatch, the file-level growth walk
//! (AGROW/SAGROW/SPRIX/PFBNS), and the shared free helpers.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | AGROW | info | no | Variable appears to change size on every loop iteration. Consider preallocating for speed. |
//! | SAGROW | info | no | Variable appears to change size on every loop iteration (within a script). Consider preallocating for speed. |
//! | AND2 | info | yes (unsafe) | When both arguments are numeric scalars, consider replacing & with && for performance. |
//! | OR2 | info | yes (unsafe) | When both arguments are numeric scalars, consider replacing \| with \|\| for performance. |
//! | MINV | info | no | INV(A)*b can be slower and less accurate than A\b. Consider using A\b for INV(A)*b or b/A for b*INV(A). |
//! | GFLD | info | no | Use dynamic fieldnames with structures instead of GETFIELD. |
//! | SFLD | info | no | Use dynamic fieldnames with structures instead of SETFIELD. |
//! | EXIST | info | no | EXIST with two input arguments is generally faster and clearer than with one input argument. |
//! | PFBNS | info | no | The entire array or structure VAR_NAME is a broadcast variable. This might result in unnecessary communication overhead. |
//! | CCAT | info | no | For improved performance, concatenate cell arrays using [] instead of extracting cell arrays and reconstructing them. |
//! | CCAT1 | info | no | { A{I} } can usually be replaced by A(I) or A(I)', which can be much faster. |
//! | ISMT | info | no | Using ISEMPTY is usually faster than comparing LENGTH to 0. |
//! | ISCL | info | no | To improve performance, use 'isscalar' instead of length comparison. |
//! | ST2NM | info | yes (unsafe) | If you are operating on scalar values, consider using 'str2double' for faster performance. |
//! | FLPST | info | yes (unsafe) | For better performance in some cases, use SORT with the 'descend' option. |
//! | MXFND | info | no | Use FIND with the 'first' or 'last' option. |
//! | EFIND | info | no | To improve performance, replace ISEMPTY(FIND(X)) with ISEMPTY(FIND( X, 1 )). |
//! | UDIM | info | no | Instead of using transpose (' or .'), consider using a different DIMENSION input argument to VAR_NAME. |
//! | FREAD | info | no | FREAD(FID,...,'*char') is more efficient than CHAR(FREAD(...)). |
//! | N2UNI | info | no | Instead of using 'native2unicode' with 'fread', specify the character encoding scheme in the call to 'fopen'. |
//! | LAXES | info | no | Calling AXES(h) in a loop can be slow. Consider moving the call to AXES outside the loop. |
//! | MMTC | info | no | This use of MAT2CELL should probably be replaced by a simpler, faster call to NUM2CELL. |
//! | MRPBW | info | yes (unsafe) | To use less memory, replace BWLABEL(bw) by LOGICAL(bw) in a call of REGIONPROPS. |
//! | SPRIX | info | no | This sparse indexing expression is likely to be slow. |
//! | TRSRT | info | no | Transposing the input to 'sort' is often unnecessary. |
//! | GRIDD | info | no | Consider replacing GRIDDATA with SCATTEREDINTERPOLANT for better performance. |
//! | CLALL | info | no | Using 'clear' with the 'all' option usually decreases code performance and is often unnecessary. |
//! | CLCLS | info | no | Using 'clear' with the 'classes' option will decrease code performance and is often unnecessary. |
//! | CLFUNC | info | no | Using 'clear' with the 'functions' option usually decreases code performance and is often unnecessary. |
//! | CLJAVA | info | no | Using 'clear' with the 'java' option usually decreases code performance and is often unnecessary. |
//! | CLMEX | info | no | Using 'clear' with the 'mex' option usually decreases code performance and is often unnecessary. |
//! | CLEAR0ARGS | info | no | Avoid using 'clear' to clear more than necessary, this decreases code performance and is usually unnecessary. |
//! | RGXP1 | info | no | Using REGEXP(str, pattern, 'ONCE') is faster in this case. |
//! | RGXPI | info | no | Using REGEXPI(str, pattern, 'ONCE') is faster in this case. |
//! | TRIM1 | info | yes (unsafe) | Use STRTRIM(str) instead of nesting FLIPLR and DEBLANK calls. |
//! | TRIM2 | info | yes (unsafe) | Use STRTRIM(str) instead of DEBLANK(STRJUST(str,'left')). |
//! | STTOK | info | no | Use one call to 'split' instead of calling 'strtok' in a loop. |
//! | STNCI | info | no | Use STRNCMPI(str1,str2) instead of using UPPER/LOWER in a call to STRNCMP. |
//! | STCCS | info | no | It appears that STRCMPI/STRNCMPI can be replaced by a faster, case sensitive compare. |
//! | FNDSB | info | no | For array or cell array, performance can be improved using logical indexing instead of 'find'. |
//!
//! ## Fix
//!
//! Rewrites the flagged call or operator into its faster equivalent:
//!
//! All rewrites are unsafe fixes (they can change behaviour):
//!
//! - `&` → `&&` and `|` → `||` in conditions (AND2, OR2).
//! - `str2num` → `str2double` (ST2NM).
//! - `fliplr(sort(x))` → `sort(x, 'descend')` (FLPST).
//! - `regionprops(bwlabel(bw), …)` → `regionprops(logical(bw), …)` (MRPBW).
//! - nested `fliplr`/`deblank` and `deblank(strjust(s, 'left'))` → `strtrim(s)` (TRIM1, TRIM2).
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! for i = 1:n
//!     x = [x, val];          % AGROW
//! end
//! if a & b                   % AND2
//! inv(A) * b                 % MINV
//! exist('foo')             % EXIST
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! x = zeros(1, n);
//! for i = 1:n
//!     x(i) = val;
//! end
//! if a && b
//! A \ b
//! exist('foo', 'file')
//! ```
//!
//! ### Fixed
//!
//! ```diff
//! - if a & b
//! + if a && b
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.PERFORMANCE_ENGINE]
//! severity = "info"
//! skip_checks = ["AGROW", "ST2NM"]
//! ```

use malt_core::{Category, Config, Diagnostic, FileContext, Fix, NodeContext, Rule, Severity};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Submodules: one file per independent node-level check
// ---------------------------------------------------------------------------

mod check_binary_operator;
mod check_boolean_context_binary;
mod check_command;
mod check_comparison;
mod check_function_call;
mod check_index_growth;

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for the performance engine.
///
/// Deserialized from the `[lint.rules.PERFORMANCE_ENGINE]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PerformanceEngineConfig {
    /// Check IDs to skip (e.g., `["AGROW", "AND2"]`).
    #[serde(default)]
    pub skip_checks: Vec<String>,
}

// ---------------------------------------------------------------------------
// Check metadata
// ---------------------------------------------------------------------------

/// Static metadata for a single performance check.
struct CheckMeta {
    id: &'static str,
    description: &'static str,
}

/// All 40 performance check definitions.
const CHECKS: &[CheckMeta] = &[
    CheckMeta {
        id: "AGROW",
        description: "Variable appears to change size on every loop iteration. Consider preallocating for speed.",
    },
    CheckMeta {
        id: "SAGROW",
        description: "Variable appears to change size on every loop iteration (within a script). Consider preallocating for speed.",
    },
    CheckMeta {
        id: "AND2",
        description: "When both arguments are numeric scalars, consider replacing & with && for performance.",
    },
    CheckMeta {
        id: "OR2",
        description: "When both arguments are numeric scalars, consider replacing | with || for performance.",
    },
    CheckMeta {
        id: "MINV",
        description: "INV(A)*b can be slower and less accurate than A\\b. Consider using A\\b for INV(A)*b or b/A for b*INV(A).",
    },
    CheckMeta {
        id: "GFLD",
        description: "Use dynamic fieldnames with structures instead of GETFIELD.",
    },
    CheckMeta {
        id: "SFLD",
        description: "Use dynamic fieldnames with structures instead of SETFIELD.",
    },
    CheckMeta {
        id: "EXIST",
        description: "EXIST with two input arguments is generally faster and clearer than with one input argument.",
    },
    CheckMeta {
        id: "PFBNS",
        description: "The entire array or structure VAR_NAME is a broadcast variable. This might result in unnecessary communication overhead.",
    },
    CheckMeta {
        id: "CCAT",
        description: "For improved performance, concatenate cell arrays using [] instead of extracting cell arrays and reconstructing them.",
    },
    CheckMeta {
        id: "CCAT1",
        description: "{ A{I} } can usually be replaced by A(I) or A(I)', which can be much faster.",
    },
    CheckMeta {
        id: "ISMT",
        description: "Using ISEMPTY is usually faster than comparing LENGTH to 0.",
    },
    CheckMeta {
        id: "ISCL",
        description: "To improve performance, use 'isscalar' instead of length comparison.",
    },
    CheckMeta {
        id: "ST2NM",
        description: "If you are operating on scalar values, consider using 'str2double' for faster performance.",
    },
    CheckMeta {
        id: "FLPST",
        description: "For better performance in some cases, use SORT with the 'descend' option.",
    },
    CheckMeta {
        id: "MXFND",
        description: "Use FIND with the 'first' or 'last' option.",
    },
    CheckMeta {
        id: "EFIND",
        description: "To improve performance, replace ISEMPTY(FIND(X)) with ISEMPTY(FIND( X, 1 )).",
    },
    CheckMeta {
        id: "UDIM",
        description: "Instead of using transpose (' or .'), consider using a different DIMENSION input argument to VAR_NAME.",
    },
    CheckMeta {
        id: "FREAD",
        description: "FREAD(FID,...,'*char') is more efficient than CHAR(FREAD(...)).",
    },
    CheckMeta {
        id: "N2UNI",
        description: "Instead of using 'native2unicode' with 'fread', specify the character encoding scheme in the call to 'fopen'.",
    },
    CheckMeta {
        id: "LAXES",
        description: "Calling AXES(h) in a loop can be slow. Consider moving the call to AXES outside the loop.",
    },
    CheckMeta {
        id: "MMTC",
        description: "This use of MAT2CELL should probably be replaced by a simpler, faster call to NUM2CELL.",
    },
    CheckMeta {
        id: "MRPBW",
        description: "To use less memory, replace BWLABEL(bw) by LOGICAL(bw) in a call of REGIONPROPS.",
    },
    CheckMeta {
        id: "SPRIX",
        description: "This sparse indexing expression is likely to be slow.",
    },
    CheckMeta {
        id: "TRSRT",
        description: "Transposing the input to 'sort' is often unnecessary.",
    },
    CheckMeta {
        id: "GRIDD",
        description: "Consider replacing GRIDDATA with SCATTEREDINTERPOLANT for better performance.",
    },
    CheckMeta {
        id: "CLALL",
        description: "Using 'clear' with the 'all' option usually decreases code performance and is often unnecessary.",
    },
    CheckMeta {
        id: "CLCLS",
        description: "Using 'clear' with the 'classes' option will decrease code performance and is often unnecessary.",
    },
    CheckMeta {
        id: "CLFUNC",
        description: "Using 'clear' with the 'functions' option usually decreases code performance and is often unnecessary.",
    },
    CheckMeta {
        id: "CLJAVA",
        description: "Using 'clear' with the 'java' option usually decreases code performance and is often unnecessary.",
    },
    CheckMeta {
        id: "CLMEX",
        description: "Using 'clear' with the 'mex' option usually decreases code performance and is often unnecessary.",
    },
    CheckMeta {
        id: "CLEAR0ARGS",
        description: "Avoid using 'clear' to clear more than necessary, this decreases code performance and is usually unnecessary.",
    },
    CheckMeta {
        id: "RGXP1",
        description: "Using REGEXP(str, pattern, 'ONCE') is faster in this case.",
    },
    CheckMeta {
        id: "RGXPI",
        description: "Using REGEXPI(str, pattern, 'ONCE') is faster in this case.",
    },
    CheckMeta {
        id: "TRIM1",
        description: "Use STRTRIM(str) instead of nesting FLIPLR and DEBLANK calls.",
    },
    CheckMeta {
        id: "TRIM2",
        description: "Use STRTRIM(str) instead of DEBLANK(STRJUST(str,'left')).",
    },
    CheckMeta {
        id: "STTOK",
        description: "Use one call to 'split' instead of calling 'strtok' in a loop.",
    },
    CheckMeta {
        id: "STNCI",
        description: "Use STRNCMPI(str1,str2) instead of using UPPER/LOWER in a call to STRNCMP.",
    },
    CheckMeta {
        id: "STCCS",
        description: "It appears that STRCMPI/STRNCMPI can be replaced by a faster, case sensitive compare.",
    },
    CheckMeta {
        id: "FNDSB",
        description: "For array or cell array, performance can be improved using logical indexing instead of 'find'.",
    },
];

/// Look up check description by ID.
pub(crate) fn check_description(id: &str) -> &'static str {
    CHECKS
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.description)
        .unwrap_or("Performance improvement suggestion")
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Hybrid performance engine covering 40 performance checks.
///
/// Node-level checks fire on `function_call`, `command`, `binary_operator`,
/// `boolean_operator`, and `comparison_operator`. File-level checks (AGROW,
/// SAGROW, PFBNS) traverse the full tree for loop growth patterns.
pub struct PerformanceEngine {
    config: PerformanceEngineConfig,
}

/// Node types for node-level dispatch.
const TARGET_NODES: &[&str] = &[
    "function_call",
    "command",
    "binary_operator",
    "boolean_operator",
    "comparison_operator",
    "cell",
];

impl PerformanceEngine {
    /// Factory constructor called by the rule registry.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let mut rule_config: PerformanceEngineConfig = config.rule_params("PERFORMANCE_ENGINE");
        // GRIDD: MATLAB stayed silent on every `griddata` form probed (3, 5 and 6
        // arguments, in a function) and on the fixture, so it is off by default.
        rule_config
            .skip_checks
            .extend(crate::off_under_mathworks(config, &["GRIDD"]));
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Whether a check ID is enabled (not in `skip_checks`).
    fn is_enabled(&self, check_id: &str) -> bool {
        !self.config.skip_checks.iter().any(|s| s == check_id)
    }

    // -----------------------------------------------------------------------
    // File-level checks: AGROW, SAGROW, SPRIX, PFBNS
    // -----------------------------------------------------------------------

    /// File-level checks: array growth in loops (AGROW/SAGROW), indexed
    /// assignment into sparse arrays (SPRIX) and broadcast variables in
    /// `parfor` loops (PFBNS).
    fn check_growth_in_loops(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let root = tree.root_node();
        self.traverse_for_growth(&root, source, false, false, &mut diags);
        self.check_sparse_assignments(root, source, &mut diags);
        if self.is_enabled("PFBNS") {
            check_broadcast_variables(root, source, &mut diags);
        }
        diags
    }

    /// SPRIX: `S(i) = v` where `S` was created by `sparse(...)`/`spalloc(...)`.
    fn check_sparse_assignments(
        &self,
        root: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_enabled("SPRIX") {
            return;
        }
        let mut sparse_vars: Vec<&str> = Vec::new();
        let mut stack = vec![root];
        let mut assignments = Vec::new();
        while let Some(n) = stack.pop() {
            if n.kind() == "assignment" {
                assignments.push(n);
            }
            let mut c = n.walk();
            for ch in n.children(&mut c) {
                stack.push(ch);
            }
        }
        assignments.sort_by_key(|n| n.start_byte());
        for a in assignments {
            let (Some(lhs), Some(rhs)) = (
                a.child_by_field_name("left"),
                a.child_by_field_name("right"),
            ) else {
                continue;
            };
            if lhs.kind() == "identifier" {
                let name = node_text(lhs, source);
                let is_sparse = rhs.kind() == "function_call"
                    && matches!(extract_func_name(rhs, source), Some("sparse" | "spalloc"));
                if is_sparse {
                    if !sparse_vars.contains(&name) {
                        sparse_vars.push(name);
                    }
                } else {
                    sparse_vars.retain(|v| *v != name);
                }
            } else if lhs.kind() == "function_call" {
                if let Some(name) = extract_func_name(lhs, source) {
                    if sparse_vars.contains(&name) {
                        diags.push(make_diag("SPRIX", a));
                    }
                }
            }
        }
    }

    /// Recursive traversal looking for growth patterns inside loops.
    fn traverse_for_growth(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        in_loop: bool,
        in_function: bool,
        diags: &mut Vec<Diagnostic>,
    ) {
        let kind = node.kind();
        let entering_loop = kind == "for_statement" || kind == "while_statement";
        let now_in_loop = in_loop || entering_loop;
        let now_in_function = in_function || kind == "function_definition";
        // AGROW applies to functions and methods; SAGROW to scripts.
        let check_id = if now_in_function { "AGROW" } else { "SAGROW" };

        // Only check assignment nodes inside loops.
        if now_in_loop && kind == "assignment" {
            self.check_assignment_growth(*node, source, check_id, diags);
        }

        // Also detect growth via full-node text for statements that
        // tree-sitter may parse as a bare call (e.g. `s.field(end+1)`).
        if now_in_loop
            && (kind == "function_call" || kind == "cell_index")
            && is_statement_level(node)
            && self.is_enabled(check_id)
        {
            let text = node_text(*node, source);
            let normalized: String = text.chars().filter(|c| !c.is_whitespace()).collect();
            if normalized.contains("end+1") {
                diags.push(make_diag(check_id, *node));
            }
        }

        for child in crate::analysis::symbols::children_of(node) {
            self.traverse_for_growth(&child, source, now_in_loop, now_in_function, diags);
        }
    }

    /// Check a single assignment node for growth patterns.
    ///
    /// Detects:
    /// 1. `x = [x, val]` or `x = [x; val]` — concatenation growth
    /// 2. `x(end+1) = val` — end+1 indexing
    /// 3. `x{end+1} = val` — cell growth
    /// 4. `s.field(end+1) = val` and `s.field = [s.field, val]` — field growth
    fn check_assignment_growth(
        &self,
        node: tree_sitter::Node,
        source: &str,
        check_id: &'static str,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_enabled(check_id) {
            return;
        }
        let lhs = match node.child_by_field_name("left").or_else(|| node.child(0)) {
            Some(n) => n,
            None => return,
        };
        let rhs = match node.child_by_field_name("right").or_else(|| node.child(2)) {
            Some(n) => n,
            None => return,
        };

        let lhs_text = node_text(lhs, source);

        // Pattern 1: x(end+1) = ..., x{end+1} = ... or x(end+1).f = ... (the growth
        // index is in the first bracket group). `end+1` grows on every initial value
        // (probed, parameters included); the size form `x(numel(x)+1)` only when `x`
        // starts empty (probed: silent after `zeros(1,0)`). `s.f(end+1) = ...` names a
        // field: MATLAB does not report it (probed).
        let field_base = lhs_text
            .split(['(', '{'])
            .next()
            .is_some_and(|base| base.contains('.'));
        let base = lhs_text.split(['(', '{']).next().unwrap_or("").trim();
        let first = check_index_growth::first_index_group(lhs_text);
        if matches!(
            lhs.kind(),
            "function_call" | "cell_index" | "field_expression"
        ) && !field_base
            && (check_index_growth::first_group_has_end_plus_one(first)
                || (check_index_growth::first_group_has_size_plus_one(first, base)
                    && check_index_growth::starts_growable(lhs, source, base)))
        {
            diags.push(make_diag(check_id, node));
            return;
        }

        // Pattern 3: an indexed write with a variable in its index into a variable
        // that is undefined, `[]`, `{}` or a number before the loop (see
        // check_index_growth.rs). A field of a struct array, `x(i).f`, counts too.
        if !field_base
            && matches!(
                lhs.kind(),
                "function_call" | "cell_index" | "field_expression"
            )
            && check_index_growth::indexed_write_grows(lhs, source)
        {
            diags.push(make_diag(check_id, node));
            return;
        }

        // Pattern 2: x = [x, val] or x = [x; val] (or the cell equivalent). MATLAB
        // reports concatenation as AGROW in scripts too (probed). A field target
        // such as `s.g = [s.g, i]` is not reported.
        if lhs.kind() == "identifier"
            && (rhs.kind() == "matrix" || rhs.kind() == "cell")
            && matrix_contains_var(rhs, source, lhs_text)
            && self.is_enabled("AGROW")
        {
            diags.push(make_diag("AGROW", node));
        }
    }
}

/// Whether `name` is the target of a plain assignment that starts before `before`.
fn defined_before(root: tree_sitter::Node, source: &str, name: &str, before: usize) -> bool {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if n.kind() == "assignment" && n.start_byte() < before {
            if let Some(l) = n.child_by_field_name("left") {
                if l.kind() == "identifier" && node_text(l, source) == name {
                    return true;
                }
            }
        }
        let mut c = n.walk();
        for ch in n.children(&mut c) {
            stack.push(ch);
        }
    }
    false
}

/// One variable read in a parfor body: its name, each indexed access as
/// `(key, node, broadcast)`, and each bare read of the whole value.
struct ParforAccess<'a> {
    name: &'a str,
    keys: Vec<String>,
    broadcast: bool,
    accesses: Vec<(String, tree_sitter::Node<'a>, bool)>,
    whole: Vec<tree_sitter::Node<'a>>,
}

/// PFBNS: a variable defined before a `parfor` loop and read in its body as a
/// broadcast variable. Probed on R2026b: a variable is reported when one of its
/// accesses is not sliced, when it is accessed in two different ways (two
/// subscript lists, a field and an index, ...), or when its whole value is read
/// (`numel(v)`, `disp(v)`, `x = v`) while it is also indexed in the body. An access
/// `v(...)` is sliced when a subscript is the loop index or the index plus or minus
/// a constant (`v(i)`, `v(i + 1)`, `v(i, 1)`, `v(1:3, i)`). `v(end)`, `v(:)`,
/// `v(2)`, `v([1 2])`, `v(1:i)`, `v(i/2)`, `v.f` and `v{k}` are not sliced. A
/// constant range on its own is neutral (`v(1:3)`, `v(1, 1:3)`). A whole read of a
/// variable that is not indexed in the body is silent, as are variables assigned in
/// the loop. The report is at the first unsliced access or whole read; without one,
/// at the first access whose key differs from the first access's.
fn check_broadcast_variables(root: tree_sitter::Node, source: &str, diags: &mut Vec<Diagnostic>) {
    let sym = crate::analysis::symbols::SymbolTable::shared(|| root, source);
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        let mut c = n.walk();
        for ch in n.children(&mut c) {
            stack.push(ch);
        }
        if n.kind() != "for_statement" || !node_text(n, source).trim_start().starts_with("parfor") {
            continue;
        }
        let index = crate::good_practices::extract_parfor_index_var(n, source).unwrap_or_default();
        let mut accesses: Vec<ParforAccess> = Vec::new();
        // Variables assigned in the loop, through any target shape.
        let mut assigned: Vec<&str> = Vec::new();
        let mut bare_names: Vec<tree_sitter::Node> = Vec::new();
        let mut inner = vec![n];
        while let Some(m) = inner.pop() {
            if m.kind() == "assignment" {
                if let Some(base) = m
                    .child_by_field_name("left")
                    .and_then(|l| lhs_base(l, source))
                {
                    assigned.push(base);
                }
            }
            if m.kind() == "identifier" && is_bare_name(m) {
                bare_names.push(m);
            }
            if let Some((name, key, broadcast, _sliced)) = broadcast_access(m, &index, source) {
                let access = (key.clone(), m, broadcast);
                match accesses.iter_mut().find(|a| a.name == name) {
                    Some(a) => {
                        if !a.keys.contains(&key) {
                            a.keys.push(key);
                        }
                        a.broadcast |= broadcast;
                        a.accesses.push(access);
                    }
                    None => accesses.push(ParforAccess {
                        name,
                        keys: vec![key],
                        broadcast,
                        accesses: vec![access],
                        whole: Vec::new(),
                    }),
                }
            }
            let mut c = m.walk();
            for ch in m.children(&mut c) {
                inner.push(ch);
            }
        }
        for a in &mut accesses {
            a.whole = bare_names
                .iter()
                .filter(|id| node_text(**id, source) == a.name)
                .copied()
                .collect();
        }
        for a in accesses {
            // Only variables defined before the loop (not function calls), never
            // assigned inside it, and broadcast, accessed in two ways, or read whole.
            if !(a.broadcast || a.keys.len() >= 2 || !a.whole.is_empty())
                || assigned.contains(&a.name)
                || !(defined_before(root, source, a.name, n.start_byte())
                    || crate::good_practices::is_defined_before_parfor(
                        &sym,
                        a.name,
                        n.start_byte(),
                    ))
                || is_pool_constant(a.name, &source[..n.start_byte()])
                || is_function_handle(a.name, &source[..n.start_byte()])
            {
                continue;
            }
            let unsliced = a
                .accesses
                .iter()
                .filter(|(_, _, broadcast)| *broadcast)
                .map(|(_, node, _)| *node)
                .chain(a.whole.iter().copied())
                .min_by_key(|node| node.start_byte());
            let at = unsliced.or_else(|| {
                let first = a
                    .accesses
                    .iter()
                    .min_by_key(|(_, node, _)| node.start_byte())?;
                let first_key = first.0.clone();
                a.accesses
                    .iter()
                    .filter(|(key, _, _)| *key != first_key)
                    .map(|(_, node, _)| *node)
                    .min_by_key(|node| node.start_byte())
            });
            if let Some(at) = at {
                diags.push(make_diag_named("PFBNS", at, a.name));
            }
        }
    }
}

/// Whether an identifier is a bare name: not the base of an index or call, and not
/// part of a field access (`v.f`, `s.v`).
fn is_bare_name(node: tree_sitter::Node) -> bool {
    let Some(p) = node.parent() else {
        return true;
    };
    match p.kind() {
        "field_expression" => false,
        "function_call" | "cell_index" => p.named_child(0).is_none_or(|c| c.id() != node.id()),
        _ => true,
    }
}

/// The variable at the root of an assignment target: `v`, `v(...)`, `v{...}`,
/// `v.f` or `v(i).f`.
fn lhs_base<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    let mut current = node;
    loop {
        match current.kind() {
            "identifier" => return Some(node_text(current, source)),
            "function_call" | "cell_index" | "field_expression" => {
                current = current.named_child(0)?;
            }
            _ => return None,
        }
    }
}

/// A PFBNS access of a variable in a `parfor` body: `(name, key, broadcast, sliced)`.
/// The key separates access shapes (the subscript list, or the field). `broadcast`
/// is true for an access that is not sliced and not a constant range.
fn broadcast_access<'a>(
    node: tree_sitter::Node<'a>,
    index: &str,
    source: &'a str,
) -> Option<(&'a str, String, bool, bool)> {
    match node.kind() {
        "function_call" | "cell_index" => {
            let base = node.named_child(0)?;
            let subs: Vec<tree_sitter::Node> = match find_arguments(node) {
                Some(args) => args.named_children(&mut args.walk()).collect(),
                None => node.named_children(&mut node.walk()).skip(1).collect(),
            };
            if subs.is_empty() {
                return None;
            }
            match base.kind() {
                "identifier" => {
                    let texts: Vec<String> = subs
                        .iter()
                        .map(|s| {
                            node_text(*s, source)
                                .chars()
                                .filter(|c| !c.is_whitespace())
                                .collect()
                        })
                        .collect();
                    // The loop index in exactly one subscript (`M(i, i)` is a broadcast).
                    let sliced = texts.iter().filter(|t| names_index(t, index)).count() == 1
                        && texts.iter().any(|t| is_sliced_subscript(t, index));
                    let neutral = texts.iter().any(|t| is_constant_range(t));
                    Some((
                        node_text(base, source),
                        texts.join(","),
                        !sliced && !neutral,
                        sliced,
                    ))
                }
                "field_expression" => {
                    let object = base.named_child(0)?;
                    (object.kind() == "identifier").then(|| {
                        (
                            node_text(object, source),
                            node_text(base, source).to_string(),
                            true,
                            false,
                        )
                    })
                }
                _ => None,
            }
        }
        "field_expression" => {
            let object = node.named_child(0)?;
            (object.kind() == "identifier").then(|| {
                (
                    node_text(object, source),
                    node_text(node, source).to_string(),
                    true,
                    false,
                )
            })
        }
        _ => None,
    }
}

/// Whether `name` is assigned a `parallel.pool.Constant` before the loop. MATLAB
/// does not report a Constant's `.Value` as a broadcast (probed).
fn is_pool_constant(name: &str, before: &str) -> bool {
    before.lines().any(|line| {
        line.trim()
            .strip_prefix(name)
            .and_then(|rest| rest.trim_start().strip_prefix('='))
            .is_some_and(|rhs| rhs.trim_start().starts_with("parallel.pool.Constant"))
    })
}

/// Whether the subscript text mentions the loop index as a whole name.
fn names_index(text: &str, index: &str) -> bool {
    !index.is_empty()
        && text
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .any(|t| t == index)
}

/// `i` or `i + k` / `i - k` for a constant `k` or a variable `k` (whitespace-free text).
/// Probed: MATLAB is silent for `data(i + k)`, so a variable offset counts as sliced.
fn is_sliced_subscript(text: &str, index: &str) -> bool {
    if index.is_empty() {
        return false;
    }
    if text == index {
        return true;
    }
    text.strip_prefix(index)
        .and_then(|rest| rest.strip_prefix(['+', '-']))
        .is_some_and(|k| {
            let is_name = k.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            !k.is_empty() && (k.chars().all(|c| c.is_ascii_digit()) || is_name)
        })
}

/// Whether `name` is assigned a function handle before the loop: a call of a handle is
/// a function call to MATLAB, not an indexed broadcast (probed: `g(1)` with `g = @(i) i`).
fn is_function_handle(name: &str, before: &str) -> bool {
    before.split(['\n', ';']).any(|statement| {
        statement
            .trim()
            .strip_prefix(name)
            .and_then(|rest| rest.trim_start().strip_prefix('='))
            .is_some_and(|rhs| rhs.trim_start().starts_with('@'))
    })
}

/// `a:b` or `a:s:b` with constant bounds (whitespace-free text).
fn is_constant_range(text: &str) -> bool {
    let parts: Vec<&str> = text.split(':').collect();
    (2..=3).contains(&parts.len())
        && parts.iter().all(|p| {
            let digits = p.strip_prefix('-').unwrap_or(p);
            !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
        })
}

impl Rule for PerformanceEngine {
    fn id(&self) -> &'static str {
        "PERFORMANCE_ENGINE"
    }

    fn description(&self) -> &'static str {
        "Performance improvement suggestions"
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn category(&self) -> Category {
        Category::Performance
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        match ctx.node.kind() {
            "function_call" => self.check_function_call(ctx.node, ctx.source),
            "command" => self.check_command(ctx.node, ctx.source),
            "binary_operator" => {
                let mut diags = self.check_binary_operator(ctx.node, ctx.source);
                // Also check for AND2/OR2 (& and | parse as binary_operator).
                diags.extend(self.check_boolean_context_binary(ctx.node, ctx.source));
                diags
            }
            "boolean_operator" => {
                // boolean_operator is for && / || which are already correct.
                Vec::new()
            }
            "comparison_operator" => self.check_comparison(ctx.node, ctx.source),
            "cell" => self.check_cell(ctx.node, ctx.source),
            _ => Vec::new(),
        }
    }

    fn active_check_ids(&self) -> Vec<&'static str> {
        CHECKS
            .iter()
            .map(|check| check.id)
            .filter(|id| self.is_enabled(id))
            .collect()
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        self.check_growth_in_loops(ctx.tree, ctx.source)
    }
}

// ---------------------------------------------------------------------------
// Helper: diagnostic construction
// ---------------------------------------------------------------------------

/// Build a diagnostic from a check ID and node.
pub(crate) fn make_diag(check_id: &'static str, node: tree_sitter::Node) -> Diagnostic {
    let start = node.start_position();
    Diagnostic {
        rule_id: check_id,
        message: check_description(check_id).to_string(),
        severity: Severity::Info,
        byte_range: node.start_byte()..node.end_byte(),
        line: start.row + 1,
        column: start.column + 1,
        fix: None,
    }
}

/// Build a diagnostic with an optional fix.
pub(crate) fn make_diag_with_fix(
    check_id: &'static str,
    node: tree_sitter::Node,
    fix: Option<Fix>,
) -> Diagnostic {
    let start = node.start_position();
    Diagnostic {
        rule_id: check_id,
        message: check_description(check_id).to_string(),
        severity: Severity::Info,
        byte_range: node.start_byte()..node.end_byte(),
        line: start.row + 1,
        column: start.column + 1,
        fix,
    }
}

/// Build a diagnostic whose message embeds a runtime name in place of the
/// `VAR_NAME` placeholder.
pub(crate) fn make_diag_named(
    check_id: &'static str,
    node: tree_sitter::Node,
    name: &str,
) -> Diagnostic {
    let start = node.start_position();
    Diagnostic {
        rule_id: check_id,
        message: check_description(check_id).replace("VAR_NAME", name),
        severity: Severity::Info,
        byte_range: node.start_byte()..node.end_byte(),
        line: start.row + 1,
        column: start.column + 1,
        fix: None,
    }
}

// ---------------------------------------------------------------------------
// Helper: node text extraction
// ---------------------------------------------------------------------------

/// Extract the raw text of a node.
pub(crate) fn node_text<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// Extract function name from a `function_call` node.
pub(crate) fn extract_func_name<'a>(
    node: tree_sitter::Node<'a>,
    source: &'a str,
) -> Option<&'a str> {
    if node.kind() != "function_call" || crate::analysis::symbols::is_qualified_call(node) {
        return None;
    }
    let name_node = node.child_by_field_name("name")?;
    Some(&source[name_node.start_byte()..name_node.end_byte()])
}

/// Extract command name from a `command` node.
pub(crate) fn extract_command_name<'a>(
    node: tree_sitter::Node<'a>,
    source: &'a str,
) -> Option<&'a str> {
    if node.kind() != "command" {
        return None;
    }
    let name_node = node.child(0)?;
    if name_node.kind() == "command_name" {
        Some(&source[name_node.start_byte()..name_node.end_byte()])
    } else {
        None
    }
}

/// Extract command arguments as a list of text fragments.
pub(crate) fn extract_command_args<'a>(
    node: tree_sitter::Node<'a>,
    source: &'a str,
) -> Vec<&'a str> {
    let mut args = Vec::new();
    for child in crate::analysis::symbols::children_of(node) {
        // Skip the command_name (first child) and punctuation.
        if child.kind() == "command_name" || child.kind() == ";" || child.kind() == "," {
            continue;
        }
        let text = node_text(child, source).trim();
        if !text.is_empty() {
            args.push(text);
        }
    }
    args
}

/// Extract the operator text from a `binary_operator` or `boolean_operator`.
pub(crate) fn extract_operator<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> &'a str {
    // The operator is typically the second child (between left and right operands),
    // or we can scan children for the operator token.
    for child in crate::analysis::symbols::children_of(node) {
        let text = node_text(child, source);
        // Operator tokens are short and not identifiers/expressions.
        match text {
            "+" | "-" | "*" | "/" | "\\" | "^" | ".*" | "./" | ".\\" | ".^" | "&" | "|" | "&&"
            | "||" | "==" | "~=" | "<" | ">" | "<=" | ">=" => return text,
            _ => continue,
        }
    }
    ""
}

// ---------------------------------------------------------------------------
// Helper: argument analysis
// ---------------------------------------------------------------------------

/// Find the `arguments` child node of a `function_call` node.
///
/// tree-sitter-matlab does not attach a field name to this child, so the
/// arguments list is located by node kind.
pub(crate) fn find_arguments<'a>(node: tree_sitter::Node<'a>) -> Option<tree_sitter::Node<'a>> {
    crate::analysis::symbols::children_of(node)
        .into_iter()
        .find(|child| child.kind() == "arguments")
}

/// Check if a function_call has a specific string argument.
pub(crate) fn has_string_arg(node: tree_sitter::Node, source: &str, value: &str) -> bool {
    for child in crate::analysis::symbols::children_of(node) {
        let text = node_text(child, source);
        // Match quoted strings: 'value' or "value"
        let trimmed = text.trim_matches('\'').trim_matches('"');
        if trimmed.eq_ignore_ascii_case(value) {
            return true;
        }
        // Recurse into arguments node.
        if child.kind() == "arguments" && has_string_arg(child, source, value) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Helper: pattern detection
// ---------------------------------------------------------------------------

/// Check if a function_call node is used as a subscript index.
/// i.e., the parent is the arguments of another function_call.
pub(crate) fn is_subscript_context(node: tree_sitter::Node) -> bool {
    if let Some(parent) = node.parent() {
        // Direct parent might be `arguments` of a function_call.
        if parent.kind() == "arguments" {
            if let Some(grandparent) = parent.parent() {
                return grandparent.kind() == "function_call";
            }
        }
        // Or the parent is directly a function_call (as an argument).
        if parent.kind() == "function_call" {
            // Make sure this node is not the function name itself.
            if let Some(name_node) = parent.child_by_field_name("name") {
                return name_node.id() != node.id();
            }
        }
    }
    false
}

/// Check if a node is at statement level (direct child of source_file or block).
pub(crate) fn is_statement_level(node: &tree_sitter::Node) -> bool {
    node.parent()
        .map(|p| matches!(p.kind(), "source_file" | "block"))
        .unwrap_or(false)
}

/// Check if a node is inside a condition expression (if/while/elseif).
pub(crate) fn is_in_condition(node: tree_sitter::Node) -> bool {
    let mut current = node.parent();
    while let Some(p) = current {
        match p.kind() {
            "if_statement" | "while_statement" | "elseif_clause" => {
                // Check if our node is in the condition part, not the body.
                // The condition is typically the first expression child after the keyword.
                if let Some(cond) = p.child_by_field_name("condition") {
                    return is_descendant_of(node, cond);
                }
                // Fallback: check if node appears before the "block" child.
                for child in crate::analysis::symbols::children_of(p) {
                    if child.kind() == "block" {
                        // If node is before the block, it's in the condition.
                        return node.start_byte() < child.start_byte();
                    }
                }
                return true;
            }
            "function_definition" => return false,
            _ => {}
        }
        current = p.parent();
    }
    false
}

/// Check if `needle` is a descendant of `haystack`.
pub(crate) fn is_descendant_of(needle: tree_sitter::Node, haystack: tree_sitter::Node) -> bool {
    needle.start_byte() >= haystack.start_byte() && needle.end_byte() <= haystack.end_byte()
}

/// Check if an `inv()` call is an operand of a binary_operator.
pub(crate) fn has_inv_operand(node: tree_sitter::Node, source: &str) -> bool {
    for child in crate::analysis::symbols::children_of(node) {
        if child.kind() == "function_call" {
            if let Some(name) = extract_func_name(child, source) {
                if name == "inv" {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if a matrix literal `[...]` contains a variable name as an element.
pub(crate) fn matrix_contains_var(node: tree_sitter::Node, source: &str, var_name: &str) -> bool {
    // Walk the matrix's children looking for an identifier matching var_name.
    for child in crate::analysis::symbols::children_of(node) {
        if child.kind() == "identifier" && node_text(child, source) == var_name {
            return true;
        }
        // Recurse into rows.
        if child.kind() == "row" && matrix_contains_var(child, source, var_name) {
            return true;
        }
    }
    false
}

/// Extract the comparison operands from a comparison like `length(x) == 0`.
/// Returns (function_name, compared_value) if the pattern matches.
pub(crate) fn extract_length_comparison<'a>(
    node: tree_sitter::Node<'a>,
    source: &'a str,
) -> Option<(&'a str, &'a str)> {
    // comparison_operator children: left op right
    let left = node.child(0)?;
    let op_node = node.child(1)?;
    let right = node.child(2)?;

    // `~=` is accepted for ISMT (`length(x) ~= 0`); callers gate ISCL on `==`.
    let op = node_text(op_node, source);
    if op != "==" && op != "~=" {
        return None;
    }

    // Check if left is a function_call to length/numel/size.
    if left.kind() == "function_call" {
        if let Some(name) = extract_func_name(left, source) {
            if name == "length" || name == "numel" {
                return Some((name, node_text(right, source).trim()));
            }
        }
    }

    // Also check reversed: 0 == length(x)
    if right.kind() == "function_call" {
        if let Some(name) = extract_func_name(right, source) {
            if name == "length" || name == "numel" {
                return Some((name, node_text(left, source).trim()));
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Helper: fix construction
// ---------------------------------------------------------------------------

/// Create a fix that renames the function in a function_call node.
pub(crate) fn fix_rename_func(node: tree_sitter::Node, source: &str, new_name: &str) -> Fix {
    if let Some(name_node) = node.child_by_field_name("name") {
        Fix::new(name_node.start_byte()..name_node.end_byte(), new_name)
    } else {
        // Fallback: try to replace the function name textually.
        let text = node_text(node, source);
        if let Some(paren_pos) = text.find('(') {
            let start = node.start_byte();
            Fix::new(start..start + paren_pos, new_name)
        } else {
            Fix::new(node.start_byte()..node.end_byte(), new_name)
        }
    }
}

/// Create a fix that replaces an operator in a binary_operator node.
pub(crate) fn fix_replace_operator(
    node: tree_sitter::Node,
    source: &str,
    old_op: &str,
    new_op: &str,
) -> Option<Fix> {
    for child in crate::analysis::symbols::children_of(node) {
        let text = node_text(child, source);
        if text == old_op {
            return Some(Fix::new(child.start_byte()..child.end_byte(), new_op));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "PERFORMANCE_ENGINE",
    PerformanceEngine::from_config
));

// ---------------------------------------------------------------------------
// Test support
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    /// Build a `PerformanceEngine` with default configuration for tests.
    pub(crate) fn engine() -> Box<dyn Rule> {
        PerformanceEngine::from_config(&Config::default())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::test_support::engine;
    use super::*;
    use crate::test_util::{has_id, lint_file, lint_nodes};
    use malt_core::Config;

    // -- AGROW (file-level) --------------------------------------------------

    #[test]
    fn agrow_fires_on_concatenation_growth() {
        let src = "function f()\nfor i = 1:10\n    x = [x, i];\nend\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "AGROW"), "got: {diags:?}");
    }

    #[test]
    fn agrow_fires_on_end_plus_one() {
        let src = "function f()\nfor i = 1:10\n    x(end+1) = i;\nend\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "AGROW"), "got: {diags:?}");
    }

    #[test]
    fn agrow_not_fire_on_indexed_assignment() {
        let src = "for i = 1:10\n    x(i) = i;\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "AGROW"), "got: {diags:?}");
    }

    // -- SAGROW (file-level) -------------------------------------------------

    #[test]
    fn sagrow_fires_on_script_growth() {
        let src = "for i = 1:10\n    x(end+1) = i;\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "SAGROW"), "got: {diags:?}");
    }

    #[test]
    fn sagrow_fires_on_numel_and_length_growth() {
        // Probed in MATLAB: `n(numel(n)+1)` and `n(length(n)+1)` report SAGROW.
        for src in [
            "n = [];\nfor i = 1:3\n    n(numel(n)+1) = i;\nend\n",
            "n = [];\nfor i = 1:3\n    n(length(n) + 1) = i;\nend\n",
        ] {
            let diags = lint_file(&*engine(), src);
            assert!(has_id(&diags, "SAGROW"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn sagrow_silent_on_size_of_another_variable() {
        // Not probed: the index counts a different variable, so it is not this growth.
        let src = "n = [];\nfor i = 1:3\n    n(numel(m)+1) = i;\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "SAGROW"), "got: {diags:?}");
    }

    #[test]
    fn agrow_fires_on_script_concatenation() {
        // Concatenation is AGROW in scripts too (probed); a field target is silent.
        let src = "for i = 1:10\n    y = [y, i];\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(
            has_id(&diags, "AGROW") && !has_id(&diags, "SAGROW"),
            "got: {diags:?}"
        );
        let field = "for i = 1:10\n    s.data = [s.data, i];\nend\n";
        let diags = lint_file(&*engine(), field);
        assert!(
            !has_id(&diags, "AGROW") && !has_id(&diags, "SAGROW"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn sagrow_not_fire_on_indexed_assignment() {
        let src = "for i = 1:10\n    s.data(i) = i;\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "SAGROW"), "got: {diags:?}");
    }

    #[test]
    fn agrow_in_function_sagrow_in_script() {
        let f = "function f()\nfor i = 1:10\n    x(end+1) = i;\nend\nend\n";
        let d = lint_file(&*engine(), f);
        assert!(has_id(&d, "AGROW") && !has_id(&d, "SAGROW"), "got: {d:?}");
        let s = "for i = 1:10\n    x(end+1) = i;\nend\n";
        let d = lint_file(&*engine(), s);
        assert!(has_id(&d, "SAGROW") && !has_id(&d, "AGROW"), "got: {d:?}");
    }

    #[test]
    fn sprix_fires_on_indexed_assignment_into_sparse() {
        let src = "S = sparse(10, 10);\nS(3) = 1;\n";
        assert!(has_id(&lint_file(&*engine(), src), "SPRIX"));
        let src = "S = zeros(10);\nS(3) = 1;\n";
        assert!(!has_id(&lint_file(&*engine(), src), "SPRIX"));
        let src = "S = sparse(10, 10);\nS = full(S);\nS(3) = 1;\n";
        assert!(!has_id(&lint_file(&*engine(), src), "SPRIX"));
    }

    #[test]
    fn pfbns_follows_matlab_broadcast_rule() {
        // Probed on R2026b: a non-sliced access is a broadcast on its own, and so are
        // two different access shapes.
        for body in [
            "r(i) = data(1);",
            "r(i) = data(end);",
            "r(i) = data(:);",
            "r(i) = data(floor(i));",
            "r(i) = data.x;",
            "r(i) = cfg.scale;",
            "r(i) = data(i) + data(i + 1);",
            "r(i) = data(i + 1) + data(1:3);",
        ] {
            let src = format!("data = foo();\ncfg = foo();\nr = zeros(1, 10);\nparfor i = 1:10\n    {body}\nend\n");
            assert!(has_id(&lint_file(&*engine(), &src), "PFBNS"), "{body}");
        }
        // Probed: a `parallel.pool.Constant` is not a broadcast variable.
        let src = "c = parallel.pool.Constant(@() 1);\nr = zeros(1, 10);\nparfor i = 1:10\n    r(i) = c.Value;\nend\n";
        assert!(!has_id(&lint_file(&*engine(), src), "PFBNS"));
        // Probed: one sliced shape, a constant range alone, a whole read and an
        // indexed field are silent.
        for body in [
            "r(i) = data(i) + data(i);",
            "r(i) = data(2:5);",
            "r(i) = numel(data);",
            "r(i) = sum(data);",
            "r(i) = data(1, 1:3);",
            "r(i) = cfg(i).scale + cfg(i).offset;",
        ] {
            let src = format!("data = foo();\ncfg = foo();\nr = zeros(1, 10);\nparfor i = 1:10\n    {body}\nend\n");
            assert!(!has_id(&lint_file(&*engine(), &src), "PFBNS"), "{body}");
        }
    }

    #[test]
    fn pfbns_follows_matlab_for_repeated_index_and_handles() {
        // Probed (bn58): the index in two subscripts of `M(i, i)` is a broadcast. Probed
        // (bn30): a variable offset `data(i + k)` is silent. Probed (lp12): a call of a
        // function handle is a function call, not an indexed broadcast.
        let src = "data = foo();\nM = foo();\nk = 1;\nr = zeros(1, 10);\nparfor i = 1:10\n    r(i) = M(i, i);\nend\n";
        assert!(has_id(&lint_file(&*engine(), src), "PFBNS"));
        let src = "data = foo();\nk = 1;\nr = zeros(1, 10);\nparfor i = 1:10\n    r(i) = data(i + k);\nend\n";
        assert!(!has_id(&lint_file(&*engine(), src), "PFBNS"));
        let src = "g = @(i) i;\nr = zeros(1, 10);\nparfor i = 1:10\n    r(i) = g(1);\nend\n";
        assert!(!has_id(&lint_file(&*engine(), src), "PFBNS"));
    }

    #[test]
    fn pfbns_fires_on_broadcast_variable_with_name() {
        let src = "c = foo();\nparfor i = 1:N\n    a(i) = c(i,1);\n    b(i) = c(i,2);\nend\n";
        let d = lint_file(&*engine(), src);
        let p = d.iter().find(|d| d.rule_id == "PFBNS").expect("PFBNS");
        assert!(
            p.message.contains("variable c is") || p.message.contains(" c "),
            "{}",
            p.message
        );
        assert!(!p.message.contains("VAR_NAME"));
        let src = "c = foo();\nparfor i = 1:N\n    a(i) = c(i,1);\n    b(i) = c(i,1);\nend\n";
        assert!(!has_id(&lint_file(&*engine(), src), "PFBNS"));
        let src = "c = foo();\nfor i = 1:N\n    a(i) = c(i,1);\n    b(i) = c(i,2);\nend\n";
        assert!(!has_id(&lint_file(&*engine(), src), "PFBNS"));
    }

    // -- Config-respected test --------------------------------------------------

    #[test]
    fn config_skip_checks_disables_individual_checks() {
        let config = Config::from_toml(
            r#"
[lint.rules.PERFORMANCE_ENGINE]
skip_checks = ["AGROW", "ST2NM"]
"#,
        )
        .unwrap();
        let rule = PerformanceEngine::from_config(&config);

        // Skipped check no longer fires.
        let src = "function f()\nfor i = 1:10\n    x = [x, i];\nend\nend\n";
        let diags = lint_file(&*rule, src);
        assert!(!has_id(&diags, "AGROW"), "got: {diags:?}");

        let src = "x = str2num('1 2');\n";
        let diags = lint_nodes(&*rule, src);
        assert!(!has_id(&diags, "ST2NM"), "got: {diags:?}");

        // Non-skipped checks still fire.
        let src = "x = inv(A) * b;\n";
        let diags = lint_nodes(&*rule, src);
        assert!(has_id(&diags, "MINV"), "got: {diags:?}");
    }
}

#[cfg(test)]
mod active_check_tests {
    use super::*;

    #[test]
    fn active_check_ids_are_the_checks_not_skipped() {
        let active = PerformanceEngine::from_config(&Config::default()).active_check_ids();
        assert!(active.contains(&"PFBNS") && active.contains(&"LAXES"));
        assert!(active.contains(&"AGROW"));
        // GRIDD is switched off under the mathworks preset. MATLAB reports an
        // unused `%#ok<GRIDD>` (probed), but malt cannot tell which off-by-default
        // checks MATLAB keeps active (CTCH is silent), so it declares none of them.
        assert!(!active.contains(&"GRIDD"));

        let config =
            Config::from_toml("[lint.rules.PERFORMANCE_ENGINE]\nskip_checks = [\"PFBNS\"]\n")
                .unwrap();
        let active = PerformanceEngine::from_config(&config).active_check_ids();
        assert!(!active.contains(&"PFBNS"));
        assert!(active.contains(&"LAXES"));
    }
}
