use std::cell::Cell;
use std::collections::HashSet;
use std::ops::Range;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use tree_sitter::Parser;

use crate::diagnostic::{Diagnostic, Fix};
use crate::limits::{self, Limits};
use crate::registry::RuleRegistry;
use crate::rule::{FileContext, NodeContext, ProjectIndex};
use crate::suppression;
use crate::Severity;

/// Checks that report a token MATLAB cannot lex or parse. The statement holding
/// such a token is not analysed by any other check.
const STATEMENT_ERROR_IDS: &[&str] = &[
    "SYNER", "BADCH", "BADFP", "BADHBH", "BADHBB", "BADHBHT", "BADHBBT", "STRIN",
];

/// Number-literal checks whose range covers the rest of a malformed literal. A
/// SYNER inside that range is the same error, so MATLAB reports only the literal.
const NUMBER_LITERAL_IDS: &[&str] = &["BADHBH", "BADHBB", "BADFP", "BADHBHT", "BADHBBT"];

/// Parse-level checks that stay even when they sit in a broken statement: the
/// statement error is itself reported by one of them, or they describe the
/// structure of the broken region (missing `end` or bracket).
const STATEMENT_ERROR_KEPT_IDS: &[&str] = &[
    "NOSEMI",
    "NOCOMMA",
    "NOPAR",
    "NOPAR2",
    "EOLPAR",
    "ENDPAR",
    "ENDCT",
    "ENDCT2",
    "ENDCT3",
    "ENDCT4",
    "RESWD",
    "FVSYN",
    "FVAMI",
    "FVACS",
    "SBTMP",
    "INBLK",
    "BADNOT",
    "BADNOTLHS",
    "EOFER",
    // Incomplete-analysis limits: MATLAB reports them for the file whatever the
    // statements hold (TMSMS at line 1 with 1,001 errors on line 1, probed).
    "TMSMS",
    "MXASET",
    "NOSPC",
    "MDEEP",
    "DEEPN",
    "DEEPC",
];

/// The parse-level checks that MATLAB keeps when a file has EOFER, taken from the syntax
/// engine's check IDs (SEPEX is kept only by [`is_loop_header_sepex`]).
const EOFER_KEPT_IDS: &[&str] = &[
    "BADCH",
    "BADNE",
    "BADNOT",
    "BADNOTLHS",
    "BADSP",
    "BDFIL",
    "DOUQT",
    "ENDCT",
    "ENDCT2",
    "ENDCT3",
    "ENDPAR",
    "EOFER",
    "EOLPAR",
    "FNDOT",
    "FVACI",
    "FVAPN",
    "FVSYN",
    "INBLK",
    "LHROW",
    "MCPLD",
    "NOCOMMA",
    "NOLHS",
    "NOPAR",
    "NOPAR2",
    "REDEF",
    "RESWD",
    "SBTMP",
    "SEPEXC",
    "SEPEXR",
    "STRIN",
    "SYNER",
    "TWOCM",
    "UNSET",
];

/// SEPEX on a `for` or `while` line with no separator after it (`while x x=0 end`) is
/// kept with the parse messages: MATLAB reports it beside the SYNER there. Other SEPEX on
/// a broken statement (`if x, y end`, `while k = 3, end`) is silent in MATLAB and is
/// dropped with the statement.
fn is_loop_header_sepex(d: &Diagnostic, source: &str) -> bool {
    if d.rule_id != "SEPEX" {
        return false;
    }
    let line_start = source[..d.byte_range.start]
        .rfind('\n')
        .map_or(0, |i| i + 1);
    let line_end = source[d.byte_range.start..]
        .find('\n')
        .map_or(source.len(), |i| d.byte_range.start + i);
    let head = source[line_start..d.byte_range.start].trim_start();
    let unseparated = !source[d.byte_range.start..line_end].contains([',', ';']);
    unseparated && matches!(head.split_whitespace().next(), Some("for" | "while"))
}

/// Drops every check but the parse messages from a file that has EOFER.
fn drop_checks_after_eofer(diagnostics: &mut Vec<Diagnostic>, source: &str) {
    if diagnostics.iter().any(|d| d.rule_id == "EOFER") {
        diagnostics.retain(|d| {
            EOFER_KEPT_IDS.contains(&d.rule_id)
                || STATEMENT_ERROR_KEPT_IDS.contains(&d.rule_id)
                || is_loop_header_sepex(d, source)
        });
    }
}

/// Removes the diagnostics of the statements that hold a token MATLAB cannot
/// parse. MATLAB reports the token and still analyses the statements that follow
/// it on the same line: statements are separated by `;`, `,` or a newline. So
/// the broken statement runs from the separator before the token to the next one.
fn drop_statements_with_errors(diagnostics: &mut Vec<Diagnostic>, source: &str) {
    // MATLAB drops every SYNER of the statement that holds the literal error
    // (`y = 0b102;` reports BADHBB only; `x = 0xFG;` reports BADHBH only).
    let literal_spans: Vec<Range<usize>> = diagnostics
        .iter()
        .filter(|d| NUMBER_LITERAL_IDS.contains(&d.rule_id))
        .map(|d| statement_span(source, d.byte_range.start))
        .collect();
    // A SYNER on the `;` that ends the literal's statement belongs to it too (`x = 1e+;`).
    diagnostics.retain(|d| {
        d.rule_id != "SYNER"
            || !literal_spans
                .iter()
                .any(|s| s.contains(&d.byte_range.start) || s.end == d.byte_range.start)
    });
    let spans: Vec<Range<usize>> = diagnostics
        .iter()
        .filter(|d| STATEMENT_ERROR_IDS.contains(&d.rule_id))
        .map(|d| statement_span(source, d.byte_range.start))
        .collect();
    if spans.is_empty() {
        return;
    }
    diagnostics.retain(|d| {
        STATEMENT_ERROR_IDS.contains(&d.rule_id)
            || STATEMENT_ERROR_KEPT_IDS.contains(&d.rule_id)
            || is_loop_header_sepex(d, source)
            || !spans.iter().any(|s| s.contains(&d.byte_range.start))
    });
}

/// Byte range of the statement that contains `pos`, with separators excluded.
fn statement_span(source: &str, pos: usize) -> Range<usize> {
    let is_separator = |b: &u8| matches!(b, b'\n' | b';' | b',');
    let bytes = source.as_bytes();
    let pos = pos.min(bytes.len());
    let start = bytes[..pos]
        .iter()
        .rposition(is_separator)
        .map_or(0, |p| p + 1);
    let end = bytes[pos..]
        .iter()
        .position(is_separator)
        .map_or(bytes.len(), |p| pos + p);
    start..end
}

// ---------------------------------------------------------------------------
// Linter
// ---------------------------------------------------------------------------

