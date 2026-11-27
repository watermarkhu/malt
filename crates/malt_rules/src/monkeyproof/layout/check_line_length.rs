//! `MP_LAYOUT_5`: lines are at most 120 characters long.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;

use super::text::Text;

/// Parameters of `MP_LAYOUT_5`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    /// The longest line allowed, in characters.
    max_length: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self { max_length: 120 }
    }
}

/// Report each line longer than `max_length` characters (default 120), comments included.
/// Characters are counted, not bytes, and a tab counts as one. The finding covers the part
/// of the line past the limit.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_LAYOUT_5");
    let text = Text::new(ctx.source, ctx.tree.root_node());
    for line in 0..text.line_count() {
        let range = text.line_range(line);
        let content = &ctx.source[range.clone()];
        let Some((offset, _)) = content.char_indices().nth(params.max_length) else {
            continue;
        };
        let length = content.chars().count();
        out.push(text.finding(
            "MP_LAYOUT_5",
            range.start + offset..range.end,
            format!(
                "Shorten this line to at most {} characters; it has {length}.",
                params.max_length
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    #[test]
    fn long_line_is_reported_past_the_limit() {
        let source = format!("x = '{}';\n", "a".repeat(130));
        let d = hits("MP_LAYOUT_5", &source, "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (1, 121));
    }

    #[test]
    fn exactly_the_limit_is_fine() {
        let source = format!("x = '{}';\n", "a".repeat(120 - 7));
        assert!(hits("MP_LAYOUT_5", &source, "").is_empty());
    }

    #[test]
    fn comments_count() {
        let source = format!("% {}\n", "word ".repeat(30));
        assert_eq!(hits("MP_LAYOUT_5", &source, "").len(), 1);
    }

    #[test]
    fn characters_are_counted_not_bytes() {
        let source = format!("% {}\n", "é".repeat(100));
        assert!(hits("MP_LAYOUT_5", &source, "").is_empty());
    }

    #[test]
    fn crlf_is_not_part_of_the_line() {
        let source = format!("x = '{}';\r\n", "a".repeat(120 - 7));
        assert!(hits("MP_LAYOUT_5", &source, "").is_empty());
    }

    #[test]
    fn max_length_is_a_parameter() {
        let toml = "[lint.rules.MP_LAYOUT_5]\nmax_length = 20\n";
        let d = hits(
            "MP_LAYOUT_5",
            "x = 'abcdefghijklmnopqrstuvwxyz';\ny = 1;\n",
            toml,
        );
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].column, 21);
    }
}
