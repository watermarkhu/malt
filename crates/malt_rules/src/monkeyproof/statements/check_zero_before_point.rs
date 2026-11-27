//! MP_STAT_21: a number written without a zero before the decimal point.

use malt_core::{Config, Diagnostic, FileContext};

use super::{text, walk};
use crate::monkeyproof::finding;

/// Flag each number literal that starts with the decimal point (`.5`, `.25e3`).
///
/// Hexadecimal and binary literals never start with a point, so they are not affected.
/// The finding covers the literal.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    walk(ctx.tree.root_node(), |node, _| {
        if node.kind() == "number" && text(node, ctx.source).starts_with('.') {
            out.push(finding(
                ctx.source,
                "MP_STAT_21",
                node.byte_range(),
                "Write a zero before the decimal point.",
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_a_leading_point() {
        assert_eq!(
            hits("MP_STAT_21", "x = .5;\ny = 2 + .25e3;\n", ""),
            vec![(1, 5), (2, 9)]
        );
    }

    #[test]
    fn leaves_other_numbers_and_fields_alone() {
        let source = "x = 0.5;\ny = 1.;\nz = s.a;\nw = [1 2.5 3e-2 0x1F];\n";
        assert!(hits("MP_STAT_21", source, "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_21", "x = (.5\nif\n", "");
    }
}
