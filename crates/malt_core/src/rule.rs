use std::path::Path;

use tree_sitter::{Node, Tree};

use crate::diagnostic::{Diagnostic, Severity};

// ---------------------------------------------------------------------------
// Rule categories (maps to MATLAB Code Analyzer check groups)
// ---------------------------------------------------------------------------

/// Categories of lint rules, mapping to MATLAB Code Analyzer check groups.
///
/// Used for bulk enable/disable/severity configuration via `[lint.categories]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Internal linter limits and analysis failures.
    IncompleteAnalysis,
    /// Parser-level syntax validation.
    SyntaxErrors,
    /// Language specification constraint violations.
    LanguageSpecification,
    /// Likely bugs and logic errors.
    Bugs,
    /// Configurable code complexity/style metrics.
    CustomChecks,
    /// Naming convention enforcement.
    Naming,
    /// Deprecated/removed functions and APIs.
    Compatibility,
    /// Forward compatibility issues.
    ForwardCompatibility,
    /// Common best practices.
    GoodPractices,
    /// Variables that may not be defined before use.
    UnsetVariables,
    /// Dead code and unused constructions.
    UnusedConstructions,
    /// Suggestions for improved code patterns.
    SuggestedImprovements,
    /// Readability improvements.
    Readability,
    /// Code formatting suggestions.
    Formatting,
    /// Performance improvement suggestions.
    Performance,
    /// MATLAB Coder / code generation constraints.
    CodeGeneration,
    /// Fixed-point toolbox specific.
    FixedPoint,
    /// MATLAB Compiler deployment constraints.
    Deployment,
    /// System object validation.
    SystemObjects,
    /// Unsupported features.
    Unsupported,
    /// Behavior changes between MATLAB versions.
    BehaviorChanges,
    /// Configuration file validation.
    ConfigurationIssues,
    /// The MonkeyProof Coding Standard for MATLAB, a standard of its own and not part of
    /// MATLAB's Code Analyzer. Off unless the `monkeyproof` preset or the category is
    /// switched on.
    MonkeyProof,
}

impl Category {
    /// Returns the string identifier used in `.malt.toml` config files.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IncompleteAnalysis => "incomplete-analysis",
            Self::SyntaxErrors => "syntax-errors",
            Self::LanguageSpecification => "language-specification",
            Self::Bugs => "bugs",
            Self::CustomChecks => "custom-checks",
            Self::Naming => "naming",
            Self::Compatibility => "compatibility",
            Self::ForwardCompatibility => "forward-compatibility",
            Self::GoodPractices => "good-practices",
            Self::UnsetVariables => "unset-variables",
            Self::UnusedConstructions => "unused-constructions",
            Self::SuggestedImprovements => "suggested-improvements",
            Self::Readability => "readability",
            Self::Formatting => "formatting",
            Self::Performance => "performance",
            Self::CodeGeneration => "code-generation",
            Self::FixedPoint => "fixed-point",
            Self::Deployment => "deployment",
            Self::SystemObjects => "system-objects",
            Self::Unsupported => "unsupported",
            Self::BehaviorChanges => "behavior-changes",
            Self::ConfigurationIssues => "configuration-issues",
            Self::MonkeyProof => "monkeyproof",
        }
    }
}

