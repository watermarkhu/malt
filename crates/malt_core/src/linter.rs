use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use tree_sitter::Parser;

use crate::diagnostic::Diagnostic;
use crate::registry::RuleRegistry;
use crate::rule::{FileContext, NodeContext};
use crate::suppression;
use crate::Severity;

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
/// Diagnostics emitted by rules carry the rule's default severity. The linter
/// stamps the effective severity from the registry (which resolves config
/// overrides) onto each diagnostic before returning.
///
/// # Inline Suppression
///
/// When [`Linter::inline_suppression`] is enabled (the default), diagnostics
/// on lines carrying a `%#ok<...>` directive comment are filtered out, mirroring
/// MATLAB's inline suppression.
pub struct Linter {
    registry: RuleRegistry,
    parser: Parser,
    inline_suppression: bool,
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
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_matlab::LANGUAGE.into())
            .expect("failed to load tree-sitter-matlab grammar");

        Self {
            registry,
            parser,
            inline_suppression: true,
        }
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
        // Step 1: Parse
        let tree = self
            .parser
            .parse(source, None)
            .ok_or(LintError::ParseFailed)?;

        // Step 2: Single-pass DFS traversal using TreeCursor
        // Steps 2-3 are wrapped in a panic guard: if any rule panics (an
        // internal analyzer error), the analysis is considered incomplete and a
        // single QUIT diagnostic is returned instead of crashing.
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut diagnostics = self.traverse(&tree, source, file_path);

            // Step 3: File-level checks (only on rules that implement check_file)
            let file_ctx = FileContext {
                tree: &tree,
                source,
                file_path,
            };
            for &idx in self.registry.file_check_rules() {
                let rule = self.registry.get_rule(idx);
                let mut file_diags = rule.check_file(&file_ctx);
                // Stamp effective severity on file-level diagnostics.
                let effective_severity = self.registry.effective_severity(idx);
                for diag in &mut file_diags {
                    diag.severity = effective_severity;
                }
                diagnostics.extend(file_diags);
            }

            diagnostics
        }));

        let mut diagnostics = match result {
            Ok(diags) => diags,
            // Internal analyzer failure — analysis did not complete.
            Err(_) => vec![Self::quit_diagnostic(source)],
        };

        // Step 4: Apply inline suppression directives (%#ok<...>).
        // A directive on a line suppresses matching diagnostics on that line only.
        if self.inline_suppression {
            let suppressions = suppression::parse(source);
            diagnostics.retain(|d| !suppressions.is_suppressed(d.line, d.rule_id));
        }

        // Step 5: Sort by position for deterministic output
        diagnostics.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));

        Ok(diagnostics)
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
                };

                for &idx in rule_indices {
                    let rule = self.registry.get_rule(idx);
                    let mut rule_diags = rule.check(&node_ctx);
                    // Stamp effective severity from config onto each diagnostic.
                    let effective_severity = self.registry.effective_severity(idx);
                    for diag in &mut rule_diags {
                        diag.severity = effective_severity;
                    }
                    diagnostics.extend(rule_diags);
                }
            }

            // DFS navigation: try child first, then sibling, then backtrack.
            if cursor.goto_first_child() {
                continue;
            }
            if cursor.goto_next_sibling() {
                continue;
            }

            // Backtrack up the tree until we find an ancestor with a next sibling.
            loop {
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

    fn lint_with(source: &str) -> Vec<Diagnostic> {
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
    fn inline_suppression_star_all_suppresses_everything_on_line() {
        let diags = lint_with("aa = 1; bb = 2; %#ok<*all*>\n");
        assert!(diags.is_empty(), "got: {diags:?}");
    }

    #[test]
    fn inline_suppression_comma_separated_ids() {
        let diags = lint_with("aa = 1; bb = 2; %#ok<RULEA,RULEB>\n");
        assert!(diags.is_empty(), "got: {diags:?}");
    }

    #[test]
    fn inline_suppression_wildcard_matching() {
        // `*A` matches RULEA (ends with "A") but not RULEB.
        let diags = lint_with("aa = 1; bb = 2; %#ok<*A>\n");
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
        assert_eq!(diags.len(), 2, "got: {diags:?}");
        assert!(diags.iter().any(|d| d.rule_id == "RULEA"));
        assert!(diags.iter().any(|d| d.rule_id == "RULEB"));
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
}
