//! FCNS-24: do not use Java in your code.
//!
//! Reports three things, each once per occurrence:
//!
//! - a call to one of the functions in `functions` (default `javaObject`, `javaMethod`,
//!   `javaArray`, `javaaddpath`, `javarmpath`, `javaclasspath`, `javaObjectEDT`,
//!   `javaMethodEDT`, `usejava`, `javachk`);
//! - an expression that starts with one of the packages in `packages` (default `java`,
//!   `javax`, `com`, `org`), such as `java.lang.String('a')` or `com.mathworks.x.Y`;
//! - an `import` of such a package, in command form (`import java.util.*`) or as a call.
//!
//! A name that is a variable where it is used, for example a struct called `org`, is not
//! a package and is left alone. Java reached through a variable, or through a function
//! that is not in the list, is not found.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use super::common::text;
use crate::analysis::symbols::{DefKind, SymbolTable};
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_24";

/// Parameters of `MP_FCNS_24`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    functions: Vec<String>,
    packages: Vec<String>,
}

impl Default for Params {
    fn default() -> Self {
        let own = |list: &[&str]| list.iter().map(|s| s.to_string()).collect();
        Self {
            functions: own(&[
                "javaObject",
                "javaMethod",
                "javaArray",
                "javaaddpath",
                "javarmpath",
                "javaclasspath",
                "javaObjectEDT",
                "javaMethodEDT",
                "usejava",
                "javachk",
            ]),
            packages: own(&["java", "javax", "com", "org"]),
        }
    }
}

/// Run the check over a file.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    let root = ctx.tree.root_node();
    let table = SymbolTable::shared(|| root, ctx.source);
    visit(root, ctx.source, &params, &table, out);
}

fn is_variable(table: &SymbolTable, name: &str, offset: usize) -> bool {
    table.scope_at(offset).is_some_and(|scope| {
        scope
            .defs
            .iter()
            .any(|d| d.name == name && d.kind != DefKind::NestedFunction)
    })
}

fn is_java_path(path: &str, params: &Params) -> bool {
    let head = path.split('.').next().unwrap_or("");
    path.contains('.') && params.packages.iter().any(|p| p == head)
}

fn visit(
    node: Node,
    source: &str,
    params: &Params,
    table: &SymbolTable,
    out: &mut Vec<Diagnostic>,
) {
    match node.kind() {
        "function_call" => {
            let name = node
                .child_by_field_name("name")
                .filter(|n| n.kind() == "identifier");
            let member = node.parent().is_some_and(|p| {
                p.kind() == "field_expression"
                    && p.named_child(0).is_some_and(|o| o.id() != node.id())
            });
            if let Some(name) = name.filter(|_| !member) {
                let word = text(name, source);
                if params.functions.iter().any(|f| f == word)
                    && !is_variable(table, word, name.start_byte())
                {
                    out.push(finding(
                        source,
                        ID,
                        name.byte_range(),
                        format!("The function {word} uses Java. Use a MATLAB function instead."),
                    ));
                } else if word == "import" && imports_java(node, source, params) {
                    out.push(java_import(source, name.byte_range()));
                }
            }
        }
        "command" => {
            let name = node.child(0).filter(|n| n.kind() == "command_name");
            if let Some(name) = name {
                let word = text(name, source);
                if params.functions.iter().any(|f| f == word)
                    && !is_variable(table, word, name.start_byte())
                {
                    out.push(finding(
                        source,
                        ID,
                        name.byte_range(),
                        format!("The function {word} uses Java. Use a MATLAB function instead."),
                    ));
                } else if word == "import" {
                    let mut cursor = node.walk();
                    let java = node
                        .children(&mut cursor)
                        .filter(|c| c.kind() == "command_argument")
                        .any(|a| is_java_path(text(a, source), params));
                    if java {
                        out.push(java_import(source, node.byte_range()));
                    }
                }
            }
            return;
        }
        "field_expression" => {
            if let Some(root) = leftmost(node).filter(|r| r.kind() == "identifier") {
                let head = text(root, source);
                if params.packages.iter().any(|p| p == head)
                    && !is_variable(table, head, root.start_byte())
                {
                    out.push(finding(
                        source,
                        ID,
                        node.byte_range(),
                        format!("The package {head} is Java. Use a MATLAB alternative."),
                    ));
                    return;
                }
            }
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, source, params, table, out);
    }
}

fn java_import(source: &str, range: std::ops::Range<usize>) -> Diagnostic {
    finding(
        source,
        ID,
        range,
        "This import brings in Java classes. Use a MATLAB alternative.",
    )
}

/// The first object of a chain of field accesses.
fn leftmost(node: Node) -> Option<Node> {
    let mut current = node;
    while current.kind() == "field_expression" {
        current = current.child_by_field_name("object")?;
    }
    Some(current)
}

/// Whether `import('java.util.Date')` imports a Java package.
fn imports_java(call: Node, source: &str, params: &Params) -> bool {
    let Some(arguments) = super::common::child_of_kind(call, "arguments") else {
        return false;
    };
    let mut cursor = arguments.walk();
    let found = arguments.named_children(&mut cursor).any(|a| {
        a.kind() == "string"
            && is_java_path(
                text(a, source).trim_matches(|c| c == '\'' || c == '"'),
                params,
            )
    });
    found
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::run;
    use super::ID;

    #[test]
    fn java_functions_fire_at_the_name() {
        let d = run(ID, "x = javaObject('java.lang.String', 'a');\n", "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (1, 5));
        assert_eq!(run(ID, "if usejava('jvm')\nend\n", "").len(), 1);
        assert_eq!(run(ID, "javaaddpath foo.jar\n", "").len(), 1);
    }

    #[test]
    fn java_packages_fire_once_per_expression() {
        let d = run(ID, "s = java.lang.String('a');\nq = com.foo.bar(1);\n", "");
        assert_eq!(d.len(), 2, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (1, 5));
        assert_eq!(run(ID, "d = java.util.Date;\n", "").len(), 1);
        assert_eq!(run(ID, "k = javax.swing.JFrame;\n", "").len(), 1);
    }

    #[test]
    fn imports_of_java_packages_fire() {
        assert_eq!(run(ID, "import java.util.*\n", "").len(), 1);
        assert_eq!(run(ID, "import('java.util.Date');\n", "").len(), 1);
        assert!(run(ID, "import matlab.net.*\n", "").is_empty());
        assert!(run(ID, "import('matlab.net.http.HTTPOptions');\n", "").is_empty());
    }

    #[test]
    fn variables_with_a_package_name_are_not_java() {
        let src = "function f(org)\nx = org.name;\nend\n";
        assert!(run(ID, src, "").is_empty());
        assert!(run(ID, "x = s.java;\ny = comment;\n", "").is_empty());
    }

    #[test]
    fn the_lists_are_parameters() {
        let toml = "[lint.rules.MP_FCNS_24]\nfunctions = [\"javaMethod\"]\npackages = []\n";
        let src = "javaObject('a');\njavaMethod('b');\nx = java.lang.Math.PI;\n";
        let d = run(ID, src, toml);
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].line, 2);
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(ID, "x = java.lang.(\nimport java.\njavaObject(\n", "");
    }
}
