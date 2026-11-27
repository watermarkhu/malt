//! `MP_LAYOUT_8`: the equals signs of a block of assignments are aligned.

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::text::{children, walk, Text};

/// One assignment of a block: the `=` token and its column in characters.
struct Member {
    equals: std::ops::Range<usize>,
    column: usize,
}

/// Report the `=` of an assignment that is not in the column of the others in its block. A
/// block is a run of assignments on consecutive lines, each starting its own line, in the same
/// body; a blank line, a comment line or any other statement ends it. The column to match is
/// the rightmost one, so padding the shorter names fixes a block.
///
/// Blocks of one assignment, assignments whose `=` is not on their first line, and blocks
/// with a tab before an `=` are skipped.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let text = Text::new(ctx.source, ctx.tree.root_node());
    walk(ctx.tree.root_node(), |node, _| {
        if !matches!(node.kind(), "source_file" | "block") {
            return;
        }
        let mut group: Vec<Member> = Vec::new();
        let mut last_line = 0usize;
        for child in children(node) {
            if !child.is_named() || child.kind() == "comment" {
                continue;
            }
            let member = member_of(&text, child);
            let start_line = child.start_position().row;
            let adjacent = !group.is_empty() && start_line == last_line + 1;
            match member {
                Some(member) => {
                    if !adjacent {
                        flush(&text, &mut group, out);
                    }
                    group.push(member);
                }
                None => flush(&text, &mut group, out),
            }
            last_line = child.end_position().row;
        }
        flush(&text, &mut group, out);
    });
}

/// The `=` of `node` when it is a one-line-header assignment that starts its line.
fn member_of(text: &Text, node: Node) -> Option<Member> {
    if node.kind() != "assignment" || node.has_error() {
        return None;
    }
    let start = node.start_byte();
    let line_start = text.line_starts[text.line_of(start)];
    let before = &text.source[line_start..start];
    if !before.chars().all(|c| c == ' ') {
        return None;
    }
    let equals = children(node).into_iter().find(|c| c.kind() == "=")?;
    if equals.start_position().row != node.start_position().row {
        return None;
    }
    let prefix = &text.source[line_start..equals.start_byte()];
    if prefix.contains('\t') {
        return None;
    }
    Some(Member {
        equals: equals.start_byte()..equals.end_byte(),
        column: prefix.chars().count(),
    })
}

/// Report the members of `group` off the rightmost column, then empty it.
fn flush(text: &Text, group: &mut Vec<Member>, out: &mut Vec<Diagnostic>) {
    if group.len() > 1 {
        let target = group.iter().map(|m| m.column).max().unwrap_or(0);
        for member in group.iter().filter(|m| m.column != target) {
            out.push(text.finding(
                "MP_LAYOUT_8",
                member.equals.clone(),
                format!(
                    "Align this `=` with the others in the block, at column {}.",
                    target + 1
                ),
            ));
        }
    }
    group.clear();
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    fn lines(source: &str) -> Vec<(usize, usize)> {
        hits("MP_LAYOUT_8", source, "")
            .iter()
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn unaligned_block_is_reported_per_assignment() {
        let source = "carWeight = 1.7 * 600;\nnWheels = 2 + 2;\nmaximumSpeed = 190;\n";
        assert_eq!(lines(source), [(1, 11), (2, 9)]);
    }

    #[test]
    fn aligned_block_is_fine() {
        let source =
            "carWeight       = 1.7 * 600;\nnWheels         = 2 + 2;\nmaximumSpeed    = 190;\n";
        assert!(lines(source).is_empty());
    }

    #[test]
    fn blank_lines_comments_and_other_statements_end_the_block() {
        assert!(lines("a = 1;\n\nbbb = 2;\n").is_empty());
        assert!(lines("a = 1;\n% note\nbbb = 2;\n").is_empty());
        assert!(lines("a = 1;\ndisp(a);\nbbb = 2;\n").is_empty());
    }

    #[test]
    fn single_assignment_and_shared_lines_are_fine() {
        assert!(lines("a = 1;\n").is_empty());
        assert!(lines("a = 1; bbb = 2;\n").is_empty());
    }

    #[test]
    fn blocks_inside_a_function_are_found() {
        let source = "function f()\n    a = 1;\n    bbb = 2;\nend\n";
        assert_eq!(lines(source), [(2, 7)]);
    }

    #[test]
    fn trailing_comments_do_not_matter() {
        let source = "a   = 1; % one\nbbb = 2; % two\n";
        assert!(lines(source).is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        let _ = lines("a = (1;\nbbb = 2;\nc =\n");
    }
}
