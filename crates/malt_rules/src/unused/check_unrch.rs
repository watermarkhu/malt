//! UNRCH check: unreachable code after return/break/continue/error.

use super::*;
use crate::analysis::control_flow::{error_call_always_throws, is_script_body};
use crate::analysis::liveness::{constant_truths, if_clauses};
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
        collect_empty_try(ctx.tree.root_node(), &mut seen, diagnostics);

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
        let table = SymbolTable::shared(|| ctx.tree.root_node(), ctx.source);
        let truths = constant_truths(&table, ctx.tree.root_node(), ctx.source);
        collect_constant_branches(
            ctx.tree.root_node(),
            ctx.source,
            &truths,
            &mut seen,
            diagnostics,
        );
    }
}

/// The body of a `while` whose condition is constant false: MATLAB reports its first
/// statement (probed: `while false`, `while 0`, `while ~true`, `while []` and a
/// variable that holds false). A `for` over an empty range is not folded.
fn dead_loop_body<'a>(
    node: Node<'a>,
    source: &str,
    truths: &HashMap<usize, bool>,
) -> Option<Node<'a>> {
    if node.kind() != "while_statement" {
        return None;
    }
    let condition = node.child_by_field_name("condition")?;
    if constant_truth(condition, source, truths) != Some(false) {
        return None;
    }
    let parts: Vec<Node> = named_children(node)
        .into_iter()
        .filter(|c| c.id() != condition.id())
        .collect();
    first_statement(&parts)
}

/// Walk the tree and report the dead branch of each constant `if` condition.
fn collect_constant_branches(
    node: Node,
    source: &str,
    truths: &HashMap<usize, bool>,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for dead in dead_constant_branches(node, source, truths)
        .into_iter()
        .chain(dead_loop_body(node, source, truths))
    {
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

/// The first statement that a constant condition makes unreachable, walking the
/// clauses of an `if` in order. A constant false clause never runs: its first
/// statement is reported (probed: `if 0 disp(1) else ... end`, `x = 0; if x a = 1
/// elseif y ... end` reports `a = 1`). After a constant true clause the next clause
/// is dead: an `elseif` is reported at its keyword, an `else` at its first statement
/// (probed: `if 1 ... elseif y ... else a = 3 end` reports the `elseif` line).
/// Numeric literals, `true` and `false` count, and so do variables whose value is
/// known at the condition (`k = 0; if k`), see `constant_truths`. A constant false
/// clause does not stop the scan: `if 0 a elseif 1 b else c end` reports `a` and `c`
/// (probed).
fn dead_constant_branches<'a>(
    node: Node<'a>,
    source: &str,
    truths: &HashMap<usize, bool>,
) -> Vec<Node<'a>> {
    let mut dead = Vec::new();
    if node.kind() != "if_statement" {
        return dead;
    }
    let mut after_true = false;
    for clause in if_clauses(node) {
        if after_true {
            let first = if clause.is_else {
                first_statement(&clause.parts)
            } else {
                Some(clause.node)
            };
            dead.extend(first);
            break;
        }
        let truth = match clause.condition {
            None => Some(true),
            Some(condition) => constant_truth(condition, source, truths),
        };
        match truth {
            Some(false) => dead.extend(first_statement(&clause.parts)),
            Some(true) => after_true = true,
            None => {}
        }
    }
    dead
}

/// The truth value of a condition that is a literal or a known variable.
fn constant_truth(condition: Node, source: &str, truths: &HashMap<usize, bool>) -> Option<bool> {
    let text = source[condition.start_byte()..condition.end_byte()].trim();
    match text {
        "true" => Some(true),
        "false" => Some(false),
        _ if text.starts_with(|c: char| c.is_ascii_digit()) => {
            text.parse::<f64>().ok().map(|v| v != 0.0)
        }
        _ => truths.get(&condition.start_byte()).copied(),
    }
}

