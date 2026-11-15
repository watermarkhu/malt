//! Live scripts: `.mlx` files (built here with the zip crate) and plain-text live code,
//! and their sections. Malformed input is an error or an empty result, never a panic.

use std::io::Write;
use std::path::{Path, PathBuf};

use malt_index::{
    parse_live_bytes, parse_live_file, parse_live_text_file, Index, Kind, LiveSection,
    ParserConfig, SectionKind,
};

/// A fresh folder under the system temp directory.
fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("malt-live-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

/// A `.mlx` archive holding `document` as `matlab/document.xml`.
fn mlx(document: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut bytes));
        zip.start_file(
            "matlab/document.xml",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        zip.write_all(document.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    bytes
}

const DOCUMENT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
<w:p><w:pPr><w:pStyle w:val="Title"/></w:pPr><w:r><w:t>Hello</w:t></w:r></w:p>
<w:p><w:r><w:t xml:space="preserve">World </w:t></w:r><w:r><w:t>é</w:t></w:r></w:p>
<w:p><w:pPr><w:pStyle w:val="matlab-Code"/></w:pPr><w:r><w:t>x = 1;</w:t></w:r></w:p>
<w:p><w:pPr><w:pStyle w:val="matlab-Code"/></w:pPr><w:r><w:t>y = 2;</w:t></w:r></w:p>
<w:p/>
<w:p><w:r><w:t>Tail</w:t></w:r></w:p>
</w:body></w:document>"#;

fn sections(entity: &malt_index::Entity) -> Vec<(SectionKind, String)> {
    entity
        .sections()
        .iter()
        .map(|s| (s.kind, s.content.clone()))
        .collect()
}

#[test]
fn mlx_paragraphs_group_into_text_and_code_runs() {
    let entity = parse_live_bytes(Path::new("Demo.mlx"), &mlx(DOCUMENT), false).unwrap();
    assert_eq!(entity.kind, Kind::LiveScript);
    assert_eq!(entity.name, "Demo");
    assert_eq!(
        sections(&entity),
        [
            (SectionKind::Text, "Hello\nWorld é".to_string()),
            (SectionKind::Code, "x = 1;\ny = 2;".to_string()),
            (SectionKind::Text, "Tail".to_string()),
        ]
    );
    // A .mlx paragraph has no line numbers.
    assert!(entity.sections().iter().all(|s| s.lineno.is_none()));
}

#[test]
fn mlx_without_namespace_prefixes_is_read_too() {
    let plain = r#"<document><body><p><pPr><pStyle val="matlab-Code"/></pPr><r><t>z = 3;</t></r></p></body></document>"#;
    let entity = parse_live_bytes(Path::new("Plain.mlx"), &mlx(plain), false).unwrap();
    assert_eq!(
        sections(&entity),
        [(SectionKind::Code, "z = 3;".to_string())]
    );
}

#[test]
fn mlx_with_malformed_xml_has_no_sections_and_no_panic() {
    let broken = "<w:document><w:body><w:p></w:x></w:body></w:document>";
    let entity = parse_live_bytes(Path::new("Broken.mlx"), &mlx(broken), false).unwrap();
    assert!(entity.sections().is_empty());
}

#[test]
fn a_file_that_is_not_a_zip_is_an_error_for_an_mlx_but_text_otherwise() {
    let err = parse_live_bytes(Path::new("Bad.mlx"), b"PK\x03\x04 not really", false).unwrap_err();
    assert!(err.to_string().contains("not a valid .mlx"), "{err}");
    // Forced as plain text, the same bytes are text.
    let text = parse_live_bytes(Path::new("Bad.m"), b"%[text] hi\n", true).unwrap();
    assert_eq!(sections(&text), [(SectionKind::Text, "hi".to_string())]);
}

#[test]
fn an_archive_without_document_xml_is_an_error() {
    let mut bytes = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut bytes));
        zip.start_file("other.xml", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"<x/>").unwrap();
        zip.finish().unwrap();
    }
    let err = parse_live_bytes(Path::new("Empty.mlx"), &bytes, false).unwrap_err();
    assert!(err.to_string().contains("document.xml"), "{err}");
}

#[test]
fn truncated_and_random_archives_never_panic() {
    let whole = mlx(DOCUMENT);
    for cut in [0, 2, 4, 10, 30, whole.len() / 2, whole.len() - 1] {
        let _ = parse_live_bytes(Path::new("Cut.mlx"), &whole[..cut], false);
    }
    let mut seed: u32 = 12345;
    for _ in 0..200 {
        let mut noise = b"PK\x03\x04".to_vec();
        for _ in 0..64 {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            noise.push((seed >> 16) as u8);
        }
        let _ = parse_live_bytes(Path::new("Noise.mlx"), &noise, false);
    }
}

