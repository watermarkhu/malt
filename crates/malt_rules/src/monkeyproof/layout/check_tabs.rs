//! `MP_LAYOUT_13`: no tab characters.

use malt_core::{Config, Diagnostic, FileContext};

use super::text::Text;

/// Report the first tab of each line that has one, in code and in comments. A tab inside a
/// string literal is data, not layout, and is left alone.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let text = Text::new(ctx.source, ctx.tree.root_node());
    for line in 0..text.line_count() {
        let range = text.line_range(line);
        let content = &ctx.source[range.clone()];
        let tab = content
            .match_indices('\t')
            .map(|(at, _)| range.start + at)
            .find(|at| !text.in_string(*at));
        if let Some(at) = tab {
            out.push(text.finding(
                "MP_LAYOUT_13",
                at..at + 1,
                "Use spaces instead of a tab character.",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    #[test]
    fn a_tab_is_reported_once_per_line() {
        let d = hits("MP_LAYOUT_13", "if x\n\t\ty = 1;\nend\nz = 1;\t% c\n", "");
        let at: Vec<_> = d.iter().map(|d| (d.line, d.column)).collect();
        assert_eq!(at, [(2, 1), (4, 7)], "{d:?}");
    }

    #[test]
    fn tabs_in_comments_count() {
        assert_eq!(hits("MP_LAYOUT_13", "% a\tb\n", "").len(), 1);
    }

    #[test]
    fn tabs_in_strings_do_not() {
        assert!(hits("MP_LAYOUT_13", "s = \"a\tb\";\nt = 'c\td';\n", "").is_empty());
    }

    #[test]
    fn spaces_are_fine() {
        assert!(hits("MP_LAYOUT_13", "if x\n    y = 1;\nend\n", "").is_empty());
    }
}
