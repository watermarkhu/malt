//! # AT* class, property, method, and event attribute checks
//!
//! These checks validate the attribute lists of `classdef`, `properties`,
//! `methods`, and `events` blocks, plus the `Input`/`Output` attributes of
//! `arguments` blocks:
//!
//! - **ATUNK** — the attribute name is not a valid MATLAB attribute.
//! - **ATLAB** — the `Input`/`Output` arguments-block attribute is assigned a
//!   value or negated.
//! - **ATNAS** — a class-level meta-class attribute (`AllowedSubclasses`,
//!   `InferiorClasses`) has a value that is neither a single meta-class object
//!   nor a cell array of meta-class objects.
//! - **ATNPI** — a class-level access attribute has an unexpected value.
//! - **ATNPP** — an events-block access attribute has an unexpected value
//!   (no `immutable`).
//! - **ATPPI** — a properties-block access attribute has an unexpected value.
//! - **ATPPP** — a methods-block access attribute has an unexpected value
//!   (no `immutable`).
//! - **ATAS** — a non-class meta-class attribute has an unexpected value.
//! - **ATVIZE** — an attribute named `Visible` is used.
//!
//! See the parent module `super` for the shared engine and helpers.

use super::*;

/// Valid class-level attribute names (ATUNK).
const CLASS_ATTRS: &[&str] = &[
    "Abstract",
    "Sealed",
    "HandleCompatible",
    "Hidden",
    "ConstructOnLoad",
    "AllowedSubclasses",
    "Enumeration",
    "InferiorClasses",
    "ContainsNonPublicMembers",
    "Access",
    "DynamicProps",
    "ExplicitProperties",
];

/// Valid properties-block attribute names (ATUNK).
const PROPERTY_ATTRS: &[&str] = &[
    "Abstract",
    "Constant",
    "Dependent",
    "GetAccess",
    "SetAccess",
    "Access",
    "AbortSet",
    "GetObservable",
    "SetObservable",
    "Transient",
    "NonCopyable",
    "PartialMatch",
    "HasDefault",
    "Default",
    "Hidden",
    "ExplicitProperties",
];

/// Valid methods-block attribute names (ATUNK).
const METHOD_ATTRS: &[&str] = &[
    "Abstract",
    "Static",
    "Access",
    "Hidden",
    "Sealed",
    "AbstractMethods",
];

/// Valid events-block attribute names (ATUNK).
const EVENT_ATTRS: &[&str] = &["ListenAccess", "NotifyAccess", "Hidden"];

/// Attribute names that take a single meta-class object or a cell array of
/// meta-class objects as their value (ATNAS/ATAS).
const METACLASS_ATTRS: &[&str] = &["AllowedSubclasses", "InferiorClasses"];

/// Which level an attribute appears at; selects the access-value check ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttrLevel {
    /// Attributes on the `classdef` line.
    Class,
    /// Attributes on a `properties` block.
    Property,
    /// Attributes on a `methods` block.
    Method,
    /// Attributes on an `events` block.
    Event,
}

impl LanguageSpecEngine {
    /// AT* checks for class, properties-block, methods-block, and events-block
    /// attributes.
    pub(crate) fn check_class_attributes(
        &self,
        class: &ClassMeta,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let range = class.byte_range.clone();
        let line = class.line;

        for attr in &class.attributes {
            self.check_one_attribute(
                &range,
                line,
                attr,
                CLASS_ATTRS,
                AttrLevel::Class,
                diagnostics,
            );
        }
        for block in &class.properties_blocks {
            for attr in &block.attributes {
                self.check_one_attribute(
                    &range,
                    line,
                    attr,
                    PROPERTY_ATTRS,
                    AttrLevel::Property,
                    diagnostics,
                );
            }
        }
        for block in &class.methods_blocks {
            for attr in &block.attributes {
                self.check_one_attribute(
                    &range,
                    line,
                    attr,
                    METHOD_ATTRS,
                    AttrLevel::Method,
                    diagnostics,
                );
            }
        }
        for block in &class.events_blocks {
            for attr in &block.attributes {
                self.check_one_attribute(
                    &range,
                    line,
                    attr,
                    EVENT_ATTRS,
                    AttrLevel::Event,
                    diagnostics,
                );
            }
        }
    }

