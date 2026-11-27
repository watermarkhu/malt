//! `check_file_name`: one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_file_name(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if !self.is_check_enabled("BDFIL") {
            return diagnostics;
        }

        if let Some(stem) = ctx.file_path.file_stem() {
            let name = stem.to_string_lossy();

            let valid = !name.is_empty()
                && name.len() <= 2048
                && name.starts_with(|c: char| c.is_ascii_alphabetic())
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

            if !valid {
                diagnostics.push(Diagnostic {
                    rule_id: "BDFIL",
                    message: "Invalid MATLAB file name. MATLAB file names must start with a letter, contain only letters, numbers, or underscores, and have no more than 2048 characters.".to_string(),
                    severity: Severity::Error,
                    byte_range: 0..ctx.source.len().min(1),
                    line: 1,
                    column: 1,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::parse;

    /// The BDFIL findings for a file whose stem is `stem`.
    fn bdfil(stem: &str) -> Vec<Diagnostic> {
        let source = "x = 1;\n";
        let tree = parse(source);
        let path = std::path::PathBuf::from(format!("{stem}.m"));
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: &path,
            project: None,
        };
        let engine = SyntaxErrorsEngine {
            disabled: Default::default(),
        };
        engine.check_file_name(&ctx)
    }

    #[test]
    fn bdfil_fires_on_invalid_stems() {
        // Probed (_under.m, 9lead.m, a-b.m, dot.v2.m): MATLAB reports BDFIL.
        for stem in ["_under", "9lead", "a-b", "dot.v2"] {
            assert_eq!(bdfil(stem).len(), 1, "{stem}");
        }
    }

    #[test]
    fn bdfil_ok_on_valid_and_long_stems() {
        // Probed: a one-letter name, and names of 63, 64, 100 and 200 characters, are
        // silent in MATLAB (its message gives a limit of 2048 characters).
        let long = "a".repeat(200);
        for stem in ["a", "Ab_1", long.as_str()] {
            assert!(bdfil(stem).is_empty(), "{stem}");
        }
    }
}