/// The core lint engine. Performs a single-pass, depth-first traversal of the
/// tree-sitter syntax tree and dispatches nodes to matching rules.
///
/// # Performance
///
/// - Uses `TreeCursor` for stack-based, allocation-free traversal.
/// - Looks up rules via the `RuleRegistry`'s O(1) node-type index.
/// - Each node is visited exactly once; rules share the traversal.
/// - After traversal, `check_file()` is called once per rule for file-level analysis.
///
/// # Severity Overrides
///
/// Each diagnostic carries the severity its check assigned. When the user
/// configured a severity for the rule (per rule or per category), the linter
/// stamps that override onto the rule's diagnostics before returning.
///
/// # Inline Suppression
///
/// When [`Linter::inline_suppression`] is enabled (the default), diagnostics
/// on lines carrying a `%#ok<...>` directive comment are filtered out, mirroring
/// MATLAB's inline suppression. A directive that names an active check which
/// reported nothing on its line gets an MSNU (`A Code Analyzer message was once
/// suppressed here...`, Info); a directive naming an ID malt does not track is
/// silent (MATLAB's MSNE is not emitted).
pub struct Linter {
    registry: Arc<RuleRegistry>,
    parser: Parser,
    inline_suppression: bool,
    /// Resource limits: file size, tree depth, bracket nesting. See [`Limits`].
    limits: Limits,
    /// Threads that run the file checks of a large file side by side; 1 (the
    /// default) runs them on the calling thread. See [`Linter::set_file_check_threads`].
    file_check_threads: usize,
    /// Stack size of each of those threads.
    file_check_stack: usize,
    /// The search path, when the host enabled the path index. See
    /// [`Linter::set_project_index`].
    project: Option<Arc<dyn ProjectIndex>>,
}

/// Trees with fewer nodes than this run their file checks on one thread even
/// when more are allowed: starting threads would cost more than it saves.
const PARALLEL_MIN_NODES: usize = 20_000;

/// The deepest syntax tree the analyzer works on by default.
///
/// Rules recurse over the tree, so a deep one costs stack, and some analyses
/// get slower than linearly with depth (500 nested `if`s, about 1,000 levels,
/// take seconds even in a release build). Real code stays far below it: every
/// nested statement adds two or three levels. Deeper input is reported, not
/// analyzed, instead of crashing the process or hanging it.
pub use crate::limits::DEFAULT_MAX_TREE_DEPTH;

/// MATLAB stops generating messages at this count and reports TMMSG; the
/// messages past it are deleted. malt keeps the first ones in source order.
pub const MATLAB_MAX_MESSAGES: usize = 10_000;

/// Number of parses made so far by [`Linter::lint`] and by test helpers, in
/// all threads. See [`note_parse`].
static PARSE_COUNT: AtomicU64 = AtomicU64::new(0);

thread_local! {
    /// The parse epoch of this thread: the parse it analyses now.
    static PARSE_EPOCH: Cell<u64> = const { Cell::new(0) };
}

/// The current parse epoch of this thread. Analyses that memoise by node id (an
/// address that a later parse may reuse) key their memo by it. Each thread has
/// its own, so threads that lint files side by side keep their memos.
pub fn parse_epoch() -> u64 {
    PARSE_EPOCH.with(Cell::get)
}

/// Start a new parse epoch in this thread. Call before each parse whose tree
/// rules analyse, so a memo built for an earlier tree is never read for this
/// one. Epochs are unique across threads.
pub fn note_parse() {
    let epoch = PARSE_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    PARSE_EPOCH.with(|current| current.set(epoch));
}

/// Errors that can occur during linting.
#[derive(Debug, thiserror::Error)]
pub enum LintError {
    #[error("failed to parse source: tree-sitter returned no tree")]
    ParseFailed,
}

impl Linter {
    /// Create a new `Linter` with the given rule registry.
    ///
    /// Initializes the tree-sitter parser with the MATLAB language grammar.
    /// Inline suppression (`%#ok<...>`) is enabled by default; disable it with
    /// [`Linter::set_inline_suppression`].
    pub fn new(registry: RuleRegistry) -> Self {
        Self::shared(Arc::new(registry))
    }