    /// Validate a single attribute against the valid-name set for its level.
    fn check_one_attribute(
        &self,
        range: &std::ops::Range<usize>,
        line: usize,
        attr: &AttributeMeta,
        valid_names: &[&str],
        level: AttrLevel,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // ATVIZE: the 'Visible' attribute is invalid for classes and events.
        if attr.name == "Visible" {
            if self.is_check_enabled("ATVIZE") {
                diagnostics.push(Diagnostic {
                    rule_id: "ATVIZE",
                    message: "The 'Visible' attribute is invalid for classes and events. \
                              Use the '~Hidden' attribute instead or omit the attribute \
                              since 'Hidden' is false by default."
                        .to_string(),
                    severity: Severity::Error,
                    byte_range: range.clone(),
                    line,
                    column: 1,
                    fix: None,
                });
            }
            return;
        }

        // Meta-class attribute value checks (ATNAS / ATAS). These also run for
        // names that are not valid at this level, so a meta-class attribute
        // placed on a property/method block still gets a value diagnostic.
        if METACLASS_ATTRS.contains(&attr.name.as_str()) {
            if let Some(value) = attr.value.as_deref() {
                if !meta_class_value_is_valid(value) {
                    let (rule_id, message) = match level {
                        AttrLevel::Class => (
                            "ATNAS",
                            "Set attribute to a single meta-class object or a cell array \
                             of meta-class objects.",
                        ),
                        _ => (
                            "ATAS",
                            "The attribute value is unexpected. Use a single meta-class \
                             object or a cell array of meta-class objects.",
                        ),
                    };
                    if self.is_check_enabled(rule_id) {
                        diagnostics.push(Diagnostic {
                            rule_id,
                            message: message.to_string(),
                            severity: Severity::Error,
                            byte_range: range.clone(),
                            line,
                            column: 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        // ATUNK: unknown attribute name.
        if !valid_names.contains(&attr.name.as_str()) {
            if self.is_check_enabled("ATUNK") {
                diagnostics.push(Diagnostic {
                    rule_id: "ATUNK",
                    message: "Unknown attribute name.".to_string(),
                    severity: Severity::Error,
                    byte_range: range.clone(),
                    line,
                    column: 1,
                    fix: None,
                });
            }
            // Do not run value checks for names that are not valid at this level.
            return;
        }

        // Access attribute value checks (ATNPI / ATNPP / ATPPI / ATPPP).
        if is_access_attr_name(&attr.name, level) {
            if let Some(value) = attr.value.as_deref() {
                let (allowed, rule_id, message) = match level {
                    AttrLevel::Class => (
                        &["public", "private", "protected", "immutable"][..],
                        "ATNPI",
                        "Set attribute to 'public', 'private', 'protected', 'immutable', \
                         or a cell array of meta-classes instead.",
                    ),
                    AttrLevel::Property => (
                        &["public", "private", "protected", "immutable"][..],
                        "ATPPI",
                        "The attribute value is unexpected. Use 'public', 'private', \
                         'protected', 'immutable', or a cell array of meta-classes instead.",
                    ),
                    AttrLevel::Method => (
                        &["public", "private", "protected"][..],
                        "ATPPP",
                        "The attribute value is unexpected. Use 'public', 'private', \
                         'protected', or a cell array of meta-classes instead.",
                    ),
                    AttrLevel::Event => (
                        &["public", "private", "protected"][..],
                        "ATNPP",
                        "Set attribute to 'public', 'private', 'protected', or a cell \
                         array of meta-classes instead.",
                    ),
                };
                if !access_value_is_valid(value, allowed) && self.is_check_enabled(rule_id) {
                    diagnostics.push(Diagnostic {
                        rule_id,
                        message: message.to_string(),
                        severity: Severity::Error,
                        byte_range: range.clone(),
                        line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }
    }

    /// ATLAB: `Input` and `Output` arguments-block attributes must not be
    /// assigned a value or negated.
    ///
    /// The tree-sitter grammar only accepts bare identifiers as arguments-block
    /// attributes, so a valued/negated form (`arguments (Input = true)`,
    /// `arguments (~Output)`) parses with an `ERROR` child inside the
    /// `attributes` node. We fire when such an `attributes` node names
    /// `Input` or `Output`.
    pub(crate) fn check_atlab(
        &self,
        root: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if node.kind() == "arguments_statement" {
                if let Some(attrs_node) = find_child_kind(node, "attributes") {
                    let mut names = Vec::new();
                    let mut has_error = false;
                    let mut cursor = attrs_node.walk();
                    for child in attrs_node.children(&mut cursor) {
                        match child.kind() {
                            "identifier" => names.push(node_text(child, source).to_string()),
                            "ERROR" => has_error = true,
                            _ => {}
                        }
                    }
                    if has_error
                        && names.iter().any(|n| n == "Input" || n == "Output")
                        && self.is_check_enabled("ATLAB")
                    {
                        self.push_diag(
                            attrs_node,
                            "ATLAB",
                            "Attribute 'Input' and 'Output' must not be assigned a value or negated.",
                            diagnostics,
                        );
                    }
                }
                continue;
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }
    }
}

/// Whether an access-attribute value is valid: one of the allowed access
/// keywords, a single meta-class object (`?Class`), or a cell array of
/// meta-classes (`{...}`).
fn access_value_is_valid(value: &str, allowed: &[&str]) -> bool {
    allowed.contains(&value) || value.starts_with('?') || value.starts_with('{')
}

/// Whether an attribute name is an access attribute at the given level.
fn is_access_attr_name(name: &str, level: AttrLevel) -> bool {
    match level {
        AttrLevel::Event => matches!(name, "ListenAccess" | "NotifyAccess"),
        _ => matches!(name, "Access" | "GetAccess" | "SetAccess"),
    }
}

/// Whether a meta-class attribute value is a single meta-class object or a
/// cell array of meta-class objects.
fn meta_class_value_is_valid(value: &str) -> bool {
    value.starts_with('?') || value.starts_with('{')
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};
    use malt_core::Rule;

    // -- ATUNK ---------------------------------------------------------------

    #[test]
    fn test_atunk_fires_unknown_class_attribute() {
        let source = "\
classdef (Frobnicate) Foo
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "ATUNK").is_empty(),
            "ATUNK should fire for an unknown class attribute"
        );
    }

    #[test]
    fn test_atunk_fires_unknown_property_and_method_attributes() {
        let source = "\
classdef Foo
    properties (Mystery)
        x
    end
    methods (Static, Weird)
        function y = f(self)
            y = 1;
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "ATUNK");
        assert_eq!(
            hits.len(),
            2,
            "ATUNK should fire for the unknown property and method attributes"
        );
    }

    #[test]
    fn test_atunk_no_fire_valid_attributes() {
        let source = "\
classdef (Sealed, AllowedSubclasses = ?Foo) Foo < handle
    properties (Access = private, Constant, GetAccess = private, SetAccess = private, Dependent, Transient)
        x double = 5
    end
    methods (Static, Access = protected, Sealed)
        function y = f(self)
            y = 1;
        end
    end
    events (ListenAccess = private, NotifyAccess = private)
        Done
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "ATUNK").is_empty(),
            "ATUNK should NOT fire for valid attributes"
        );
    }

    // -- ATLAB ---------------------------------------------------------------

    #[test]
    fn test_atlab_fires_valued_input_attribute() {
        let source = "\
function f()
    arguments (Input = true)
        x
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "ATLAB").is_empty(),
            "ATLAB should fire for 'arguments (Input = true)'"
        );
    }

    #[test]
    fn test_atlab_fires_negated_output_attribute() {
        let source = "\
function y = f(x)
    arguments (Output)
        y
    end
    arguments (~Output)
        z
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "ATLAB").is_empty(),
            "ATLAB should fire for 'arguments (~Output)'"
        );
    }

