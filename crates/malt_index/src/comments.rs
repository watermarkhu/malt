//! Comment blocks as docstrings: which comment lines form the text, with the
//! comment markers removed and the common indentation taken off.
//!
//! This only extracts the text (no section parsing). The rules follow maxx's
//! `_comment_docstring`: a block is the run of comments on adjacent lines, `%%`
//! lines are cell titles, `%{ ... %}` is a block comment, `%#` pragmas and lines
//! containing `--8<--` are dropped.

use tree_sitter::Node;

use crate::model::Docstring;

/// The text of a node, decoded lossily (MATLAB files are not always UTF-8).
pub(crate) fn node_text(src: &[u8], node: Node) -> String {
    let bytes = src.get(node.start_byte()..node.end_byte()).unwrap_or(&[]);
    String::from_utf8_lossy(bytes).into_owned()
}

/// The docstring of a run of comment nodes, or `None` when it has no text.
///
/// Only the first block is used: a blank line between two comments ends the block.
pub(crate) fn comment_docstring(src: &[u8], nodes: &[Node]) -> Option<Docstring> {
    let mut nodes: Vec<Node> = nodes.to_vec();
    nodes.sort_by_key(|n| n.start_byte());
    if nodes.is_empty() {
        return None;
    }
    if let Some(gap) = nodes.windows(2).position(|w| {
        w[1].start_position()
            .row
            .saturating_sub(w[0].end_position().row)
            > 1
    }) {
        nodes.truncate(gap + 1);
    }
    let lineno = nodes[0].start_position().row + 1;
    let endlineno = nodes[nodes.len() - 1].end_position().row + 1;
    let texts: Vec<String> = nodes.iter().map(|n| node_text(src, *n)).collect();
    let raw = texts.join("\n");
    let lines: Vec<String> = texts
        .iter()
        .flat_map(|t| t.lines().map(str::to_string))
        .collect();

    let value = docstring_text(&lines)?;
    Some(Docstring {
        value: normalize(&value),
        raw,
        lineno,
        endlineno,
    })
}

/// The docstring value as maxx (through griffe) keeps it: `inspect.cleandoc`, then
/// trailing whitespace removed.
pub(crate) fn normalize(text: &str) -> String {
    cleandoc(text).trim_end().to_string()
}

/// Python's `inspect.cleandoc`: tabs expanded to 8, the first line stripped, the
/// common indentation of the other lines removed, and leading and trailing blank
/// lines dropped.
pub(crate) fn cleandoc(text: &str) -> String {
    let expanded: Vec<String> = text.split('\n').map(expand_tabs).collect();
    let mut lines: Vec<String> = expanded;
    let margin = lines
        .iter()
        .skip(1)
        .filter(|l| !l.trim_start().is_empty())
        .map(|l| l.chars().count() - l.trim_start().chars().count())
        .min();
    if let Some(first) = lines.first_mut() {
        *first = first.trim_start().to_string();
    }
    if let Some(margin) = margin {
        for line in lines.iter_mut().skip(1) {
            *line = line.chars().skip(margin).collect();
        }
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    let start = lines.iter().take_while(|l| l.is_empty()).count();
    lines.drain(..start);
    lines.join("\n")
}

/// Tabs to spaces at multiples of 8 columns, as Python's `str.expandtabs(8)`.
fn expand_tabs(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut column = 0;
    for c in line.chars() {
        match c {
            '\t' => {
                let pad = 8 - column % 8;
                out.extend(std::iter::repeat_n(' ', pad));
                column += pad;
            }
            '\n' | '\r' => {
                out.push(c);
                column = 0;
            }
            _ => {
                out.push(c);
                column += 1;
            }
        }
    }
    out
}

/// The docstring text of comment lines (markers removed), or `None` when empty.
fn docstring_text(lines: &[String]) -> Option<String> {
    let mut docstring: Vec<String> = Vec::new();
    let mut uncommented: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim_start().to_string();
        i += 1;
        if line.starts_with("%#") || line.contains("--8<--") {
            continue;
        }
        if line.starts_with("%{") || line.starts_with("%%") {
            if !uncommented.is_empty() {
                docstring.extend(dedent(&uncommented));
                uncommented.clear();
            }
            if let Some(title) = line.strip_prefix("%%") {
                docstring.push(title.trim_start().to_string());
                continue;
            }
            let mut block: Vec<String> = Vec::new();
            let mut current = line[2..].to_string();
            loop {
                if let Some(end) = current.find("%}") {
                    let last = &current[..end];
                    if !last.is_empty() {
                        block.push(last.to_string());
                    }
                    break;
                }
                block.push(current);
                if i >= lines.len() {
                    break;
                }
                current = lines[i].clone();
                i += 1;
            }
            if let Some(first) = block.first() {
                docstring.push(first.clone());
                docstring.extend(dedent(&block[1..]));
            }
        } else if let Some(rest) = line.strip_prefix('%') {
            uncommented.push(rest.to_string());
        } else if line.is_empty() {
            uncommented.push(String::new());
        }
        // Any other line cannot come from a comment node: nothing to take.
    }
    if !uncommented.is_empty() {
        docstring.extend(dedent(&uncommented));
    }
    if docstring.is_empty() {
        None
    } else {
        Some(docstring.join("\n"))
    }
}

/// Remove the common leading whitespace of the lines (Python's `textwrap.dedent`):
/// blank lines become empty.
fn dedent(lines: &[String]) -> Vec<String> {
    let margin = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| leading_whitespace(l))
        .reduce(common_prefix)
        .unwrap_or("");
    lines
        .iter()
        .map(|l| {
            if l.trim().is_empty() {
                String::new()
            } else {
                l[margin.len()..].to_string()
            }
        })
        .collect()
}

fn leading_whitespace(line: &str) -> &str {
    let end = line
        .char_indices()
        .find(|(_, c)| *c != ' ' && *c != '\t')
        .map(|(i, _)| i)
        .unwrap_or(line.len());
    &line[..end]
}

fn common_prefix<'a>(a: &'a str, b: &str) -> &'a str {
    let n = a
        .char_indices()
        .zip(b.chars())
        .take_while(|((_, x), y)| x == y)
        .last()
        .map(|((i, x), _)| i + x.len_utf8())
        .unwrap_or(0);
    &a[..n]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(lines: &[&str]) -> Option<String> {
        let owned: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        docstring_text(&owned)
    }

    #[test]
    fn line_comments_lose_their_marker_and_common_indent() {
        assert_eq!(
            text(&["% Hello", "%   indented"]).as_deref(),
            Some("Hello\n  indented")
        );
    }

    #[test]
    fn cell_titles_and_pragmas() {
        assert_eq!(
            text(&["%#codegen", "%% Title", "% body"]).as_deref(),
            Some("Title\nbody")
        );
        assert_eq!(text(&["%#ok<NOPRT>"]), None);
    }

    #[test]
    fn block_comment_keeps_its_lines() {
        assert_eq!(
            text(&["%{", "  Block doc", "    more", "%}"]).as_deref(),
            Some("\nBlock doc\n  more")
        );
    }

    #[test]
    fn single_line_block() {
        assert_eq!(text(&["%{ inline %}"]).as_deref(), Some(" inline "));
        assert_eq!(text(&["%{%}"]), None);
    }

    #[test]
    fn empty_comment_is_empty_text() {
        assert_eq!(text(&["%"]).as_deref(), Some(""));
    }

    #[test]
    fn dedent_ignores_blank_lines() {
        let lines = vec!["  a".to_string(), "   ".to_string(), "    b".to_string()];
        assert_eq!(dedent(&lines), vec!["a", "", "  b"]);
    }
}
