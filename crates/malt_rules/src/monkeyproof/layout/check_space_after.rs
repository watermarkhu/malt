//! `MP_LAYOUT_4`: commas, semicolons and keywords are followed by a space.

use malt_core::{Config, Diagnostic, FileContext};

use super::text::{walk, Text};

/// Keywords that take an expression after them.
const KEYWORDS: &[&str] = &["if", "elseif", "while", "for", "parfor", "switch", "case"];

/// Statements the keywords start.
const OWNERS: &[&str] = &[
    "if_statement",
    "elseif_clause",
    "while_statement",
    "for_statement",
    "switch_statement",
    "case_clause",
];

/// Report a comma or semicolon, and a keyword of `if`, `elseif`, `while`, `for`, `parfor`,
/// `switch` and `case`, that is directly followed by something other than a space.
///
/// Nothing is reported at the end of a line, before a comment or a `...` continuation, before
/// a closing bracket or another separator. Commas and semicolons inside strings and comments
/// are not looked at. A semicolon in a matrix (`[1 2;3 4]`) is checked like any other. The
/// space before a comma is not part of this check.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let text = Text::new(ctx.source, ctx.tree.root_node());
    let code = &text.code;
    let follows_ok = |end: usize, separators: bool| match code.get(end) {
        None => true,
        Some(b' ' | b'\t' | b'\n' | b'\r') => true,
        Some(b')' | b']' | b'}') => separators,
        Some(b',' | b';') => separators,
        Some(_) => false,
    };
    for (index, byte) in code.iter().enumerate() {
        if matches!(byte, b',' | b';') && !follows_ok(index + 1, true) {
            let name = if *byte == b',' { "comma" } else { "semicolon" };
            out.push(text.finding(
                "MP_LAYOUT_4",
                index..index + 1,
                format!("Follow the {name} with a space."),
            ));
        }
    }
    walk(ctx.tree.root_node(), |node, _| {
        if !KEYWORDS.contains(&node.kind()) || node.is_named() || node.child_count() > 0 {
            return;
        }
        let owned = node.parent().is_some_and(|p| OWNERS.contains(&p.kind()));
        // `;` and `,` after a keyword (`if x, ...`) cannot follow it directly in a valid
        // statement, so only whitespace and the end of a line count here.
        if owned && !follows_ok(node.end_byte(), false) && code.get(node.end_byte()) != Some(&b';')
        {
            out.push(text.finding(
                "MP_LAYOUT_4",
                node.start_byte()..node.end_byte(),
                format!("Follow the keyword `{}` with a space.", node.kind()),
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    fn columns(source: &str) -> Vec<usize> {
        hits("MP_LAYOUT_4", source, "")
            .iter()
            .map(|d| d.column)
            .collect()
    }

    #[test]
    fn keyword_glued_to_its_condition_is_reported() {
        assert_eq!(columns("while(x)\nend\n"), [1]);
        assert_eq!(columns("if(x)\nend\n"), [1]);
        assert_eq!(columns("for(i = 1:3)\nend\n"), [1]);
        assert_eq!(columns("switch(x)\n    case{1}\n    end\nend\n").len(), 2);
    }

    #[test]
    fn keyword_with_a_space_is_fine() {
        assert!(columns("while ismember(x, [3, 5])\n    x = x + 2;\nend\n").is_empty());
        assert!(columns("try\n    a = 1;\ncatch\n    a = 2;\nend\n").is_empty());
    }

    #[test]
    fn commas_need_a_space() {
        assert_eq!(columns("y = [3,5, 7];\n"), [7]);
        assert_eq!(columns("f(a,b);\n"), [4]);
        assert_eq!(columns("function [a,b] = f(x,y)\nend\n"), [12, 21]);
    }

    #[test]
    fn semicolons_need_a_space_unless_the_statement_ends() {
        assert_eq!(columns("a = 1;b = 2;\n"), [6]);
        assert_eq!(columns("m = [1, 2;3, 4];\n"), [10]);
        assert!(columns("a = 1;\nb = 2; % c\nm = [1, 2; 3, 4];\n").is_empty());
    }

    #[test]
    fn end_of_statement_comment_and_continuation_are_fine() {
        assert!(columns("a = 1;% c\nb = [1, ...\n    2];\nc = [1,... x\n    2];\n").is_empty());
        assert!(columns("m = [1, 2;];\nx = {1,};\n").is_empty());
    }

    #[test]
    fn strings_and_comments_are_masked() {
        assert!(columns("s = 'a,b;c';  % x,y;z\nt = \"a,b\";\n").is_empty());
        assert_eq!(columns("c = {'a','b'};\n"), [9]);
    }

    #[test]
    fn broken_code_does_not_panic() {
        let _ = columns("if(x\nwhile(((\n");
    }
}