impl std::str::FromStr for Category {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "incomplete-analysis" => Ok(Self::IncompleteAnalysis),
            "syntax-errors" => Ok(Self::SyntaxErrors),
            "language-specification" => Ok(Self::LanguageSpecification),
            "bugs" => Ok(Self::Bugs),
            "custom-checks" => Ok(Self::CustomChecks),
            "naming" => Ok(Self::Naming),
            "compatibility" => Ok(Self::Compatibility),
            "forward-compatibility" => Ok(Self::ForwardCompatibility),
            "good-practices" => Ok(Self::GoodPractices),
            "unset-variables" => Ok(Self::UnsetVariables),
            "unused-constructions" => Ok(Self::UnusedConstructions),
            "suggested-improvements" => Ok(Self::SuggestedImprovements),
            "readability" => Ok(Self::Readability),
            "formatting" => Ok(Self::Formatting),
            "performance" => Ok(Self::Performance),
            "code-generation" => Ok(Self::CodeGeneration),
            "fixed-point" => Ok(Self::FixedPoint),
            "deployment" => Ok(Self::Deployment),
            "system-objects" => Ok(Self::SystemObjects),
            "unsupported" => Ok(Self::Unsupported),
            "behavior-changes" => Ok(Self::BehaviorChanges),
            "configuration-issues" => Ok(Self::ConfigurationIssues),
            "monkeyproof" => Ok(Self::MonkeyProof),
            _ => Err(format!("unknown category: {s}")),
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Maturity
// ---------------------------------------------------------------------------

/// How far a check has been verified against MATLAB's Code Analyzer.
///
/// Metadata only: it does not change what a check reports. Experimental
/// checks are off by default and run with `[lint] experimental = true`, the
/// `all` preset, or an explicit rule setting. The classification lives in
/// `malt_rules::maturity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Maturity {
    /// Verified against MATLAB (recorded findings), or part of the default
    /// analysis that the conformance suite covers.
    #[default]
    Stable,
    /// Verified against the documentation only, never reported by MATLAB in
    /// the recorded fixtures, or of low reliability.
    Experimental,
}

impl Maturity {
    /// The word used in the docs and in the data file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Experimental => "experimental",
        }
    }
}

impl std::fmt::Display for Maturity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Context types
// ---------------------------------------------------------------------------

/// Context passed to [`Rule::check`] during the single-pass node traversal.
///
/// Provides the current node, the full source text (for extracting snippets),
/// and the file path. Since tree-sitter `Node` supports `.parent()`,
/// `.next_sibling()`, etc., rules can walk the tree locally as needed.
pub struct NodeContext<'a> {
    pub node: Node<'a>,
    pub source: &'a str,
    pub file_path: &'a Path,
    /// The ancestors of `node`, the root first, when the caller walked down to
    /// it and knows them (the linter does). `None` makes [`NodeContext::ancestors`]
    /// ask tree-sitter instead.
    pub ancestors: Option<&'a [Node<'a>]>,
}

impl<'a> NodeContext<'a> {
    /// The ancestors of the node, nearest (the parent) first.
    ///
    /// Prefer it to climbing with `Node::parent`, which walks down from the root
    /// for every step: as costly as the siblings of every ancestor.
    pub fn ancestors(&self) -> Box<dyn Iterator<Item = Node<'a>> + 'a> {
        match self.ancestors {
            Some(known) => Box::new(known.iter().rev().copied()),
            None => Box::new(std::iter::successors(self.node.parent(), |n| n.parent())),
        }
    }
}

/// What a host knows about the MATLAB search path beyond the file being linted.
///
/// MATLAB's Code Analyzer reads one file at a time, so by default no rule asks
/// for this. A host supplies an implementation only when the user turns the path
/// index on (`[index] enabled`), and then a rule may report what the other files
/// on the path show. Rules must give the same result as before without one.
pub trait ProjectIndex: Send + Sync {
    /// Whether the class `name`, named as a superclass by a classdef in the file
    /// `from`, is a handle class. `Some(true)`: `handle`, or a class that derives
    /// from it. `Some(false)`: a class whose whole hierarchy is known and has no
    /// `handle` in it. `None`: unknown, because the name or one of its bases is
    /// not on the path.
    fn is_handle_class(&self, name: &str, from: &Path) -> Option<bool>;
}

