//! Source rewriting before parsing, for `arguments` declarations that
//! tree-sitter-matlab 1.3 cannot parse.
//!
//! Two shapes lose the whole block (or part of it) to an ERROR node, although
//! MATLAB accepts them:
//! - a value in the block's attribute list, `arguments (Input = 'x')`;
//! - a class-typed name-value declaration, `opts.?pkg.Class = 1`.
//!
//! [`parse_text`] rewrites those spans into forms the grammar accepts. Each
//! rewrite replaces bytes with spaces one-for-one and keeps every newline, so
//! byte offsets and line numbers match the original source. The linter builds
//! the tree from the rewritten text and gives rules the original text, so checks
//! that need a value (ATLAB) or the `?` of a class name read it from the source.

/// The text to parse in place of `source`, or `None` when no rewrite applies.
/// The linter and the rule test helpers parse through this, so both see the same tree.
pub fn parse_text(source: &str) -> Option<String> {
    let mut out = source.as_bytes().to_vec();
    let mut changed = false;
    let mut in_arguments = false;
    let mut line_start = 0;
    for line in source.split_inclusive('\n') {
        let start = line_start;
        line_start += line.len();
        let trimmed = line.trim_start();
        if trimmed.starts_with('%') {
            continue;
        }
        let indent = start + (line.len() - trimmed.len());
        if !in_arguments {
            if let Some(rest) = trimmed.strip_prefix("arguments") {
                let rest = rest.trim_start();
                // `arguments(i) = 3` is an indexed assignment, not a block header.
                let after_close = line.rfind(')').map_or("", |i| &line[i + 1..]);
                let is_header =
                    rest.is_empty() || rest.starts_with('(') && !after_close.contains('=');
                if is_header {
                    in_arguments = true;
                    changed |= blank_attribute_values(line, start, &mut out);
                }
            }
        } else if is_end_line(trimmed) {
            in_arguments = false;
        } else {
            changed |= rewrite_class_name_default(trimmed, indent, &mut out);
        }
    }
    if changed {
        String::from_utf8(out).ok()
    } else {
        None
    }
}

/// True for a line that closes a block: `end`, optionally followed by a comment.
fn is_end_line(trimmed: &str) -> bool {
    trimmed
        .strip_prefix("end")
        .is_some_and(|rest| !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
}

/// Blanks the value of each attribute on an `arguments (...)` header line:
/// `Input = 'x', Repeating` becomes `Input         , Repeating`. The attribute
/// name stays, so the tree keeps the attribute; the value (and its `=` or `~`)
/// is read from the source by ATLAB. A list that does not close on the same
/// line is left as it is.
fn blank_attribute_values(line: &str, start: usize, out: &mut [u8]) -> bool {
    let bytes = line.as_bytes();
    let Some(open) = line.find('(') else {
        return false;
    };
    let mut segments = Vec::new();
    let mut segment_start = open + 1;
    let mut depth = 0usize;
    let mut quote: Option<u8> = None;
    let mut close = None;
    for (i, &b) in bytes.iter().enumerate().skip(open + 1) {
        if let Some(q) = quote {
            if b == q {
                quote = None;
            }
            continue;
        }
        match b {
            b'\'' | b'"' => quote = Some(b),
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' if depth == 0 => {
                close = Some(i);
                break;
            }
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                segments.push((segment_start, i));
                segment_start = i + 1;
            }
            _ => {}
        }
    }
    let Some(close) = close else {
        return false;
    };
    segments.push((segment_start, close));
    let mut changed = false;
    for (s, e) in segments {
        if let Some(eq) = bytes[s..e].iter().position(|&b| b == b'=') {
            for slot in &mut out[start + s + eq..start + e] {
                *slot = b' ';
            }
            changed |= s + eq < e;
        }
    }
    changed
}

/// Rewrites `name.?pkg.Class` at the start of an arguments-block line to
/// `name  pkg.Class`, the form of a typed declaration (`x pkg.Class = ...`), which
/// the grammar parses. The `.?` becomes two spaces.
fn rewrite_class_name_default(trimmed: &str, indent: usize, out: &mut [u8]) -> bool {
    let name_len = trimmed
        .bytes()
        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
        .count();
    let rest = &trimmed[name_len..];
    let is_class_name = rest.starts_with(".?")
        && name_len > 0
        && rest[2..].starts_with(|c: char| c.is_ascii_alphabetic());
    if is_class_name {
        out[indent + name_len] = b' ';
        out[indent + name_len + 1] = b' ';
    }
    is_class_name
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(source: &str) -> String {
        parse_text(source).unwrap_or_else(|| source.to_string())
    }

    #[test]
    fn unchanged_source_needs_no_rewrite() {
        assert!(parse_text("x = 1;\narguments (Input)\n    x double\nend\n").is_none());
    }

    #[test]
    fn attribute_values_are_blanked_and_offsets_kept() {
        let source = "arguments (Input = 'x', Repeating = {1, 2})\n    y double\nend\n";
        let text = parsed(source);
        assert_eq!(text.len(), source.len());
        let expected = format!(
            "arguments (Input{}, Repeating{})\n    y double\nend\n",
            " ".repeat(6),
            " ".repeat(9)
        );
        assert_eq!(text, expected);
    }

    #[test]
    fn attribute_values_with_brackets_and_negation_are_blanked() {
        let text = parsed("arguments (Input = ~true, Output = [1 2])\n    y\nend\n");
        assert!(text.starts_with("arguments (Input "));
        assert!(!text.contains('~') && !text.contains('['), "{text}");
    }

    #[test]
    fn plain_attributes_are_left_alone() {
        assert!(parse_text("arguments (Input, Repeating)\n    x double\nend\n").is_none());
    }

    #[test]
    fn unclosed_attribute_list_is_left_alone() {
        assert!(parse_text("arguments (Input = 1,\n    Repeating)\n    x double\nend\n").is_none());
    }

    #[test]
    fn class_name_default_becomes_typed_declaration() {
        let source = "arguments\n    opts.?matlab.unittest.TestCase = 1\n    x double\nend\n";
        let text = parsed(source);
        assert_eq!(text.len(), source.len());
        assert_eq!(
            text,
            "arguments\n    opts  matlab.unittest.TestCase = 1\n    x double\nend\n"
        );
    }

    #[test]
    fn class_name_outside_arguments_is_left_alone() {
        assert!(parse_text("opts.?pkg.Class = 1;\n").is_none());
    }

    #[test]
    fn indexed_assignment_to_arguments_is_not_a_block() {
        assert!(parse_text("arguments(1) = 3;\nx.?y = 2;\n").is_none());
    }

    #[test]
    fn variable_named_arguments_is_not_a_block() {
        assert!(parse_text("arguments = 1;\nx.?y = 2;\n").is_none());
    }
}
