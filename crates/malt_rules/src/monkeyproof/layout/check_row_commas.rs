//! `MP_LAYOUT_9`: the elements of a matrix or cell row are separated by commas.

use malt_core::{Config, Diagnostic, FileContext};

use super::text::{children, walk, Text};

/// Report a gap between two elements of a `[...]` or `{...}` row that has no comma, and a
/// comma that is separated from its element by a space.
///
/// The grammar decides what is a separate element: `[1 -2]` is two elements, `[1 - 2]` is
/// one, and the finding for a missing comma sits on the whitespace between the elements.
/// Rows split over lines with a `;` or a line break are separate rows and need no comma.
/// Commas in argument lists are not looked at.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let text = Text::new(ctx.source, ctx.tree.root_node());
    let bytes = ctx.source.as_bytes();
    walk(ctx.tree.root_node(), |node, _| {
        if node.kind() != "row" {
            return;
        }
        let items = children(node);
        for (index, item) in items.iter().enumerate() {
            if item.kind() != "," || index == 0 {
                continue;
            }
            let previous = &items[index - 1];
            if item.start_byte() == item.end_byte() {
                let start = previous.end_byte();
                let end = item.start_byte().max(start + 1).min(bytes.len());
                out.push(text.finding(
                    "MP_LAYOUT_9",
                    start..end,
                    "Separate the elements of a row with commas.",
                ));
            } else {
                let mut start = item.start_byte();
                while start > 0 && matches!(bytes[start - 1], b' ' | b'\t') {
                    start -= 1;
                }
                let at_line_start = start == 0 || matches!(bytes[start - 1], b'\n' | b'\r');
                if start < item.start_byte() && !at_line_start && start >= previous.end_byte() {
                    out.push(text.finding(
                        "MP_LAYOUT_9",
                        start..item.start_byte(),
                        "Put the comma directly after the element.",
                    ));
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    fn at(source: &str) -> Vec<(usize, usize)> {
        hits("MP_LAYOUT_9", source, "")
            .iter()
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn missing_commas_are_reported_in_each_gap() {
        assert_eq!(at("p = [0 1000 1.5];\n"), [(1, 7), (1, 12)]);
    }

    #[test]
    fn cells_and_strings_are_rows_too() {
        assert_eq!(at("c = {1 'a'};\n"), [(1, 7)]);
        assert_eq!(at("s = ['An ' x ' b'];\n"), [(1, 11), (1, 13)]);
    }

    #[test]
    fn comma_separated_rows_are_fine() {
        assert!(at("p = [0, 1000, 1.5];\nm = [1, 2; 3, 4];\nc = {1, 'a'};\n").is_empty());
    }

    #[test]
    fn a_space_before_the_comma_is_reported() {
        assert_eq!(at("p = [1 , 2];\n"), [(1, 7)]);
    }

    #[test]
    fn binary_operators_and_new_lines_are_not_gaps() {
        assert!(at("p = [1 - 2, 3];\nq = [1, 2\n    3, 4];\n").is_empty());
    }

    #[test]
    fn calls_and_strings_are_left_alone() {
        assert!(at("f(a , b);\ns = '[1 2]';\n").is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        let _ = at("p = [1 2\nq = {1 [\n");
    }
}
