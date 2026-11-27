//! The indentation scan behind `MP_LAYOUT_2` and `MP_LAYOUT_6`.
//!
//! Every construct that has a body (a function, `if`, `for`, a class block, ...) is looked at
//! when it starts its own line. The statements of its body that start their own line must be
//! indented by exactly one level relative to the construct. Lines the scan cannot judge
//! (indentation with a tab, a statement sharing its line with the header, a construct with a
//! syntax error in it) are skipped.

use std::ops::Range;

use tree_sitter::Node;

use super::text::{children, walk, Text};

/// What the scan found.
pub(crate) enum Problem {
    /// The body of a function starts at or left of the `function` keyword.
    FunctionBodyFlat(Range<usize>),
    /// A body is indented, but not by one level.
    Width {
        /// The first offending line of the body.
        range: Range<usize>,
        /// The indentation the line should have.
        expected: usize,
    },
}

/// Containers whose `block` children hold the statements of the body.
const BLOCK_CONTAINERS: &[&str] = &[
    "if_statement",
    "elseif_clause",
    "else_clause",
    "for_statement",
    "while_statement",
    "case_clause",
    "otherwise_clause",
    "try_statement",
    "catch_clause",
    "spmd_statement",
];

/// Class blocks and the kind of their members.
const MEMBER_BLOCKS: &[(&str, &[&str])] = &[
    ("properties", &["property"]),
    ("methods", &["function_definition", "function_signature"]),
    ("events", &["identifier"]),
    ("enumeration", &["enum"]),
    ("arguments_statement", &["property"]),
];

/// Scan the tree for indentation problems with `indent` spaces per level.
pub(crate) fn scan(text: &Text, root: Node, indent: usize) -> Vec<Problem> {
    let mut problems = Vec::new();
    walk(root, |node, _| {
        let kind = node.kind();
        let items: Vec<Node> = if kind == "function_definition" {
            let mut items = Vec::new();
            for child in children(node) {
                match child.kind() {
                    "arguments_statement" => items.push(child),
                    "block" => items.extend(statements(child)),
                    _ => {}
                }
            }
            items
        } else if BLOCK_CONTAINERS.contains(&kind) {
            children(node)
                .into_iter()
                .filter(|child| child.kind() == "block")
                .flat_map(statements)
                .collect()
        } else if kind == "class_definition" {
            children(node)
                .into_iter()
                .filter(|child| {
                    MEMBER_BLOCKS
                        .iter()
                        .any(|(block, _)| *block == child.kind() && child.child_count() > 1)
                })
                .collect()
        } else if let Some((_, members)) = MEMBER_BLOCKS
            .iter()
            .find(|(block, _)| *block == kind && node.child_count() > 1)
        {
            children(node)
                .into_iter()
                .filter(|child| members.contains(&child.kind()))
                .collect()
        } else {
            return;
        };
        if items.is_empty() || node.has_error() {
            return;
        }
        let Some(base) = text.indent_if_first(node.start_byte()) else {
            return;
        };
        let expected = base + indent;
        let mut first_seen = false;
        for item in items {
            let Some(found) = text.indent_if_first(item.start_byte()) else {
                continue;
            };
            let line_end = text.line_range(text.line_of(item.start_byte())).end;
            let range = item.start_byte()..item.end_byte().min(line_end);
            if kind == "function_definition" && !first_seen && found <= base {
                problems.push(Problem::FunctionBodyFlat(range));
                return;
            }
            first_seen = true;
            if found != expected {
                problems.push(Problem::Width { range, expected });
                return;
            }
        }
    });
    problems
}

/// The statements of a `block`: its named children without comments and continuations.
fn statements(block: Node) -> Vec<Node> {
    children(block)
        .into_iter()
        .filter(|child| {
            child.is_named()
                && !matches!(child.kind(), "comment" | "line_continuation" | "ERROR")
                && child.start_byte() < child.end_byte()
        })
        .collect()
}