    /// Create a `Linter` on a registry that other linters share, such as one
    /// linter per thread linting files side by side.
    pub fn shared(registry: Arc<RuleRegistry>) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_matlab::LANGUAGE.into())
            .expect("failed to load tree-sitter-matlab grammar");

        Self {
            registry,
            parser,
            inline_suppression: true,
            limits: Limits::native(),
            file_check_threads: 1,
            file_check_stack: 0,
            project: None,
        }
    }

    /// Give the rules the search path of the run (`None` to take it away, the
    /// default). Off, a file is linted on its own, as MATLAB does; on, rules
    /// that use it may report more (see [`ProjectIndex`]).
    pub fn set_project_index(&mut self, index: Option<Arc<dyn ProjectIndex>>) {
        self.project = index;
    }

    /// Run the file-level checks of a large file on up to `threads` threads
    /// (including the calling one), each with a stack of `stack_bytes`: rules
    /// recurse over the tree, so a deep file needs as much stack as the calling
    /// thread has. The diagnostics are the same as with one thread.
    pub fn set_file_check_threads(&mut self, threads: usize, stack_bytes: usize) {
        self.file_check_threads = threads.max(1);
        self.file_check_stack = stack_bytes;
    }

    /// Set the deepest syntax tree that is analyzed (default
    /// [`DEFAULT_MAX_TREE_DEPTH`]). Hosts with a small stack, such as
    /// WebAssembly, lower it.
    pub fn set_max_tree_depth(&mut self, depth: usize) {
        self.limits.max_tree_depth = depth;
    }

    /// Set the resource limits (file size, tree depth, bracket nesting). The
    /// thread limit is the host's to apply: see [`Linter::set_file_check_threads`].
    pub fn set_limits(&mut self, limits: Limits) {
        self.limits = limits;
    }

    /// The resource limits in force.
    pub fn limits(&self) -> Limits {
        self.limits
    }

    /// Enable or disable inline suppression directives (`%#ok<...>`).
    ///
    /// Enabled by default; call with `false` to mirror a config file that sets
    /// `inline_suppression = false`.
    pub fn set_inline_suppression(&mut self, enabled: bool) {
        self.inline_suppression = enabled;
    }

    /// Lint a source file, returning all diagnostics sorted by position.
    ///
    /// This is the main entry point. It:
    /// 1. Parses `source` into a tree-sitter `Tree`.
    /// 2. Performs a single DFS traversal, dispatching nodes to subscribed rules.
    /// 3. Calls `check_file()` on all rules for file-level analysis.
    /// 4. Filters out diagnostics suppressed by inline `%#ok<...>` directives.
    /// 5. Returns diagnostics sorted by (line, column).
    ///
    /// Severity overrides from config are stamped onto diagnostics as they are
    /// collected during steps 2 and 3.
    pub fn lint(&mut self, source: &str, file_path: &Path) -> Result<Vec<Diagnostic>, LintError> {
        // Resource limits come first, before the parse allocates: a file over
        // `max_file_size`, or one that nests brackets past `max_nesting`, is
        // reported and not parsed.
        if source.len() as u64 > self.limits.max_file_size {
            return Ok(vec![limits::too_big_diagnostic(
                source.len() as u64,
                self.limits.max_file_size,
            )]);
        }
        if limits::exceeds_nesting(source, self.limits.max_nesting) {
            return Ok(vec![limits::too_nested_diagnostic(self.limits.max_nesting)]);
        }

        // Step 1: Parse
        note_parse();
        let tree = self
            .parser
            .parse(source, None)
            .ok_or(LintError::ParseFailed)?;

        // MATLAB's own limits (NOSPC, MXASET) stop the analysis: MATLAB reports
        // nothing else for such a file (probed). A tree too deep for malt to walk
        // safely gets MALT_TOO_DEEP unless one of those limits applies.
        let mut limits = Vec::new();
        for &idx in self.registry.file_check_rules() {
            let mut found = self.registry.get_rule(idx).check_limits(&tree, source);
            if let Some(severity) = self.registry.severity_override(idx) {
                for diag in &mut found {
                    diag.severity = severity;
                }
            }
            limits.extend(found);
        }
        if !limits.is_empty() {
            limits.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));
            return Ok(limits);
        }
        let depth = tree_depth(&tree);
        if depth > self.limits.max_tree_depth {
            return Ok(vec![Self::too_deep_diagnostic(
                source,
                depth,
                self.limits.max_tree_depth,
            )]);
        }

        // Step 2: Single-pass DFS traversal using TreeCursor
        // Steps 2-3 are wrapped in a panic guard: if any rule panics (an
        // internal analyzer error), the analysis is considered incomplete and a
        // single QUIT diagnostic is returned instead of crashing.
        let result = catch_unwind(AssertUnwindSafe(|| {
            // Step 3: File-level checks (only on rules that implement check_file)
            let file_ctx = FileContext {
                tree: &tree,
                source,
                file_path,
                project: self.project.as_deref(),
            };
            let threads = self
                .file_check_threads
                .min(self.registry.file_check_rules().len());
            if threads > 1 && tree.root_node().descendant_count() >= PARALLEL_MIN_NODES {
                return self.check_in_parallel(&file_ctx, threads);
            }
            let mut diagnostics = self.traverse(&tree, source, file_path);
            for &idx in self.registry.file_check_rules() {
                diagnostics.extend(self.check_file_with(idx, &file_ctx));
            }
            diagnostics
        }));

        let mut diagnostics = match result {
            Ok(diags) => diags,
            // Internal analyzer failure — analysis did not complete.
            Err(_) => vec![Self::quit_diagnostic(source)],
        };

        // Step 3b: MATLAB does not analyse the statement holding a token it cannot parse.
        drop_statements_with_errors(&mut diagnostics, source);
        // Step 3b': with EOFER ("Code analysis did not complete") MATLAB reports only the
        // parse messages for the file (probed: NOPTS and NASGU vanish, SEPEX and SYNER stay).
        drop_checks_after_eofer(&mut diagnostics, source);

        // Per-check settings of a MATLAB configuration file, before the message limit
        // counts the findings (MATLAB counts only the messages it reports).
        self.registry.apply_check_settings(&mut diagnostics);

        // Step 3c: MATLAB stops at 10,000 messages (TMMSG), counting every rule's
        // messages before the %#ok directives remove any.
        limit_parse_errors(&mut diagnostics);
        limit_messages(&mut diagnostics, source);

        // Step 4: Apply inline suppression directives (%#ok<...>).
        // A `%#ok<*ID>` wildcard suppresses its ID in the whole file and is applied
        // first (probed: a named directive whose message a wildcard removed gets MSNU).
        // A line directive suppresses matching diagnostics on that line only.
        if self.inline_suppression {
            let suppressions = suppression::from_tree(source, &tree);
            diagnostics.retain(|d| !suppressions.is_file_wide(d.rule_id));
            // MSNU reads what the checks reported before the directives removed it.
            // An analysis that did not complete cannot say what was generated.
            let analysis_complete = !diagnostics.iter().any(|d| d.rule_id == "QUIT");
            let unused = if analysis_complete {
                let mut unused = unused_suppressions(
                    &diagnostics,
                    &suppressions,
                    &self.registry.active_check_ids(),
                );
                unused.extend(unused_bare_suppressions(&diagnostics, &suppressions));
                unused
            } else {
                Vec::new()
            };
            // MSNE needs no messages from the checks, so it is not tied to the
            // analysis completing.
            let unknown = unknown_suppressions(&suppressions);
            diagnostics.retain(|d| !suppressions.is_suppressed(d.line, d.rule_id));
            // A wildcard also removes MSNU (probed: `%#ok<*MSNU>` silences it).
            diagnostics.extend(
                unused
                    .into_iter()
                    .filter(|d| !suppressions.is_file_wide(d.rule_id)),
            );
            diagnostics.extend(unknown);
        }

        // Step 5: Sort by position for deterministic output
        diagnostics.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));

        Ok(diagnostics)
    }

    /// The file-level diagnostics of rule `idx`, with the configured severity
    /// override, if any.
    fn check_file_with(&self, idx: usize, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut found = self.registry.get_rule(idx).check_file(ctx);
        if let Some(severity) = self.registry.severity_override(idx) {
            for diag in &mut found {
                diag.severity = severity;
            }
        }
        found
    }

    /// Steps 2 and 3 with the file checks on `threads` threads: they pull rules
    /// from a queue while this thread walks the tree for the node checks and
    /// then helps. The diagnostics come out in the order of the serial run.
    ///
    /// Each thread keeps its own memos (symbol table, dead stores...), so some
    /// analyses run more than once; that costs CPU time, not wall time.
    fn check_in_parallel(&self, ctx: &FileContext, threads: usize) -> Vec<Diagnostic> {
        let rules = self.registry.file_check_rules();
        let next = AtomicUsize::new(0);
        let results: Mutex<Vec<Option<Vec<Diagnostic>>>> =
            Mutex::new(rules.iter().map(|_| None).collect());
        let work = || loop {
            let i = next.fetch_add(1, Ordering::Relaxed);
            let Some(&idx) = rules.get(i) else {
                break;
            };
            let found = self.check_file_with(idx, ctx);
            results.lock().expect("no thread panics holding the lock")[i] = Some(found);
        };
        let mut diagnostics = std::thread::scope(|scope| {
            for _ in 1..threads {
                // A thread that cannot start leaves its share to the others.
                let _ = std::thread::Builder::new()
                    .stack_size(self.file_check_stack)
                    .spawn_scoped(scope, || {
                        // Memos of this thread belong to this tree only.
                        note_parse();
                        work();
                    });
            }
            let diagnostics = self.traverse(ctx.tree, ctx.source, ctx.file_path);
            work();
            diagnostics
        });
        // A panicking thread has made `scope` panic: every result is here.
        let results = results
            .into_inner()
            .expect("no thread panics holding the lock");
        for found in results {
            diagnostics.extend(found.expect("every rule was checked"));
        }
        diagnostics
    }

    /// The MALT_TOO_DEEP diagnostic for code nested beyond malt's depth limit.
    ///
    /// Not a MATLAB check: MATLAB has no such limit on the parse tree. malt stops
    /// here to keep its own stack from overflowing, and says so under its own ID.
    fn too_deep_diagnostic(source: &str, depth: usize, limit: usize) -> Diagnostic {
        Diagnostic {
            rule_id: "MALT_TOO_DEEP",
            message: format!(
                "malt did not analyze this file: the code is nested too deeply \
                 ({depth} levels; malt stops at {limit}, max_tree_depth). To analyze it, raise \
                 max_tree_depth under [lint.limits] in .malt.toml; deeper trees can use very \
                 large amounts of memory or stack. This is not a MATLAB message."
            ),
            severity: Severity::Error,
            byte_range: 0..source.len().min(1),
            line: 1,
            column: 1,
            fix: None,
        }
    }

    /// Build the QUIT diagnostic ("Code analysis did not complete...").
    ///
    /// Emitted when an internal analyzer error (a panicking rule) prevents the
    /// analysis of a file from completing. Part of the Incomplete Analysis
    /// checks; `can_be_disabled` is not applicable because this is an engine
    /// guard, not a registered rule.
    fn quit_diagnostic(source: &str) -> Diagnostic {
        Diagnostic {
            rule_id: "QUIT",
            message: "Code analysis did not complete. Code Analyzer encountered an error."
                .to_string(),
            severity: Severity::Error,
            byte_range: 0..source.len().min(1),
            line: 1,
            column: 1,
            fix: None,
        }
    }

    /// Perform the single-pass depth-first traversal.
    ///
    /// Uses `TreeCursor` which is the most efficient way to traverse a
    /// tree-sitter tree — it avoids allocating `Node` objects and uses an
    /// internal stack.
    fn traverse(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        file_path: &Path,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut cursor = tree.walk();
        // The ancestors of the cursor's node, the root first.
        let mut ancestors: Vec<tree_sitter::Node> = Vec::new();

        // Depth-first traversal using TreeCursor's goto_* methods.
        // We visit every node exactly once.
        loop {
            let node = cursor.node();
            let node_kind = node.kind();

            // O(1) lookup: are there rules interested in this node type?
            let rule_indices = self.registry.rules_for_node_type(node_kind);
            if !rule_indices.is_empty() {
                let node_ctx = NodeContext {
                    node,
                    source,
                    file_path,
                    ancestors: Some(&ancestors),
                };

                for &idx in rule_indices {
                    let rule = self.registry.get_rule(idx);
                    let mut rule_diags = rule.check(&node_ctx);
                    // Apply the configured severity override, if any; otherwise
                    // the diagnostic keeps the severity its check assigned.
                    if let Some(severity) = self.registry.severity_override(idx) {
                        for diag in &mut rule_diags {
                            diag.severity = severity;
                        }
                    }
                    diagnostics.extend(rule_diags);
                }
            }

            // DFS navigation: try child first, then sibling, then backtrack.
            if cursor.goto_first_child() {
                ancestors.push(node);
                continue;
            }
            if cursor.goto_next_sibling() {
                continue;
            }

            // Backtrack up the tree until we find an ancestor with a next sibling.
            loop {
                ancestors.pop();
                if !cursor.goto_parent() {
                    // We've returned to the root — traversal complete.
                    return diagnostics;
                }
                if cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }
}

/// Message of MATLAB's MSNU: a `%#ok<ID>` directive whose check is active but
/// generated no message on its line.
const MSNU_MESSAGE: &str =
    "A Code Analyzer message was once suppressed here, but the message is no longer generated.";

/// MSNU diagnostics for the named check IDs of `suppressions` that are active
/// but produced no diagnostic with that ID on their line.
///
/// Only the IDs in `active` are considered: an ID malt does not declare, or
/// has switched off, is never reported, so a missing check cannot produce it.
/// The MSNU itself is not suppressible (MATLAB reports it even when named by
/// a directive on its line).
fn unused_suppressions(
    diagnostics: &[Diagnostic],
    suppressions: &suppression::Suppressions,
    active: &HashSet<&'static str>,
) -> Vec<Diagnostic> {
    let unused: Vec<&suppression::NamedId> = suppressions
        .named_ids()
        .iter()
        .filter(|named| {
            active.contains(named.id.as_str())
                && !diagnostics.iter().any(|d| {
                    d.rule_id == named.id
                        && suppressions.suppresses_from(named.line, d.line, &named.id)
                })
        })
        .collect();
    unused
        .iter()
        .map(|named| {
            let same_directive = unused.iter().filter(|o| o.comment == named.comment).count();
            Diagnostic {
                rule_id: "MSNU",
                message: MSNU_MESSAGE.to_string(),
                severity: Severity::Info,
                byte_range: named.byte_range.clone(),
                line: named.line,
                column: named.column,
                fix: msnu_fix(named, same_directive),
            }
        })
        .collect()
}

/// The MSNU fix for one unused ID of a `%#ok<...>` directive: MATLAB's own
/// edit, probed on every shape below. Comment text only, so the code's
/// behaviour cannot change; what it can change is which messages show (the
/// directive was suppressing nothing).
///
/// * One ID and nothing after the `>`: the whole comment goes (`x = 1; %#ok<NOPRT>`
///   becomes `x = 1; `).
/// * One ID and a remark after it: only `#ok<ID>` goes, the `%` and the remark
///   stay (`%#ok<NOPRT> note` becomes `% note`).
/// * Several IDs: the ID goes with the comma after it, or with the comma in
///   front when it is the last one (`<NOPRT,NASGU>` and `<NASGU,NOPRT>` both
///   become `<NASGU>`; white space around the commas is left).
///
/// No fix when every ID of a multi-ID directive is unused (MATLAB's edit
/// would leave `%#ok<>`; unprobed) or when the list is not well formed.
fn msnu_fix(named: &suppression::NamedId, unused_in_directive: usize) -> Option<Fix> {
    let end = named.list_end?;
    if named.parts == 1 {
        let range = if end == named.comment.end {
            named.comment.clone()
        } else {
            named.comment.start + 1..end
        };
        return Some(Fix::new(range, ""));
    }
    if unused_in_directive >= named.parts {
        return None;
    }
    let range = if named.index + 1 < named.parts {
        // The comma after the ID.
        named.part.start..named.part.end + 1
    } else {
        // The comma in front of the last ID.
        named.part.start - 1..named.part.end
    };
    Some(Fix::new(range, ""))
}

/// Message of MATLAB's MSNE: a `%#ok<ID>` directive naming an ID MATLAB does not know.
const MSNE_MESSAGE: &str = "No Code Analyzer check is found for this check ID.";

/// MSNE diagnostics for the IDs named by `suppressions` that are not check IDs.
fn unknown_suppressions(suppressions: &suppression::Suppressions) -> Vec<Diagnostic> {
    suppressions
        .named_ids()
        .iter()
        .filter(|named| !crate::known_ids::is_known_check_id(&named.id))
        .map(|named| Diagnostic {
            rule_id: "MSNE",
            message: MSNE_MESSAGE.to_string(),
            severity: Severity::Info,
            byte_range: named.byte_range.clone(),
            line: named.line,
            column: named.column,
            fix: None,
        })
        .collect()
}

/// MSNU diagnostics for the bare `%#ok` directives that suppressed nothing: no
/// message of `diagnostics` falls on the lines the directive covers.
fn unused_bare_suppressions(
    diagnostics: &[Diagnostic],
    suppressions: &suppression::Suppressions,
) -> Vec<Diagnostic> {
    suppressions
        .bare_directives()
        .iter()
        .filter(|bare| {
            !diagnostics
                .iter()
                .any(|d| suppressions.covers(bare.line, d.line))
        })
        .map(|bare| Diagnostic {
            rule_id: "MSNU",
            message: MSNU_MESSAGE.to_string(),
            severity: Severity::Info,
            byte_range: bare.byte_range.clone(),
            line: bare.line,
            column: bare.column,
            // MATLAB deletes the whole comment of a bare `%#ok` (probed:
            // `y = 2 %#ok` becomes `y = 2 `); one with a remark is left alone.
            fix: (bare.comment.len() == "%#ok".len()).then(|| Fix::new(bare.comment.clone(), "")),
        })
        .collect()
}

/// Parse errors MATLAB keeps before it reports TMSMS (the message itself comes from
/// the incomplete-analysis engine). Probed: 1,100 and 1,500 erroneous statements
/// both give 1,000 SYNER and one TMSMS.
const MATLAB_MAX_PARSE_ERRORS: usize = 1_000;

/// Keeps the first `MATLAB_MAX_PARSE_ERRORS` SYNER messages in source order and drops
/// the rest, as MATLAB does once its parse-error budget is spent.
fn limit_parse_errors(diagnostics: &mut Vec<Diagnostic>) {
    let syner = diagnostics.iter().filter(|d| d.rule_id == "SYNER").count();
    if syner <= MATLAB_MAX_PARSE_ERRORS {
        return;
    }
    diagnostics.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));
    let mut kept = 0;
    diagnostics.retain(|d| {
        if d.rule_id != "SYNER" {
            return true;
        }
        kept += 1;
        kept <= MATLAB_MAX_PARSE_ERRORS
    });
}

