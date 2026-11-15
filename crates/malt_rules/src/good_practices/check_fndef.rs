use super::*;

/// True when the header line (up to a comment) has a `_` name or a `~` in the output
/// list, which MATLAB cannot parse. A `~` among the inputs (`f(~, x)`) is valid.
fn header_has_invalid_token(text: &str) -> bool {
    let header = text.lines().next().unwrap_or("");
    let header = header.split('%').next().unwrap_or("");
    let outputs = header.split('=').next().filter(|_| header.contains('='));
    outputs.is_some_and(|out| out.contains('~'))
        || header
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .any(|token| token.starts_with('_'))
}

impl GoodPracticesEngine {
    /// FNDEF: the main function of a function file has a name other than the
    /// file name. MATLAB calls the function by its file name, so the name in the
    /// `function` line is ignored.
    pub(crate) fn check_fndef(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        file_path: &std::path::Path,
    ) -> Vec<Diagnostic> {
        if !self.is_check_enabled("FNDEF") {
            return Vec::new();
        }
        let Some(stem) = file_path.file_stem().and_then(|s| s.to_str()) else {
            return Vec::new();
        };
        // MATLAB only names a file after a valid identifier. A file such as
        // `my-file.m` gets BDFIL instead, and MATLAB reports no FNDEF for it.
        if !is_identifier(stem) {
            return Vec::new();
        }

        // MATLAB reports nothing when a `function` line sits inside an arguments
        // block: the block is then not closed, and the function is not analysed as
        // the file's main function (probed with `function` inside `arguments`).
        if function_keyword_in_arguments_block(source) {
            return Vec::new();
        }

        let (name, line, byte_range) = match FileMeta::build(tree, source).main_function() {
            Some(main) => (main.name.clone(), main.line, main.byte_range.clone()),
            None => match broken_function_header(tree.root_node(), source) {
                Some(header) => header,
                None => return Vec::new(),
            },
        };
        if name.is_empty() || name == stem {
            return Vec::new();
        }
        // A header MATLAB cannot parse (`_y` or a `~` output) is not the main function (probed).
        if header_has_invalid_token(&source[byte_range.start..]) {
            return Vec::new();
        }
        let column = name_column(source, byte_range.start, &name);
        vec![Diagnostic {
            rule_id: "FNDEF",
            message: format!(
                "Function name {} is known to MATLAB by its file name: {}.",
                name, stem
            ),
            severity: Severity::Warning,
            byte_range,
            line,
            column,
            fix: None,
        }]
    }
}

/// The 1-based column of `name` on the `function` line that holds byte `start`.
/// MATLAB points at the name (probed: column 10 for `function other()`).
fn name_column(source: &str, start: usize, name: &str) -> usize {
    let line_start = source[..start].rfind('\n').map_or(0, |p| p + 1);
    let line = source[line_start..].lines().next().unwrap_or("");
    let keyword = line.find("function").map_or(0, |i| i + "function".len());
    line[keyword..]
        .find(name)
        .map_or(1, |i| line[..keyword + i].chars().count() + 1)
}

/// The name, line and byte range of a function whose header tree-sitter could
/// not parse: the root is an `ERROR` node that starts with `function`, with no
/// outputs (`function t3(x)`). A later syntax error makes the whole file an
/// error, and MATLAB still names the function (probed with `fprintf' hi;` in
/// the body).
fn broken_function_header(
    root: tree_sitter::Node,
    source: &str,
) -> Option<(String, usize, std::ops::Range<usize>)> {
    if root.kind() != "ERROR" || root.child(0)?.kind() != "function" {
        return None;
    }
    let mut cursor = root.walk();
    let children: Vec<_> = root.children(&mut cursor).collect();
    if children.iter().any(|c| c.kind() == "function_output") {
        return None;
    }
    let name = children.iter().find(|c| c.kind() == "identifier")?;
    let text = source[name.start_byte()..name.end_byte()].to_string();
    Some((text, root.start_position().row + 1, name.byte_range()))
}

/// True when a line that starts with `function` lies inside an `arguments`
/// block: the block starts at a line that is `arguments` (with an optional
/// attribute list) and ends at the next line that is `end`.
fn function_keyword_in_arguments_block(source: &str) -> bool {
    let mut in_arguments = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if in_arguments {
            if trimmed == "end" {
                in_arguments = false;
            } else if trimmed
                .strip_prefix("function")
                .is_some_and(|rest| rest.starts_with([' ', '\t']))
            {
                return true;
            }
        } else if trimmed == "arguments"
            || trimmed
                .strip_prefix("arguments")
                .is_some_and(|rest| rest.starts_with(['(', ' ']))
        {
            in_arguments = true;
        }
    }
    false
}