/// The first statement of a clause body: its block's first named child, or the
/// statement written after the clause header on the same line.
fn first_statement<'a>(parts: &[Node<'a>]) -> Option<Node<'a>> {
    parts
        .iter()
        .flat_map(|part| {
            if part.kind() == "block" {
                named_children(*part)
            } else {
                vec![*part]
            }
        })
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

/// A `try` whose body cannot throw before it leaves: MATLAB reports its `catch` as
/// unreachable (probed: `try, catch end` and `try, catch ME, end` report the `catch`
/// keyword, and so does `try, return, catch, ..., end`).
fn collect_empty_try(
    node: Node,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "try_statement" {
        let children = named_children(node);
        let first_statement = children
            .iter()
            .filter(|c| c.kind() == "block")
            .flat_map(|block| named_children(*block))
            .find(|c| c.kind() != "comment");
        let body_cannot_throw = first_statement.is_none_or(|c| {
            matches!(
                c.kind(),
                "return_statement" | "break_statement" | "continue_statement"
            )
        });
        if body_cannot_throw {
            if let Some(catch) = children.iter().find(|c| c.kind() == "catch_clause") {
                let pos = catch.start_position();
                emit_unrch(
                    catch.start_byte()..catch.end_byte(),
                    pos.row + 1,
                    pos.column + 1,
                    seen,
                    diagnostics,
                );
            }
        }
    }
    for child in named_children(node) {
        collect_empty_try(child, seen, diagnostics);
    }
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
    let text = source[condition.start_byte()..condition.end_byte()].trim();
    let constant_true = text == "true" || text.parse::<f64>().is_ok_and(|v| v != 0.0);
    if !constant_true {
        return false;
    }
    !contains_live_break(node, source)
}

/// Returns `true` if `node` is a `try` whose body and `catch` both end in a
/// terminator: an error in the body runs the `catch`, which does not come back
/// (probed: `try, error('x'), catch, return, end` makes the next statement dead).
fn is_terminating_try(node: Node, source: &str) -> bool {
    if node.kind() != "try_statement" {
        return false;
    }
    let children = named_children(node);
    let body = children.iter().find(|c| c.kind() == "block");
    let catch = children.iter().find(|c| c.kind() == "catch_clause");
    let catch_block =
        catch.and_then(|c| named_children(*c).into_iter().find(|b| b.kind() == "block"));
    body.is_some_and(|b| block_terminates(*b, source))
        && catch_block.is_some_and(|b| block_terminates(b, source))
}

/// Returns `true` if a block has a statement that never lets execution go on.
fn block_terminates(block: Node, source: &str) -> bool {
    named_children(block)
        .into_iter()
        .any(|c| statement_terminates(c, source))
}

/// Returns `true` if a statement never lets execution go on to the next one.
fn statement_terminates(node: Node, source: &str) -> bool {
    matches!(
        node.kind(),
        "return_statement" | "break_statement" | "continue_statement"
    ) || is_terminator(node, source)
}

/// Returns `true` if a `break` below `node` can run: a `break` in a clause of an
/// `if` whose literal condition is false never does (probed: `while 1, if 0, break,
/// end, end` never ends).
fn contains_live_break(node: Node, source: &str) -> bool {
    match node.kind() {
        "break_statement" => true,
        "if_statement" => {
            let truths = HashMap::new();
            if_clauses(node).iter().any(|clause| {
                let dead = clause
                    .condition
                    .is_some_and(|c| constant_truth(c, source, &truths) == Some(false));
                !dead
                    && clause
                        .parts
                        .iter()
                        .any(|part| contains_live_break(*part, source))
            })
        }
        _ => named_children(node)
            .into_iter()
            .any(|c| contains_live_break(c, source)),
    }
}