/// The budget units one message uses against MATLAB's message limit. NOPTS,
/// FNDEF and INUSD count three (probed: 3,334 NOPTS messages are kept, the 3,335th
/// is cut and TMMSG added; a function file with 10,500 unused assignments keeps
/// 9,997 of them beside FNDEF). NASGU, UNRCH, AGROW, ALIGN, COMPNOT and COMPNOP
/// count one, as do the IDs not probed.
fn message_units(rule_id: &str) -> usize {
    match rule_id {
        "NOPTS" | "FNDEF" | "INUSD" => 3,
        _ => 1,
    }
}

/// Keep the messages in source order while the units used so far are below
/// `MATLAB_MAX_MESSAGES` and add TMMSG when some are dropped, as MATLAB does (the
/// messages past the limit are deleted). The message that crosses the limit is kept.
fn limit_messages(diagnostics: &mut Vec<Diagnostic>, source: &str) {
    let units: usize = diagnostics.iter().map(|d| message_units(d.rule_id)).sum();
    if units <= MATLAB_MAX_MESSAGES {
        return;
    }
    diagnostics.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));
    let mut used = 0;
    let mut kept = 0;
    for diagnostic in diagnostics.iter() {
        if used >= MATLAB_MAX_MESSAGES {
            break;
        }
        used += message_units(diagnostic.rule_id);
        kept += 1;
    }
    diagnostics.truncate(kept);
    diagnostics.push(Diagnostic {
        rule_id: "TMMSG",
        message:
            "More than 10,000 Code Analyzer messages were generated, leading to some being deleted."
                .to_string(),
        severity: Severity::Error,
        byte_range: 0..source.len().min(1),
        line: 1,
        column: 1,
        fix: None,
    });
}