/// True when `s` is a MATLAB identifier: a letter, then letters, digits or `_`.
fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use crate::test_util::{has_id, parse};
    use malt_core::{Config, FileContext};
    use std::path::Path;

    fn fndef(src: &str, file: &str) -> Vec<malt_core::Diagnostic> {
        let engine = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        let tree = parse(src);
        let ctx = FileContext {
            tree: &tree,
            source: src,
            file_path: Path::new(file),
            project: None,
        };
        engine.check_file(&ctx)
    }

    #[test]
    fn fndef_fires_when_the_name_differs_from_the_file() {
        let d = fndef("function r = other(x)\n r = x;\nend\n", "mine.m");
        let m = d.iter().find(|d| d.rule_id == "FNDEF").expect("FNDEF");
        // MATLAB names the file without its extension ('mine', not 'mine.m').
        assert!(
            m.message.contains("other") && m.message.ends_with(": mine."),
            "{}",
            m.message
        );
    }

    #[test]
    fn fndef_silent_when_file_name_is_not_an_identifier() {
        // MATLAB reports BDFIL for `my-file.m` and no FNDEF (probe, R2026b).
        assert!(!has_id(
            &fndef("function r = other(x)\n r = x;\nend\n", "my-file.m"),
            "FNDEF"
        ));
        assert!(!has_id(
            &fndef("function r = other(x)\n r = x;\nend\n", "1fd.m"),
            "FNDEF"
        ));
    }

    #[test]
    fn fndef_ok_when_names_agree_or_in_scripts() {
        assert!(!has_id(
            &fndef("function r = mine(x)\n r = x;\nend\n", "mine.m"),
            "FNDEF"
        ));
        assert!(!has_id(&fndef("x = 1;\ndisp(x);\n", "mine.m"), "FNDEF"));
        // Local functions are not the file's main function.
        assert!(!has_id(
            &fndef("function a()\nend\nfunction b()\nend\n", "a.m"),
            "FNDEF"
        ));
    }

    #[test]
    fn fndef_fires_after_a_leading_blank_line_or_help_comment() {
        // Probed: MATLAB reports FNDEF at the name in fd04 (`% help` first) and fd11 (blank lines first).
        let d = fndef("% help text\nfunction other()\nend\n", "fd04.m");
        let m = d.iter().find(|d| d.rule_id == "FNDEF").expect("FNDEF");
        assert_eq!((m.line, m.column), (2, 10));
        assert!(has_id(
            &fndef("\n\nfunction other()\nend\n", "fd11.m"),
            "FNDEF"
        ));
    }

    #[test]
    fn fndef_fires_for_a_broken_header_with_a_later_syntax_error() {
        // MATLAB probe (readability__t3f): `function t3(x, ...)` in a file t3f.m
        // gets FNDEF on line 1 although a later line is a parse error.
        let src = "function t3(x)\nr1 = 1;\nz = fprintf' hi;\n";
        let d = fndef(src, "t3f.m");
        let m = d.iter().find(|d| d.rule_id == "FNDEF").expect("FNDEF");
        assert_eq!(m.line, 1, "{}", m.message);
        assert!(
            m.message.contains("t3") && m.message.ends_with(": t3f."),
            "{}",
            m.message
        );
    }

    #[test]
    fn fndef_silent_when_a_function_line_sits_in_an_arguments_block() {
        // MATLAB probe (t__fv4, and the same shape in a file pf5.m): no FNDEF
        // when `function q = inner()` is written inside the arguments block.
        let src = "function r = fvA(n)\n  arguments\n    n\n    function q = inner(), q = 1; end\n  end\n  r = 1;\nend\n";
        assert!(!has_id(&fndef(src, "pf5.m"), "FNDEF"));
    }

    #[test]
    fn fndef_fires_when_the_arguments_block_is_closed_before_a_function() {
        // MATLAB probe (pf4.m): the block is closed by its `end`, so the nested
        // function after it does not hide FNDEF.
        let src = "function r = fvA(n)\narguments\n n\nend\nfunction q = inner(), q = 1; end\nr = 1;\nend\n";
        assert!(has_id(&fndef(src, "pf4.m"), "FNDEF"));
    }
}