/// Context passed to [`Rule::check_file`] after the full traversal completes.
///
/// Provides the parsed tree (for rules that need their own traversal strategy),
/// the full source text, the file path, and the search path when the host
/// enabled it (see [`ProjectIndex`]).
pub struct FileContext<'a> {
    pub tree: &'a Tree,
    pub source: &'a str,
    pub file_path: &'a Path,
    /// The search path of the run, `None` unless the host enabled the path index.
    pub project: Option<&'a dyn ProjectIndex>,
}

// ---------------------------------------------------------------------------
// Rule trait
// ---------------------------------------------------------------------------

/// The core trait that all lint rules must implement.
///
/// Rules declare which tree-sitter node types they are interested in via
/// [`target_node_types`]. During the single-pass traversal, the engine
/// dispatches matching nodes to [`check`]. After traversal, [`check_file`]
/// is called on rules that override it for file-level analysis.
///
/// # Design Notes
///
/// - **Node-level rules**: Override `check()`, return `target_node_types()`.
/// - **File-level rules**: Override `check_file()`, return empty `target_node_types()`.
/// - **Hybrid rules**: Override both (unusual, but supported).
///
/// Rules must be `Send + Sync` to support future parallel file processing.
pub trait Rule: Send + Sync {
    /// Unique rule identifier matching MATLAB Code Analyzer check IDs
    /// (e.g., `"NOSEMI"`, `"AGROW"`, `"naming.class.casing"`).
    fn id(&self) -> &'static str;

    /// Human-readable one-line description of what this rule checks.
    fn description(&self) -> &'static str;

    /// Default severity for diagnostics produced by this rule.
    fn severity(&self) -> Severity;

    /// The category this rule belongs to (used for bulk config).
    fn category(&self) -> Category;

    /// Tree-sitter node type names this rule subscribes to.
    ///
    /// Return an empty slice for file-level-only rules (which use `check_file`).
    /// The engine uses this to build an index for O(1) dispatch during traversal.
    fn target_node_types(&self) -> &'static [&'static str];

    /// Whether this rule can be disabled by the user. Most rules return `true`.
    /// The 17 "Incomplete Analysis" checks cannot be disabled.
    fn can_be_disabled(&self) -> bool {
        true
    }

    /// Whether this rule is enabled by default (no config present).
    /// Most rules return `true`. Specialized engines (code generation,
    /// deployment, ...) return `false` so a default run stays focused.
    fn enabled_by_default(&self) -> bool {
        true
    }

    /// Whether this rule implements file-level checking.
    ///
    /// Override to return `true` if the rule implements [`check_file`].
    /// This allows the engine to skip calling `check_file()` on rules
    /// that only do node-level checking.
    fn has_file_check(&self) -> bool {
        false
    }

    /// The check IDs this rule reports under the current configuration: the
    /// IDs it is set up to report, minus those switched off.
    ///
    /// The linter uses them for MSNU (a `%#ok<ID>` directive whose check
    /// reported nothing on its line). The default declares none, so a rule that
    /// does not list its IDs never causes an MSNU.
    fn active_check_ids(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// Check a single node encountered during the single-pass traversal.
    ///
    /// Only called for nodes whose `kind()` is in [`target_node_types`].
    /// Default implementation returns no diagnostics.
    fn check(&self, _ctx: &NodeContext) -> Vec<Diagnostic> {
        Vec::new()
    }

    /// Check the file after the full traversal has completed.
    ///
    /// Called once per file for rules where [`has_file_check`] returns `true`.
    /// Use this for rules that require full-file context (e.g., unused variables,
    /// duplicate definitions). Default implementation returns no diagnostics.
    fn check_file(&self, _ctx: &FileContext) -> Vec<Diagnostic> {
        Vec::new()
    }

    /// MATLAB's own limits that stop the analysis of a file (NOSPC, MXASET), run
    /// before any other check on every tree. MATLAB then reports only these, so
    /// a file that reaches one is not analysed further. Default returns none.
    fn check_limits(&self, _tree: &tree_sitter::Tree, _source: &str) -> Vec<Diagnostic> {
        Vec::new()
    }
}
