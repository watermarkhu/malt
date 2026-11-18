//! Handle-default-value checks (MCHDP, MCHDT).
//!
//! A property default value that is (or resolves to) a handle instance is
//! created once when the class definition file is loaded and shared by every
//! instance, which is almost always unintended.
//!
//! - MCHDP fires when the default value *directly constructs* a handle: a
//!   known handle class constructor call (e.g. `containers.Map(...)`). Probed
//!   silent: `handle()`, `onCleanup(...)`, `figure()`, a bare `timer`, a
//!   constant property, and a constructor call to the file's own class (MATLAB
//!   reports MCPIN there).
//! - MCHDT fires when the default value is a bare identifier/expression that
//!   the type environment proves to be a handle-typed value.
//!
//! Heuristic note: external handle classes that are not in the known list and
//! not the file's own class cannot be resolved statically, so MCHDP only
//! recognizes the constructor-call patterns above and MCHDT only fires on
//! identifiers the type environment can prove to be handles.

use super::*;
use crate::analysis::typing::{TypeEnv, TypeKind};

/// Built-in / toolbox handle classes whose constructor calls are recognized by
/// MCHDP. A default value whose text starts with one of these names followed
/// by `(` is treated as a direct handle instantiation.
const KNOWN_HANDLE_CLASSES: &[&str] = &["containers.Map"];