const TEXT: &str = "%% Title\n\
%[text] # Head\n\
%[text]{\"align\":\"center\"} Some words.\n\
x = 1;\n\
\n\
y = 2;\n\
%[output:abc]{\"type\":\"text\"}\n\
%   data: {\"x\":1}\n\
%---\n\
%[text:table]\n\
%[text] | a | b |\n\
%[appendix]{\"version\":\"1.0\"}\n\
%---\n\
%[text] after the appendix\n";

#[test]
fn plain_text_live_code_is_sections_with_spans() {
    let entity = parse_live_bytes(Path::new("live.m"), TEXT.as_bytes(), true).unwrap();
    let got: Vec<&LiveSection> = entity.sections().iter().collect();
    assert_eq!(got.len(), 3, "{got:?}");
    assert_eq!(got[0].kind, SectionKind::Text);
    assert_eq!(got[0].content, "# Head\nSome words.");
    assert_eq!((got[0].lineno, got[0].endlineno), (Some(2), Some(3)));
    assert_eq!(got[1].kind, SectionKind::Code);
    assert_eq!(got[1].content, "x = 1;\n\ny = 2;");
    assert_eq!((got[1].lineno, got[1].endlineno), (Some(4), Some(6)));
    assert_eq!(got[2].kind, SectionKind::Text);
    assert_eq!(got[2].content, "| a | b |");
    assert_eq!((got[2].lineno, got[2].endlineno), (Some(11), Some(11)));
}

#[test]
fn outputs_and_the_appendix_are_not_sections_and_crlf_is_read() {
    let crlf = TEXT.replace('\n', "\r\n");
    let entity = parse_live_bytes(Path::new("crlf.m"), crlf.as_bytes(), true).unwrap();
    let all: String = sections(&entity).iter().map(|(_, c)| c.clone()).collect();
    assert!(!all.contains("data"), "{all}");
    assert!(!all.contains("appendix"), "{all}");
    assert!(!all.contains("after the appendix"), "{all}");
    assert!(!all.contains('\r'), "{all:?}");
}

#[test]
fn plain_text_without_markers_is_one_code_section() {
    let entity = parse_live_bytes(Path::new("code.m"), b"a = 1;\nb = a + 1;\n", true).unwrap();
    assert_eq!(
        sections(&entity),
        [(SectionKind::Code, "a = 1;\nb = a + 1;".to_string())]
    );
}

#[test]
fn a_live_script_file_is_read_from_disk_by_its_extension_and_contents() {
    let dir = scratch("disk");
    let mlx_path = dir.join("Shot.mlx");
    std::fs::write(&mlx_path, mlx(DOCUMENT)).unwrap();
    let text_path = dir.join("Text.m");
    std::fs::write(&text_path, TEXT).unwrap();
    let from_mlx = parse_live_file(&mlx_path).unwrap();
    assert_eq!(from_mlx.sections().len(), 3);
    let from_text = parse_live_text_file(&text_path).unwrap();
    assert_eq!(from_text.sections().len(), 3);
    assert!(parse_live_file(&dir.join("missing.mlx")).is_err());
}

#[test]
fn live_scripts_on_the_path_only_with_the_parser_flag() {
    let dir = scratch("flag");
    std::fs::write(dir.join("Shot.mlx"), mlx(DOCUMENT)).unwrap();
    std::fs::write(dir.join("plain.m"), "y = 1;\n").unwrap();

    let off = Index::scan(std::slice::from_ref(&dir), &ParserConfig::default());
    assert!(off.get("Shot").unwrap().is_none());

    let config = ParserConfig {
        parse_live_scripts: true,
        ..ParserConfig::default()
    };
    let on = Index::scan(std::slice::from_ref(&dir), &config);
    let shot = on.get("Shot").unwrap().unwrap();
    assert_eq!(shot.kind, Kind::LiveScript);
    assert_eq!(shot.sections().len(), 3);
    // A plain-text `.m` is never a live script on the path, even with the flag.
    assert_eq!(on.get("plain").unwrap().unwrap().kind, Kind::Script);
    assert!(on.member_names().contains(&"Shot"));
}

#[test]
fn a_section_has_no_docstring_and_the_dump_names_it() {
    let entity = parse_live_bytes(Path::new("d.m"), TEXT.as_bytes(), true).unwrap();
    assert!(entity.docstring.is_none());
    assert_eq!(entity.kind.as_str(), "live_script");
}
