//! Shared text helpers of the layout checks: where the comments and strings are, a line
//! table, and a masked copy of the source with those removed.

use std::ops::Range;

use malt_core::{Diagnostic, Severity};
use tree_sitter::Node;

/// Byte that stands in for the contents of a string literal in [`Text::code`]. It is not
/// whitespace, so `'a','b'` still shows a comma followed by something.
pub(crate) const STRING_MASK: u8 = 0x01;

/// The source of one file with its comments and strings located.
pub(crate) struct Text<'a> {
    /// The original source.
    pub source: &'a str,
    /// Start byte of every line.
    pub line_starts: Vec<usize>,
    /// Byte ranges of comments (a run of comment lines is one range), sorted.
    pub comments: Vec<Range<usize>>,
    /// Byte ranges of string literals including their quotes, sorted.
    pub strings: Vec<Range<usize>>,
    /// The source with comments and `...` continuations replaced by spaces and string
    /// literals by [`STRING_MASK`]; newlines are kept, and the length is unchanged.
    pub code: Vec<u8>,
}

impl<'a> Text<'a> {
    /// Locate comments and strings of `root`'s tree.
    pub fn new(source: &'a str, root: Node) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(
            source
                .bytes()
                .enumerate()
                .filter(|(_, b)| *b == b'\n')
                .map(|(i, _)| i + 1),
        );
        let mut comments = Vec::new();
        let mut strings = Vec::new();
        let mut continuations = Vec::new();
        let mut cursor = root.walk();
        'walk: loop {
            let node = cursor.node();
            match node.kind() {
                "comment" => comments.push(node.start_byte()..node.end_byte()),
                "line_continuation" => continuations.push(node.start_byte()..node.end_byte()),
                "string" => strings.push(node.start_byte()..node.end_byte()),
                _ => {}
            }
            // Comments and strings have nothing to look at below them.
            if !matches!(node.kind(), "comment" | "string" | "line_continuation")
                && cursor.goto_first_child()
            {
                continue;
            }
            loop {
                if cursor.goto_next_sibling() {
                    break;
                }
                if !cursor.goto_parent() {
                    break 'walk;
                }
            }
        }
        let mut code = source.as_bytes().to_vec();
        let mut mask = |range: &Range<usize>, with: u8| {
            for byte in &mut code[range.clone()] {
                if *byte != b'\n' && *byte != b'\r' {
                    *byte = with;
                }
            }
        };
        for range in comments.iter().chain(&continuations) {
            mask(range, b' ');
        }
        for range in &strings {
            mask(range, STRING_MASK);
        }
        Self {
            source,
            line_starts,
            comments,
            strings,
            code,
        }
    }

    /// Number of lines.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// The zero-based line of `byte`.
    pub fn line_of(&self, byte: usize) -> usize {
        self.line_starts.partition_point(|&start| start <= byte) - 1
    }

    /// The byte range of `line` without its line break.
    pub fn line_range(&self, line: usize) -> Range<usize> {
        let start = self.line_starts[line];
        let mut end = self
            .line_starts
            .get(line + 1)
            .map_or(self.source.len(), |next| next - 1);
        if end > start && self.source.as_bytes()[end - 1] == b'\r' {
            end -= 1;
        }
        start..end
    }

    /// Whether `byte` is inside a comment.
    pub fn in_comment(&self, byte: usize) -> bool {
        contains(&self.comments, byte)
    }

    /// Whether `byte` is inside a string literal.
    pub fn in_string(&self, byte: usize) -> bool {
        contains(&self.strings, byte)
    }

    /// The indentation of the line `byte` is on, when `byte` is the first thing on its line
    /// and the indentation is spaces only. `None` otherwise (so the caller skips what it
    /// cannot judge).
    pub fn indent_if_first(&self, byte: usize) -> Option<usize> {
        let start = self.line_starts[self.line_of(byte)];
        let before = &self.source.as_bytes()[start..byte];
        before.iter().all(|b| *b == b' ').then_some(before.len())
    }

    /// A finding of `id` at `range`, with the position taken from the line table.
    pub fn finding(
        &self,
        id: &'static str,
        range: Range<usize>,
        message: impl Into<String>,
    ) -> Diagnostic {
        let line = self.line_of(range.start);
        let column = self.source[self.line_starts[line]..range.start.min(self.source.len())]
            .chars()
            .count()
            + 1;
        Diagnostic {
            rule_id: id,
            message: message.into(),
            severity: Severity::Info,
            byte_range: range,
            line: line + 1,
            column,
            fix: None,
        }
    }
}

/// Whether the sorted, disjoint `ranges` contain `byte`.
fn contains(ranges: &[Range<usize>], byte: usize) -> bool {
    let index = ranges.partition_point(|range| range.end <= byte);
    ranges.get(index).is_some_and(|range| range.start <= byte)
}

/// Visit every node in document order with its depth, without recursion.
pub(crate) fn walk<'t>(root: Node<'t>, mut visit: impl FnMut(Node<'t>, usize)) {
    let mut cursor = root.walk();
    let mut depth = 0usize;
    'walk: loop {
        visit(cursor.node(), depth);
        if cursor.goto_first_child() {
            depth += 1;
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                break 'walk;
            }
            depth -= 1;
        }
    }
}

/// The children of `node` as a vector.
pub(crate) fn children<'t>(node: Node<'t>) -> Vec<Node<'t>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).collect()
}