    #[test]
    fn test_atlab_no_fire_bare_attributes() {
        let source = "\
function y = f(x)
    arguments (Input)
        x
    end
    arguments (Output)
        y
    end
    arguments (Input, Repeating)
        z
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "ATLAB").is_empty(),
            "ATLAB should NOT fire for bare Input/Output attributes"
        );
    }

    // -- ATNAS ---------------------------------------------------------------

    #[test]
    fn test_atnas_fires_bad_metaclass_value() {
        let source = "\
classdef (AllowedSubclasses = Foo) MyClass
end
";
        let diags = check_source(source, "MyClass.m");
        assert!(
            !filter_by_id(&diags, "ATNAS").is_empty(),
            "ATNAS should fire when AllowedSubclasses is not a meta-class or cell array"
        );
    }

    #[test]
    fn test_atnas_no_fire_valid_metaclass_values() {
        let source = "\
classdef (AllowedSubclasses = {?Foo, ?Bar}, InferiorClasses = ?Baz) MyClass
end
";
        let diags = check_source(source, "MyClass.m");
        assert!(
            filter_by_id(&diags, "ATNAS").is_empty(),
            "ATNAS should NOT fire for valid meta-class values"
        );
    }

    // -- ATNPI / ATNPP / ATPPI / ATPPP ---------------------------------------

    #[test]
    fn test_atnpi_fires_bad_class_access_value() {
        let source = "\
classdef (Access = 42) Foo
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "ATNPI").is_empty(),
            "ATNPI should fire for a bad class Access value"
        );
    }

    #[test]
    fn test_atnpi_no_fire_valid_class_access_values() {
        let source = "\
classdef (Access = private) Foo
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "ATNPI").is_empty(),
            "ATNPI should NOT fire for 'Access = private'"
        );
    }

    #[test]
    fn test_atppi_fires_bad_property_access_value() {
        let source = "\
classdef Foo
    properties (Access = 42)
        x
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "ATPPI").is_empty(),
            "ATPPI should fire for a bad property Access value"
        );
    }

    #[test]
    fn test_atppi_no_fire_immutable_and_metaclass_values() {
        let source = "\
classdef Foo
    properties (Access = immutable, GetAccess = ?Foo, SetAccess = {?Bar, ?Baz})
        x
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "ATPPI").is_empty(),
            "ATPPI should NOT fire for immutable or meta-class access values"
        );
    }

    #[test]
    fn test_atppp_fires_bad_method_access_value() {
        let source = "\
classdef Foo
    methods (Access = immutable)
        function y = f(self)
            y = 1;
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "ATPPP").is_empty(),
            "ATPPP should fire for 'Access = immutable' on a methods block"
        );
    }

    #[test]
    fn test_atppp_no_fire_valid_method_access_value() {
        let source = "\
classdef Foo
    methods (Access = protected)
        function y = f(self)
            y = 1;
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "ATPPP").is_empty(),
            "ATPPP should NOT fire for 'Access = protected'"
        );
    }

    #[test]
    fn test_atnpp_fires_bad_event_access_value() {
        let source = "\
classdef Foo < handle
    events (ListenAccess = immutable)
        Done
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "ATNPP").is_empty(),
            "ATNPP should fire for 'ListenAccess = immutable'"
        );
    }

    #[test]
    fn test_atnpp_no_fire_valid_event_access_value() {
        let source = "\
classdef Foo < handle
    events (ListenAccess = private, NotifyAccess = ?Foo)
        Done
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "ATNPP").is_empty(),
            "ATNPP should NOT fire for valid event access values"
        );
    }

    // -- ATAS ----------------------------------------------------------------

    #[test]
    fn test_atas_fires_bad_metaclass_value_on_property_block() {
        let source = "\
classdef Foo
    properties (AllowedSubclasses = 5)
        x
    end
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "ATAS").is_empty(),
            "ATAS should fire for a bad meta-class value on a property block"
        );
    }

    // -- ATVIZE --------------------------------------------------------------

    #[test]
    fn test_atvize_fires_visible_attribute() {
        let source = "\
classdef (Visible) Foo
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "ATVIZE").is_empty(),
            "ATVIZE should fire for a 'Visible' attribute"
        );
    }

    #[test]
    fn test_atvize_no_fire_hidden_attribute() {
        let source = "\
classdef (~Hidden) Foo
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "ATVIZE").is_empty(),
            "ATVIZE should NOT fire for a '~Hidden' attribute"
        );
    }

    // -- Config respect ------------------------------------------------------

    #[test]
    fn test_at_checks_respect_disabled_config() {
        let mut rule_config = crate::language_spec::LanguageSpecConfig::default();
        rule_config.disabled_checks.push("ATUNK".to_string());
        let engine = crate::language_spec::LanguageSpecEngine {
            config: rule_config,
        };
        let source = "\
classdef (Frobnicate) Foo
end
";
        let tree = crate::language_spec::tests::parse_matlab(source);
        let path = std::path::Path::new("Foo.m");
        let ctx = malt_core::FileContext {
            tree: &tree,
            source,
            file_path: path,
        };
        let diags = engine.check_file(&ctx);
        assert!(
            filter_by_id(&diags, "ATUNK").is_empty(),
            "ATUNK should be disabled via config disabled_checks"
        );
    }
}
