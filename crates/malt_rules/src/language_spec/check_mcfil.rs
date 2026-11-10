//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCFIL: Class name must match file name.
    pub(crate) fn check_mcfil(
        &self,
        class: &ClassMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let file_stem = ctx
            .file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        // MATLAB reports BDFIL and no MCFIL for a file name that is not a valid
        // MATLAB identifier (for example one containing a hyphen).
        let valid_file_name = file_stem
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
            && file_stem
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_');
        if valid_file_name && file_stem != class.name {
            // Don't fire if the file is in a @directory (MCDIR handles that)
            let in_at_dir = ctx
                .file_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .is_some_and(|dir| dir.starts_with('@'));
            if !in_at_dir {
                diagnostics.push(Diagnostic {
                    rule_id: "MCFIL",
                    message: format!(
                        "Class name {} and file name do not agree: {}. Update the class name and constructor, if defined, or change the file name to match the class name.",
                        class.name, file_stem
                    ),
                    severity: Severity::Error,
                    byte_range: class.byte_range.clone(),
                    line: class.line,
                    column: 1,
                    fix: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mcfil_class_file_name_mismatch() {
        let source = "\
classdef MyClass
    methods
    end
end
";
        // File is named "WrongName.m" but class is "MyClass"
        let diags = check_source(source, "WrongName.m");
        let mcfil = filter_by_id(&diags, "MCFIL");
        assert!(
            !mcfil.is_empty(),
            "MCFIL should fire when class name != file name"
        );
    }

    #[test]
    fn test_mcfil_no_fire_for_invalid_file_name() {
        // MATLAB reports BDFIL (not MCFIL) for a file name with a hyphen.
        let source = "\
classdef MyClass
    methods
    end
end
";
        let diags = check_source(source, "my-file.m");
        assert!(filter_by_id(&diags, "MCFIL").is_empty());
    }

    #[test]
    fn test_mcfil_no_fire_when_match() {
        let source = "\
classdef MyClass
    methods
    end
end
";
        let diags = check_source(source, "MyClass.m");
        let mcfil = filter_by_id(&diags, "MCFIL");
        assert!(mcfil.is_empty(), "MCFIL should NOT fire when names match");
    }
}
