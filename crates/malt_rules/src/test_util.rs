//! Shared utilities for rule-module unit tests.
//!
//! These helpers mirror the `Linter` dispatch so tests can exercise both
//! node-level (`check`) and file-level (`check_file`) rules directly,
//! without going through the full CLI.
//!
//! This module is only compiled under `#[cfg(test)]`.

use std::path::Path;

use malt_core::{Diagnostic, FileContext, NodeContext, ProjectIndex, Rule};
use tree_sitter::{Node, Parser};

/// A search path that knows only the classes it lists: each name with whether it
/// is a handle class. `handle` is always a handle class. Any other name is unknown.
pub struct StubProject(pub Vec<(&'static str, bool)>);

impl ProjectIndex for StubProject {
    fn is_handle_class(&self, name: &str, _from: &Path) -> Option<bool> {
        if name == "handle" {
            return Some(true);
        }
        self.0
            .iter()
            .find(|(known, _)| *known == name)
            .map(|(_, handle)| *handle)
    }
}

/// Parse MATLAB source and return the tree.
pub fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_matlab::LANGUAGE.into())
        .expect("failed to load tree-sitter-matlab");
    malt_core::note_parse();
    malt_core::note_parse();
    parser.parse(source, None).expect("parse failed")
}

/// Build a `FileContext` with a fixed test file path.
pub fn file_ctx<'a>(tree: &'a tree_sitter::Tree, source: &'a str) -> FileContext<'a> {
    FileContext {
        tree,
        source,
        file_path: std::path::Path::new("test.m"),
        project: None,
    }
}

/// Run a rule's node-level `check()` over every node in the tree, mimicking
/// the Linter's depth-first dispatch. Returns all diagnostics.
pub fn lint_nodes(rule: &dyn Rule, source: &str) -> Vec<Diagnostic> {
    let tree = parse(source);
    let path = std::path::Path::new("test.m");
    let targets: Vec<&str> = rule.target_node_types().to_vec();
    let mut diagnostics = Vec::new();

    fn walk(
        node: Node,
        source: &str,
        path: &std::path::Path,
        targets: &[&str],
        rule: &dyn Rule,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if targets.contains(&node.kind()) {
            let ctx = NodeContext {
                node,
                source,
                file_path: path,
                ancestors: None,
            };
            diagnostics.extend(rule.check(&ctx));
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk(child, source, path, targets, rule, diagnostics);
        }
    }

    walk(
        tree.root_node(),
        source,
        path,
        &targets,
        rule,
        &mut diagnostics,
    );
    diagnostics
}

/// Run a rule's file-level `check_file()`. Returns all diagnostics.
pub fn lint_file(rule: &dyn Rule, source: &str) -> Vec<Diagnostic> {
    let tree = parse(source);
    let ctx = file_ctx(&tree, source);
    rule.check_file(&ctx)
}

/// Returns true if any diagnostic has the given rule ID.
pub fn has_id(diags: &[Diagnostic], id: &str) -> bool {
    diags.iter().any(|d| d.rule_id == id)
}

/// Every diagnostic of `rule` on `source`, node-level and file-level.
pub fn lint_all(rule: &dyn Rule, source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = lint_nodes(rule, source);
    diagnostics.extend(lint_file(rule, source));
    diagnostics
}

/// `source` after the fixes of `rule`, unsafe ones included (one pass).
pub fn fixed_all(rule: &dyn Rule, source: &str) -> String {
    let diagnostics = lint_all(rule, source);
    malt_core::apply_fixes_with(source, &diagnostics, malt_core::FixMode::All).0
}

/// The fix of the single diagnostic `id` on `source`: whether it is safe, and
/// the source with only that diagnostic's fix applied. `None` when the
/// diagnostic is absent or has no fix.
pub fn fix_of(rule: &dyn Rule, source: &str, id: &str) -> Option<(bool, String)> {
    let diagnostics: Vec<Diagnostic> = lint_all(rule, source)
        .into_iter()
        .filter(|d| d.rule_id == id)
        .collect();
    let first = diagnostics.first()?;
    let safe = first.fix.as_ref()?.is_safe();
    let fixed = malt_core::apply_fixes_with(source, &diagnostics, malt_core::FixMode::All).0;
    Some((safe, fixed))
}

/// Whether `id` is reported on `source` with no fix.
pub fn reported_without_fix(rule: &dyn Rule, source: &str, id: &str) -> bool {
    let diagnostics: Vec<Diagnostic> = lint_all(rule, source)
        .into_iter()
        .filter(|d| d.rule_id == id)
        .collect();
    !diagnostics.is_empty() && diagnostics.iter().all(|d| d.fix.is_none())
}