/// Returns `true` if `node` never lets execution continue to the next statement.
/// Probes: `rethrow` and `exit` calls, `error` calls whose first argument is not a
/// variable (`error(s)` is not a terminator, gp-b__t2; `error(sprintf(...))` is,
/// readability__t3), `while true` and `while 1` loops with no `break`, and a `try`
/// whose body and `catch` both terminate.
fn is_terminator(node: Node, source: &str) -> bool {
    is_call_to(node, source, &["throw", "rethrow", "exit", "quit"])
        || is_bare_name_of(node, source, &["exit", "quit"])
        || (is_call_to(node, source, &["error"]) && error_call_always_throws(node, source))
        || is_infinite_loop(node, source)
        || is_terminating_try(node, source)
        || is_terminating_if(node, source)
}

/// Returns `true` if `node` is a bare `exit;` or `quit` statement (no parentheses),
/// which the grammar parses as a name or a command.
fn is_bare_name_of(node: Node, source: &str, names: &[&str]) -> bool {
    let name = match node.kind() {
        "identifier" => node,
        "command" if node.named_child_count() == 1 => match node.named_child(0) {
            Some(child) if child.kind() == "command_name" => child,
            _ => return false,
        },
        _ => return false,
    };
    names.contains(&&source[name.byte_range()])
}

/// Returns `true` if `node` is an `if` with an `else` whose every clause ends in a
/// terminator (probed: `if c, error('a'), else, error('b'), end` makes the next
/// statement dead).
fn is_terminating_if(node: Node, source: &str) -> bool {
    if node.kind() != "if_statement" {
        return false;
    }
    let clauses = if_clauses(node);
    clauses.iter().any(|c| c.is_else)
        && clauses.iter().all(|clause| {
            clause.parts.iter().any(|part| {
                if part.kind() == "block" {
                    block_terminates(*part, source)
                } else {
                    statement_terminates(*part, source)
                }
            })
        })
}

/// Returns `true` if the first argument of a call is a bare identifier.
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

/// Byte ranges of the code that cannot run: `unreachable_regions` and the dead
/// clauses of constant conditions (`if false ... end`). STOUT does not count an
/// assignment in them (probed: `if false, r = 1; end` and `return; r = 1;` leave
/// the output unset).
pub(crate) fn dead_constant_regions(root: Node, source: &str) -> Vec<std::ops::Range<usize>> {
    let table = SymbolTable::shared(|| root, source);
    let truths = constant_truths(&table, root, source);
    let mut regions = Vec::new();
    collect_dead_constant_regions(root, source, &truths, &mut regions);
    regions
}

pub(crate) fn dead_regions(tree: &tree_sitter::Tree, source: &str) -> Vec<std::ops::Range<usize>> {
    let mut regions = unreachable_regions(tree, source);
    let table = SymbolTable::shared(|| tree.root_node(), source);
    let truths = constant_truths(&table, tree.root_node(), source);
    collect_dead_constant_regions(tree.root_node(), source, &truths, &mut regions);
    regions
}

