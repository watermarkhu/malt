//! `MP_LAYOUT_12`: no whitespace at the end of a line.

use malt_core::{Config, Diagnostic, FileContext};

use super::text::Text;

/// Report spaces and tabs at the end of a line, blank lines holding only whitespace
/// included. Whitespace before a comment on the same line is not at the end of the line.
/// The finding covers the trailing whitespace.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let text = Text::new(ctx.source, ctx.tree.root_node());
    for line in 0..text.line_count() {
        let range = text.line_range(line);
        let content = &ctx.source[range.clone()];
        let trimmed = content.trim_end_matches([' ', '\t']);
        if trimmed.len() < content.len() {
            out.push(text.finding(
                "MP_LAYOUT_12",
                range.start + trimmed.len()..range.end,
                "Remove the whitespace at the end of the line.",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    #[test]
    fn trailing_spaces_are_reported_with_their_position() {
        let d = hits("MP_LAYOUT_12", "x = 1;  \ny = 2;\n", "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (1, 7));
        assert_eq!(d[0].byte_range, 6..8);
    }

    #[test]
    fn whitespace_only_lines_and_tabs_are_reported() {
        let d = hits("MP_LAYOUT_12", "x = 1;\n    \ny = 2;\t\n", "");
        let lines: Vec<_> = d.iter().map(|d| d.line).collect();
        assert_eq!(lines, [2, 3], "{d:?}");
    }

    #[test]
    fn clean_lines_and_crlf_are_fine() {
        assert!(hits("MP_LAYOUT_12", "x = 1; % c\r\ny = 2;\r\n\r\n", "").is_empty());
        assert!(hits("MP_LAYOUT_12", "s = 'a ';\n", "").is_empty());
    }

    #[test]
    fn whitespace_before_a_comment_is_fine() {
        assert!(hits("MP_LAYOUT_12", "x = 1;   % note\n", "").is_empty());
    }

    #[test]
    fn last_line_without_newline() {
        assert_eq!(hits("MP_LAYOUT_12", "x = 1; ", "").len(), 1);
    }
}
