//! MP_STAT_25: instantiating an abstract class.

use std::collections::HashSet;

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::{call_identifier, is_member, text, walk};
use crate::monkeyproof::finding;

/// Whether an `attributes` node switches `Abstract` on: `(Abstract)`, `(Abstract = true)`.
fn has_abstract(attributes: Node, source: &str) -> bool {
    let mut cursor = attributes.walk();
    let found = attributes
        .named_children(&mut cursor)
        .filter(|attribute| attribute.kind() == "attribute")
        .any(|attribute| {
            let mut inner = attribute.walk();
            let mut parts = attribute.named_children(&mut inner);
            parts
                .next()
                .is_some_and(|name| text(name, source) == "Abstract")
                && parts
                    .next()
                    .is_none_or(|value| text(value, source) != "false")
        });
    found
}

/// Whether a `class_definition` is abstract: it carries `(Abstract)` itself or has a
/// `methods (Abstract)` block.
fn is_abstract(class: Node, source: &str) -> bool {
    let mut cursor = class.walk();
    let found = class.children(&mut cursor).any(|child| match child.kind() {
        "attributes" => has_abstract(child, source),
        "methods" => {
            let mut inner = child.walk();
            let found = child
                .children(&mut inner)
                .any(|c| c.kind() == "attributes" && has_abstract(c, source));
            found
        }
        _ => false,
    });
    found
}

/// Flag a call of the constructor of an abstract class that is defined in the same file.
/// The finding covers the call.
///
/// A class counts as abstract when its `classdef` has the `Abstract` attribute or it has a
/// `methods (Abstract)` block. Classes defined in other files are not known to this check.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let root = ctx.tree.root_node();
    let mut abstract_classes: HashSet<&str> = HashSet::new();
    walk(root, |node, _| {
        if node.kind() == "class_definition" && is_abstract(node, ctx.source) {
            if let Some(name) = node.child_by_field_name("name") {
                abstract_classes.insert(text(name, ctx.source));
            }
        }
    });
    if abstract_classes.is_empty() {
        return;
    }
    walk(root, |node, parent| {
        if node.kind() != "function_call" || parent.is_some_and(|p| is_member(node, p)) {
            return;
        }
        if let Some(name) = call_identifier(node, ctx.source) {
            if abstract_classes.contains(name) {
                out.push(finding(
                    ctx.source,
                    "MP_STAT_25",
                    node.byte_range(),
                    format!("Do not instantiate the abstract class {name}."),
                ));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_the_constructor_of_an_abstract_class() {
        let source = "classdef (Abstract) Shape\n  methods (Static)\n    function o = make()\n      o = Shape(3);\n    end\n  end\nend\n";
        assert_eq!(hits("MP_STAT_25", source, ""), vec![(4, 11)]);
    }

    #[test]
    fn a_class_with_abstract_methods_is_abstract() {
        let source = "classdef Shape\n  methods (Abstract)\n    area(obj)\n  end\n  methods (Static)\n    function o = make()\n      o = Shape();\n    end\n  end\nend\n";
        assert_eq!(hits("MP_STAT_25", source, ""), vec![(7, 11)]);
    }

    #[test]
    fn concrete_classes_and_static_calls_are_fine() {
        let source = "classdef Shape\n  methods (Static)\n    function o = make()\n      o = Shape(3);\n    end\n  end\nend\n";
        assert!(hits("MP_STAT_25", source, "").is_empty());
        let source = "classdef (Abstract) Shape\n  methods (Static)\n    function o = make()\n      o = Shape.build(3);\n      p = Other(1);\n    end\n  end\nend\n";
        assert!(hits("MP_STAT_25", source, "").is_empty());
    }

    #[test]
    fn abstract_false_is_concrete() {
        let source = "classdef (Abstract = false) Shape\n  methods (Static)\n    function o = make()\n      o = Shape(3);\n    end\n  end\nend\n";
        assert!(hits("MP_STAT_25", source, "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_25", "classdef (Abstract) A\n x = A(\n", "");
    }
}
