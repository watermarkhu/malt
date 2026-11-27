//! `MP_LAYOUT_7`: comments do not justify poorly written code.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;

use super::text::Text;

/// Parameters of `MP_LAYOUT_7`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    /// Words and phrases that mark such a comment. Matched case-sensitively as whole words.
    words: Vec<String>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            words: ["TODO", "FIXME", "HACK", "XXX", "KLUDGE"]
                .map(String::from)
                .to_vec(),
        }
    }
}

/// Report a comment that contains one of the marker words (`TODO`, `FIXME`, `HACK`, `XXX`,
/// `KLUDGE` by default; set `words` to change the list). A word counts when it stands alone:
/// `TODOS` and `MY_TODO` do not match, `TODO:` does. Matching is case-sensitive, so list
/// the lowercase spelling too if you want it. Only comments are searched, never strings.
///
/// This is a list of markers, not a judgement of the code: a comment that excuses bad code
/// in other words is not caught.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_LAYOUT_7");
    let text = Text::new(ctx.source, ctx.tree.root_node());
    let word_byte = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    for comment in &text.comments {
        let body = &ctx.source[comment.clone()];
        let mut found = Vec::new();
        for word in params.words.iter().filter(|w| !w.is_empty()) {
            for (at, _) in body.match_indices(word.as_str()) {
                let before = body.as_bytes()[..at].last().copied();
                let after = body.as_bytes().get(at + word.len()).copied();
                if before.is_some_and(word_byte) || after.is_some_and(word_byte) {
                    continue;
                }
                found.push((at, word));
            }
        }
        found.sort();
        for (at, word) in found {
            let start = comment.start + at;
            out.push(text.finding(
                "MP_LAYOUT_7",
                start..start + word.len(),
                format!("Fix the code instead of leaving a `{word}` comment."),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    #[test]
    fn markers_in_comments_are_reported_at_the_word() {
        let source = "x = 2;\ny = 3 * x; % TODO: take z into account.\n% FIXME ugly\n";
        let d = hits("MP_LAYOUT_7", source, "");
        let at: Vec<_> = d.iter().map(|d| (d.line, d.column)).collect();
        assert_eq!(at, [(2, 14), (3, 3)], "{d:?}");
    }

    #[test]
    fn markers_in_strings_and_longer_words_stay_silent() {
        for source in [
            "s = 'TODO';\n",
            "% TODOS are tracked elsewhere\n",
            "% MY_TODO\n",
            "% a todo list\n",
            "% nothing to see\n",
        ] {
            assert!(hits("MP_LAYOUT_7", source, "").is_empty(), "{source:?}");
        }
    }

    #[test]
    fn every_line_of_a_comment_run_is_searched() {
        let d = hits("MP_LAYOUT_7", "% one\n% HACK here\n% XXX there\n", "");
        let lines: Vec<_> = d.iter().map(|d| d.line).collect();
        assert_eq!(lines, [2, 3], "{d:?}");
    }

    #[test]
    fn the_word_list_is_a_parameter() {
        let toml = "[lint.rules.MP_LAYOUT_7]\nwords = [\"ugly code\"]\n";
        let source = "% this is ugly code\n% TODO later\n";
        let d = hits("MP_LAYOUT_7", source, toml);
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].line, 1);
    }
}