/// Depth of the deepest node of `tree` (the root is level 0). Iterative, so
/// measuring a pathological tree cannot overflow the stack either.
fn tree_depth(tree: &tree_sitter::Tree) -> usize {
    let mut cursor = tree.walk();
    let (mut depth, mut deepest) = (0usize, 0usize);
    loop {
        if cursor.goto_first_child() {
            depth += 1;
            deepest = deepest.max(depth);
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return deepest;
            }
            depth -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Category, Config, Rule};
    use std::path::Path;

    /// A test-only rule that panics during `check_file`, to exercise the QUIT guard.
    struct PanicRule;

    impl Rule for PanicRule {
        fn id(&self) -> &'static str {
            "PANIC_TEST_QUIT"
        }

        fn description(&self) -> &'static str {
            "test-only panicking rule"
        }

        fn severity(&self) -> Severity {
            Severity::Error
        }

        fn category(&self) -> Category {
            Category::Bugs
        }

        fn target_node_types(&self) -> &'static [&'static str] {
            &["identifier"]
        }

        fn has_file_check(&self) -> bool {
            true
        }

        fn check_file(&self, _ctx: &FileContext) -> Vec<Diagnostic> {
            panic!("intentional test panic");
        }
    }

    #[test]
    fn quit_fires_when_a_rule_panics() {
        let registry = RuleRegistry::new(vec![Box::new(PanicRule)], &Config::default());
        let mut linter = Linter::new(registry);
        let diags = linter
            .lint("x = 1;\n", Path::new("test.m"))
            .expect("lint should not propagate the panic");
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].rule_id, "QUIT");
        assert_eq!(diags[0].severity, Severity::Error);
        assert_eq!(
            diags[0].message,
            "Code analysis did not complete. Code Analyzer encountered an error."
        );
        assert_eq!(diags[0].line, 1);
        assert_eq!(diags[0].column, 1);
    }

    #[test]
    fn quit_does_not_fire_on_clean_file() {
        let registry = RuleRegistry::new(Vec::new(), &Config::default());
        let mut linter = Linter::new(registry);
        let diags = linter
            .lint("x = 1;\n", Path::new("test.m"))
            .expect("lint should succeed");
        assert!(!diags.iter().any(|d| d.rule_id == "QUIT"), "got: {diags:?}");
    }

    /// Test-only rule that emits diagnostics with different rule ids depending on
    /// the identifier text: `aa` → "RULEA", `bb` → "RULEB".
    ///
    /// Lets suppression tests exercise per-rule matching on the same line.
    struct MultiRule;

    impl Rule for MultiRule {
        fn id(&self) -> &'static str {
            "MULTI"
        }

        fn description(&self) -> &'static str {
            "test-only multi-id rule"
        }

        fn severity(&self) -> Severity {
            Severity::Warning
        }

        fn category(&self) -> Category {
            Category::Bugs
        }

        fn target_node_types(&self) -> &'static [&'static str] {
            &["identifier"]
        }

        fn active_check_ids(&self) -> Vec<&'static str> {
            vec!["RULEA", "RULEB"]
        }

        fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
            let node = ctx.node;
            let text = &ctx.source[node.byte_range()];
            let rule_id = match text {
                "aa" => "RULEA",
                "bb" => "RULEB",
                _ => return Vec::new(),
            };
            let (line, column) = line_col_of(ctx.source, node.start_byte());
            vec![Diagnostic {
                rule_id,
                message: format!("{rule_id} fired"),
                severity: Severity::Warning,
                byte_range: node.byte_range(),
                line,
                column,
                fix: None,
            }]
        }
    }

    /// Compute the 1-indexed (line, column) of a byte offset in `source`.
    fn line_col_of(source: &str, byte: usize) -> (usize, usize) {
        let prefix = &source[..byte];
        let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
        let line_start = prefix.rfind('\n').map_or(0, |p| p + 1);
        (line, byte - line_start + 1)
    }

    /// Lint with the test rule, without MSNE: the test rule's IDs (`RULEA`,
    /// `RULEB`) are not MATLAB's, so a directive naming them is an MSNE. MSNE is
    /// tested through [`lint_unfiltered`].
    fn lint_with(source: &str) -> Vec<Diagnostic> {
        lint_unfiltered(source)
            .into_iter()
            .filter(|d| d.rule_id != "MSNE")
            .collect()
    }

    fn lint_unfiltered(source: &str) -> Vec<Diagnostic> {
        let registry = RuleRegistry::new(vec![Box::new(MultiRule)], &Config::default());
        let mut linter = Linter::new(registry);
        linter
            .lint(source, Path::new("test.m"))
            .expect("lint should succeed")
    }

    #[test]
    fn inline_suppression_single_rule_on_line() {
        let diags = lint_with("aa = 1; %#ok<RULEA>\nbb = 2;\n");
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].rule_id, "RULEB");
        assert_eq!(diags[0].line, 2);
    }

    #[test]
    fn inline_suppression_star_all_suppresses_nothing() {
        // Probe: `%#ok<*all*>` and `%#ok<*>` silence no message in MATLAB.
        let diags = lint_with("aa = 1; bb = 2; %#ok<*all*>\n");
        assert_eq!(diags.len(), 2, "got: {diags:?}");
        let diags = lint_with("aa = 1; bb = 2; %#ok<*>\n");
        assert_eq!(diags.len(), 2, "got: {diags:?}");
    }

    #[test]
    fn inline_suppression_wildcard_is_file_wide() {
        // Probe: `%#ok<*RULEA>` on line 2 silences RULEA on lines 1 and 3 too.
        let diags = lint_with("aa = 1;\nbb = 2 %#ok<*RULEA>\naa = 3;\n");
        assert!(diags.iter().all(|d| d.rule_id != "RULEA"), "got: {diags:?}");
        assert!(diags.iter().any(|d| d.rule_id == "RULEB" && d.line == 2));
        assert!(!diags.iter().any(|d| d.rule_id == "MSNU"), "got: {diags:?}");
    }

    #[test]
    fn wildcard_claims_a_message_before_a_named_directive_on_its_line() {
        // Probe: `%#ok<*RULEB>` on line 3 claims the RULEB of line 1, so the named
        // `%#ok<RULEB>` there is unused (MSNU).
        let diags = lint_with("aa = 1; bb = 2; %#ok<RULEB>\naa = 3;\nbb = 4; %#ok<*RULEB>\n");
        assert!(diags.iter().all(|d| d.rule_id != "RULEB"), "got: {diags:?}");
        assert_eq!(
            diags
                .iter()
                .filter(|d| d.rule_id == "MSNU")
                .map(|d| d.line)
                .collect::<Vec<_>>(),
            vec![1]
        );
    }

    #[test]
    fn wildcard_silences_msnu() {
        // Probe: `%#ok<*MSNU>` silences the MSNU of `x = 1; %#ok<NOPTS>`.
        let diags = lint_with("cc = 3; %#ok<RULEA>\naa = 1; %#ok<*MSNU>\n");
        assert!(msnu_of(&diags).is_empty(), "got: {diags:?}");
    }

    #[test]
    fn inline_suppression_comma_separated_ids() {
        let diags = lint_with("aa = 1; bb = 2; %#ok<RULEA,RULEB>\n");
        assert!(diags.is_empty(), "got: {diags:?}");
    }

    #[test]
    fn inline_suppression_wildcard_matching() {
        // IDs are exact: `*A` names no ID, so nothing is suppressed.
        let diags = lint_with("aa = 1; bb = 2; %#ok<*A>\n");
        assert_eq!(diags.len(), 2, "got: {diags:?}");
        let diags = lint_with("aa = 1; bb = 2; %#ok<*RULEA>\n");
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].rule_id, "RULEB");
    }

    #[test]
    fn inline_suppression_non_matching_rule_on_same_line_not_suppressed() {
        let diags = lint_with("aa = 1; bb = 2; %#ok<RULEA>\n");
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].rule_id, "RULEB");
        assert_eq!(diags[0].line, 1);
    }

    #[test]
    fn inline_suppression_directive_on_other_line_does_not_suppress() {
        let diags = lint_with("aa = 1;\n%#ok<RULEA>\nbb = 2;\n");
        assert_eq!(diags.len(), 3, "got: {diags:?}");
        assert!(diags.iter().any(|d| d.rule_id == "RULEA"));
        assert!(diags.iter().any(|d| d.rule_id == "RULEB"));
        // The directive on line 2 names RULEA, which reported nothing there.
        assert!(diags.iter().any(|d| d.rule_id == "MSNU" && d.line == 2));
    }

    #[test]
    fn inline_suppression_enabled_by_default() {
        // Linter::new defaults inline suppression to on; a directive suppresses
        // without any explicit config/setter call.
        let diags = lint_with("aa = 1; %#ok<RULEA>\n");
        assert!(diags.is_empty(), "got: {diags:?}");
    }

    #[test]
    fn inline_suppression_disabled_via_config() {
        let config = Config::from_toml("[lint]\ninline_suppression = false\n").unwrap();
        let registry = RuleRegistry::new(vec![Box::new(MultiRule)], &config);
        let mut linter = Linter::new(registry);
        linter.set_inline_suppression(config.inline_suppression);
        let diags = linter
            .lint("aa = 1; %#ok<RULEA>\nbb = 2;\n", Path::new("test.m"))
            .expect("lint should succeed");
        assert_eq!(diags.len(), 2, "got: {diags:?}");
        assert!(diags.iter().any(|d| d.rule_id == "RULEA"));
        assert!(diags.iter().any(|d| d.rule_id == "RULEB"));
    }

    // -- MSNU: a directive naming an active check that reported nothing ----------

    fn msnu_of(diags: &[Diagnostic]) -> Vec<(usize, usize)> {
        diags
            .iter()
            .filter(|d| d.rule_id == "MSNU")
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn unused_single_id_is_reported_at_the_percent_sign() {
        // Probe: `y = x + 1; %#ok<PFBNS>` gives MSNU at column 12, the `%`.
        let diags = lint_with("bb = 2; %#ok<RULEA>\n");
        assert_eq!(msnu_of(&diags), vec![(1, 9)]);
        let msnu = diags.iter().find(|d| d.rule_id == "MSNU").unwrap();
        assert_eq!(msnu.severity, Severity::Info);
        assert_eq!(
            msnu.message,
            "A Code Analyzer message was once suppressed here, but the message is no longer generated."
        );
    }

    #[test]
    fn unused_ids_of_a_several_id_directive_are_reported_at_their_own_columns() {
        // Probe: `z = x; %#ok<AGROW,NASGU>` gives MSNU at column 13 (AGROW's own column).
        // Here RULEA (column 14) and RULEB (column 20) each report nothing on the line.
        let diags = lint_with("cc = 3; %#ok<RULEA,RULEB>\n");
        assert_eq!(msnu_of(&diags), vec![(1, 14), (1, 20)]);
    }

    #[test]
    fn a_fired_and_suppressed_id_is_used() {
        // RULEA fires on the line and is suppressed: the directive is in use.
        let diags = lint_with("aa = 1; %#ok<RULEA>\n");
        assert!(msnu_of(&diags).is_empty(), "got: {diags:?}");
    }

    #[test]
    fn only_the_unused_id_of_a_directive_is_reported() {
        // RULEB fires and is suppressed; RULEA (column 14) reports nothing.
        let diags = lint_with("bb = 2; %#ok<RULEA,RULEB>\n");
        assert_eq!(msnu_of(&diags), vec![(1, 14)]);
        assert!(
            !diags.iter().any(|d| d.rule_id == "RULEB"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn an_id_the_linter_does_not_track_is_never_reported() {
        // NOSUCHID is not an active check, so no MSNU (MSNE is not emitted).
        let diags = lint_with("cc = 3; %#ok<NOSUCHID>\n");
        assert!(msnu_of(&diags).is_empty(), "got: {diags:?}");
    }

    #[test]
    fn wildcard_directives_are_never_reported() {
        let diags = lint_with("cc = 3; %#ok<*RULE*>\n");
        assert!(msnu_of(&diags).is_empty(), "got: {diags:?}");
    }

    #[test]
    fn bare_directive_with_nothing_to_suppress_is_msnu() {
        // Probed: a bare `%#ok` on a line with no message gets MSNU at the `%`
        // (`if x, end %#ok` gives column 11 in MATLAB), also on its own line.
        let diags = lint_unfiltered("cc = 3; %#ok\n");
        assert_eq!(msnu_of(&diags), vec![(1, 9)], "got: {diags:?}");
        let diags = lint_unfiltered("aa = 1;\n%#ok\n");
        assert_eq!(msnu_of(&diags), vec![(2, 1)], "got: {diags:?}");
    }

    #[test]
    fn bare_directive_that_suppresses_a_message_is_not_msnu() {
        // `aa` fires RULEA on line 1, which the bare directive covers.
        let diags = lint_unfiltered("aa = 1; %#ok\n");
        assert!(msnu_of(&diags).is_empty(), "got: {diags:?}");
        assert!(diags.iter().all(|d| d.rule_id != "RULEA"), "got: {diags:?}");
    }

    #[test]
    fn msne_for_an_id_that_is_not_a_check() {
        // Probed: `%#ok<NOSUCHID>` gives MSNE, not MSNU; the column is the `%`.
        let diags = lint_unfiltered("x = 1; %#ok<NOSUCHID>\n");
        let msne: Vec<_> = diags.iter().filter(|d| d.rule_id == "MSNE").collect();
        assert_eq!(msne.len(), 1, "got: {diags:?}");
        assert_eq!((msne[0].line, msne[0].column), (1, 8));
        assert_eq!(msne[0].severity, Severity::Info);
        assert!(msnu_of(&diags).is_empty(), "got: {diags:?}");
    }

    #[test]
    fn msne_follows_exact_ids_and_lists() {
        // Probed: lowercase IDs and IDs with spaces are unknown (the directive
        // suppresses nothing); an unknown element of a list gets MSNE alone.
        for source in [
            "x = 1; %#ok<nasgu>\n",
            "x = 1; %#ok< NASGU >\n",
            "x = 1; %#ok<NASGU,NOSUCHID>\n",
        ] {
            let diags = lint_unfiltered(source);
            assert_eq!(
                diags.iter().filter(|d| d.rule_id == "MSNE").count(),
                1,
                "{source}: got {diags:?}"
            );
        }
        // A known ID, an empty list and a wildcard give no MSNE.
        for source in [
            "x = 1; %#ok<NASGU>\n",
            "x = 1; %#ok<>\n",
            "x = 1; %#ok<*NASGU>\n",
        ] {
            let diags = lint_unfiltered(source);
            assert!(
                diags.iter().all(|d| d.rule_id != "MSNE"),
                "{source}: got {diags:?}"
            );
        }
    }

    #[test]
    fn msnu_is_not_suppressed_by_a_directive_that_matches_it() {
        // Probe: `disp(x); %#ok<AGROW,MSNU>` reports MSNU for AGROW and for MSNU
        // itself, so a directive does not hide an MSNU.
        let diags = lint_with("cc = 3; %#ok<RULEA,MSNU>\n");
        assert_eq!(msnu_of(&diags), vec![(1, 14)]);
    }

    #[test]
    fn no_msnu_when_suppression_is_disabled() {
        let registry = RuleRegistry::new(vec![Box::new(MultiRule)], &Config::default());
        let mut linter = Linter::new(registry);
        linter.set_inline_suppression(false);
        let diags = linter
            .lint("bb = 2; %#ok<RULEA>\n", Path::new("test.m"))
            .expect("lint should succeed");
        assert!(msnu_of(&diags).is_empty(), "got: {diags:?}");
    }

    #[test]
    fn no_msnu_when_the_analysis_did_not_complete() {
        // A panicking rule makes the file QUIT: nothing can be said about what was generated.
        let registry = RuleRegistry::new(
            vec![Box::new(PanicRule), Box::new(MultiRule)],
            &Config::default(),
        );
        let mut linter = Linter::new(registry);
        let diags = linter
            .lint("bb = 2; %#ok<RULEA>\n", Path::new("test.m"))
            .expect("lint should succeed");
        assert!(diags.iter().any(|d| d.rule_id == "QUIT"), "got: {diags:?}");
        assert!(msnu_of(&diags).is_empty(), "got: {diags:?}");
    }

    #[test]
    fn a_directive_in_a_string_or_after_a_continuation_reports_nothing() {
        // Neither is a directive (probes q3 and q1), so RULEA is not named.
        assert!(msnu_of(&lint_with("disp('%#ok<RULEA>'); bb = 2;\n")).is_empty());
        assert!(msnu_of(&lint_with("bb = [1, ... %#ok<RULEA>\n 2];\n")).is_empty());
    }

    // -- depth limit -----------------------------------------------------------

    fn empty_linter() -> Linter {
        Linter::new(RuleRegistry::new(Vec::new(), &Config::default()))
    }

    fn nested_ifs(depth: usize) -> String {
        format!(
            "{}y = 1;\n{}",
            "if x\n".repeat(depth),
            "end\n".repeat(depth)
        )
    }

    #[test]
    fn code_nested_beyond_the_limit_is_reported_not_analyzed() {
        let mut linter = empty_linter();
        let diags = linter
            .lint(&nested_ifs(2_000), Path::new("deep.m"))
            .unwrap();
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].rule_id, "MALT_TOO_DEEP");
        assert!(
            diags[0].message.contains("nested too deeply"),
            "{}",
            diags[0].message
        );
        assert!(diags[0].message.contains("not a MATLAB message"));
    }

    #[test]
    fn ordinary_nesting_is_analyzed() {
        let mut linter = empty_linter();
        let diags = linter.lint(&nested_ifs(50), Path::new("ok.m")).unwrap();
        assert!(diags.is_empty(), "got: {diags:?}");
    }

    #[test]
    fn the_limit_can_be_lowered_and_raised() {
        let source = nested_ifs(20);
        let mut linter = empty_linter();
        linter.set_max_tree_depth(10);
        assert_eq!(
            linter.lint(&source, Path::new("a.m")).unwrap()[0].rule_id,
            "MALT_TOO_DEEP"
        );
        linter.set_max_tree_depth(DEFAULT_MAX_TREE_DEPTH);
        assert!(linter.lint(&source, Path::new("a.m")).unwrap().is_empty());
    }

    /// A test-only rule that reports `count` messages for one file.
    struct ManyRule {
        count: usize,
    }

    impl Rule for ManyRule {
        fn id(&self) -> &'static str {
            "MANY_TEST"
        }

        fn description(&self) -> &'static str {
            "test-only rule with many messages"
        }

        fn severity(&self) -> Severity {
            Severity::Warning
        }

        fn category(&self) -> Category {
            Category::Bugs
        }

        fn target_node_types(&self) -> &'static [&'static str] {
            &[]
        }

        fn has_file_check(&self) -> bool {
            true
        }

        fn check_file(&self, _ctx: &FileContext) -> Vec<Diagnostic> {
            (0..self.count)
                .map(|i| Diagnostic {
                    rule_id: "MANY_TEST",
                    message: "many".to_string(),
                    severity: Severity::Warning,
                    byte_range: 0..1,
                    line: i + 1,
                    column: 1,
                    fix: None,
                })
                .collect()
        }
    }

    fn lint_many(count: usize) -> Vec<Diagnostic> {
        let registry = RuleRegistry::new(vec![Box::new(ManyRule { count })], &Config::default());
        Linter::new(registry)
            .lint("x = 1;\n", Path::new("many.m"))
            .unwrap()
    }

    #[test]
    fn messages_past_the_matlab_limit_are_dropped_and_tmmsg_added() {
        let diags = lint_many(MATLAB_MAX_MESSAGES + 1);
        let kept = diags.iter().filter(|d| d.rule_id == "MANY_TEST").count();
        assert_eq!(kept, MATLAB_MAX_MESSAGES, "got {} diagnostics", diags.len());
        assert_eq!(diags.iter().filter(|d| d.rule_id == "TMMSG").count(), 1);
    }

    #[test]
    fn exactly_the_matlab_limit_is_kept_without_tmmsg() {
        let diags = lint_many(MATLAB_MAX_MESSAGES);
        assert_eq!(diags.len(), MATLAB_MAX_MESSAGES);
        assert!(diags.iter().all(|d| d.rule_id != "TMMSG"));
    }
}

