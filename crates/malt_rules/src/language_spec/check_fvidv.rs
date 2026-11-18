//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVIDV: Validation or default values on ignored arguments are not supported.
    pub(crate) fn check_fvidv(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVIDV") {
            return;
        }
        for block in blocks {
            // MATLAB reports an ignored output as FVOOI, not FVIDV (probed).
            if block.attributes.iter().any(|a| a == "Output") {
                continue;
            }
            for prop in block_properties(block.node) {
                if !prop_has_ignored(prop) {
                    continue;
                }
                // MATLAB reports at the first token after `~`: the size or the class,
                // which may be dotted (probed: `~ (1,1) double`, `~ double {mustBeNumeric}`
                // and `~ pkg.Class` all report there). Without a size or class, at the
                // validators. A default value is reported at its value (probed: `~ (1,1)
                // double = 1` reports at the size and at the `1`).
                let mut cursor = prop.walk();
                let type_or_size = prop
                    .children(&mut cursor)
                    .filter(|c| matches!(c.kind(), "dimensions" | "identifier" | "property_name"))
                    .min_by_key(|c| c.start_byte());
                let head = type_or_size.or_else(|| find_child_kind(prop, "validation_functions"));
                if let Some(head) = head {
                    self.push_diag_bytes(
                        source,
                        head.start_byte(),
                        head.end_byte(),
                        "FVIDV",
                        "Specifying validation or default value for ignored arguments is not supported.",
                        diagnostics,
                    );
                }
                if let Some(default) = find_child_kind(prop, "default_value") {
                    let text = node_text(default, source);
                    let blanks = |after: &str| after.len() - after.trim_start().len();
                    let value_start = text.find('=').map_or(default.start_byte(), |eq| {
                        default.start_byte() + eq + 1 + blanks(&text[eq + 1..])
                    });
                    self.push_diag_bytes(
                        source,
                        value_start,
                        default.end_byte(),
                        "FVIDV",
                        "Specifying validation or default value for ignored arguments is not supported.",
                        diagnostics,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvidv_fires_validation_on_ignored() {
        let source = "\
function f(a, ~)
    arguments
        a (1,1) double
        ~ (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVIDV").is_empty(),
            "FVIDV should fire for validation on an ignored argument"
        );
    }

    #[test]
    fn test_fvidv_no_fire_bare_ignored() {
        let source = "\
function f(a, ~)
    arguments
        a (1,1) double
        ~
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVIDV").is_empty(),
            "FVIDV should NOT fire for a bare ignored argument"
        );
    }

    #[test]
    fn test_fvidv_reports_size_and_default_of_ignored_argument() {
        // Probed: `~ (1,1) double = 1` reports at the size (column 7) and at the value (column 22).
        let source = "\
function f(~, b)
arguments
    ~ (1,1) double = 1
    b double
end
";
        let diags = check_source(source, "f.m");
        let fvidv = filter_by_id(&diags, "FVIDV");
        assert_eq!(fvidv.len(), 2, "got: {diags:?}");
        assert_eq!((fvidv[0].line, fvidv[0].column), (3, 7));
        assert_eq!((fvidv[1].line, fvidv[1].column), (3, 22));
    }

    #[test]
    fn test_fvidv_silent_ignored_output() {
        // MATLAB reports an ignored output as FVOOI, not FVIDV (probed).
        let source = "\
function y = f(a)
arguments (Input)
    a double
end
arguments (Output)
    ~ (1,1) double
end
y = a;
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVIDV").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_fvidv_reports_class_name_of_ignored_argument() {
        // Probed: `~ pkg.Class` reports at the class name (column 7).
        let source = "\
function f(~, b)
arguments
    ~ pkg.Class
    b double
end
";
        let diags = check_source(source, "f.m");
        let fvidv = filter_by_id(&diags, "FVIDV");
        assert_eq!(fvidv.len(), 1, "got: {diags:?}");
        assert_eq!((fvidv[0].line, fvidv[0].column), (3, 7));
    }
}
