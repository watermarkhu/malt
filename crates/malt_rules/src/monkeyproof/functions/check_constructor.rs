//! FCNS-3: every constructor has exactly one output.
//!
//! A constructor is a method named like its class. It is reported when it declares no
//! output or more than one. Declarations without a body, for constructors implemented in
//! a separate file, are checked as well.

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::common::{function_name, name_node, name_range, outputs, text};
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_3";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    visit(ctx.tree.root_node(), ctx.source, out);
}

fn visit(node: Node, source: &str, out: &mut Vec<Diagnostic>) {
    if node.kind() == "class_definition" {
        if let Some(class) = name_node(node).map(|n| text(n, source)) {
            let mut cursor = node.walk();
            for block in node.children(&mut cursor).filter(|c| c.kind() == "methods") {
                let mut inner = block.walk();
                for method in block.named_children(&mut inner) {
                    if !matches!(method.kind(), "function_definition" | "function_signature")
                        || function_name(method, source) != class
                    {
                        continue;
                    }
                    let count = outputs(method).len();
                    if count != 1 {
                        out.push(finding(
                            source,
                            ID,
                            name_range(method),
                            format!(
                                "Constructor of {class} has {count} outputs. Give it exactly one, the object."
                            ),
                        ));
                    }
                }
            }
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, source, out);
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::run;
    use super::ID;

    #[test]
    fn a_second_output_fires_at_the_constructor() {
        let src = "classdef Computer\nmethods\nfunction [obj, ok] = Computer(in)\nend\nend\nend\n";
        let d = run(ID, src, "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (3, 22));
    }

    #[test]
    fn no_output_fires_too() {
        let src = "classdef Computer\nmethods\nfunction Computer()\nend\nend\nend\n";
        assert_eq!(run(ID, src, "").len(), 1);
    }

    #[test]
    fn one_output_and_other_methods_are_fine() {
        let src = "classdef Computer\nmethods\nfunction obj = Computer(in)\nend\nfunction [a, b] = other(obj)\nend\nend\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn a_local_function_named_like_the_class_is_not_a_constructor() {
        let src = "classdef Computer\nend\nfunction [a, b] = Computer()\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn declared_constructors_are_checked_and_case_matters() {
        let src =
            "classdef Computer\nmethods\n[o, b] = Computer(in)\n[o, b] = computer(in)\nend\nend\n";
        assert_eq!(run(ID, src, "").len(), 1);
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(ID, "classdef C\nmethods\nfunction [ = C(\n", "");
    }
}
