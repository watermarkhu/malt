//! Expressions: the source text of a type, default value, validator or base class,
//! with the span it was read from.
//!
//! An [`Expr`] is raw text, as written. It is not evaluated, and it is not resolved
//! (a type `double` is the text `double`; what it means is for the consumer). The
//! text joins the tokens of the expression with nothing between them, as maxx does
//! (so `uint8 scalar` reads as `uint8scalar`); the span runs from the first token to
//! the last. Lines are 1-based, columns are 0-based and count characters.

use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tree_sitter::{Node, Point};

use crate::comments::node_text;

/// Raw source text with its span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
    /// The text as written (tokens joined with nothing between them).
    pub text: String,
    /// First line (1-based).
    pub lineno: usize,
    /// First column (0-based, in characters).
    pub column: usize,
    /// Last line (1-based, inclusive).
    pub endlineno: usize,
    /// Column just after the last character (0-based, in characters).
    pub endcolumn: usize,
}

impl Expr {
    /// An expression with a text and a span.
    pub fn new(
        text: impl Into<String>,
        lineno: usize,
        column: usize,
        endlineno: usize,
        endcolumn: usize,
    ) -> Self {
        Expr {
            text: text.into(),
            lineno,
            column,
            endlineno,
            endcolumn,
        }
    }

    /// The expression of some tokens of `src`: the text of each token joined, the span
    /// from the first to the last. `None` when there are no tokens.
    pub(crate) fn from_nodes(src: &[u8], nodes: &[Node]) -> Option<Expr> {
        let first = nodes.first()?;
        let last = nodes.last()?;
        let text: String = nodes.iter().map(|n| node_text(src, *n)).collect();
        let (lineno, column) = position(src, first.start_byte(), first.start_position());
        let (endlineno, endcolumn) = position(src, last.end_byte(), last.end_position());
        Some(Expr::new(text, lineno, column, endlineno, endcolumn))
    }

    /// The expression of one node, with `text` in place of the node's text (for a part of
    /// the node, as the class path of `opts.?Class.path`).
    pub(crate) fn from_node_text(src: &[u8], node: Node, text: &str) -> Expr {
        let (lineno, column) = position(src, node.start_byte(), node.start_position());
        let (endlineno, endcolumn) = position(src, node.end_byte(), node.end_position());
        Expr::new(text, lineno, column, endlineno, endcolumn)
    }
}

/// The 1-based line and the character column of a byte offset, from its tree-sitter point.
fn position(src: &[u8], byte: usize, point: Point) -> (usize, usize) {
    let line_start = byte.saturating_sub(point.column);
    let prefix = String::from_utf8_lossy(&src[line_start..byte.min(src.len())]);
    (point.row + 1, prefix.chars().count())
}

/// The text of the expression: `Expr` derefs to its text, so `&str` methods work on it.
impl Deref for Expr {
    type Target = str;

    fn deref(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl PartialEq<str> for Expr {
    fn eq(&self, other: &str) -> bool {
        self.text == other
    }
}

impl PartialEq<&str> for Expr {
    fn eq(&self, other: &&str) -> bool {
        self.text == *other
    }
}

impl PartialEq<String> for Expr {
    fn eq(&self, other: &String) -> bool {
        &self.text == other
    }
}

/// An expression is written as its text (the span is not part of the dump).
impl Serialize for Expr {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.text)
    }
}

/// A text read back has no span: the lines and columns are 0.
impl<'de> Deserialize<'de> for Expr {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(|text| Expr::new(text, 0, 0, 0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derefs_to_text_and_compares_with_strings() {
        let e = Expr::new("double", 3, 4, 3, 10);
        assert_eq!(&*e, "double");
        assert!(e.starts_with("dou"));
        assert_eq!(e, "double");
        assert_eq!(e.to_string(), "double");
        assert_eq!(serde_json::to_string(&e).unwrap(), "\"double\"");
    }
}
