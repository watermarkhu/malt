//! `MP_LAYOUT_14`: all `properties` blocks come before all `methods` blocks.

use malt_core::{Config, Diagnostic, FileContext};

use super::text::{children, walk, Text};

/// Report each `properties` block of a class that follows a `methods` block. The finding
/// sits on the `properties` keyword. `events` and `enumeration` blocks are not ordered by this
/// check.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let text = Text::new(ctx.source, ctx.tree.root_node());
    walk(ctx.tree.root_node(), |node, _| {
        if node.kind() != "class_definition" {
            return;
        }
        let mut methods_seen = false;
        for block in children(node) {
            // The keyword token has the kind of its block but no children.
            if block.child_count() == 0 {
                continue;
            }
            match block.kind() {
                "methods" => methods_seen = true,
                "properties" if methods_seen => {
                    let keyword = block.child(0).unwrap_or(block);
                    out.push(text.finding(
                        "MP_LAYOUT_14",
                        keyword.start_byte()..keyword.end_byte(),
                        "Move this properties block before the methods blocks.",
                    ));
                }
                _ => {}
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    #[test]
    fn properties_after_methods_are_reported() {
        let source = "classdef A\n    properties\n        P\n    end\n    methods\n    end\n    properties (Constant)\n        Q = 1\n    end\nend\n";
        let d = hits("MP_LAYOUT_14", source, "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (7, 5));
    }

    #[test]
    fn every_late_block_is_reported() {
        let source = "classdef A\n    methods\n    end\n    properties\n    end\n    properties (Hidden)\n    end\nend\n";
        assert_eq!(hits("MP_LAYOUT_14", source, "").len(), 2);
    }

    #[test]
    fn the_right_order_is_fine() {
        let source = "classdef A\n    properties\n    end\n    properties (Constant)\n    end\n    methods\n    end\n    methods (Static)\n    end\nend\n";
        assert!(hits("MP_LAYOUT_14", source, "").is_empty());
    }

    #[test]
    fn events_do_not_count() {
        let source = "classdef A\n    methods\n    end\n    events\n        E\n    end\nend\n";
        assert!(hits("MP_LAYOUT_14", source, "").is_empty());
    }

    #[test]
    fn functions_and_scripts_are_fine() {
        assert!(hits("MP_LAYOUT_14", "function f()\nend\n", "").is_empty());
    }
}
