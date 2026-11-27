//! FCNS-22: do not call functions with more input arguments than expected.
//!
//! Covers the functions the file itself defines: the main function, local functions,
//! nested functions, and static methods called as `ClassName.method(...)` inside the
//! class file. A call is reported when it passes more arguments than the definition has
//! inputs. Skipped: functions that end in `varargin`, functions with a name-value or
//! `Repeating` entry in their `arguments` block, a name defined more than once in the
//! file, a name that is a variable where it is called, and calls with a comma-separated
//! list argument such as `c{:}` whose length is unknown. Calls to functions in other
//! files are not checked. MATLAB's Code Analyzer reports some of these calls too; the set
//! is off by default, so that overlap is harmless.

use std::collections::HashMap;

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::common::{child_of_kind, function_name, has_true_attribute, inputs, name_node, text};
use crate::analysis::symbols::{DefKind, SymbolTable};
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_22";

/// How many inputs a definition takes, or `None` when any number may be passed.
type Limit = Option<usize>;

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let root = ctx.tree.root_node();
    let mut plain: HashMap<&str, Vec<Limit>> = HashMap::new();
    let mut statics: HashMap<&str, Vec<Limit>> = HashMap::new();
    let mut class_name = None;
    collect(
        root,
        ctx.source,
        false,
        &mut plain,
        &mut statics,
        &mut class_name,
    );
    plain.retain(|_, defs| defs.len() == 1);
    statics.retain(|_, defs| defs.len() == 1);
    if plain.is_empty() && statics.is_empty() {
        return;
    }
    let table = SymbolTable::shared(|| root, ctx.source);
    visit(
        root,
        ctx.source,
        &Known {
            plain: &plain,
            statics: &statics,
            class_name,
            table: &table,
        },
        out,
    );
}

struct Known<'a> {
    plain: &'a HashMap<&'a str, Vec<Limit>>,
    statics: &'a HashMap<&'a str, Vec<Limit>>,
    class_name: Option<&'a str>,
    table: &'a SymbolTable,
}

fn limit_of(node: Node, source: &str) -> Limit {
    let params = inputs(node);
    if params
        .last()
        .is_some_and(|p| text(*p, source) == "varargin")
    {
        return None;
    }
    if let Some(block) = child_of_kind(node, "arguments_statement") {
        let body = text(block, source);
        let mut cursor = block.walk();
        let name_value = block.named_children(&mut cursor).any(|p| {
            p.child_by_field_name("name")
                .is_some_and(|n| n.kind() == "property_name")
        });
        if name_value || body.contains("Repeating") {
            return None;
        }
    }
    Some(params.len())
}

fn collect<'a>(
    node: Node<'a>,
    source: &'a str,
    in_static_block: bool,
    plain: &mut HashMap<&'a str, Vec<Limit>>,
    statics: &mut HashMap<&'a str, Vec<Limit>>,
    class_name: &mut Option<&'a str>,
) {
    let mut in_static = in_static_block;
    match node.kind() {
        "class_definition" => {
            *class_name = name_node(node).map(|n| text(n, source));
        }
        "methods" => in_static = has_true_attribute(node, "Static", source),
        "function_definition" | "function_signature" => {
            let name = function_name(node, source);
            let limit = limit_of(node, source);
            let in_methods = node.parent().is_some_and(|p| p.kind() == "methods");
            if in_methods {
                if in_static {
                    statics.entry(name).or_default().push(limit);
                } else {
                    // Instance methods are not called by their bare name; keep the name
                    // out of the plain table by recording an unknown limit.
                    plain.entry(name).or_default().push(None);
                }
            } else if node.kind() == "function_definition" {
                plain.entry(name).or_default().push(limit);
            }
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect(child, source, in_static, plain, statics, class_name);
    }
}

fn visit(node: Node, source: &str, known: &Known, out: &mut Vec<Diagnostic>) {
    if node.kind() == "function_call" {
        check_call(node, source, known, out);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, source, known, out);
    }
}

