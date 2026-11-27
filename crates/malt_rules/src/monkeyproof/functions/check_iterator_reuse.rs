//! FCNS-13: do not reuse iterator variables within a function.
//!
//! Reports a `for` or `parfor` loop whose iterator variable is already the iterator of an
//! earlier loop in the same function, nested or not. In a script, the file is the scope.
//! A nested function has a scope of its own. Loops over a variable that is also
//! assigned elsewhere are not special: only the loop headers are compared.

use std::collections::HashSet;

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_13";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let mut seen = HashSet::new();
    visit(ctx.tree.root_node(), ctx.source, &mut seen, out);
}

fn visit<'s>(node: Node, source: &'s str, seen: &mut HashSet<&'s str>, out: &mut Vec<Diagnostic>) {
    if node.kind() == "function_definition" {
        let mut own = HashSet::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(child, source, &mut own, out);
        }
        return;
    }
    if node.kind() == "for_statement" {
        if let Some(variable) = iterator_variable(node, source) {
            if !seen.insert(&source[variable.byte_range()]) {
                out.push(finding(
                    source,
                    ID,
                    variable.byte_range(),
                    format!(
                        "The loop variable {} is used by an earlier loop in this function. Give each loop its own variable.",
                        &source[variable.byte_range()]
                    ),
                ));
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, source, seen, out);
    }
}

/// The identifier a `for_statement` iterates with.
pub(crate) fn iterator_variable<'t>(for_statement: Node<'t>, _source: &str) -> Option<Node<'t>> {
    let mut cursor = for_statement.walk();
    let iterator = for_statement
        .children(&mut cursor)
        .find(|c| c.kind() == "iterator")?;
    iterator.named_child(0).filter(|n| n.kind() == "identifier")
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::run;
    use super::ID;

    #[test]
    fn a_second_loop_with_the_same_variable_fires() {
        let src = "function f(n)\nfor i = 1:n\nend\nfor i = 1:2:11\nend\nend\n";
        let d = run(ID, src, "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (4, 5));
    }

    #[test]
    fn a_nested_loop_with_the_same_variable_fires() {
        let src = "function f(n)\nfor i = 1:n\nfor i = 1:n\nend\nend\nend\n";
        assert_eq!(run(ID, src, "").len(), 1);
    }

    #[test]
    fn distinct_variables_and_other_functions_are_fine() {
        let src = "function f(n)\nfor i = 1:n\nend\nfor j = 1:n\nend\nend\nfunction g(n)\nfor i = 1:n\nend\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn names_differing_in_case_are_distinct_and_parfor_counts() {
        let src = "function f(n)\nfor i = 1:n\nend\nfor I = 1:n\nend\nparfor i = 1:n\nend\nend\n";
        let d = run(ID, src, "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].line, 6);
    }

    #[test]
    fn a_nested_function_has_its_own_scope() {
        let src = "function f(n)\nfor i = 1:n\nend\nfunction g(n)\nfor i = 1:n\nend\nend\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(ID, "for i = \nfor i =\n", "");
    }
}
