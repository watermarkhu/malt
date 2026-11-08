//! Generic (AST-pattern) compatibility checks.
//!
//! These checks cannot be matched by the function-name lookup table (their
//! data-file entries have empty `function_name`), so each is implemented as a
//! pattern check over the tree-sitter AST. The checks are grouped by pattern
//! family; each group lives in its own `check_*.rs` module and exposes a
//! `pub(crate) fn collect_checks(engine, node, source, diagnostics)`.
//!
//! `collect_node_checks` (below) is the single dispatch point called from the
//! engine's file-level walker.

use super::CompatibilityEngine;
use malt_core::Diagnostic;

mod check_behavior_prop; // 4.5-D: behavior-change figure/axes properties
mod check_forward_gates; // 4.5-C: forward-compat version gates + scope/unset
mod check_input_syntax; // 4.5-B: input-arg renames + class/scripting syntax
mod check_property_attr; // 4.5-A: property/attribute + option removals
mod check_unset_vars; // Phase 7: IDISVARLOW/IDISVARHIGH/SHVAI unset vars

/// Dispatch a single node to every generic check group.
pub(crate) fn collect_node_checks(
    engine: &CompatibilityEngine,
    node: tree_sitter::Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_property_attr::collect_checks(engine, node, source, diagnostics);
    check_input_syntax::collect_checks(engine, node, source, diagnostics);
    check_forward_gates::collect_checks(engine, node, source, diagnostics);
    check_behavior_prop::collect_checks(engine, node, source, diagnostics);
}

/// Dispatch the file-level (whole-tree) generic checks. Called once per file
/// from the engine's `check_file` pass, after the per-node walk.
pub(crate) fn collect_file_checks(
    engine: &CompatibilityEngine,
    tree: &tree_sitter::Tree,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_forward_gates::collect_file_checks(engine, tree, source, diagnostics);
    check_unset_vars::collect_file_checks(engine, tree, source, diagnostics);
}
