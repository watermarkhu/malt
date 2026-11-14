//! # FORMATTING_ENGINE: Formatting Suggestion Checks
//!
//! ```malt
//! id = "FORMATTING_ENGINE"
//! title = "Formatting Suggestion Checks"
//! category = "formatting"
//! severity = "info"
//! fix = true
//! icon = "lucide/align-left"
//! slug = "formatting"
//! ```
//!
//! ## Rule
//!
//! Suggests formatting improvements for MATLAB code. All 6 checks share a
//! single `FormattingEngine` that runs as a file-level rule, walking the whole
//! tree once and applying every enabled check per node. Each diagnostic
//! carries the specific check ID (e.g. `NOCOMMA`, `NO4LP`). Individual checks
//! can be enabled, disabled, or reconfigured through their own
//! `[lint.rules.*]` tables.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | NOCOMMA | info | yes | Extra comma is unnecessary. |
//! | NO4LP | info | yes | Parentheses are not needed in a FOR statement. |
//! | ALIGN | info | yes | This keyword might not be aligned with its matching END on line VAR_NUMBER. |
//! | NOPTS | info | yes | Add a semicolon after the statement to hide the output (in a script). |
//! | NOPRT | info | yes | Add a semicolon after the statement to hide the output (in a function). |
//! | NCOMMA | info | yes | Best practice is to separate output variables with commas. |
//!
//! ## Fix
//!
//! Each check rewrites the flagged construct directly:
//!
//! - NOCOMMA removes a comma that repeats a separator (`x = 1;,` → `x = 1;`).
//! - NCOMMA inserts a comma between space-separated output variables (`[a b]` → `[a, b]`).
//! - NO4LP removes the parentheses around the `for` header (`for (i = 1:n)` → `for i = 1:n`).
//! - ALIGN re-indents the `end` to the column of its keyword (only when the keyword starts its line).
//! - NOPTS and NOPRT insert the missing semicolon after the statement (`x = 5` → `x = 5;`).
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! x = 1;,                % NOCOMMA: extra comma
//! [a b] = f();          % NCOMMA: separate outputs with commas
//! function g()
//!     y = compute()     % NOPRT: semicolon to hide output (in a function)
//! end
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! x = [1, 2];
//! [a, b] = f();
//! function g()
//!     y = compute();
//! end
//! ```
//!
//! ### Fixed
//!
//! ```diff
//! - x = [1, 2,];
//! + x = [1, 2];
//! - [a b] = f();
//! + [a, b] = f();
//! - y = compute()
//! + y = compute();
//! ```
//!
//! ## Configuration
//!
//! Each check is configured independently through its own rule table:
//!
//! ```toml
//! [lint.rules.NOCOMMA]
//! severity = "info"
//!
//! [lint.rules.NO4LP]
//! severity = "info"
//! ```

mod check_align;
mod check_ncomma;
mod check_no4lp;
mod check_nocomma;
mod check_noprt;
mod check_nopts;

use malt_core::{Category, Config, Diagnostic, FileContext, Fix, Rule, Severity};
use tree_sitter::Node;

use check_nopts::ScopeCache;

// ---------------------------------------------------------------------------
// Check IDs
// ---------------------------------------------------------------------------

/// All formatting check IDs handled by this engine.
pub(crate) const CHECK_IDS: &[&str] = &["NOCOMMA", "NO4LP", "ALIGN", "NOPTS", "NOPRT", "NCOMMA"];

// ---------------------------------------------------------------------------
// Per-check configuration
// ---------------------------------------------------------------------------

/// Parent node types that indicate statement-level context.
pub(crate) const STATEMENT_PARENTS: &[&str] = &["source_file", "block"];

// ---------------------------------------------------------------------------
// FormattingEngine rule
// ---------------------------------------------------------------------------

/// Tracks which individual checks are enabled.
struct EnabledChecks {
    nocomma: bool,
    no4lp: bool,
    align: bool,
    nopts: bool,
    noprt: bool,
    ncomma: bool,
}

