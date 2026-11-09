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
        let rule_config: PerformanceEngineConfig = config.rule_params("PERFORMANCE_ENGINE");
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

        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                self.traverse_for_growth(&child, source, now_in_loop, now_in_function, diags);
            }
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
        let rhs_text = node_text(rhs, source);

        // Pattern 1: x(end+1) = ... or x{end+1} = ...
        if (lhs.kind() == "function_call" || lhs.kind() == "cell_index")
            && contains_end_plus_pattern(lhs, source)
        {
            diags.push(make_diag(check_id, node));
            return;
        }

        // Pattern 1b: s.field(end+1) = ... — field_expression as LHS.
        if lhs.kind() == "field_expression" {
            let assign_text = node_text(node, source);
            if let Some(eq_pos) = assign_text.find('=') {
                let lhs_full = &assign_text[..eq_pos];
                let normalized: String = lhs_full.chars().filter(|c| !c.is_whitespace()).collect();
                if normalized.contains("end+1") {
                    diags.push(make_diag(check_id, node));
                    return;
                }
            }
        }

        // Pattern 2: x = [x, val] or x = [x; val] (or the cell equivalent).
        if lhs.kind() == "identifier"
            && (rhs.kind() == "matrix" || rhs.kind() == "cell")
            && matrix_contains_var(rhs, source, lhs_text)
        {
            diags.push(make_diag(check_id, node));
            return;
        }

        // Pattern 3: s.field = [s.field, val]
        if lhs.kind() == "field_expression" && rhs.kind() == "matrix" && rhs_text.contains(lhs_text)
        {
            diags.push(make_diag(check_id, node));
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

/// PFBNS: a variable indexed with different expressions inside a `parfor`
/// body (and never assigned there) is a broadcast variable.
fn check_broadcast_variables(root: tree_sitter::Node, source: &str, diags: &mut Vec<Diagnostic>) {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        let mut c = n.walk();
        for ch in n.children(&mut c) {
            stack.push(ch);
        }
        if n.kind() != "for_statement" || !node_text(n, source).trim_start().starts_with("parfor") {
            continue;
        }
        // Collect indexed uses (name -> (distinct index texts, first node)) and assigned names.
        let mut uses: Vec<(&str, Vec<String>, tree_sitter::Node)> = Vec::new();
        let mut assigned: Vec<&str> = Vec::new();
        let mut inner = vec![n];
        while let Some(m) = inner.pop() {
            if m.kind() == "assignment" {
                if let Some(l) = m.child_by_field_name("left") {
                    let base = if l.kind() == "function_call" {
                        extract_func_name(l, source)
                    } else {
                        Some(node_text(l, source))
                    };
                    if let Some(b) = base {
                        assigned.push(b);
                    }
                }
            }
            if m.kind() == "function_call" {
                if let (Some(name), Some(args)) = (extract_func_name(m, source), find_arguments(m))
                {
                    let idx = node_text(args, source)
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect::<String>();
                    match uses.iter_mut().find(|(u, _, _)| *u == name) {
                        Some((_, v, _)) => {
                            if !v.contains(&idx) {
                                v.push(idx);
                            }
                        }
                        None => uses.push((name, vec![idx], m)),
                    }
                }
            }
            let mut c = m.walk();
            for ch in m.children(&mut c) {
                inner.push(ch);
            }
        }
        for (name, idxs, first) in uses {
            // Only variables defined before the loop (not function calls), never
            // assigned inside it, indexed in at least two different ways.
            if idxs.len() >= 2
                && !assigned.contains(&name)
                && defined_before(root, source, name, n.start_byte())
            {
                diags.push(make_diag_named("PFBNS", first, name));
            }
        }
    }
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
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            // Skip the command_name (first child) and punctuation.
            if child.kind() == "command_name" || child.kind() == ";" || child.kind() == "," {
                continue;
            }
            let text = node_text(child, source).trim();
            if !text.is_empty() {
                args.push(text);
            }
        }
    }
    args
}

/// Extract the operator text from a `binary_operator` or `boolean_operator`.
pub(crate) fn extract_operator<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> &'a str {
    // The operator is typically the second child (between left and right operands),
    // or we can scan children for the operator token.
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            let text = node_text(child, source);
            // Operator tokens are short and not identifiers/expressions.
            match text {
                "+" | "-" | "*" | "/" | "\\" | "^" | ".*" | "./" | ".\\" | ".^" | "&" | "|"
                | "&&" | "||" | "==" | "~=" | "<" | ">" | "<=" | ">=" => return text,
                _ => continue,
            }
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
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            if child.kind() == "arguments" {
                return Some(child);
            }
        }
    }
    None
}

/// Check if a function_call has a specific string argument.
pub(crate) fn has_string_arg(node: tree_sitter::Node, source: &str, value: &str) -> bool {
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
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

/// Check if node is inside a loop (for_statement or while_statement).
pub(crate) fn is_inside_loop(node: tree_sitter::Node) -> bool {
    let mut current = node.parent();
    while let Some(p) = current {
        match p.kind() {
            "for_statement" | "while_statement" => return true,
            "function_definition" => return false, // Stop at function boundary.
            _ => {}
        }
        current = p.parent();
    }
    false
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
                let count = p.child_count();
                for i in 0..count {
                    if let Some(child) = p.child(i) {
                        if child.kind() == "block" {
                            // If node is before the block, it's in the condition.
                            return node.start_byte() < child.start_byte();
                        }
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
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            if child.kind() == "function_call" {
                if let Some(name) = extract_func_name(child, source) {
                    if name == "inv" {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Check if an indexing expression contains `end+1` or `end + 1` pattern.
pub(crate) fn contains_end_plus_pattern(node: tree_sitter::Node, source: &str) -> bool {
    let text = node_text(node, source);
    // Simple textual check: look for "end+1" or "end + 1" within parentheses or braces.
    let normalized: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    normalized.contains("end+1")
}

/// Check if a matrix literal `[...]` contains a variable name as an element.
pub(crate) fn matrix_contains_var(node: tree_sitter::Node, source: &str, var_name: &str) -> bool {
    // Walk the matrix's children looking for an identifier matching var_name.
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" && node_text(child, source) == var_name {
                return true;
            }
            // Recurse into rows.
            if child.kind() == "row" && matrix_contains_var(child, source, var_name) {
                return true;
            }
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

    let op = node_text(op_node, source);
    if op != "==" {
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
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            let text = node_text(child, source);
            if text == old_op {
                return Some(Fix::new(child.start_byte()..child.end_byte(), new_op));
            }
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
        let src = "for i = 1:10\n    s.data(end+1) = i;\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "SAGROW"), "got: {diags:?}");
    }

    #[test]
    fn sagrow_fires_on_script_concatenation() {
        let src = "for i = 1:10\n    s.data = [s.data, i];\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "SAGROW"), "got: {diags:?}");
    }

    #[test]
    fn sagrow_not_fire_on_indexed_assignment() {
        let src = "for i = 1:10\n    s.data(i) = i;\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "SAGROW"), "got: {diags:?}");
    }

    #[test]
    fn agrow_in_function_sagrow_in_script() {
        let f = "function f()\nfor i = 1:10\n    s.data(end+1) = i;\nend\nend\n";
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
