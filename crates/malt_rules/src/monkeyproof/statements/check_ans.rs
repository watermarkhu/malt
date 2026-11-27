//! MP_STAT_22: the workspace variable `ans`.

use malt_core::{Config, Diagnostic, FileContext};

use super::{text, walk};
use crate::monkeyproof::finding;

/// Flag each use of the name `ans`, read or written.
///
/// A field called `ans` (`s.ans`) is not the variable. An expression statement that
/// leaves its value in `ans` without the name appearing is not reported.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    walk(ctx.tree.root_node(), |node, parent| {
        let named_ans = match node.kind() {
            "identifier" | "command_name" => text(node, ctx.source) == "ans",
            _ => false,
        };
        if !named_ans {
            return;
        }
        let is_field = parent.is_some_and(|p| {
            p.kind() == "field_expression" && p.child_by_field_name("object") != Some(node)
        });
        if !is_field {
            out.push(finding(
                ctx.source,
                "MP_STAT_22",
                node.byte_range(),
                "Do not use the variable ans, store the value in a named variable.",
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_reads_and_writes() {
        let source = "x = ans + 1;\nans = 3;\nans\ndisp(ans)\n";
        assert_eq!(
            hits("MP_STAT_22", source, ""),
            vec![(1, 5), (2, 1), (3, 1), (4, 6)]
        );
    }

    #[test]
    fn leaves_fields_and_longer_names_alone() {
        let source = "x = s.ans;\ny = answer + ans2;\n";
        assert!(hits("MP_STAT_22", source, "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_22", "x = ans +\n(", "");
    }
}
