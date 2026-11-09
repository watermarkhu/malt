//! UNRCH check: unreachable code after return/break/continue/error.

use super::*;
use crate::analysis::control_flow::is_script_body;

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

/// Returns `true` if `node` is a statement-level call to one of `names`.
fn is_call_to(node: Node, source: &str, names: &[&str]) -> bool {
    if node.kind() != "function_call" {
        return false;
    }
    let Some(name_node) = node.child_by_field_name("name") else {
        return false;
    };
    let text = &source[name_node.start_byte()..name_node.end_byte()];
    names.iter().any(|n| text.eq_ignore_ascii_case(n))
}

/// Returns `true` if `node` is a `while true` loop with no `break` anywhere in
/// it. Such a loop never ends, so the statements after it cannot be reached.
fn is_infinite_loop(node: Node, source: &str) -> bool {
    if node.kind() != "while_statement" {
        return false;
    }
    let Some(condition) = node.child_by_field_name("condition") else {
        return false;
    };
    if source[condition.start_byte()..condition.end_byte()].trim() != "true" {
        return false;
    }
    !contains_kind(node, "break_statement")
}

/// Returns `true` if `node` or any node below it has the given kind.
fn contains_kind(node: Node, kind: &str) -> bool {
    node.kind() == kind
        || named_children(node)
            .into_iter()
            .any(|c| contains_kind(c, kind))
}

/// Returns `true` if `node` never lets execution continue to the next statement.
/// Probes: `error` and `rethrow` calls, and `while true` loops with no `break`.
fn is_terminator(node: Node, source: &str) -> bool {
    is_call_to(node, source, &["error", "rethrow"]) || is_infinite_loop(node, source)
}

/// Walk the tree; within each `block` or script body (`source_file`), mark
/// statements following a terminator as unreachable.
fn collect_unreachable_after_error(
    node: Node,
    source: &str,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "block" || is_script_body(node) {
        let mut terminator_seen = false;
        for child in named_children(node) {
            if terminator_seen {
                if child.kind() == "comment" {
                    continue;
                }
                // One report per run: MATLAB flags the first statement only.
                let pos = child.start_position();
                emit_unrch(
                    child.start_byte()..child.end_byte(),
                    pos.row + 1,
                    pos.column + 1,
                    seen,
                    diagnostics,
                );
                break;
            } else if is_terminator(child, source) {
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

    fn unrch_lines(diags: &[Diagnostic]) -> Vec<usize> {
        diags
            .iter()
            .filter(|d| d.rule_id == "UNRCH")
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn unrch_fires_after_rethrow() {
        // Probe: MATLAB treats rethrow like error.
        let diags = lint_file(
            &*engine(),
            "function foo()\n    rethrow(MException('a:b', 'c'));\n    x = 1;\nend\n",
        );
        assert_eq!(unrch_lines(&diags), vec![3], "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_after_while_true_without_break() {
        // Probe: the loop never ends, so the statement after it is unreachable.
        let diags = lint_file(
            &*engine(),
            "function foo()\n    while true\n        disp(1);\n    end\n    x = 2;\nend\n",
        );
        assert_eq!(unrch_lines(&diags), vec![5], "got: {diags:?}");
    }

    #[test]
    fn unrch_ok_after_while_true_with_break() {
        // Probe: a break inside the loop makes the statement after it reachable.
        let diags = lint_file(
            &*engine(),
            "function foo(x)\n    while true\n        if x\n            break;\n        end\n        disp(1);\n    end\n    y = 2;\nend\n",
        );
        assert!(!has_id(&diags, "UNRCH"), "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_after_return_in_script() {
        // Probe: scripts have a body too; statements after return are unreachable.
        let diags = lint_file(&*engine(), "disp(1);\nreturn;\ndisp(2);\n");
        assert_eq!(unrch_lines(&diags), vec![3], "got: {diags:?}");
    }

    #[test]
    fn unrch_ok_after_top_level_break_in_script() {
        // Probe: MATLAB reports nothing after a break outside a loop (a syntax error).
        let diags = lint_file(&*engine(), "disp(1);\nbreak\ndisp(2);\n");
        assert!(!has_id(&diags, "UNRCH"), "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_after_error_in_script() {
        let diags = lint_file(&*engine(), "disp(1);\nerror('a:b', 'boom');\ndisp(2);\n");
        assert_eq!(unrch_lines(&diags), vec![3], "got: {diags:?}");
    }

    #[test]
    fn unrch_ok_after_error_inside_if() {
        // The error is conditional, so the next statement is reachable.
        let diags = lint_file(
            &*engine(),
            "function foo(x)\n    if x\n        error('boom');\n    end\n    y = 1;\nend\n",
        );
        assert!(!has_id(&diags, "UNRCH"), "got: {diags:?}");
    }

    #[test]
    fn unrch_reports_only_first_statement_of_run() {
        // Probe: MATLAB reports one statement for a run of unreachable code.
        let diags = lint_file(
            &*engine(),
            "function foo()\n    return;\n    x = 1;\n    y = 2;\n    z = 3;\nend\n",
        );
        assert_eq!(unrch_lines(&diags), vec![3], "got: {diags:?}");
    }

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