fn check_call(call: Node, source: &str, known: &Known, out: &mut Vec<Diagnostic>) {
    let Some(name_node) = call
        .child_by_field_name("name")
        .filter(|n| n.kind() == "identifier")
    else {
        return;
    };
    let name = text(name_node, source);
    let parent = call.parent().filter(|p| p.kind() == "field_expression");
    let limit = match parent {
        None => {
            if is_variable(known.table, name, call.start_byte()) {
                return;
            }
            known.plain.get(name).map(|defs| defs[0])
        }
        Some(field) => {
            // `Class.method(...)`: a field expression of exactly the class name and the call.
            let object = field.child_by_field_name("object");
            let is_class = field.named_child_count() == 2
                && object.is_some_and(|o| {
                    o.kind() == "identifier"
                        && Some(text(o, source)) == known.class_name
                        && o.id() != call.id()
                })
                && !is_variable(
                    known.table,
                    known.class_name.unwrap_or(""),
                    call.start_byte(),
                );
            if is_class {
                known.statics.get(name).map(|defs| defs[0])
            } else {
                None
            }
        }
    };
    let Some(Some(max)) = limit else { return };
    let Some((count, certain)) = argument_count(call, source) else {
        return;
    };
    if certain && count > max {
        out.push(finding(
            source,
            ID,
            name_node.byte_range(),
            format!(
                "{name} is called with {count} inputs but takes at most {max}. Remove the extra arguments."
            ),
        ));
    }
}

/// The number of arguments of a call, and whether it is certain (no list argument).
fn argument_count(call: Node, source: &str) -> Option<(usize, bool)> {
    let Some(arguments) = child_of_kind(call, "arguments") else {
        return Some((0, true));
    };
    let mut count = 0;
    let mut cursor = arguments.walk();
    for argument in arguments.named_children(&mut cursor) {
        if argument.kind() == "comment" {
            continue;
        }
        if text(argument, source).trim_end().ends_with('}') {
            return None;
        }
        count += 1;
    }
    Some((count, true))
}

fn is_variable(table: &SymbolTable, name: &str, offset: usize) -> bool {
    table.scope_at(offset).is_some_and(|scope| {
        scope
            .defs
            .iter()
            .any(|d| d.name == name && d.kind != DefKind::NestedFunction)
    })
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::run;
    use super::ID;

    #[test]
    fn a_call_with_too_many_arguments_fires_at_the_callee() {
        let src = "function p(a, b, c)\nx = m(a, b, c);\nend\nfunction out = m(a, b)\nout = max(a, b);\nend\n";
        let d = run(ID, src, "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (2, 5));
    }

    #[test]
    fn the_exact_number_and_fewer_are_fine() {
        let src = "function p(a)\nx = m(a, a);\ny = m(a);\nz = m();\nend\nfunction out = m(a, b)\nout = 1;\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn varargin_and_name_value_functions_accept_more() {
        let src = "function p(a)\nm(a, a, a);\nn(a, a, a);\nend\nfunction m(a, varargin)\nend\nfunction n(a, opts)\narguments\na\nopts.Name = 1\nend\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn a_list_argument_and_a_variable_of_that_name_are_skipped() {
        let src = "function p(c, m)\nx = q(c{:}, 1, 2);\ny = m(1, 2, 3);\nend\nfunction q(a)\nend\nfunction m(a)\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn other_functions_and_members_are_fine() {
        let src = "function p(a, s)\nmax(a, a, a);\ns.m(1, 2, 3);\nend\nfunction m(a)\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn static_methods_called_through_the_class_are_checked() {
        let src = "classdef C\nmethods (Static)\nfunction r = make(a)\nr = a;\nend\nend\nmethods\nfunction t(obj)\nC.make(1, 2);\nC.make(1);\nend\nend\nend\n";
        let d = run(ID, src, "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].line, 9);
    }

    #[test]
    fn a_name_defined_twice_is_skipped() {
        let src = "function p()\nm(1, 2, 3);\nend\nfunction m(a)\nend\nfunction m(a, b)\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(ID, "function f(a)\nf(1, 2,\nend\n", "");
    }
}
