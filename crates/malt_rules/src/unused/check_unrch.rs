//! UNRCH check: unreachable code after return/break/continue/error.

use super::*;
use crate::analysis::control_flow::is_script_body;
use crate::analysis::liveness::constant_truths;
use crate::analysis::symbols::SymbolTable;
use std::collections::HashMap;

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

        // Dead branches of constant conditions (`if true ... else`, `k = 0; if k`).
        let table = SymbolTable::build(ctx.tree, ctx.source);
        let truths = constant_truths(&table, ctx.tree, ctx.source);
        collect_constant_branches(
            ctx.tree.root_node(),
            ctx.source,
            &truths,
            &mut seen,
            diagnostics,
        );
    }
}

/// Walk the tree and report the dead branch of each constant `if` condition.
fn collect_constant_branches(
    node: Node,
    source: &str,
    truths: &HashMap<usize, bool>,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(dead) = dead_constant_branch(node, source, truths) {
        emit_unrch(
            dead.start_byte()..dead.end_byte(),
            dead.start_position().row + 1,
            dead.start_position().column + 1,
            seen,
            diagnostics,
        );
    }
    for child in named_children(node) {
        collect_constant_branches(child, source, truths, seen, diagnostics);
    }
}

/// The first statement of the branch that a constant condition never runs, for an
/// `if` with no `elseif`. Probed: `if true ... else y = 1; end` reports the `else`
/// statement, `if false y = 1; end` and `if 0 disp(1) else ... end` the body.
/// Numeric literals, `true` and `false` count, and so do variables whose value is
/// known at the condition (`k = 0; if k`), see `constant_truths`.
fn dead_constant_branch<'a>(
    node: Node<'a>,
    source: &str,
    truths: &HashMap<usize, bool>,
) -> Option<Node<'a>> {
    if node.kind() != "if_statement" {
        return None;
    }
    let condition = node.child_by_field_name("condition")?;
    let text = source[condition.start_byte()..condition.end_byte()].trim();
    let truth = match text {
        "true" => true,
        "false" => false,
        _ if text.starts_with(|c: char| c.is_ascii_digit()) => text.parse::<f64>().ok()? != 0.0,
        _ => *truths.get(&condition.start_byte())?,
    };
    let children = named_children(node);
    if children.iter().any(|c| c.kind() == "elseif_clause") {
        return None;
    }
    let block = if truth {
        let else_clause = children.iter().find(|c| c.kind() == "else_clause")?;
        named_children(*else_clause)
            .into_iter()
            .find(|c| c.kind() == "block")?
    } else {
        *children.iter().find(|c| c.kind() == "block")?
    };
    named_children(block)
        .into_iter()
        .find(|c| c.kind() != "comment")
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
/// Probes: `rethrow` calls, `error` calls whose first argument is not a variable
/// (`error(s)` is not a terminator, gp-b__t2; `error(sprintf(...))` is, readability__t3),
/// and `while true` loops with no `break`.
fn is_terminator(node: Node, source: &str) -> bool {
    is_call_to(node, source, &["rethrow"])
        || (is_call_to(node, source, &["error"]) && !has_variable_first_argument(node))
        || is_infinite_loop(node, source)
}

/// Returns `true` if the first argument of a call is a bare identifier.
fn has_variable_first_argument(node: Node) -> bool {
    named_children(node)
        .into_iter()
        .find(|c| c.kind() == "arguments")
        .and_then(|args| named_children(args).into_iter().next())
        .is_some_and(|arg| arg.kind() == "identifier")
}

/// Byte ranges of the code that UNRCH reports as unreachable: every statement
/// after a `return`, `break`, `continue`, `error` or infinite loop in the same
/// block. Other checks use them to stay silent where MATLAB does not analyse.
pub(crate) fn unreachable_regions(
    tree: &tree_sitter::Tree,
    source: &str,
) -> Vec<std::ops::Range<usize>> {
    let mut regions: Vec<std::ops::Range<usize>> = find_unreachable(tree, source)
        .into_iter()
        .map(|span| span.byte_range)
        .collect();
    collect_regions_after_terminator(tree.root_node(), source, &mut regions);
    regions
}

fn collect_regions_after_terminator(
    node: Node,
    source: &str,
    regions: &mut Vec<std::ops::Range<usize>>,
) {
    if node.kind() == "block" || is_script_body(node) {
        let mut terminator_seen = false;
        for child in named_children(node) {
            if terminator_seen {
                if !matches!(child.kind(), "comment" | "function_definition") {
                    regions.push(child.start_byte()..node.end_byte());
                }
                break;
            } else if is_terminator(child, source) {
                terminator_seen = true;
            }
        }
    }
    for child in named_children(node) {
        collect_regions_after_terminator(child, source, regions);
    }
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
                if matches!(child.kind(), "comment" | "function_definition") {
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

    #[test]
    fn unrch_dead_else_of_constant_true() {
        // Probed: MATLAB reports the `else` statement of `if true`.
        let src = "function f()\nif true\n    disp(1);\nelse\n    y = 1;\nend\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "UNRCH"), "got: {diags:?}");
    }

    #[test]
    fn unrch_dead_body_of_constant_false_and_zero() {
        // Probed: `if false` and `if 0` report the body; the `else` is live.
        let src = "function f()\nif false\n    y = 1;\nend\nend\n";
        assert!(has_id(&lint_file(&*engine(), src), "UNRCH"));
        let src = "function f()\nif 0\n    disp(1)\nelse\n    y = 1;\nend\nend\n";
        assert!(has_id(&lint_file(&*engine(), src), "UNRCH"));
    }

    #[test]
    fn unrch_dead_body_of_variable_known_false() {
        // Probed: `k = 0; if k` never runs its body.
        let src = "function f()\n  k = 0;\n  if k\n    disp(1)\n  end\nend\n";
        assert_eq!(unrch_lines(&lint_file(&*engine(), src)), vec![4]);
    }

    #[test]
    fn unrch_dead_body_of_negated_variable() {
        // Probed: `k = 1; if ~k` never runs its body.
        let src = "function f()\n  k = 1;\n  if ~k\n    disp(1)\n  end\nend\n";
        assert_eq!(unrch_lines(&lint_file(&*engine(), src)), vec![4]);
    }

    #[test]
    fn unrch_silent_on_variable_assigned_under_condition() {
        let src =
            "function f(x)\n  k = 0;\n  if x\n    k = 1;\n  end\n  if k\n    disp(1)\n  end\nend\n";
        assert!(!has_id(&lint_file(&*engine(), src), "UNRCH"));
    }

    #[test]
    fn unrch_silent_on_variable_condition() {
        let src = "function f(c)\nif c\n    disp(1);\nelse\n    y = 1;\nend\nend\n";
        assert!(!has_id(&lint_file(&*engine(), src), "UNRCH"));
    }
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
    fn unrch_ok_after_error_with_variable_message() {
        // Probe gp-b__t2: `error(s)` with a variable does not end the flow.
        let diags = lint_file(
            &*engine(),
            "function f(s)\n    error(s);\n    disp(1);\nend\n",
        );
        assert!(unrch_lines(&diags).is_empty(), "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_after_error_with_computed_message() {
        // Probe readability__t3: `error(sprintf(...))` ends the flow.
        let diags = lint_file(
            &*engine(),
            "function f(x)\n    error(sprintf('bad %d', x));\n    disp(1);\nend\n",
        );
        assert_eq!(unrch_lines(&diags), vec![3], "got: {diags:?}");
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
