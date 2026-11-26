//! Live scripts: `.mlx` files and plain-text live code (`.m` files of MATLAB R2025a).
//!
//! A live script is read into runs of [`LiveSection`]s: consecutive text paragraphs,
//! or consecutive code lines. Outputs, images and the `%[appendix]` (metadata) are not
//! sections. Both formats are read without evaluating anything.
//!
//! - `.mlx` is a zip archive whose `matlab/document.xml` holds the paragraphs. A
//!   paragraph with the style `matlab-Code` is code, any other paragraph is text. A
//!   file that is not a zip archive is read as plain text (as maxx does).
//! - Plain-text live code: a `%%` line starts a new cell. Lines `%[text] ...` are text
//!   (the marker, and any `{...}` attributes after it, are removed). A `%[output...]`
//!   or other `%[...]` marker line is followed by its `%`-prefixed lines, which are
//!   skipped. Everything else is code. The text stops at `%[appendix]`.
//!
//! The live script's name is its file stem. It has no docstring.

use std::io::{Cursor, Read};
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::IndexError;
use crate::model::{Detail, Entity, Kind, LiveSection, SectionKind};

/// The largest `document.xml` read from a `.mlx` (a guard against archive bombs).
const MAX_DOCUMENT_BYTES: u64 = 64 * 1024 * 1024;

/// Whether the file is a `.mlx` (by its extension, in any case).
pub fn is_mlx(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("mlx"))
}

/// Read a live script from disk: a `.mlx` (binary or plain text) by its contents.
pub fn parse_live_file(path: &Path) -> Result<Entity, IndexError> {
    let bytes = std::fs::read(path).map_err(|e| IndexError::io(path, &e))?;
    parse_live_bytes(path, &bytes, false)
}

/// Read a plain-text live code file from disk, whatever its extension.
pub fn parse_live_text_file(path: &Path) -> Result<Entity, IndexError> {
    let bytes = std::fs::read(path).map_err(|e| IndexError::io(path, &e))?;
    parse_live_bytes(path, &bytes, true)
}

/// Read a live script from its bytes. With `plaintext` the bytes are read as
/// plain-text live code, whatever they are; otherwise a zip archive is a `.mlx`.
pub fn parse_live_bytes(path: &Path, bytes: &[u8], plaintext: bool) -> Result<Entity, IndexError> {
    log::debug!("parsing live script {}", path.display());
    let sections = if !plaintext && bytes.starts_with(b"PK") {
        sections_of_mlx(path, bytes)?
    } else {
        sections_of_text(&decode(bytes))
    };
    log::info!(
        "parsed live script {}: {} section(s)",
        path.display(),
        sections.len()
    );
    let mut entity = Entity::new(Kind::LiveScript, &stem(path));
    entity.path = Some(path.to_path_buf());
    entity.detail = Detail::LiveScript { sections };
    Ok(entity)
}

/// The file stem, or an empty string.
fn stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Text as UTF-8, or as Latin-1 when it is not UTF-8; a UTF-8 BOM is dropped.
pub(crate) fn decode(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => bytes.iter().map(|b| char::from(*b)).collect(),
    }
}

// ---------------------------------------------------------------------------
// .mlx
// ---------------------------------------------------------------------------

/// The sections of a `.mlx`: the paragraphs of `matlab/document.xml`, in order, grouped
/// into runs of one kind. A document that is not well-formed XML has no sections (with a
/// warning), as maxx reads it.
fn sections_of_mlx(path: &Path, bytes: &[u8]) -> Result<Vec<LiveSection>, IndexError> {
    let format = |message: &str| IndexError::Format {
        path: path.to_path_buf(),
        message: message.to_string(),
    };
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| format("not a valid .mlx (zip) file"))?;
    let name = archive
        .file_names()
        .find(|n| n.ends_with("document.xml"))
        .map(str::to_string)
        .ok_or_else(|| format("no document.xml inside the .mlx archive"))?;
    let entry = archive
        .by_name(&name)
        .map_err(|_| format("cannot read document.xml inside the .mlx archive"))?;
    let mut xml = Vec::new();
    entry
        .take(MAX_DOCUMENT_BYTES + 1)
        .read_to_end(&mut xml)
        .map_err(|_| format("cannot read document.xml inside the .mlx archive"))?;
    if xml.len() as u64 > MAX_DOCUMENT_BYTES {
        return Err(format("document.xml is larger than 64 MiB"));
    }
    let paragraphs = match paragraphs_of_xml(&xml) {
        Ok(paragraphs) => paragraphs,
        Err(err) => {
            log::warn!("{}: cannot read document.xml: {err}", path.display());
            return Ok(Vec::new());
        }
    };
    Ok(group_paragraphs(paragraphs))
}

