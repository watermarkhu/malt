//! # AT* class, property, method, and event attribute checks
//!
//! These checks validate the attribute lists of `classdef`, `properties`,
//! `methods`, and `events` blocks, plus the `Input`/`Output` attributes of
//! `arguments` blocks:
//!
//! - **ATUNK** — the attribute name is not a valid MATLAB attribute.
//! - **ATTF** — a logical attribute (`Abstract`, `Sealed`, `Hidden`, `Dependent`, ...)
//!   has a value other than `true`, `false`, `1` or `0`, such as `TRUE` or `~true`.
//! - **ATTOF** — the class attribute `Abstract` is set to `false` or `0`.
//! - **ATLAB** — the `Input`/`Output` arguments-block attribute is assigned a
//!   value or negated.
//! - **ATAS** — a meta-class attribute (`AllowedSubclasses`,
//!   `InferiorClasses`) has a value that is neither a single meta-class object
//!   nor a cell array of meta-class objects.
//! - **ATNPI** — a class-level access attribute has an unexpected value.
//! - **ATPPP** — an events-block access attribute has an unexpected value
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
    "DynamicProps",
    "ExplicitProperties",
];

/// Valid properties-block attribute names (ATUNK).
const PROPERTY_ATTRS: &[&str] = &[
    "Abstract",
    "Constant",
    "Dependent",
    "WeakHandle",
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
    // matlab.unittest test classes.
    "TestParameter",
    "ClassSetupParameter",
    "MethodSetupParameter",
];

/// Valid methods-block attribute names (ATUNK).
const METHOD_ATTRS: &[&str] = &[
    "Abstract",
    "Static",
    "Access",
    "Hidden",
    "Sealed",
    "AbstractMethods",
    // matlab.unittest test classes.
    "Test",
    "TestMethodSetup",
    "TestMethodTeardown",
    "TestClassSetup",
    "TestClassTeardown",
    "TestTags",
    "ParameterCombination",
];

/// Valid events-block attribute names (ATUNK).
const EVENT_ATTRS: &[&str] = &["ListenAccess", "NotifyAccess", "Hidden"];

/// Attribute names that take a single meta-class object or a cell array of
/// meta-class objects as their value (ATNAS/ATAS).
const METACLASS_ATTRS: &[&str] = &["AllowedSubclasses", "InferiorClasses"];

