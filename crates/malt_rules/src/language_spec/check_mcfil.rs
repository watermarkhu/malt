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
            // Probed (plan 6.2): MCFIL is independent of an `@directory`; a class
            // in `@Qux/Qux.m` named `Bar` gets MCDIR and MCFIL.
            {
                // MATLAB's own fix renames the classdef AND every constructor
                // method to the file name (probed on fix/t__Cls.m:
                // `classdef Cls` becomes `classdef t__Cls` and
                // `function r = Cls(~, x)` becomes `function r = t__Cls(~, x)`).
                // The rewrite is UNSAFE: a method that was never the
                // constructor (its name did not match the class the runtime
                // uses, which is the file name) becomes one, so constructing
                // the object starts running it.
                let mut edits = Vec::new();
                if let Some(range) = class_name_range(ctx.source, &class.byte_range, &class.name) {
                    edits.push(malt_core::Fix::new(range, file_stem.to_string()));
                }
                for method in class
                    .methods_blocks
                    .iter()
                    .flat_map(|b| b.methods.iter())
                    .filter(|m| m.is_constructor)
                {
                    if let Some(range) = crate::good_practices::declared_name_range(
                        ctx.source,
                        method.byte_range.start,
                        &method.name,
                    ) {
                        edits.push(malt_core::Fix::new(range, file_stem.to_string()));
                    }
                }
                let fix = (!edits.is_empty()).then(|| {
                    let (primary, additional) = (edits[0].clone(), edits[1..].to_vec());
                    malt_core::Fix::with_additional(
                        primary.byte_range,
                        primary.replacement,
                        additional,
                    )
                    .unsafe_fix()
                });
                diagnostics.push(Diagnostic {
                    rule_id: "MCFIL",
                    message: format!(
                        "Class name '{}' and file name do not agree: '{}'. Update the class name and constructor, if defined, or change the file name to match the class name.",
                        class.name, file_stem
                    ),
                    severity: Severity::Error,
                    byte_range: class.byte_range.clone(),
                    line: class.line,
                    column: 1,
                    fix,
                });
            }
        }
    }
}

/// The byte range of the declared class `name` on the `classdef` line that
/// `class_range` starts. `None` when the name is not on that line.
fn class_name_range(
    source: &str,
    class_range: &std::ops::Range<usize>,
    name: &str,
) -> Option<std::ops::Range<usize>> {
    let start = class_range.start;
    let line = source[start..].lines().next()?;
    let after = line.find("classdef")? + "classdef".len();
    let tail = &line[after..];
    // Offsets inside `line` are relative to `start`; make them absolute.
    let offset = start + after + tail.find(name)?;
    let end = offset + name.len();
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let before_ok = source[..offset].chars().last().is_none_or(|c| !word(c));
    let after_ok = source[end..].chars().next().is_none_or(|c| !word(c));
    (before_ok && after_ok).then_some(offset..end)
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
    fn test_mcfil_fires_inside_an_at_directory() {
        // Probed (plan 6.2): `@Qux/Qux.m` holding `classdef Bar` is MCDIR and MCFIL;
        // `@Zed/other.m` holding `classdef Zed` is MCFIL alone.
        let diags = check_source("classdef Bar\nend\n", "@Qux/Qux.m");
        assert_eq!(filter_by_id(&diags, "MCFIL").len(), 1);
        let mcdir = filter_by_id(&diags, "MCDIR");
        assert_eq!(mcdir.len(), 1);
        // MATLAB quotes the class name and names the file by its absolute path.
        let message = &mcdir[0].message;
        assert!(message.starts_with("Class name 'Bar' and @directory name do not agree: '/"));
        assert!(message.ends_with("Qux.m'."), "{message}");
        let diags = check_source("classdef Zed\nend\n", "@Zed/other.m");
        assert_eq!(filter_by_id(&diags, "MCFIL").len(), 1);
        assert!(filter_by_id(&diags, "MCDIR").is_empty());
        let diags = check_source("classdef Foo\nend\n", "@Foo/Foo.m");
        assert!(filter_by_id(&diags, "MCFIL").is_empty());
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

    #[test]
    fn test_mcfil_fix_renames_classdef_and_constructor() {
        // Probed: MATLAB's fix on `classdef Cls` in t__Cls.m renames the
        // classdef and the constructor method to the file name, in one edit.
        let source = "\
classdef Cls
    methods
        function r = Cls(~, x)
            r = x;
        end
    end
end
";
        let diags = check_source(source, "t__Cls.m");
        let mcfil = filter_by_id(&diags, "MCFIL");
        let m = mcfil.first().expect("MCFIL");
        let fix = m.fix.as_ref().expect("the rename fix");
        assert!(!fix.is_safe(), "the constructor starts being one");
        // Back to front, as the applier does.
        let mut edits: Vec<(std::ops::Range<usize>, String)> = fix
            .additional_edits
            .iter()
            .map(|e| (e.byte_range.clone(), e.replacement.clone()))
            .chain(std::iter::once((
                fix.byte_range.clone(),
                fix.replacement.clone(),
            )))
            .collect();
        edits.sort_by_key(|(r, _)| std::cmp::Reverse(r.start));
        let mut out = String::from(source);
        for (range, text) in edits {
            out.replace_range(range, &text);
        }
        assert_eq!(
            out,
            "\
classdef t__Cls
    methods
        function r = t__Cls(~, x)
            r = x;
        end
    end
end
"
        );
    }
}
