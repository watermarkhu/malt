//! MP_STAT_6: a `switch` without an `otherwise`.

use malt_core::{Config, Diagnostic, FileContext};

use super::walk;
use crate::monkeyproof::finding;

/// Flag a `switch` statement with no `otherwise` branch. The finding covers the
/// `switch` keyword.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    walk(ctx.tree.root_node(), |node, _| {
        if node.kind() != "switch_statement" {
            return;
        }
        let mut cursor = node.walk();
        let has_otherwise = node
            .children(&mut cursor)
            .any(|c| c.kind() == "otherwise_clause");
        if !has_otherwise {
            let range = node.child(0).map_or(node.byte_range(), |c| c.byte_range());
            out.push(finding(
                ctx.source,
                "MP_STAT_6",
                range,
                "Add an otherwise branch to this switch.",
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_a_switch_without_otherwise() {
        let source = "switch a\n  case 1\n    x = 1;\nend\n";
        assert_eq!(hits("MP_STAT_6", source, ""), vec![(1, 1)]);
    }

    #[test]
    fn leaves_a_complete_switch_alone() {
        let source = "switch a\n  case 1\n    x = 1;\n  otherwise\n    x = 0;\nend\n";
        assert!(hits("MP_STAT_6", source, "").is_empty());
    }

    #[test]
    fn an_empty_otherwise_counts() {
        let source = "switch a\n  case 1\n    x = 1;\n  otherwise\nend\n";
        assert!(hits("MP_STAT_6", source, "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_6", "switch\ncase\n", "");
    }
}
