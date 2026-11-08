//! Shared utilities for rule-module unit tests.
//!
//! These helpers mirror the `Linter` dispatch so tests can exercise both
//! node-level (`check`) and file-level (`check_file`) rules directly,
//! without going through the full CLI.
//!
//! This module is only compiled under `#[cfg(test)]`.

use malt_core::{Diagnostic, FileContext, NodeContext, Rule};
use tree_sitter::{Node, Parser};

/// Parse MATLAB source and return the tree.
pub fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_matlab::LANGUAGE.into())
        .expect("failed to load tree-sitter-matlab");
    parser.parse(source, None).expect("parse failed")
}

/// Build a `FileContext` with a fixed test file path.
pub fn file_ctx<'a>(tree: &'a tree_sitter::Tree, source: &'a str) -> FileContext<'a> {
    FileContext {
        tree,
        source,
        file_path: std::path::Path::new("test.m"),
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