/// The paragraphs (kind, text) of a document, in order: the `p` elements inside `body`,
/// whatever their namespace prefix. The text of a paragraph is its `t` elements in runs
/// (`r`), concatenated.
fn paragraphs_of_xml(xml: &[u8]) -> Result<Vec<(SectionKind, String)>, quick_xml::Error> {
    let mut reader = Reader::from_reader(xml);
    let mut paragraphs = Vec::new();
    let mut body_depth = 0usize;
    // The open paragraph: its kind, its text, and the depth of runs and text elements.
    let mut current: Option<(SectionKind, String)> = None;
    let mut run_depth = 0usize;
    let mut in_text = false;
    let mut para_depth = 0usize;
    let mut depth = 0usize;
    loop {
        let event = reader.read_event()?;
        match event {
            Event::Start(e) => {
                depth += 1;
                let name = e.local_name();
                match name.as_ref() {
                    "body" => body_depth += 1,
                    "p" if body_depth > 0 && current.is_none() => {
                        current = Some((SectionKind::Text, String::new()));
                        para_depth = depth;
                    }
                    "pStyle" if current.is_some() => {
                        if style_is_code(&e) {
                            if let Some(paragraph) = current.as_mut() {
                                paragraph.0 = SectionKind::Code;
                            }
                        }
                    }
                    "r" if current.is_some() => run_depth += 1,
                    "t" if current.is_some() && run_depth > 0 => in_text = true,
                    _ => {}
                }
            }
            Event::Empty(e) => {
                if e.local_name().as_ref() == "pStyle" && current.is_some() && style_is_code(&e) {
                    if let Some(paragraph) = current.as_mut() {
                        paragraph.0 = SectionKind::Code;
                    }
                }
            }
            Event::Text(t) if in_text => {
                if let Some(paragraph) = current.as_mut() {
                    paragraph.1.push_str(&t.xml10_content());
                }
            }
            // Entity and character references arrive as events of their own.
            Event::GeneralRef(r) if in_text => {
                if let Some(paragraph) = current.as_mut() {
                    if let Some(c) = r.resolve_char_ref()? {
                        paragraph.1.push(c);
                    } else if let Some(s) = quick_xml::escape::resolve_predefined_entity(&r) {
                        paragraph.1.push_str(s);
                    }
                }
            }
            Event::CData(c) if in_text => {
                if let Some(paragraph) = current.as_mut() {
                    paragraph.1.push_str(&c.into_inner());
                }
            }
            Event::End(e) => {
                let name = e.local_name();
                match name.as_ref() {
                    "body" => body_depth = body_depth.saturating_sub(1),
                    "t" => in_text = false,
                    "r" => run_depth = run_depth.saturating_sub(1),
                    "p" if current.is_some() && depth == para_depth => {
                        paragraphs.extend(current.take());
                    }
                    _ => {}
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(paragraphs)
}

/// Whether a `pStyle` element names a code style: `code` (written by MATLAB) or
/// `matlab-Code`.
fn style_is_code(e: &quick_xml::events::BytesStart) -> bool {
    e.attributes().flatten().any(|a| {
        a.key.local_name().as_ref() == "val" && matches!(a.value.as_ref(), "code" | "matlab-Code")
    })
}

/// Group paragraphs into runs of one kind. Empty paragraphs add nothing, and runs
/// without text are dropped.
fn group_paragraphs(paragraphs: Vec<(SectionKind, String)>) -> Vec<LiveSection> {
    let mut sections = Vec::new();
    let mut current: Option<SectionKind> = None;
    let mut lines: Vec<String> = Vec::new();
    let mut flush = |kind: Option<SectionKind>, lines: &mut Vec<String>| {
        if let Some(kind) = kind {
            if !lines.is_empty() {
                let content = lines.join("\n");
                if !content.trim().is_empty() {
                    sections.push(LiveSection {
                        kind,
                        content,
                        lineno: None,
                        endlineno: None,
                    });
                }
            }
        }
        lines.clear();
    };
    for (kind, text) in paragraphs {
        if Some(kind) != current {
            flush(current, &mut lines);
            current = Some(kind);
            if !text.is_empty() {
                lines.push(text);
            }
        } else if !text.is_empty() {
            lines.push(text);
        }
    }
    flush(current, &mut lines);
    sections
}

// ---------------------------------------------------------------------------
// Plain-text live code
// ---------------------------------------------------------------------------

/// The sections of plain-text live code, as described in the module docs.
pub(crate) fn sections_of_text(source: &str) -> Vec<LiveSection> {
    let mut sections: Vec<LiveSection> = Vec::new();
    // The run being built: its kind, and its lines as (lineno, text).
    let mut run: Option<(SectionKind, Vec<(usize, String)>)> = None;
    let mut in_metadata = false;

    let push = |run: &mut Option<(SectionKind, Vec<(usize, String)>)>,
                sections: &mut Vec<LiveSection>| {
        if let Some((kind, lines)) = run.take() {
            if let Some(section) = finish_run(kind, lines) {
                sections.push(section);
            }
        }
    };

    for (index, line) in source.lines().enumerate() {
        let lineno = index + 1;
        if let Some(marker) = marker_of(line) {
            in_metadata = false;
            if marker.name == "appendix" {
                break;
            }
            if marker.name == "text" || marker.name.starts_with("text:") {
                let text = text_content(marker.rest);
                if text.is_empty() {
                    continue;
                }
                if !matches!(run, Some((SectionKind::Text, _))) {
                    push(&mut run, &mut sections);
                    run = Some((SectionKind::Text, Vec::new()));
                }
                if let Some((_, lines)) = run.as_mut() {
                    lines.push((lineno, text.to_string()));
                }
                continue;
            }
            // Outputs, images and controls: their `%` lines follow and are skipped.
            in_metadata = true;
            continue;
        }
        if is_cell_marker(line) {
            push(&mut run, &mut sections);
            in_metadata = false;
            continue;
        }
        if in_metadata && line.starts_with('%') {
            continue;
        }
        in_metadata = false;
        if line.trim().is_empty() {
            // A blank line belongs to the code run it sits in, and never starts one.
            if let Some((SectionKind::Code, lines)) = run.as_mut() {
                lines.push((lineno, String::new()));
            }
            continue;
        }
        if !matches!(run, Some((SectionKind::Code, _))) {
            push(&mut run, &mut sections);
            run = Some((SectionKind::Code, Vec::new()));
        }
        if let Some((_, lines)) = run.as_mut() {
            lines.push((lineno, line.to_string()));
        }
    }
    push(&mut run, &mut sections);
    sections
}

/// A `%[name]...` marker line.
struct Marker<'a> {
    /// The name inside the brackets, with any colon part (`text`, `text:table`,
    /// `output:id`, `appendix`).
    name: &'a str,
    /// The rest of the line after the closing bracket.
    rest: &'a str,
}

/// The marker of a line that starts with `%[`, if any.
fn marker_of(line: &str) -> Option<Marker<'_>> {
    let inner = line.strip_prefix("%[")?;
    let close = inner.find(']')?;
    Some(Marker {
        name: &inner[..close],
        rest: &inner[close + 1..],
    })
}

/// The content of a text marker: the rest of the line after any `{...}` attributes,
/// trimmed.
fn text_content(rest: &str) -> &str {
    let rest = rest.trim_start();
    let rest = match rest
        .strip_prefix('{')
        .and_then(|r| r.find('}').map(|i| &r[i + 1..]))
    {
        Some(after) => after,
        None => rest,
    };
    rest.trim()
}

/// A cell marker: `%%` alone or followed by a space (a cell title).
fn is_cell_marker(line: &str) -> bool {
    let t = line.trim_start();
    match t.strip_prefix("%%") {
        Some(after) => after.is_empty() || after.starts_with(char::is_whitespace),
        None => false,
    }
}

/// A run's section: the content with leading and trailing blank lines removed, and the
/// span of its first and last non-blank line. `None` for a run without content.
fn finish_run(kind: SectionKind, mut lines: Vec<(usize, String)>) -> Option<LiveSection> {
    while lines.last().is_some_and(|(_, l)| l.trim().is_empty()) {
        lines.pop();
    }
    let first = lines.iter().position(|(_, l)| !l.trim().is_empty())?;
    let lines = &lines[first..];
    let lineno = lines.first()?.0;
    let endlineno = lines.last()?.0;
    let content = lines
        .iter()
        .map(|(_, l)| l.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    Some(LiveSection {
        kind,
        content,
        lineno: Some(lineno),
        endlineno: Some(endlineno),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_markers_and_text_markers() {
        assert!(is_cell_marker("%%"));
        assert!(is_cell_marker("%% Title"));
        assert!(!is_cell_marker("%%%"));
        assert!(!is_cell_marker("%[text] %%"));
        assert_eq!(text_content("{\"align\":\"center\"} $ x $"), "$ x $");
        assert_eq!(text_content(" # Title "), "# Title");
    }

    #[test]
    fn entities_and_cdata_are_part_of_the_text() {
        let xml = br#"<w:document><w:body><w:p><w:r><w:t>a &amp; b &lt; &#65;</w:t></w:r><w:r><w:t><![CDATA[<c>]]></w:t></w:r></w:p></w:body></w:document>"#;
        let paragraphs = paragraphs_of_xml(xml).unwrap();
        assert_eq!(paragraphs.len(), 1);
        assert_eq!(paragraphs[0].1, "a & b < A<c>");
    }
}
