//! UNRCH check: unreachable code after return/break/continue/error.

use super::*;

impl UnusedEngine {
    /// Run UNRCH check: statements that cannot be reached because they follow a
    /// `return`, `break`, `continue`, or `error(...)` call in the same block.
    pub(crate) fn check_unrch(&self, ctx: &FileContext, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("UNRCH") {
            return;
        }

        let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();

        // Statements after return/break/continue.
        for span in find_unreachable(ctx.tree, ctx.source) {
            emit_unrch(
                span.byte_range,
                span.line,
                span.column,
                &mut seen,
                diagnostics,
            );
        }

        // Statements after `error(...)` calls (which always throw).
        collect_unreachable_after_error(ctx.tree.root_node(), ctx.source, &mut seen, diagnostics);
    }
}

/// Emit a single UNRCH diagnostic, deduplicating by byte range.
fn emit_unrch(
    byte_range: std::ops::Range<usize>,
    line: usize,
    column: usize,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !seen.insert((byte_range.start, byte_range.end)) {
        return;
    }
    diagnostics.push(Diagnostic {
        rule_id: "UNRCH",
        message: "This statement (and possibly following ones) cannot be reached.".to_string(),
        severity: Severity::Warning,
        byte_range,
        line,
        column,
        fix: None,
    });
}

/// Collect named children of a node.
fn named_children(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

/// Returns `true` if `node` is a statement-level `error(...)` call.
fn is_error_call(node: Node, source: &str) -> bool {
    if node.kind() != "function_call" {
        return false;
    }
    let Some(name_node) = node.child_by_field_name("name") else {
        return false;
    };
    source[name_node.start_byte()..name_node.end_byte()].eq_ignore_ascii_case("error")
}

/// Walk the tree; within each `block`, mark statements following an `error`
/// call as unreachable.
fn collect_unreachable_after_error(
    node: Node,
    source: &str,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "block" {
        let mut terminator_seen = false;
        for child in named_children(node) {
            if terminator_seen {
                if child.kind() == "comment" {
                    continue;
                }
                let pos = child.start_position();
                emit_unrch(
                    child.start_byte()..child.end_byte(),
                    pos.row + 1,
                    pos.column + 1,
                    seen,
                    diagnostics,
                );
            } else if is_error_call(child, source) {
                terminator_seen = true;
            }
        }
    }

    for child in named_children(node) {
        collect_unreachable_after_error(child, source, seen, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    // -- UNRCH: unreachable code --------------------------------------------

    #[test]
    fn unrch_fires_after_return() {
        let diags = lint_file(&*engine(), "function foo()\n    return;\n    x = 1;\nend\n");
        assert!(has_id(&diags, "UNRCH"), "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_after_error() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    error('boom');\n    x = 1;\nend\n",
        );
        assert!(has_id(&diags, "UNRCH"), "got: {diags:?}");
    }

    #[test]
    fn unrch_ok_without_terminator() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    x = 1;\n    disp(x);\nend\n",
        );
        assert!(!has_id(&diags, "UNRCH"), "got: {diags:?}");
    }
}