#[cfg(test)]
mod statement_error_tests {
    use super::*;

    #[test]
    fn syner_past_the_parse_error_budget_is_dropped() {
        // MATLAB probe: 1,100 and 1,500 erroneous statements give 1,000 SYNER and TMSMS.
        let mut diags: Vec<Diagnostic> = (0..1500).map(|i| diag("SYNER", i)).collect();
        limit_parse_errors(&mut diags);
        assert_eq!(diags.len(), 1000);
        assert_eq!(diags.last().unwrap().byte_range.start, 999);

        let mut diags: Vec<Diagnostic> = (0..1000).map(|i| diag("SYNER", i)).collect();
        limit_parse_errors(&mut diags);
        assert_eq!(diags.len(), 1000);
    }

    #[test]
    fn nopts_counts_three_units_towards_the_matlab_limit() {
        // MATLAB probe: 3,400 NOPTS messages keep 3,334 and add TMMSG; 3,000 are all kept.
        let mut diags: Vec<Diagnostic> = (0..3400).map(|i| diag("NOPTS", i)).collect();
        limit_messages(&mut diags, "");
        assert_eq!(diags.iter().filter(|d| d.rule_id == "NOPTS").count(), 3334);
        assert_eq!(diags.last().unwrap().rule_id, "TMMSG");

        let mut diags: Vec<Diagnostic> = (0..3000).map(|i| diag("NOPTS", i)).collect();
        limit_messages(&mut diags, "");
        assert_eq!(diags.len(), 3000);
        assert!(diags.iter().all(|d| d.rule_id == "NOPTS"));
    }

