//! Helpers shared by the fixes of several rule engines.

use malt_core::Fix;
use tree_sitter::Node;

/// A fix that deletes the whole line of the statement `node`, when that is
/// all that stands on the line.
///
/// The statement must be a child of a `block` or the `source_file`, start the
/// line (blanks before it only) and end it (an optional `;` or `,` and blanks
/// after it, then the line break). Statements that share a line with another
/// statement or with a comment get no fix: removing the line would take the
/// neighbour along, and removing the statement alone would leave a stray
/// separator.
pub(crate) fn delete_statement_line(node: Node, source: &str) -> Option<Fix> {
    let parent = node.parent()?;
    if !matches!(parent.kind(), "block" | "source_file") {
        return None;
    }
    let bytes = source.as_bytes();
    let mut start = node.start_byte();
    while start > 0 && matches!(bytes[start - 1], b' ' | b'\t') {
        start -= 1;
    }
    if start > 0 && bytes[start - 1] != b'\n' {
        return None;
    }
    let mut end = node.end_byte();
    while end < bytes.len() && matches!(bytes[end], b' ' | b'\t') {
        end += 1;
    }
    if end < bytes.len() && matches!(bytes[end], b';' | b',') {
        end += 1;
        while end < bytes.len() && matches!(bytes[end], b' ' | b'\t') {
            end += 1;
        }
    }
    if end < bytes.len() {
        if bytes[end] == b'\r' {
            end += 1;
        }
        if end < bytes.len() && bytes[end] != b'\n' {
            return None;
        }
        if end < bytes.len() {
            end += 1;
        }
    }
    Some(Fix::new(start..end, ""))
}

/// Whether `text` is a plain MATLAB identifier (a letter or underscore, then
/// letters, digits and underscores) that is not a keyword.
pub(crate) fn is_plain_identifier(text: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "break",
        "case",
        "catch",
        "classdef",
        "continue",
        "else",
        "elseif",
        "end",
        "for",
        "function",
        "global",
        "if",
        "otherwise",
        "parfor",
        "persistent",
        "return",
        "spmd",
        "switch",
        "try",
        "while",
        "arguments",
        "enumeration",
        "events",
        "methods",
        "properties",
    ];
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !KEYWORDS.contains(&text)
}

/// The text between the quotes of a quoted literal (`'abc'` or `"abc"`), when
/// it contains no quote escape and no backslash.
pub(crate) fn simple_literal_text<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    if node.kind() != "string" {
        return None;
    }
    let text = &source[node.byte_range()];
    let quote = text.chars().next()?;
    if !matches!(quote, '\'' | '"') || text.len() < 2 || !text.ends_with(quote) {
        return None;
    }
    let inner = &text[1..text.len() - 1];
    (!inner.contains(quote)).then_some(inner)
}

/// Whether `text` parses as MATLAB code without a syntax error.
pub(crate) fn parses_cleanly(text: &str) -> bool {
    malt_core::parses_cleanly(text)
}