fn collect_dead_constant_regions(
    node: Node,
    source: &str,
    truths: &HashMap<usize, bool>,
    regions: &mut Vec<std::ops::Range<usize>>,
) {
    for dead in dead_constant_branches(node, source, truths) {
        let range = match dead.parent() {
            Some(parent) if dead.kind() != "elseif_clause" && parent.kind() == "block" => {
                parent.byte_range()
            }
            _ => dead.byte_range(),
        };
        regions.push(range);
    }
    for child in named_children(node) {
        collect_dead_constant_regions(child, source, truths, regions);
    }
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
    fn unrch_dead_branch_of_strcmp_strcmpi_and_ischar_on_known_values() {
        // Probed: `strcmp` and `strcmpi` of known text (also through a copy of the
        // result) and `ischar` of a known value fold; the other branch is dead.
        let src = "function f()\n  b = 'x';\n  if strcmp(b, 'y')\n    disp(1)\n  end\nend\n";
        assert_eq!(unrch_lines(&lint_file(&*engine(), src)), vec![4]);
        let src = "function f()\n  b = 'x';\n  if strcmpi(b, 'X')\n    disp(1)\n  else\n    disp(2)\n  end\nend\n";
        assert_eq!(unrch_lines(&lint_file(&*engine(), src)), vec![6]);
        let src = "function f()\n  b = 5;\n  if ischar(b)\n    disp(1)\n  end\nend\n";
        assert_eq!(unrch_lines(&lint_file(&*engine(), src)), vec![4]);
        let src =
            "function f()\n  b = 'x';\n  c = strcmp(b, 'y');\n  if c\n    disp(1)\n  end\nend\n";
        assert_eq!(unrch_lines(&lint_file(&*engine(), src)), vec![5]);
    }

    #[test]
    fn unrch_silent_on_unknown_or_unfolded_strcmp_and_ischar() {
        // Probed: a value that is one of two, a parameter, `==` on a known number, and
        // `isempty`/`isnumeric` on known values do not fold: the branches stay live.
        for src in [
            "function f()\n  b = 'x';\n  if rand > 2\n    b = 'y';\n  end\n  if strcmp(b, 'x')\n    disp(1)\n  end\nend\n",
            "function f(b)\n  if strcmp(b, 'x')\n    disp(1)\n  end\nend\n",
            "function f()\n  n = 3;\n  if n == 4\n    disp(1)\n  end\nend\n",
            "function f()\n  b = [];\n  if isempty(b)\n    disp(1)\n  else\n    disp(2)\n  end\nend\n",
            "function f()\n  n = 3;\n  if isnumeric(n)\n    disp(1)\n  else\n    disp(2)\n  end\nend\n",
        ] {
            assert!(!has_id(&lint_file(&*engine(), src), "UNRCH"), "{src}");
        }
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
    fn unrch_fires_after_top_level_break_in_script() {
        // Probe: a `break` outside a loop is BRKFOR, and the statement after it is UNRCH.
        let diags = lint_file(&*engine(), "disp(1);\nbreak\ndisp(2);\n");
        assert_eq!(unrch_lines(&diags), vec![3], "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_after_top_level_break_in_script_with_syntax_error() {
        // Probe t__t1: the `break` ends the flow even though the file has a syntax error.
        let diags = lint_file(&*engine(), "disp(1);\nbreak\ncontinue\ng = end;\n");
        assert_eq!(unrch_lines(&diags), vec![3], "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_after_top_level_continue_in_script() {
        // Probe: `for ... break ... end; break; continue; a = 1;` reports the `continue`.
        let diags = lint_file(
            &*engine(),
            "for i = 1:2\n  break\n  disp(3)\nend\nbreak\ncontinue\na = 1;\n",
        );
        let mut lines = unrch_lines(&diags);
        lines.sort_unstable();
        assert_eq!(lines, vec![3, 6], "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_after_if_where_all_branches_return() {
        // Probe unused__a3: both branches of the `if` return, so `disp(1)` is unreachable.
        let diags = lint_file(
            &*engine(),
            "function r = a3(x)\nr = 1;\nif x\n    return\nelse\n    return\nend\ndisp(1);\nend\n",
        );
        assert_eq!(unrch_lines(&diags), vec![8], "got: {diags:?}");
    }

    #[test]
    fn unrch_ok_after_if_where_one_branch_falls_through() {
        let diags = lint_file(
            &*engine(),
            "function r = f(x)\nif x\n    return\nend\ndisp(1);\nend\n",
        );
        assert!(unrch_lines(&diags).is_empty(), "got: {diags:?}");
    }

    #[test]
    fn unrch_ok_for_nested_function_after_return() {
        // Probe custom__a: `function g()` nested after `return` is not unreachable code.
        let diags = lint_file(&*engine(), "function f()\nreturn\nfunction g()\nend\nend\n");
        assert!(unrch_lines(&diags).is_empty(), "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_at_elseif_after_constant_true_if() {
        // Probe formatting__b / s6: `x = 1` makes `if x` true, so `elseif y` is unreachable.
        let diags = lint_file(
            &*engine(),
            "function f(y)\nx = 1;\nif x\n  a = 1;\nelseif y\n  a = 2;\nelse\n  a = 3;\nend\ndisp(a)\nend\n",
        );
        assert_eq!(unrch_lines(&diags), vec![5], "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_at_else_body_after_constant_true_elseif() {
        // Probe t7: the `elseif x` is true, so the `else` body is unreachable.
        let diags = lint_file(
            &*engine(),
            "function f(y)\nx = 1;\nif y\n  a = 1;\nelseif x\n  a = 2;\nelse\n  a = 3;\nend\ndisp(a)\nend\n",
        );
        assert_eq!(unrch_lines(&diags), vec![8], "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_in_body_of_constant_false_first_clause_with_elseif() {
        // Probe t5: `x = 0; if x a = 1 elseif y a = 2 end` reports the first body statement.
        let diags = lint_file(
            &*engine(),
            "function f(y)\nx = 0;\nif x\n  a = 1;\nelseif y\n  a = 2;\nend\ndisp(a)\nend\n",
        );
        assert_eq!(unrch_lines(&diags), vec![4], "got: {diags:?}");
    }

    #[test]
    fn unrch_fires_in_body_of_constant_false_elseif() {
        // Probe t8: `elseif 0` makes its body unreachable.
        let diags = lint_file(
            &*engine(),
            "function f(y)\nx = 1;\nif y\n  a = 1;\nelseif 0\n  a = 2;\nend\ndisp(a)\nend\n",
        );
        assert_eq!(unrch_lines(&diags), vec![6], "got: {diags:?}");
    }

    #[test]
    fn unrch_ok_after_loop_assigns_different_value() {
        // Probe t1: `x = 0; for ... x = 1; end` leaves `x` unknown, so no branch is dead.
        let diags = lint_file(
            &*engine(),
            "function f(y)\nx = 0;\nfor i = 1:3\n  x = 1;\nend\nif x\n  a = 1;\nelseif y\n  a = 2;\nend\ndisp(a)\nend\n",
        );
        assert!(unrch_lines(&diags).is_empty(), "got: {diags:?}");
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

#[cfg(test)]
mod literal_condition_tests {
    use super::*;
    use crate::test_util::lint_file;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    fn unrch(src: &str) -> Vec<(usize, usize)> {
        lint_file(&*engine(), src)
            .iter()
            .filter(|d| d.rule_id == "UNRCH")
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn unrch_after_constant_true_if_that_returns() {
        // Probed: `if 1, return, end` (line 2), `if true` and `if ~0` (lines 2-4).
        assert_eq!(
            unrch("function f()\nif 1, return, end\ndisp(1)\nend\n"),
            vec![(3, 1)]
        );
        assert_eq!(
            unrch("function f()\nif true\n return\nend\ndisp(2)\nend\n"),
            vec![(5, 1)]
        );
        assert_eq!(
            unrch("function f()\nif ~0\n return\nend\ndisp(1)\nend\n"),
            vec![(5, 1)]
        );
    }

    #[test]
    fn unrch_after_constant_true_if_else_is_dead() {
        // Probed-consistent: the else of a constant true `if` is dead, and the code
        // after an `if` whose every live branch returns is unreachable (line 7).
        let found = unrch("function f()\nif 1\n return\nelse\n disp(1)\nend\ndisp(2)\nend\n");
        assert!(found.iter().any(|&(line, _)| line == 7), "{found:?}");
    }

    #[test]
    fn unrch_silent_after_constant_true_if_that_falls_through() {
        // Probed (unrch_one_nobr.m): `if 1` without a return leaves the rest reachable.
        assert!(unrch("function f()\nif 1\n disp(1)\nend\ndisp(2)\nend\n").is_empty());
    }

    #[test]
    fn unrch_silent_after_if_on_unknown_condition() {
        // Probed (unrch_neg.m).
        assert!(unrch("function f(x)\nif ~x\n return\nend\ndisp(1)\nend\n").is_empty());
    }

    #[test]
    fn unrch_at_catch_of_empty_try() {
        // Probed (try_empty.m, try_c3.m): the `catch` at column 6 is unreachable.
        assert_eq!(unrch("try, catch end\n"), vec![(1, 6)]);
        assert_eq!(unrch("try, catch ME, end\n"), vec![(1, 6)]);
    }

    #[test]
    fn unrch_silent_for_try_with_a_statement() {
        // Probed (try_multi.m).
        assert!(unrch("try\n x = 1;\ncatch, end\n").is_empty());
    }

    #[test]
    fn unrch_after_while_one_without_break() {
        // Probed (u08, u18, x13): `while 1` is an infinite loop, as `while true` is.
        assert_eq!(
            unrch("function f()\nwhile 1\n    disp(1);\nend\ndisp(2);\nend\n"),
            vec![(5, 1)]
        );
        assert!(unrch("function f()\nwhile 1\n    break;\nend\ndisp(1);\nend\n").is_empty());
    }

    #[test]
    fn unrch_after_exit_and_quit() {
        // Probed (f33, x01, x02, x03): bare `exit;` and `quit;` and calls with arguments.
        assert_eq!(unrch("function f()\nexit;\ndisp(1);\nend\n"), vec![(3, 1)]);
        assert_eq!(unrch("function f()\nquit;\ndisp(1);\nend\n"), vec![(3, 1)]);
        assert_eq!(
            unrch("function f()\nquit(0);\ndisp(1);\nend\n"),
            vec![(3, 1)]
        );
    }

    #[test]
    fn unrch_after_if_where_every_branch_errors() {
        // Probed (x05).
        assert_eq!(
            unrch(
                "function f(c)\nif c\n    error('a');\nelse\n    error('b');\nend\ndisp(1);\nend\n"
            ),
            vec![(7, 1)]
        );
    }

    #[test]
    fn unrch_after_try_whose_body_errors_and_catch_leaves() {
        // Probed (x06, x15, f31): the catch runs and leaves, so the next statement is dead.
        assert_eq!(
            unrch("function f()\ntry\n    error('x');\ncatch e\n    rethrow(e);\nend\ndisp(1);\nend\n"),
            vec![(7, 1)]
        );
        assert_eq!(
            unrch("function f()\ntry\n    error('x');\ncatch\n    return;\nend\ndisp(1);\nend\n"),
            vec![(7, 1)]
        );
        // Probed (x07): the body does not end, so the next statement is reachable.
        assert!(unrch(
            "function f()\ntry\n    disp(1);\ncatch\n    error('x');\nend\ndisp(2);\nend\n"
        )
        .is_empty());
    }

    #[test]
    fn unrch_at_catch_when_try_body_returns_first() {
        // Probed (x16): nothing in the body can throw before the return.
        assert_eq!(
            unrch("function f()\ntry\n    return;\ncatch\n    disp(1);\nend\ndisp(2);\nend\n"),
            vec![(4, 1)]
        );
    }

    #[test]
    fn unrch_dead_break_does_not_end_a_loop() {
        // Probed (x04): a `break` in `if 0` never runs, so the loop never ends.
        let mut got =
            unrch("function f()\nwhile 1\n    if 0\n        break;\n    end\nend\ndisp(1);\nend\n");
        got.sort();
        assert_eq!(got, vec![(4, 9), (7, 1)]);
    }

    #[test]
    fn unrch_dead_elseif_and_clause_bodies_after_false_conditions() {
        // Probed (x11): the `if 0` and `elseif 0` bodies are dead; the `else` runs.
        let mut got = unrch(
            "function f()\nif 0\n    disp(1);\nelseif 0\n    disp(2);\nelse\n    disp(3);\nend\nend\n",
        );
        got.sort();
        assert_eq!(got, vec![(3, 5), (5, 5)]);
    }
}