    #[test]
    fn fndef_and_inusd_count_three_units_towards_the_matlab_limit() {
        // MATLAB probe: a function file with 10,500 unused assignments keeps 9,997 of
        // them beside FNDEF (3 units): 3 + 9,997 = 10,000. Also INUSD (3 units).
        let mut diags = vec![diag("FNDEF", 0)];
        diags.extend((1..=10_500).map(|i| diag("NASGU", i)));
        limit_messages(&mut diags, "");
        assert_eq!(diags.iter().filter(|d| d.rule_id == "NASGU").count(), 9997);
        assert_eq!(diags.last().unwrap().rule_id, "TMMSG");

        let mut diags = vec![diag("INUSD", 0), diag("FNDEF", 0)];
        diags.extend((1..=9_994).map(|i| diag("NASGU", i)));
        diags.push(diag("NASGU", 9_995));
        limit_messages(&mut diags, "");
        assert_eq!(diags.iter().filter(|d| d.rule_id == "NASGU").count(), 9_994);
    }

    fn diag(rule_id: &'static str, start: usize) -> Diagnostic {
        Diagnostic {
            rule_id,
            message: String::new(),
            severity: Severity::Error,
            byte_range: start..start + 1,
            line: 1,
            column: 1,
            fix: None,
        }
    }

