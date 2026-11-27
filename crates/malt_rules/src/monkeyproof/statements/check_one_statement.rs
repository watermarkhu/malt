//! MP_STAT_4: more than one statement on a line.

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::{statements_of, walk};
use crate::monkeyproof::finding;

/// Whether nothing but blanks separates `previous` from `start` on one line.
fn same_line(source: &str, previous: Node, start: usize) -> bool {
    if matches!(previous.kind(), "\n" | "\r") || previous.end_byte() > start {
        return false;
    }
    source
        .get(previous.end_byte()..start)
        .is_some_and(|gap| !gap.contains(['\n', '\r']))
}

/// Flag a statement that starts on the line where another statement, or the header of
/// the construct around it, already starts. One finding per line, at the first extra
/// statement.
///
/// This covers `a = 1; b = 2`, `persistent x, y` and one-line blocks such as
/// `for k = 1:3, x = k; end`.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let mut starts: Vec<(usize, Node)> = Vec::new();
    walk(ctx.tree.root_node(), |node, _| {
        if !matches!(node.kind(), "block" | "source_file") {
            return;
        }
        let statements = statements_of(node);
        let mut previous = if node.kind() == "block" {
            node.prev_sibling()
        } else {
            None
        };
        for statement in statements {
            if statement.kind() == "ERROR" {
                previous = Some(statement);
                continue;
            }
            if previous.is_some_and(|prev| same_line(ctx.source, prev, statement.start_byte())) {
                starts.push((statement.start_byte(), statement));
            }
            previous = Some(statement);
        }
    });
    starts.sort_by_key(|(start, _)| *start);
    let mut last_row = None;
    for (_, statement) in starts {
        let row = statement.start_position().row;
        if last_row == Some(row) {
            continue;
        }
        last_row = Some(row);
        out.push(finding(
            ctx.source,
            "MP_STAT_4",
            statement.byte_range(),
            "Put each statement on its own line.",
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_two_statements_on_a_line() {
        let source = "a = 1; b = 2;\nc = 3;\n";
        assert_eq!(hits("MP_STAT_4", source, ""), vec![(1, 8)]);
    }

    #[test]
    fn flags_one_line_blocks_once() {
        let source = "for k = 1:3, x = x + k; disp(x); end\n";
        assert_eq!(hits("MP_STAT_4", source, ""), vec![(1, 14)]);
    }

    #[test]
    fn flags_a_second_persistent_name() {
        let source = "function f()\n  persistent x, y\nend\n";
        assert_eq!(hits("MP_STAT_4", source, "").len(), 1);
    }

    #[test]
    fn flags_a_statement_after_a_clause_keyword() {
        let source = "if a, x = 1; else, x = 2; end\n";
        assert_eq!(hits("MP_STAT_4", source, ""), vec![(1, 7)]);
    }

    #[test]
    fn leaves_one_statement_per_line_alone() {
        let source =
            "a = 1;\nb = 2; % note\nif a\n  c = 3;\nelse\n  c = 4;\nend\nx = [1, 2; 3, 4];\n";
        assert!(hits("MP_STAT_4", source, "").is_empty());
    }

    #[test]
    fn a_continued_statement_is_one_statement() {
        let source = "x = 1 + ...\n    2;\ny = 3;\n";
        assert!(hits("MP_STAT_4", source, "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_4", "a = 1; b = (\nif x, y\n", "");
    }
}
