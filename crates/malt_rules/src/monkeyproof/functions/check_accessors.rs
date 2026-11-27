//! FCNS-6: minimize getters and setters.
//!
//! Reports each method named `get.<Property>` or `set.<Property>` in a class, with its
//! body or as a bare declaration.

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::common::{function_name, name_range};
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_6";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    visit(ctx.tree.root_node(), ctx.source, out);
}

fn visit(node: Node, source: &str, out: &mut Vec<Diagnostic>) {
    if node.kind() == "methods" {
        let mut cursor = node.walk();
        for method in node.named_children(&mut cursor) {
            if !matches!(method.kind(), "function_definition" | "function_signature") {
                continue;
            }
            // The grammar reads `get.a` as the token `get.` followed by the name `a`.
            let mut inner = method.walk();
            let token = method
                .children(&mut inner)
                .find(|c| matches!(c.kind(), "get." | "set."));
            let Some(token) = token else { continue };
            let prefix = if token.kind() == "get." { "get" } else { "set" };
            let kind = if prefix == "get" { "getter" } else { "setter" };
            let name = format!("{prefix}.{}", function_name(method, source));
            out.push(finding(
                source,
                ID,
                token.start_byte()..name_range(method).end,
                format!("Avoid the {kind} {name}. Write an explicit method instead."),
            ));
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
    fn getters_and_setters_fire() {
        let src = "classdef C\nmethods\nfunction v = get.a(obj)\nend\nfunction obj = set.b(obj, v)\nend\nend\nend\n";
        let d = run(ID, src, "");
        assert_eq!(d.len(), 2, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (3, 14));
        assert!(d[1].message.contains("setter"));
    }

    #[test]
    fn explicit_methods_are_fine() {
        let src = "classdef C\nmethods\nfunction v = getA(obj)\nend\nfunction obj = setA(obj, v)\nend\nend\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn a_plain_function_named_like_an_accessor_is_fine() {
        assert!(run(ID, "function v = get(a)\nend\n", "").is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(ID, "classdef C\nmethods\nfunction v = get.(\n", "");
    }
}
