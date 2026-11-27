//! MP_STAT_11: global variables.

use malt_core::{Config, Diagnostic, FileContext};

use super::walk;
use crate::monkeyproof::finding;

/// Flag each `global` declaration. The finding covers the declaration with its names.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    walk(ctx.tree.root_node(), |node, _| {
        if node.kind() == "global_operator" {
            out.push(finding(
                ctx.source,
                "MP_STAT_11",
                node.byte_range(),
                "Do not use global variables, pass the values as arguments.",
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_a_global_declaration() {
        let source = "function f()\n  global A B\n  persistent C\nend\n";
        assert_eq!(hits("MP_STAT_11", source, ""), vec![(2, 3)]);
    }

    #[test]
    fn a_name_containing_global_is_fine() {
        assert!(hits("MP_STAT_11", "isglobal = 1;\nglobalx(2) = 3;\n", "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_11", "global (\n", "");
    }
}
