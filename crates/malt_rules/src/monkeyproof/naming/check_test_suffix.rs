//! NAMING-19: the suffix `Test` outside unit test files.

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::decls::text;
use crate::monkeyproof::finding;

const ID: &str = "MP_NAMING_19";

/// Whether `name` ends in `Test` with something in front of it.
fn has_suffix(name: &str) -> bool {
    name.len() > 4 && name.ends_with("Test")
}

/// The superclasses of a classdef, as dotted names.
fn superclasses<'a>(class: Node, source: &'a str) -> Vec<&'a str> {
    let mut cursor = class.walk();
    let list = class
        .children(&mut cursor)
        .find(|c| c.kind() == "superclasses");
    let Some(list) = list else {
        return Vec::new();
    };
    let mut inner = list.walk();
    list.named_children(&mut inner)
        .filter(|c| c.kind() == "property_name")
        .map(|c| text(c, source))
        .collect()
}

/// Whether the file is a unit test: a class deriving from a `TestCase` (or a class
/// whose name ends in `Test`, a shared test base), or a function file that calls
/// `functiontests`.
fn is_unit_test(root: Node, source: &str) -> bool {
    let mut cursor = root.walk();
    for node in root.named_children(&mut cursor) {
        if node.kind() == "class_definition" {
            return superclasses(node, source).iter().any(|s| {
                let last = s.rsplit('.').next().unwrap_or(s);
                last.ends_with("TestCase") || last.ends_with("Test")
            });
        }
    }
    uses_functiontests(root, source)
}

fn uses_functiontests(node: Node, source: &str) -> bool {
    if node.kind() == "function_call"
        && node
            .child_by_field_name("name")
            .is_some_and(|n| text(n, source) == "functiontests")
    {
        return true;
    }
    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();
    children.into_iter().any(|c| uses_functiontests(c, source))
}

/// A script has no definition to name; a test script cannot be told from another one.
fn is_script(root: Node) -> bool {
    let mut cursor = root.walk();
    let first = root
        .named_children(&mut cursor)
        .find(|c| !matches!(c.kind(), "comment" | "line_continuation"));
    first.is_some_and(|c| !matches!(c.kind(), "function_definition" | "class_definition"))
}

/// Report a class, function or file name that ends in `Test` in a file that is not a
/// unit test.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let root = ctx.tree.root_node();
    if is_script(root) || is_unit_test(root, ctx.source) {
        return;
    }
    let mut names = Vec::new();
    definitions(root, false, &mut names);
    let mut reported = false;
    for name in &names {
        if has_suffix(text(*name, ctx.source)) {
            reported = true;
            out.push(finding(
                ctx.source,
                ID,
                name.byte_range(),
                format!(
                    "The name '{}' ends in 'Test', which is reserved for unit tests.",
                    text(*name, ctx.source)
                ),
            ));
        }
    }
    let stem = ctx
        .file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if !reported && has_suffix(stem) {
        if let Some(first) = names.first() {
            out.push(finding(
                ctx.source,
                ID,
                first.byte_range(),
                format!("The file name '{stem}' ends in 'Test', which is reserved for unit tests."),
            ));
        }
    }
}

/// The name nodes of the classes and functions of the file, methods left out.
fn definitions<'a>(node: Node<'a>, in_methods: bool, out: &mut Vec<Node<'a>>) {
    let in_methods = in_methods || node.kind() == "methods";
    if !in_methods && matches!(node.kind(), "class_definition" | "function_definition") {
        if let Some(name) = node.child_by_field_name("name") {
            if name.kind() == "identifier" {
                out.push(name);
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        definitions(child, in_methods, out);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use malt_core::{Config, FileContext};

    use crate::test_util::parse;

    fn run(src: &str, path: &str) -> Vec<(usize, usize, String)> {
        let tree = parse(src);
        let ctx = FileContext {
            tree: &tree,
            source: src,
            file_path: Path::new(path),
            project: None,
        };
        let mut out = Vec::new();
        super::run(&Config::default(), &ctx, &mut out);
        out.into_iter()
            .map(|d| (d.line, d.column, d.message))
            .collect()
    }

    #[test]
    fn non_test_class_with_the_suffix() {
        let found = run("classdef SpeedTest\nend\n", "SpeedTest.m");
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].0, found[0].1), (1, 10));
    }

    #[test]
    fn test_case_class_is_fine() {
        assert!(run(
            "classdef SpeedTest < matlab.unittest.TestCase\nend\n",
            "SpeedTest.m"
        )
        .is_empty());
    }

    #[test]
    fn function_based_test_is_fine() {
        let src = "function tests = speedTest\ntests = functiontests(localfunctions);\nend\nfunction checkTest(testCase)\nend\n";
        assert!(run(src, "speedTest.m").is_empty());
    }

    #[test]
    fn plain_function_with_the_suffix() {
        let found = run("function r = runTest\nr = 1;\nend\n", "runTest.m");
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].0, found[0].1), (1, 14));
    }

    #[test]
    fn file_name_alone_is_reported_at_the_first_definition() {
        let found = run("function r = speed\nr = 1;\nend\n", "speedTest.m");
        assert_eq!(found.len(), 1);
        assert!(found[0].2.contains("file name"));
    }

    #[test]
    fn name_and_file_are_reported_once() {
        assert_eq!(
            run("function r = runTest\nr = 1;\nend\n", "runTest.m").len(),
            1
        );
    }

    #[test]
    fn lookalikes_are_silent() {
        assert!(run("function r = latest\nr = 1;\nend\n", "latest.m").is_empty());
        assert!(run("function r = Test\nr = 1;\nend\n", "Test.m").is_empty());
        assert!(run("function r = testRun\nr = 1;\nend\n", "testRun.m").is_empty());
    }

    #[test]
    fn methods_and_scripts_are_not_checked() {
        let src =
            "classdef C\n    methods\n        function runTest(obj)\n        end\n    end\nend\n";
        assert!(run(src, "C.m").is_empty());
        assert!(run("x = 1;\n", "scriptTest.m").is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run("classdef <\nfunction\n", "xTest.m");
    }
}