/// The formatting suggestions engine that handles all 6 formatting checks.
///
/// Registered as `"FORMATTING_ENGINE"` with inventory. Uses file-level
/// analysis to walk the tree once and apply all enabled formatting checks.
pub struct FormattingEngine {
    enabled: EnabledChecks,
}

impl FormattingEngine {
    /// Factory constructor invoked by inventory.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let enabled = EnabledChecks {
            nocomma: config.is_rule_enabled("NOCOMMA"),
            no4lp: config.is_rule_enabled("NO4LP"),
            align: config.is_rule_enabled("ALIGN"),
            nopts: config.is_rule_enabled("NOPTS"),
            noprt: config.is_rule_enabled("NOPRT"),
            ncomma: config.is_rule_enabled("NCOMMA"),
        };
        Box::new(Self { enabled })
    }

    /// Returns true if any sub-check is enabled.
    fn any_enabled(&self) -> bool {
        self.enabled.nocomma
            || self.enabled.no4lp
            || self.enabled.align
            || self.enabled.nopts
            || self.enabled.noprt
            || self.enabled.ncomma
    }
}

impl Rule for FormattingEngine {
    fn id(&self) -> &'static str {
        "FORMATTING_ENGINE"
    }

    fn description(&self) -> &'static str {
        "Formatting suggestion checks (NOCOMMA, NO4LP, ALIGN, NOPTS, NOPRT, NCOMMA)"
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn category(&self) -> Category {
        Category::Formatting
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        // File-level only — no node subscriptions.
        &[]
    }

    fn can_be_disabled(&self) -> bool {
        true
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        if !self.any_enabled() {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let root = ctx.tree.root_node();
        let frame = Frame {
            statement_level: false,
            following: &[],
            scope: root,
            in_function: false,
            in_parfor: false,
        };
        let mut names = ScopeCache::new();
        self.visit_recursive(root, frame, ctx.source, &mut names, &mut diagnostics);
        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Recursive tree traversal
// ---------------------------------------------------------------------------

/// What the walk knows about the surroundings of the node it visits, so the
/// checks need not ask tree-sitter: `Node::parent` and `Node::next_sibling`
/// walk down from the root, which costs as much as the statements before the
/// node and makes a file of many statements quadratic.
#[derive(Clone, Copy)]
pub(crate) struct Frame<'t, 'a> {
    /// Whether the parent is a `source_file` or a `block` ([`STATEMENT_PARENTS`]).
    pub(crate) statement_level: bool,
    /// The siblings after the node, in order.
    pub(crate) following: &'a [Node<'t>],
    /// The nearest enclosing `function_definition`, else the root.
    pub(crate) scope: Node<'t>,
    /// Whether an ancestor is a `function_definition`.
    pub(crate) in_function: bool,
    /// Whether an ancestor is a `parfor` loop: a `for_statement` whose first
    /// token is `parfor`. Probed: MATLAB gives no NOPTS or NOPRT for statements there.
    pub(crate) in_parfor: bool,
}

impl FormattingEngine {
    /// Walk the tree recursively, applying enabled checks to each node.
    fn visit_recursive<'t>(
        &self,
        node: Node<'t>,
        frame: Frame<'t, '_>,
        source: &str,
        names: &mut ScopeCache,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let kind = node.kind();
        // Note: multiple checks can apply to the same node type, so dispatch
        // each independently rather than using mutually-exclusive match arms.

        // NOCOMMA: a comma that repeats a statement separator
        if kind == "," && self.enabled.nocomma {
            self.check_nocomma(node, source, diagnostics);
        }

        // NO4LP: parentheses around a `for` header
        if kind == "for_statement" && self.enabled.no4lp {
            self.check_no4lp(node, source, diagnostics);
        }

        // ALIGN: a block keyword whose matching END is indented differently
        if matches!(
            kind,
            "if_statement"
                | "for_statement"
                | "while_statement"
                | "switch_statement"
                | "try_statement"
        ) && self.enabled.align
        {
            self.check_align(node, source, diagnostics);
        }

        // NOPTS / NOPRT: a statement that produces output without a semicolon,
        // in a script or in a function.
        if (self.enabled.nopts || self.enabled.noprt)
            && (kind == "assignment"
                || check_nopts::is_expression_statement(node, &frame, names, source))
        {
            if self.enabled.nopts {
                self.check_nopts(node, &frame, source, diagnostics);
            }
            if self.enabled.noprt {
                self.check_noprt(node, &frame, source, diagnostics);
            }
        }

        // NCOMMA: output variables not separated by commas
        if kind == "multioutput_variable" && self.enabled.ncomma {
            self.check_ncomma(node, source, diagnostics);
        }

        // Recurse into children.
        let children = crate::analysis::symbols::children_of(node);
        let statement_level = STATEMENT_PARENTS.contains(&kind);
        let scope = if kind == "function_definition" {
            node
        } else {
            frame.scope
        };
        let in_function = frame.in_function || kind == "function_definition";
        let in_parfor = frame.in_parfor
            || (kind == "for_statement" && node.child(0).map(|c| c.kind()) == Some("parfor"));
        for (i, &child) in children.iter().enumerate() {
            let child_frame = Frame {
                statement_level,
                following: &children[i + 1..],
                scope,
                in_function,
                in_parfor,
            };
            self.visit_recursive(child, child_frame, source, names, diagnostics);
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Get the byte offset of the start of the line containing `byte_offset`.
pub(crate) fn line_start_byte(source: &str, byte_offset: usize) -> usize {
    source[..byte_offset]
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0)
}

/// Determine whether a statement node is followed by a semicolon.
///
/// In tree-sitter-matlab's grammar, semicolons (`;`) and newlines are statement
/// terminators that appear as sibling nodes after the statement. We check the
/// source text immediately following the node for a `;` (skipping whitespace
/// on the same line).
pub(crate) fn has_trailing_semicolon(node: Node, source: &str) -> bool {
    let following = std::iter::successors(node.next_sibling(), |sib| sib.next_sibling());
    semicolon_follows(node, following, source)
}

/// [`has_trailing_semicolon`] given the siblings after `node`, in order.
pub(crate) fn semicolon_follows<'t>(
    node: Node<'t>,
    following: impl IntoIterator<Item = Node<'t>>,
    source: &str,
) -> bool {
    // Strategy 1: Check next siblings for ";".
    for sib in following {
        let kind = sib.kind();
        if kind == "comment" || kind == "line_continuation" {
            continue;
        }
        if kind == ";" {
            return true;
        }
        break;
    }

    // Strategy 2: Fallback — scan the source text after the node end.
    let after = &source[node.end_byte()..];
    for ch in after.chars() {
        match ch {
            ';' => return true,
            '\n' | '\r' => return false,
            ' ' | '\t' => continue,
            _ => return false,
        }
    }

    false
}

// ---------------------------------------------------------------------------
// Check ID validation (used in tests)
// ---------------------------------------------------------------------------

/// Returns all check IDs managed by this engine.
#[allow(dead_code)]
pub fn check_ids() -> &'static [&'static str] {
    CHECK_IDS
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "FORMATTING_ENGINE",
    FormattingEngine::from_config
));

// ---------------------------------------------------------------------------
// Shared test helpers
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use malt_core::Config;
    use tree_sitter::Parser;

    /// Parse MATLAB source and return the tree.
    pub(super) fn parse(source: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_matlab::LANGUAGE.into())
            .expect("failed to load tree-sitter-matlab");
        malt_core::note_parse();
        parser.parse(source, None).expect("parse failed")
    }

    /// Run the formatting engine on source code and return diagnostics.
    pub(super) fn lint(source: &str) -> Vec<Diagnostic> {
        let config = Config::default();
        let rule = FormattingEngine::from_config(&config);
        let tree = parse(source);
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: std::path::Path::new("test.m"),
        };
        rule.check_file(&ctx)
    }

    /// Returns `true` if any diagnostic carries the given rule ID.
    pub(super) fn has_id(diags: &[Diagnostic], id: &str) -> bool {
        diags.iter().any(|d| d.rule_id == id)
    }
}
