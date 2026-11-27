//! FCNS-1: keep functions short and simple.
//!
//! Reports a function whose number of code lines or cyclomatic complexity is above the
//! limit for its kind: main, local, method of a class, or nested. Code lines are the
//! lines of the function that are neither blank nor comment-only, leaving out the lines
//! of functions nested in it. Complexity is one plus the number of `if`, `elseif`,
//! `for`, `while`, `case` and `catch` branches and short-circuit operators (`&&`, `||`),
//! also leaving out nested functions.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use super::common::{function_name, functions, name_range, FnKind};
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_1";

/// The limits of one kind of function; a missing value keeps the default of that kind.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default)]
struct Limits {
    max_complexity: Option<usize>,
    max_lines: Option<usize>,
}

/// Parameters of `MP_FCNS_1`: a table per kind of function. A limit of 0 switches it off.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct Params {
    main: Limits,
    local: Limits,
    method: Limits,
    nested: Limits,
}

/// Run the check over a file.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    for func in functions(ctx.tree.root_node()) {
        let (limits, default_complexity, default_lines) = match func.kind {
            FnKind::Main => (params.main, 20, 100),
            FnKind::Local => (params.local, 15, 50),
            FnKind::Method => (params.method, 10, 20),
            FnKind::Nested => (params.nested, 5, 10),
        };
        let max_complexity = limits.max_complexity.unwrap_or(default_complexity);
        let max_lines = limits.max_lines.unwrap_or(default_lines);
        let name = function_name(func.node, ctx.source);
        if max_lines > 0 {
            let lines = code_lines(func.node, ctx.source);
            if lines > max_lines {
                out.push(finding(
                    ctx.source,
                    ID,
                    name_range(func.node),
                    format!(
                        "Function {name} has {lines} lines of code, more than the {max_lines} allowed. Split it up."
                    ),
                ));
            }
        }
        if max_complexity > 0 {
            let complexity = complexity(func.node);
            if complexity > max_complexity {
                out.push(finding(
                    ctx.source,
                    ID,
                    name_range(func.node),
                    format!(
                        "Function {name} has a cyclomatic complexity of {complexity}, more than the {max_complexity} allowed. Simplify it."
                    ),
                ));
            }
        }
    }
}

/// Lines of `func` with code, not counting nested functions.
fn code_lines(func: Node, source: &str) -> usize {
    let mut nested = Vec::new();
    collect_nested(func, true, &mut nested);
    let mut count = 0;
    let mut offset = func.start_byte();
    let end = func.end_byte().min(source.len());
    let mut in_block_comment = false;
    for line in source[offset..end].split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        if nested.iter().any(|r| r.contains(&start)) {
            continue;
        }
        let trimmed = line.trim();
        if in_block_comment {
            if trimmed == "%}" {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed == "%{" {
            in_block_comment = true;
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('%') {
            continue;
        }
        count += 1;
    }
    count
}

fn collect_nested(node: Node, is_root: bool, out: &mut Vec<std::ops::Range<usize>>) {
    if !is_root && node.kind() == "function_definition" {
        out.push(node.byte_range());
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_nested(child, false, out);
    }
}

/// Cyclomatic complexity of a function, nested functions excluded.
fn complexity(func: Node) -> usize {
    let mut count = 1;
    count_branches(func, true, &mut count);
    count
}

fn count_branches(node: Node, is_root: bool, count: &mut usize) {
    match node.kind() {
        "function_definition" if !is_root => return,
        "if_statement" | "elseif_clause" | "for_statement" | "while_statement" | "case_clause"
        | "catch_clause" => *count += 1,
        "boolean_operator" => {
            let mut cursor = node.walk();
            if node
                .children(&mut cursor)
                .any(|c| matches!(c.kind(), "&&" | "||"))
            {
                *count += 1;
            }
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        count_branches(child, false, count);
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::run;
    use super::ID;

    fn branches(n: usize) -> String {
        named_branches("f", n)
    }

    fn named_branches(name: &str, n: usize) -> String {
        let mut s = format!("function {name}(x)\n");
        for _ in 0..n {
            s.push_str("if x\nend\n");
        }
        s.push_str("end\n");
        s
    }

    #[test]
    fn complexity_over_the_main_limit_fires_at_the_name() {
        let d = run(ID, &branches(20), "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (1, 10));
        assert!(run(ID, &branches(19), "").is_empty());
    }

    #[test]
    fn local_functions_have_a_lower_limit() {
        let src = format!("function g()\nend\n{}", named_branches("h", 15));
        assert_eq!(run(ID, &src, "").len(), 1);
        let src = format!("function g()\nend\n{}", named_branches("h", 14));
        assert!(run(ID, &src, "").is_empty());
    }

    #[test]
    fn methods_and_nested_functions_have_their_own_limits() {
        let mut body = String::new();
        for _ in 0..10 {
            body.push_str("if x, end\n");
        }
        let class = format!("classdef C\nmethods\nfunction m(x)\n{body}end\nend\nend\n");
        assert_eq!(run(ID, &class, "").len(), 1);
        let nested = "function f(x)\nfunction g()\nif x, end\nif x, end\nif x, end\nif x, end\nif x, end\nend\nend\n";
        let d = run(ID, nested, "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert!(d[0].message.contains("Function g"));
    }

    #[test]
    fn long_functions_fire_and_blank_and_comment_lines_do_not_count() {
        let mut s = String::from("function f()\n");
        for _ in 0..98 {
            s.push_str("x = 1;\n");
        }
        s.push_str("\n% comment\n%{\nblock\n%}\nend\n");
        assert!(run(ID, &s, "").is_empty(), "100 code lines are allowed");
        let more = s.replace("x = 1;\n", "x = 1;\nx = 1;\n");
        assert_eq!(run(ID, &more, "").len(), 1);
    }

    #[test]
    fn nested_functions_do_not_count_for_their_parent() {
        let mut s = String::from("function f()\nfunction g()\n");
        for _ in 0..60 {
            s.push_str("x = 1;\n");
        }
        s.push_str("end\nend\n");
        let d = run(ID, &s, "[lint.rules.MP_FCNS_1.nested]\nmax_lines = 1000\n");
        assert!(d.is_empty(), "{d:?}");
    }

    #[test]
    fn short_circuit_operators_and_cases_count() {
        let src = "function f(x)\nif x && x || x\nend\nswitch x\ncase 1\ncase 2\nend\nend\n";
        let cfg = "[lint.rules.MP_FCNS_1.main]\nmax_complexity = 6\n";
        assert!(run(ID, src, cfg).is_empty());
        let cfg = "[lint.rules.MP_FCNS_1.main]\nmax_complexity = 5\n";
        assert_eq!(run(ID, src, cfg).len(), 1);
    }

    #[test]
    fn a_zero_limit_switches_the_limit_off() {
        let cfg = "[lint.rules.MP_FCNS_1.main]\nmax_complexity = 0\nmax_lines = 0\n";
        assert!(run(ID, &branches(40), cfg).is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(ID, "function f(\nif x &&\n", "");
    }
}
