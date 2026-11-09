use super::*;

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

        let meta = FileMeta::build(tree, source);
        let Some(main) = meta.main_function() else {
            return Vec::new();
        };
        if main.name.is_empty() || main.name == stem {
            return Vec::new();
        }
        vec![Diagnostic {
            rule_id: "FNDEF",
            message: format!(
                "Function name {} is known to MATLAB by its file name: {}.",
                main.name, stem
            ),
            severity: Severity::Warning,
            byte_range: main.byte_range.clone(),
            line: main.line,
            column: 1,
            fix: None,
        }]
    }
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
}
