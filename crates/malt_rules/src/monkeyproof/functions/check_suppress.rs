//! FCNS-5: prevent or suppress Code Analyzer messages.
//!
//! The standard asks that a message is fixed or, when that is not possible, suppressed
//! on the line it occurs on. The part of that a single file shows is the broad
//! suppression: a `%#ok<*ID>` directive, where the `*` extends the suppression of `ID`
//! to the whole file. Each such directive is reported. A plain `%#ok` or `%#ok<ID>` on a
//! line is not reported, and neither are other pragmas such as `%#codegen`. Whether a
//! suppression has a reason in the comment is not checked.

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_5";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    visit(ctx.tree.root_node(), ctx.source, out);
}

fn visit(node: Node, source: &str, out: &mut Vec<Diagnostic>) {
    if node.kind() == "comment" {
        if let Some(text) = source.get(node.byte_range()) {
            if has_file_wide_directive(text) {
                out.push(finding(
                    source,
                    ID,
                    node.byte_range(),
                    "This directive suppresses a message in the whole file. Suppress it on the line that needs it.",
                ));
            }
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, source, out);
    }
}

/// Whether the comment holds a `#ok<...>` directive with an entry that starts with `*`.
fn has_file_wide_directive(comment: &str) -> bool {
    let mut rest = comment;
    while let Some(at) = rest.find("#ok<") {
        // `#ok` must follow the comment marker: `%#ok<..>` or `% #ok<..>`.
        let before = rest[..at].trim_end_matches([' ', '\t']);
        let directive = before.ends_with('%') || before.is_empty();
        let after = &rest[at + 4..];
        let Some(close) = after.find('>') else {
            return false;
        };
        if directive
            && after[..close]
                .split(',')
                .any(|entry| entry.trim_start().starts_with('*'))
        {
            return true;
        }
        rest = &after[close..];
    }
    false
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::run;
    use super::ID;

    #[test]
    fn a_star_directive_fires_at_the_comment() {
        let d = run(ID, "x = 1; %#ok<*NASGU> keep\n", "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (1, 8));
    }

    #[test]
    fn a_star_among_several_ids_fires() {
        assert_eq!(run(ID, "x = 1; % #ok<AGROW,*NASGU>\n", "").len(), 1);
    }

    #[test]
    fn line_directives_and_pragmas_are_fine() {
        assert!(run(ID, "x = 1; %#ok<NASGU>\ny = 2; %#ok\n%#codegen\n", "").is_empty());
    }

    #[test]
    fn text_that_only_mentions_a_directive_is_fine() {
        assert!(run(ID, "% see the docs for ok<*ID> and so on\n", "").is_empty());
        assert!(run(ID, "x = '%#ok<*ID>';\n", "").is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(ID, "x = (; %#ok<*A\n", "");
    }
}