impl GoodPracticesEngine {
    /// MCHDP / MCHDT: property default values that are (or resolve to) handle
    /// instances cause all class instances to share the same object data.
    pub(crate) fn check_handle_defaults(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        let any_enabled = self.is_check_enabled("MCHDP") || self.is_check_enabled("MCHDT");
        if !any_enabled {
            return Vec::new();
        }

        let meta = FileMeta::build(tree, source);
        let env = TypeEnv::build(tree, source);
        // MATLAB is silent on the default of a constant property (probed).
        let constants: std::collections::HashSet<&str> = meta
            .class
            .as_ref()
            .map(|c| {
                c.properties_blocks
                    .iter()
                    .filter(|pb| Attributes(&pb.attributes).is_present("Constant"))
                    .flat_map(|pb| pb.properties.iter().map(|p| p.name.as_str()))
                    .collect()
            })
            .unwrap_or_default();

        let mut diagnostics = Vec::new();
        for prop in meta.all_properties() {
            if constants.contains(prop.name.as_str()) {
                continue;
            }
            let Some(default) = prop.default_value.as_deref() else {
                continue;
            };
            let text = default.trim();

            // MCHDP: direct handle construction as the default value.
            if self.is_check_enabled("MCHDP") && is_handle_constructor_default(text) {
                diagnostics.push(Diagnostic {
                    rule_id: "MCHDP",
                    message: "A property default value that is a handle will cause all instances \
                              to share the same object data. To avoid sharing, create the property \
                              value in the constructor. For intentional sharing, consider using a \
                              Constant property."
                        .to_string(),
                    severity: Severity::Warning,
                    byte_range: prop.byte_range.clone(),
                    line: prop.line,
                    column: 1,
                    fix: None,
                });
                continue;
            }

            // MCHDT: the default is an identifier that resolves to a handle.
            if self.is_check_enabled("MCHDT")
                && is_simple_identifier(text)
                && env.known_kind(text) == Some(TypeKind::Handle)
            {
                diagnostics.push(Diagnostic {
                    rule_id: "MCHDT",
                    message: "Declaring the value of a property as a handle might cause all \
                              instances to share the same default handle. To avoid sharing, \
                              create the handle for this property in the constructor. To express \
                              that sharing is intentional, use the Constant property attribute."
                        .to_string(),
                    severity: Severity::Warning,
                    byte_range: prop.byte_range.clone(),
                    line: prop.line,
                    column: 1,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

/// Whether a property default value text directly constructs a handle:
/// a known handle class (or the file's own handle class) followed by `(`.
fn is_handle_constructor_default(text: &str) -> bool {
    KNOWN_HANDLE_CLASSES
        .iter()
        .any(|class| text.starts_with(class) && text[class.len()..].trim_start().starts_with('('))
}

/// Whether a default value text is a plain identifier (no call, indexing, or
/// dot access), so the type environment can be consulted for it.
fn is_simple_identifier(text: &str) -> bool {
    !text.is_empty()
        && !text.contains('(')
        && !text.contains('.')
        && text
            .chars()
            .enumerate()
            .all(|(i, c)| c == '_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle_ids(source: &str, check: &str) -> Vec<&'static str> {
        let tree = parse(source);
        let eng = engine();
        eng.check_handle_defaults(&tree, source)
            .iter()
            .filter(|d| d.rule_id == check)
            .map(|d| d.rule_id)
            .collect()
    }

    // -- MCHDP --------------------------------------------------------------

    #[test]
    fn test_mchdp_fires_on_handle_constructor_default() {
        let source = "\
classdef Foo < handle
    properties
        p1 = handle()
        p2 = onCleanup(@cleanup)
        p3 = containers.Map()
        p4 = timer()
        p5 = Foo()
    end
end
";
        // Probed: only `containers.Map()` is reported; `handle()` (P2c01), `onCleanup`
        // (P2c02), `timer()` (P2c03) and the own-class default (P2a115, MCPIN) are silent.
        let ids = handle_ids(source, "MCHDP");
        assert_eq!(ids.len(), 1, "got: {:?}", handle_ids(source, "MCHDP"));
    }

    #[test]
    fn test_mchdp_silent_for_value_defaults() {
        let source = "\
classdef Foo < handle
    properties
        p1 = 42
        p2 = 'hello'
        p3 = [1 2 3]
        p4 = false
    end
end
";
        assert!(handle_ids(source, "MCHDP").is_empty());
    }

    #[test]
    fn test_mchdp_silent_for_value_class_own_constructor() {
        let source = "\
classdef Foo
    properties
        p = Foo()
    end
end
";
        assert!(handle_ids(source, "MCHDP").is_empty());
    }

    // -- MCHDT --------------------------------------------------------------

    #[test]
    fn test_mchdt_fires_when_default_resolves_to_handle() {
        let source = "\
classdef Foo < handle
    properties
        p = sharedH
    end
    methods
        function f(obj)
            sharedH = onCleanup(@cleanup);
        end
    end
end
";
        let ids = handle_ids(source, "MCHDT");
        assert_eq!(ids.len(), 1, "got: {:?}", handle_ids(source, "MCHDT"));
    }

    #[test]
    fn test_mchdt_silent_when_default_is_unknown_identifier() {
        let source = "\
classdef Foo < handle
    properties
        p = unknownValue
    end
end
";
        assert!(handle_ids(source, "MCHDT").is_empty());
    }

    #[test]
    fn test_mchdt_silent_when_default_is_numeric() {
        let source = "\
classdef Foo < handle
    properties
        p = 42
    end
end
";
        assert!(handle_ids(source, "MCHDT").is_empty());
    }

    // -- config -------------------------------------------------------------

    #[test]
    fn test_handle_defaults_checks_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec!["MCHDP".to_string(), "MCHDT".to_string()],
            },
        };
        let source = "\
classdef Foo < handle
    properties
        p1 = handle()
    end
end
";
        let tree = parse(source);
        assert!(eng.check_handle_defaults(&tree, source).is_empty());
    }
}

#[cfg(test)]
mod probe_p2a_tests {
    use crate::test_util::lint_file;
    use malt_core::Config;

    /// Lines of the `id` diagnostics that the default (mathworks) engine reports for `src`.
    fn lines(src: &str, id: &str) -> Vec<usize> {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_file(&*e, src)
            .iter()
            .filter(|d| d.rule_id == id)
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn mchdp_is_silent_on_constants_and_on_handles_other_than_containers_map() {
        // Probed: `containers.Map()` (113) fires; a constant (118), `onCleanup` (114),
        // `figure()` (119) and an own-class handle default (115) are silent.
        let map = "classdef C\n properties\n  H = containers.Map()\n end\nend\n";
        let constant = "classdef C\n properties (Constant)\n  H = containers.Map()\n end\nend\n";
        let cleanup = "classdef C\n properties\n  H = onCleanup(@() 1)\n end\nend\n";
        let own = "classdef C < handle\n properties\n  H = C()\n end\nend\n";
        assert_eq!(lines(map, "MCHDP"), vec![3]);
        assert!(lines(constant, "MCHDP").is_empty());
        assert!(lines(cleanup, "MCHDP").is_empty());
        assert!(lines(own, "MCHDP").is_empty());
    }
}