    fn remaining(source: &str, mut diagnostics: Vec<Diagnostic>) -> Vec<&'static str> {
        drop_statements_with_errors(&mut diagnostics, source);
        diagnostics.iter().map(|d| d.rule_id).collect()
    }

    #[test]
    fn statement_holding_a_parse_error_is_not_analysed() {
        // MATLAB probe on `x = 1 @ 2; w = 3`: SYNER at `@`, no NOPTS on `x = 1 @ 2`,
        // but NOPTS on `w = 3` after the `;`.
        let source = "x = 1 @ 2; w = 3\n";
        let diags = vec![diag("NOPTS", 0), diag("SYNER", 6), diag("NOPTS", 11)];
        assert_eq!(remaining(source, diags), ["SYNER", "NOPTS"]);
    }

    #[test]
    fn statement_after_a_comma_is_analysed() {
        // MATLAB probe on `if w = 3, u = 5, end`: SYNER at `=`, NOLHS-like findings
        // on the broken `if w = 3` are dropped, `u = 5` is still analysed.
        let source = "if w = 3, u = 5, end\n";
        let diags = vec![diag("NOLHS", 0), diag("SYNER", 5), diag("NASGU", 10)];
        assert_eq!(remaining(source, diags), ["SYNER", "NASGU"]);
    }

    #[test]
    fn formatting_and_structure_checks_survive_a_parse_error() {
        let source = "x = 1 @ 2;;\n";
        let diags = vec![diag("NOSEMI", 10), diag("ENDCT2", 0), diag("SYNER", 6)];
        assert_eq!(remaining(source, diags), ["NOSEMI", "ENDCT2", "SYNER"]);
    }

    #[test]
    fn next_line_is_not_affected() {
        let source = "x = 1 @ 2\ny = 2\n";
        let diags = vec![diag("NOPTS", 10), diag("SYNER", 6)];
        assert_eq!(remaining(source, diags), ["NOPTS", "SYNER"]);
    }

    #[test]
    fn syner_inside_a_malformed_number_is_the_same_error() {
        // MATLAB probe on `y = 0b102;`: BADHBB at the literal only, no SYNER at `2`.
        let source = "y = 0b102;\n";
        let mut bad = diag("BADHBB", 4);
        bad.byte_range = 4..9;
        let diags = vec![bad, diag("SYNER", 8)];
        assert_eq!(remaining(source, diags), ["BADHBB"]);
    }

    #[test]
    fn syner_later_in_a_statement_with_a_malformed_number_is_dropped() {
        // MATLAB probe on `x = 0xFG;`: BADHBH at `0x`, no SYNER at `G`.
        let source = "x = 0xFG;\n";
        let mut bad = diag("BADHBH", 4);
        bad.byte_range = 4..8;
        let diags = vec![bad, diag("SYNER", 7)];
        assert_eq!(remaining(source, diags), ["BADHBH"]);
    }

    #[test]
    fn syner_on_a_later_line_than_a_malformed_number_is_kept() {
        // The literal's statement ends at the newline: the next line is analysed.
        let source = "x = 0xFG;\nu = 1 @ 2;\n";
        let mut bad = diag("BADHBH", 4);
        bad.byte_range = 4..8;
        let diags = vec![bad, diag("SYNER", 17)];
        assert_eq!(remaining(source, diags), ["BADHBH", "SYNER"]);
    }

    #[test]
    fn statement_with_an_unterminated_string_is_not_analysed() {
        // MATLAB probe on `x = 'abc` then `y = 2;`: STRIN only, no NOPTS on `x`.
        let source = "x = 'abc\ny = 2;\n";
        let mut strin = diag("STRIN", 4);
        strin.byte_range = 4..8;
        let diags = vec![diag("NOPTS", 0), strin, diag("NOPTS", 9)];
        assert_eq!(remaining(source, diags), ["STRIN", "NOPTS"]);
    }

    #[test]
    fn without_a_statement_error_nothing_is_dropped() {
        let source = "x = 1 @ 2; w = 3\n";
        let diags = vec![diag("NOPTS", 0), diag("NASGU", 11)];
        assert_eq!(remaining(source, diags), ["NOPTS", "NASGU"]);
    }
}
