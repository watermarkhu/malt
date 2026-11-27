//! `MP_LAYOUT_1`: block comments (`%{ ... %}`) are not used.

use malt_core::{Config, Diagnostic, FileContext};

use super::text::Text;

/// Report each `%{` that opens a block comment: a line with nothing else on it.
///
/// Only the opening line is reported, once per block. A `%{` that follows code or other
/// comment text on its line is an ordinary comment in MATLAB and is left alone.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let text = Text::new(ctx.source, ctx.tree.root_node());
    for comment in &text.comments {
        let first = text.line_of(comment.start);
        let last = text.line_of(comment.end.saturating_sub(1).max(comment.start));
        for line in first..=last {
            let range = text.line_range(line);
            let content = &ctx.source[range.clone()];
            if content.trim() != "%{" {
                continue;
            }
            let start = range.start + content.find("%{").unwrap_or(0);
            if text.in_comment(start) {
                out.push(text.finding(
                    "MP_LAYOUT_1",
                    start..start + 2,
                    "Use line comments starting with `%` instead of a `%{ %}` block.",
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    #[test]
    fn block_comment_is_reported_at_its_opening() {
        let d = hits("MP_LAYOUT_1", "x = 1;\n%{\nold text\n%}\ny = 2;\n", "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (2, 1));
    }

    #[test]
    fn indented_block_comment_is_reported() {
        let d = hits(
            "MP_LAYOUT_1",
            "function f()\n    %{\n    a\n    %}\nend\n",
            "",
        );
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (2, 5));
    }

    #[test]
    fn block_comment_after_a_line_comment_is_reported() {
        let d = hits("MP_LAYOUT_1", "% intro\n%{\na\n%}\n", "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].line, 2);
    }

    #[test]
    fn lookalikes_stay_silent() {
        for source in [
            "x = 1; %{ not a block\n",
            "% %{\n",
            "s = '%{';\n",
            "% plain comment\n",
            "%{ text on the same line\n%}\n",
        ] {
            assert!(hits("MP_LAYOUT_1", source, "").is_empty(), "{source:?}");
        }
    }

    #[test]
    fn two_blocks_are_two_findings() {
        let d = hits("MP_LAYOUT_1", "%{\na\n%}\nx = 1;\n%{\nb\n%}\n", "");
        assert_eq!(d.len(), 2, "{d:?}");
    }
}
