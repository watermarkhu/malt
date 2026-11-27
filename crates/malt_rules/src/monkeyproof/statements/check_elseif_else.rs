//! MP_STAT_24: an `if` with an `elseif` and no `else`.

use malt_core::{Config, Diagnostic, FileContext};

use super::walk;
use crate::monkeyproof::finding;

/// Flag an `if` statement that has at least one `elseif` branch and no `else` branch.
/// The finding covers the `if` keyword.
///
/// An `if` with no `elseif` is the business of `MP_STAT_5`.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    walk(ctx.tree.root_node(), |node, _| {
        if node.kind() != "if_statement" {
            return;
        }
        let mut cursor = node.walk();
        let kinds: Vec<&str> = node.children(&mut cursor).map(|c| c.kind()).collect();
        if kinds.contains(&"elseif_clause") && !kinds.contains(&"else_clause") {
            let range = node.child(0).map_or(node.byte_range(), |c| c.byte_range());
            out.push(finding(
                ctx.source,
                "MP_STAT_24",
                range,
                "Add an else branch to this if with elseif.",
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_elseif_without_else() {
        let source = "if a\n  x = 1;\nelseif b\n  x = 2;\nend\n";
        assert_eq!(hits("MP_STAT_24", source, ""), vec![(1, 1)]);
    }

    #[test]
    fn leaves_complete_and_plain_ifs_alone() {
        let source =
            "if a\n  x = 1;\nelseif b\n  x = 2;\nelse\n  x = 3;\nend\nif c\n  y = 1;\nend\n";
        assert!(hits("MP_STAT_24", source, "").is_empty());
    }

    #[test]
    fn each_nested_if_is_judged_on_its_own() {
        let source =
            "if a\n  if b\n    x = 1;\n  elseif c\n    x = 2;\n  end\nelse\n  x = 3;\nend\n";
        assert_eq!(hits("MP_STAT_24", source, ""), vec![(2, 3)]);
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_24", "if a\nelseif\n", "");
    }
}
