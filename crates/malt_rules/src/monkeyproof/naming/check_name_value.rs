//! NAMING-18: name-value pair names that are not UpperCamelCase.

use std::collections::HashSet;

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use super::decls::text;
use crate::monkeyproof::finding;

const ID: &str = "MP_NAMING_18";

/// Parameters of `MP_NAMING_18`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct Params {
    /// Also check `addOptional`, which declares a positional argument rather than a
    /// name-value pair.
    include_optional: bool,
}

/// Report a name-value pair name that does not start with an uppercase letter, or that
/// has an underscore or a digit, in an `arguments` block (`options.Name`) or in a call
/// of `addParameter` or `addParamValue` on an `inputParser` variable.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    let source = ctx.source;
    let mut parsers = HashSet::new();
    collect_parsers(ctx.tree.root_node(), source, &mut parsers);
    visit(ctx.tree.root_node(), source, &parsers, &params, out);
}

/// The names of the variables that the file assigns `inputParser` to.
fn collect_parsers<'a>(node: Node, source: &'a str, parsers: &mut HashSet<&'a str>) {
    if node.kind() == "assignment" {
        let left = node.child_by_field_name("left");
        let right = node.child_by_field_name("right");
        if let (Some(left), Some(right)) = (left, right) {
            let constructs = match right.kind() {
                "identifier" => text(right, source) == "inputParser",
                "function_call" => right
                    .child_by_field_name("name")
                    .is_some_and(|n| text(n, source) == "inputParser"),
                _ => false,
            };
            if constructs && left.kind() == "identifier" {
                parsers.insert(text(left, source));
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_parsers(child, source, parsers);
    }
}

/// UpperCamelCase without digits: an uppercase first letter, then letters only.
fn is_upper_camel(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && name.chars().all(|c| c.is_ascii_alphabetic())
}

fn report(name_node: Node, name: &str, source: &str, out: &mut Vec<Diagnostic>) {
    if !name.is_empty() && !is_upper_camel(name) {
        out.push(finding(
            source,
            ID,
            name_node.byte_range(),
            format!("The name-value pair '{name}' should be UpperCamelCase without digits."),
        ));
    }
}

fn visit(
    node: Node,
    source: &str,
    parsers: &HashSet<&str>,
    params: &Params,
    out: &mut Vec<Diagnostic>,
) {
    match node.kind() {
        "arguments_statement" => {
            let mut cursor = node.walk();
            for property in node.children(&mut cursor) {
                // `options.Name`: a dotted name, so a name-value pair of a struct input.
                let Some(name) = property.child_by_field_name("name") else {
                    continue;
                };
                if property.kind() == "property" && name.kind() == "property_name" {
                    let mut inner = name.walk();
                    let parts: Vec<Node> = name.named_children(&mut inner).collect();
                    if let [_, field] = parts.as_slice() {
                        report(*field, text(*field, source), source, out);
                    }
                }
            }
            return;
        }
        "function_call" => check_call(node, source, parsers, params, out),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, source, parsers, params, out);
    }
}

/// Check `addParameter(p, 'Name', ...)` and `p.addParameter('Name', ...)`.
fn check_call(
    call: Node,
    source: &str,
    parsers: &HashSet<&str>,
    params: &Params,
    out: &mut Vec<Diagnostic>,
) {
    let Some(name) = call.child_by_field_name("name") else {
        return;
    };
    let method = text(name, source);
    let wanted = matches!(method, "addParameter" | "addParamValue")
        || (params.include_optional && method == "addOptional");
    if name.kind() != "identifier" || !wanted {
        return;
    }
    let mut cursor = call.walk();
    let args: Vec<Node> = call
        .children(&mut cursor)
        .filter(|c| c.kind() == "arguments")
        .flat_map(|a| {
            let mut inner = a.walk();
            a.named_children(&mut inner).collect::<Vec<_>>()
        })
        .collect();
    // Method form: the parser is the object in front of the call.
    let parent = call.parent();
    let method_form = parent.is_some_and(|p| {
        p.kind() == "field_expression" && p.named_child(0).is_some_and(|o| o.id() != call.id())
    });
    let (parser, name_arg) = if method_form {
        let object = parent
            .and_then(|p| p.named_child(0))
            .filter(|o| o.kind() == "identifier")
            .map(|o| text(o, source));
        (object, args.first())
    } else {
        let first = args
            .first()
            .filter(|a| a.kind() == "identifier")
            .map(|a| text(*a, source));
        (first, args.get(1))
    };
    let (Some(parser), Some(name_arg)) = (parser, name_arg) else {
        return;
    };
    if !parsers.contains(parser) || name_arg.kind() != "string" {
        return;
    }
    let mut inner = name_arg.walk();
    let content = name_arg
        .named_children(&mut inner)
        .find(|c| c.kind() == "string_content");
    if let Some(content) = content {
        report(content, text(content, source), source, out);
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::lint;
    use crate::monkeyproof::naming::from_config;

    fn run(src: &str, toml: &str) -> Vec<(usize, usize, String)> {
        lint(from_config, src, toml)
            .into_iter()
            .filter(|d| d.rule_id == super::ID)
            .map(|d| (d.line, d.column, d.message))
            .collect()
    }

    #[test]
    fn arguments_block_names() {
        let src = "function f(options)\narguments\n    options.number_of_lines = 3\n    options.Precision = 2\n    options.Level2 (1,1) double = 1\n    options.SHOW_ALL = true\nend\nend\n";
        let found = run(src, "");
        assert_eq!(found.len(), 3);
        assert_eq!((found[0].0, found[0].1), (3, 13));
        assert_eq!(found[1].0, 5);
        assert_eq!(found[2].0, 6);
    }

    #[test]
    fn plain_inputs_of_an_arguments_block_are_silent() {
        let src = "function f(x_in, y)\narguments\n    x_in (1,1) double\n    y\nend\nend\n";
        assert!(run(src, "").is_empty());
    }

    #[test]
    fn input_parser_both_call_forms() {
        let src = "p = inputParser;\np.addParameter('my_x', 1);\naddParameter(p, 'b2', 2);\naddParamValue(p, 'Good', 2);\np.addParameter(\"lower\", 1);\n";
        let found = run(src, "");
        assert_eq!(found.len(), 3);
        assert_eq!((found[0].0, found[0].1), (2, 17));
        assert_eq!(found[1].0, 3);
        assert_eq!(found[2].0, 5);
    }

    #[test]
    fn only_input_parser_variables_are_checked() {
        let src = "q = other();\nq.addParameter('bad_name', 1);\naddParameter(q, 'bad_name', 1);\n";
        assert!(run(src, "").is_empty());
        let src = "p = inputParser();\naddParameter(p, 'bad_name', 1);\n";
        assert_eq!(run(src, "").len(), 1);
    }

    #[test]
    fn add_optional_is_off_by_default() {
        let src = "p = inputParser;\np.addOptional('first_arg', 1);\n";
        assert!(run(src, "").is_empty());
        assert_eq!(
            run(src, "[lint.rules.MP_NAMING_18]\ninclude_optional = true\n").len(),
            1
        );
    }

    #[test]
    fn computed_names_are_skipped() {
        assert!(run("p = inputParser;\np.addParameter(name, 1);\n", "").is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(
            "p = inputParser;\np.addParameter('a_b'\narguments\n opt.\n",
            "",
        );
    }
}