/// Attributes whose value is a logical (ATTF). MATLAB evaluates only literal values.
const LOGICAL_ATTRS: &[&str] = &[
    "Abstract",
    "Sealed",
    "Hidden",
    "ConstructOnLoad",
    "HandleCompatible",
    "Dependent",
    "Constant",
    "Static",
    "Transient",
    "WeakHandle",
    "NonCopyable",
    "AbortSet",
    "SetObservable",
    "GetObservable",
];

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
        // matlab.unittest test classes take `TestParameterDefinition` on a methods
        // block (MCMTP checks it); other classes report it as unknown.
        let mut method_attrs: Vec<&str> = METHOD_ATTRS.to_vec();
        if class
            .superclasses
            .iter()
            .any(|s| s == "matlab.unittest.TestCase")
        {
            method_attrs.push("TestParameterDefinition");
        }

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
                    block.line,
                    attr,
                    PROPERTY_ATTRS,
                    AttrLevel::Property,
                    diagnostics,
                );
            }
        }
        // An enumeration block takes no attributes in MATLAB: each one is unknown.
        for block in &class.enumeration_blocks {
            for _attr in &block.attributes {
                if self.is_check_enabled("ATUNK") {
                    diagnostics.push(Diagnostic {
                        rule_id: "ATUNK",
                        message: "Unknown attribute name.".to_string(),
                        severity: Severity::Error,
                        byte_range: range.clone(),
                        line: block.line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }
        for block in &class.methods_blocks {
            for attr in &block.attributes {
                self.check_one_attribute(
                    &range,
                    block.line,
                    attr,
                    &method_attrs,
                    AttrLevel::Method,
                    diagnostics,
                );
            }
        }
        for block in &class.events_blocks {
            for attr in &block.attributes {
                self.check_one_attribute(
                    &range,
                    block.line,
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
                    let (rule_id, message) = (
                        "ATAS",
                        "The attribute value is unexpected. Use a single meta-class \
                         object or a cell array of meta-class objects.",
                    );
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

        // ATTF: a logical attribute must be `true`/`false` (or `1`/`0`) for MATLAB to
        // evaluate it. ATTOF: a class-level `Abstract` set to false or 0.
        if LOGICAL_ATTRS.contains(&attr.name.as_str()) {
            if let Some(value) = attr.value.as_deref() {
                if !matches!(value, "true" | "false" | "1" | "0") && self.is_check_enabled("ATTF") {
                    diagnostics.push(Diagnostic {
                        rule_id: "ATTF",
                        message: format!(
                            "The Code Analyzer is unable to determine if the expression \
                             assigned to the '{}' attribute evaluates to true or false.",
                            attr.name
                        ),
                        severity: Severity::Warning,
                        byte_range: range.clone(),
                        line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }
        if matches!(level, AttrLevel::Class)
            && attr.name == "Abstract"
            && matches!(attr.value.as_deref(), Some("false") | Some("0"))
            && self.is_check_enabled("ATTOF")
        {
            diagnostics.push(Diagnostic {
                rule_id: "ATTOF",
                message: "Setting the class attribute Abstract to false is not recommended."
                    .to_string(),
                severity: Severity::Warning,
                byte_range: range.clone(),
                line,
                column: 1,
                fix: None,
            });
        }

        // Access attribute value checks (ATNPI / ATPPP / ATPPI).
        if is_access_attr_name(&attr.name, level) {
            if let Some(value) = attr.value.as_deref() {
                let (allowed, rule_id, message) = match level {
                    AttrLevel::Class => (
                        &["public", "private", "protected", "immutable"][..],
                        "ATNPI",
                        "Set attribute to 'public', 'private', 'protected', 'immutable', \
                         or a cell array of meta-classes instead.",
                    ),
                    // Probed: `Access` and `GetAccess` values on a properties block are
                    // ATPPP (the methods message); only `SetAccess` is ATPPI.
                    AttrLevel::Property if attr.name != "SetAccess" => (
                        &["public", "private", "protected", "immutable"][..],
                        "ATPPP",
                        "The attribute value is unexpected. Use 'public', 'private', \
                         'protected', or a cell array of meta-classes instead.",
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
                        "ATPPP",
                        "The attribute value is unexpected. Use 'public', 'private', \
                         'protected', or a cell array of meta-classes instead.",
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
                    // `Input = value` has a value; `~Output` is negated.
                    let assigned = attribute_entries(attrs_node, source, AttrForms::Written)
                        .iter()
                        .any(|attr| {
                            (attr.value.is_some() || attr.negated)
                                && (attr.name == "Input" || attr.name == "Output")
                        });
                    if assigned && self.is_check_enabled("ATLAB") {
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
    let value = value.trim_matches(|c| c == '\'' || c == '"');
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
    if let Some(inner) = value.strip_prefix('{').and_then(|v| v.strip_suffix('}')) {
        return inner.split(',').all(|e| e.trim().starts_with('?'));
    }
    value.starts_with('?')
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
            !filter_by_id(&diags, "ATAS").is_empty(),
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
            filter_by_id(&diags, "ATAS").is_empty(),
            "ATNAS should NOT fire for valid meta-class values"
        );
    }

    // -- ATNPI / ATNPP / ATPPI / ATPPP ---------------------------------------

    #[test]
    fn test_atunk_fires_class_access() {
        let source = "\
classdef (Access = 42) Foo
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "ATUNK").is_empty(),
            "ATUNK should fire for Access on a class: MATLAB rejects it as unknown"
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
    properties (SetAccess = 42)
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
            !filter_by_id(&diags, "ATPPP").is_empty(),
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
            filter_by_id(&diags, "ATPPP").is_empty(),
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
            project: None,
        };
        let diags = engine.check_file(&ctx);
        assert!(
            filter_by_id(&diags, "ATUNK").is_empty(),
            "ATUNK should be disabled via config disabled_checks"
        );
    }

    #[test]
    fn test_atunk_no_fire_weak_handle_property_attribute() {
        let source = "\
classdef C
    properties (WeakHandle = true)
        p
    end
end
";
        let diags = check_source(source, "C.m");
        assert!(filter_by_id(&diags, "ATUNK").is_empty());
    }

    #[test]
    fn test_atppp_no_fire_quoted_access_value() {
        let source = "\
classdef C
    methods (Access = 'private')
        function f(obj)
        end
    end
end
";
        let diags = check_source(source, "C.m");
        assert!(filter_by_id(&diags, "ATPPP").is_empty());
    }

    #[test]
    fn test_atppi_atppp_report_bad_properties_values_at_block_line() {
        // Probed (B1, B2, B3, B6): a bad Access or GetAccess value on a properties block is
        // ATPPP and a bad SetAccess value is ATPPI; each is reported at the block's line.
        for (source, id, line) in [
            (
                "classdef B1\n    properties (SetAccess = bogus)\n        x = 1;\n    end\nend\n",
                "ATPPI",
                2,
            ),
            (
                "classdef B2\n    properties\n        a = 1;\n    end\n    properties (GetAccess = bogus)\n        x = 1;\n    end\nend\n",
                "ATPPP",
                5,
            ),
            (
                "classdef B3\n    properties (Access = bogus)\n        x = 1;\n    end\nend\n",
                "ATPPP",
                2,
            ),
            (
                "classdef B6\n    properties (Access = Private)\n        x = 1;\n    end\nend\n",
                "ATPPP",
                2,
            ),
        ] {
            let diags = check_source(source, "C.m");
            let hits = filter_by_id(&diags, id);
            assert_eq!(hits.len(), 1, "{source}: got: {diags:?}");
            assert_eq!(hits[0].line, line, "{source}: got: {diags:?}");
        }
    }

    #[test]
    fn test_atppi_atppp_silent_for_valid_or_quoted_property_values() {
        // Probed (Q1, Q2, Q3, Q4, B5): valid Access, GetAccess and SetAccess values, and a
        // quoted SetAccess value, are silent in MATLAB.
        for source in [
            "classdef Q1\n    properties (Access = private)\n        x = 1;\n    end\nend\n",
            "classdef Q2\n    properties (GetAccess = private)\n        x = 1;\n    end\nend\n",
            "classdef Q3\n    properties (SetAccess = private)\n        x = 1;\n    end\nend\n",
            "classdef Q4\n    properties (SetAccess = private, GetAccess = public)\n        x = 1;\n    end\nend\n",
            "classdef B5\n    properties (SetAccess = 'private')\n        x = 1;\n    end\nend\n",
        ] {
            let diags = check_source(source, "C.m");
            assert!(
                filter_by_id(&diags, "ATPPI").is_empty() && filter_by_id(&diags, "ATPPP").is_empty(),
                "{source}: got: {diags:?}"
            );
        }
    }

    #[test]
    fn test_attf_fires_uppercase_logical_value() {
        let source = "classdef (Abstract = TRUE) Foo < handle\nend\n";
        let diags = crate::language_spec::tests::check_source(source, "Foo.m");
        assert!(!crate::language_spec::tests::filter_by_id(&diags, "ATTF").is_empty());
    }

    #[test]
    fn test_attf_fires_non_literal_property_value() {
        let source =
            "classdef Foo < handle\n    properties (Dependent = TRUE)\n        x\n    end\nend\n";
        let diags = crate::language_spec::tests::check_source(source, "Foo.m");
        assert!(!crate::language_spec::tests::filter_by_id(&diags, "ATTF").is_empty());
    }

    #[test]
    fn test_attf_no_fire_literal_values() {
        let source = "classdef (Abstract = true, Sealed = 1) Foo < handle\n    properties (Dependent = false)\n        x\n    end\nend\n";
        let diags = crate::language_spec::tests::check_source(source, "Foo.m");
        assert!(crate::language_spec::tests::filter_by_id(&diags, "ATTF").is_empty());
    }

    #[test]
    fn test_attof_fires_class_abstract_false() {
        let source = "classdef (Abstract = false) Foo < handle\nend\n";
        let diags = crate::language_spec::tests::check_source(source, "Foo.m");
        assert!(!crate::language_spec::tests::filter_by_id(&diags, "ATTOF").is_empty());
    }

    #[test]
    fn test_attof_no_fire_property_abstract_false() {
        let source =
            "classdef Foo < handle\n    properties (Abstract = false)\n        x\n    end\nend\n";
        let diags = crate::language_spec::tests::check_source(source, "Foo.m");
        assert!(crate::language_spec::tests::filter_by_id(&diags, "ATTOF").is_empty());
    }

    #[test]
    fn test_atas_fires_cell_of_names_and_bare_name() {
        let cell = "classdef (AllowedSubclasses = {ValueBasic}) Foo\nend\n";
        let bare = "classdef (InferiorClasses = double) Foo\nend\n";
        for source in [cell, bare] {
            let diags = crate::language_spec::tests::check_source(source, "Foo.m");
            assert!(
                !crate::language_spec::tests::filter_by_id(&diags, "ATAS").is_empty(),
                "{source}"
            );
            assert!(crate::language_spec::tests::filter_by_id(&diags, "ATNAS").is_empty());
        }
    }

    #[test]
    fn test_atas_no_fire_meta_class_cell() {
        let source = "classdef (AllowedSubclasses = {?ValueBasic, ?HandleSub}) Foo\nend\n";
        let diags = crate::language_spec::tests::check_source(source, "Foo.m");
        assert!(crate::language_spec::tests::filter_by_id(&diags, "ATAS").is_empty());
    }

    #[test]
    fn test_atppp_fires_event_access_value() {
        let source = "classdef Foo < handle\n    events (NotifyAccess = Private)\n        Ch\n    end\nend\n";
        let diags = crate::language_spec::tests::check_source(source, "Foo.m");
        assert!(!crate::language_spec::tests::filter_by_id(&diags, "ATPPP").is_empty());
        assert!(crate::language_spec::tests::filter_by_id(&diags, "ATNPP").is_empty());
    }
}

#[cfg(test)]
mod declaration_value_tests {
    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn atlab_fires_for_a_valued_or_negated_input_output_attribute() {
        // MATLAB probes: ATLAB on `Input = 'x'`, `Output = 1` and `~Output`, and no SYNER.
        for attrs in [
            "Input = 'x'",
            "Output = 1",
            "~Output",
            "Input = [1 2], Repeating",
        ] {
            let source =
                format!("function r = f(x)\narguments ({attrs})\n    x double\nend\nr = x;\nend\n");
            let diags = check_source(&source, "f.m");
            assert!(
                !filter_by_id(&diags, "ATLAB").is_empty(),
                "{attrs}: got: {diags:?}"
            );
        }
    }

    #[test]
    fn atlab_silent_for_plain_and_repeating_attributes() {
        for attrs in ["Input", "Input, Repeating", "Repeating = true"] {
            let source =
                format!("function r = f(x)\narguments ({attrs})\n    x double\nend\nr = x;\nend\n");
            let diags = check_source(&source, "f.m");
            assert!(
                filter_by_id(&diags, "ATLAB").is_empty(),
                "{attrs}: got: {diags:?}"
            );
        }
    }

    #[test]
    fn enumeration_block_attributes_are_unknown() {
        // Probed: `enumeration (Foo)` and `enumeration (Sealed)` give ATUNK; a plain
        // `enumeration` block gives nothing.
        let diags = check_source(
            "classdef Foo\n    enumeration (Foo)\n        Red (1)\n    end\nend\n",
            "Foo.m",
        );
        let hits = filter_by_id(&diags, "ATUNK");
        assert_eq!(hits.len(), 1, "got: {diags:?}");
        assert_eq!(hits[0].line, 2);
        let diags = check_source(
            "classdef Foo\n    enumeration (Sealed)\n        Red (1)\n    end\nend\n",
            "Foo.m",
        );
        assert_eq!(filter_by_id(&diags, "ATUNK").len(), 1, "got: {diags:?}");
        let diags = check_source(
            "classdef Foo\n    enumeration\n        Red (1)\n    end\nend\n",
            "Foo.m",
        );
        assert!(filter_by_id(&diags, "ATUNK").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_parameter_definition_is_known_in_test_case_only() {
        // Probed: the block attribute is valid in a matlab.unittest.TestCase
        // (MCMTP checks the static form) and unknown elsewhere (ATUNK).
        let in_test = "classdef Foo < matlab.unittest.TestCase\n    methods (TestParameterDefinition, Static)\n        function p = pdef()\n            p = {1, 2};\n        end\n    end\nend\n";
        assert!(filter_by_id(&check_source(in_test, "Foo.m"), "ATUNK").is_empty());
        let plain = "classdef Foo\n    methods (TestParameterDefinition)\n        function p = pdef()\n            p = {1, 2};\n        end\n    end\nend\n";
        assert_eq!(
            filter_by_id(&check_source(plain, "Foo.m"), "ATUNK").len(),
            1
        );
    }
}
