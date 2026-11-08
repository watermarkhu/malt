//! # Function/Class Metadata Extraction
//!
//! Extracts structural metadata from a parsed MATLAB file, including:
//!
//! - **File type** detection (script, function file, class file)
//! - **Function metadata**: name, inputs, outputs, argument validation blocks,
//!   setter/getter/constructor detection, method attributes
//! - **Class metadata**: name, superclasses, attributes, properties blocks,
//!   methods blocks, events blocks, enumeration blocks
//! - **Property metadata**: name, dimensions, type constraints, validators,
//!   default values
//!
//! ## Usage
//!
//! ```ignore
//! use malt_rules::analysis::metadata::FileMeta;
//!
//! let meta = FileMeta::build(tree, source);
//! match meta.file_type {
//!     FileType::ClassFile => {
//!         let class = meta.class.as_ref().unwrap();
//!         println!("Class: {}", class.name);
//!         for prop in meta.all_properties() {
//!             println!("  property: {}", prop.name);
//!         }
//!     }
//!     FileType::FunctionFile => {
//!         if let Some(main) = meta.main_function() {
//!             println!("Main function: {}", main.name);
//!         }
//!     }
//!     FileType::Script => println!("Script file"),
//! }
//! ```

use std::ops::Range;
use tree_sitter::{Node, Tree};

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// The type of MATLAB file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Script file (no `function_definition` at top level, or function after statements).
    Script,
    /// Function file (starts with `function_definition`).
    FunctionFile,
    /// Class definition file (contains `class_definition`).
    ClassFile,
}

/// Metadata about a function definition.
#[derive(Debug, Clone)]
pub struct FunctionMeta {
    /// Function name.
    pub name: String,
    /// Input argument names.
    pub inputs: Vec<String>,
    /// Output argument names (includes `~` as placeholder).
    pub outputs: Vec<String>,
    /// Whether this is a constructor (name matches class name).
    pub is_constructor: bool,
    /// Whether this is a property setter (name starts with `set.`).
    pub is_setter: bool,
    /// Whether this is a property getter (name starts with `get.`).
    pub is_getter: bool,
    /// Whether this function has an `arguments` validation block.
    pub has_arguments_block: bool,
    /// The argument validation entries (if any).
    pub argument_validations: Vec<ArgValidation>,
    /// Byte range of the function node.
    pub byte_range: Range<usize>,
    /// 1-indexed start line.
    pub line: usize,
    /// 1-indexed end line.
    pub end_line: usize,
    /// Whether defined inside a methods block (is a method).
    pub is_method: bool,
    /// Whether this is an abstract method (`function_signature`, no body).
    pub is_abstract: bool,
    /// Attributes from the enclosing methods block (if any).
    pub method_attributes: Vec<AttributeMeta>,
}

/// An argument validation entry from an `arguments` block.
#[derive(Debug, Clone)]
pub struct ArgValidation {
    /// Argument name.
    pub name: String,
    /// Size constraint (e.g., `"(1,3)"`), if specified.
    pub dimensions: Option<String>,
    /// Class/type constraint (e.g., `"double"`), if specified.
    pub type_constraint: Option<String>,
    /// Validation functions (e.g., `["mustBePositive"]`).
    pub validators: Vec<String>,
    /// Default value expression text, if specified.
    pub default_value: Option<String>,
}

/// An attribute on a class, properties block, methods block, or events block.
#[derive(Debug, Clone)]
pub struct AttributeMeta {
    /// Attribute name (e.g., `"Access"`, `"Sealed"`, `"Abstract"`).
    pub name: String,
    /// Attribute value as source text (e.g., `"private"`, `"true"`), or `None` if bare/negated.
    pub value: Option<String>,
    /// Whether the attribute is negated (e.g., `~Hidden`).
    pub negated: bool,
}

/// Metadata about a class property.
#[derive(Debug, Clone)]
pub struct PropertyMeta {
    /// Property name.
    pub name: String,
    /// Size constraint, if specified.
    pub dimensions: Option<String>,
    /// Type constraint, if specified.
    pub type_constraint: Option<String>,
    /// Validation functions.
    pub validators: Vec<String>,
    /// Default value expression text.
    pub default_value: Option<String>,
    /// Whether this property has a default value.
    pub has_default: bool,
    /// Byte range.
    pub byte_range: Range<usize>,
    /// 1-indexed line.
    pub line: usize,
}

/// Metadata about a `properties ... end` block.
#[derive(Debug, Clone)]
pub struct PropertiesBlockMeta {
    /// Attributes on this properties block.
    pub attributes: Vec<AttributeMeta>,
    /// Properties defined in this block.
    pub properties: Vec<PropertyMeta>,
}

/// Metadata about a `methods ... end` block.
#[derive(Debug, Clone)]
pub struct MethodsBlockMeta {
    /// Attributes on this methods block.
    pub attributes: Vec<AttributeMeta>,
    /// Functions (methods) defined in this block.
    pub methods: Vec<FunctionMeta>,
}

/// Metadata about an `events ... end` block.
#[derive(Debug, Clone)]
pub struct EventsBlockMeta {
    /// Attributes on this events block.
    pub attributes: Vec<AttributeMeta>,
    /// Event names.
    pub events: Vec<String>,
}

/// Metadata about an `enumeration ... end` block.
#[derive(Debug, Clone)]
pub struct EnumerationBlockMeta {
    /// Enumeration member names.
    pub members: Vec<String>,
}

/// Metadata about a class definition.
#[derive(Debug, Clone)]
pub struct ClassMeta {
    /// Class name.
    pub name: String,
    /// Superclass names.
    pub superclasses: Vec<String>,
    /// Class-level attributes.
    pub attributes: Vec<AttributeMeta>,
    /// Properties blocks.
    pub properties_blocks: Vec<PropertiesBlockMeta>,
    /// Methods blocks.
    pub methods_blocks: Vec<MethodsBlockMeta>,
    /// Events blocks.
    pub events_blocks: Vec<EventsBlockMeta>,
    /// Enumeration blocks.
    pub enumeration_blocks: Vec<EnumerationBlockMeta>,
    /// Byte range of the `class_definition` node.
    pub byte_range: Range<usize>,
    /// 1-indexed start line.
    pub line: usize,
}

/// Complete file metadata extracted from a single parse tree.
#[derive(Debug, Clone)]
pub struct FileMeta {
    /// The type of this file (script, function, or class).
    pub file_type: FileType,
    /// The class definition, if this is a class file.
    pub class: Option<ClassMeta>,
    /// All function definitions (including methods if class file).
    /// For function files: first is the main function, rest are local functions.
    pub functions: Vec<FunctionMeta>,
    /// Local functions defined after the class definition (in class files)
    /// or after the main function (in function files).
    pub local_functions: Vec<FunctionMeta>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the source text covered by a node.
fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// Extract attributes from an `attributes` node.
///
/// Each `attribute` child represents a single attribute entry.
/// Attributes can be:
/// - Bare: `Sealed` (value is `None`, treated as `true`)
/// - Valued: `Access = private`
/// - Negated: `~Hidden` (negated flag set)
fn extract_attributes(attrs_node: Node, source: &str) -> Vec<AttributeMeta> {
    let mut attrs = Vec::new();
    let mut cursor = attrs_node.walk();

    for child in attrs_node.children(&mut cursor) {
        if child.kind() != "attribute" {
            continue;
        }

        let text = node_text(child, source).trim();
        let negated = text.starts_with('~');

        // Find the attribute name — first identifier child.
        let name = find_first_child_of_kind(child, "identifier")
            .map(|n| node_text(n, source).to_string())
            .unwrap_or_default();

        if name.is_empty() {
            continue;
        }

        // Look for `= value` portion.  The value is everything after the `=`.
        // In the grammar an attribute with a value has the identifier, then `=`, then
        // one or more value children (which may be identifiers, strings, etc.).
        let value = extract_attribute_value(child, source);

        attrs.push(AttributeMeta {
            name,
            value,
            negated,
        });
    }

    attrs
}

/// Extract the value portion of an `attribute` node (everything after `=`).
///
/// Returns `None` if no `=` is present (bare attribute).
fn extract_attribute_value(attr_node: Node, source: &str) -> Option<String> {
    let mut found_eq = false;
    let mut cursor = attr_node.walk();

    for child in attr_node.children(&mut cursor) {
        let kind = child.kind();
        let text = node_text(child, source);

        if text == "=" {
            found_eq = true;
            continue;
        }

        if found_eq && kind != "=" {
            // Capture the first meaningful token after `=` as the value.
            return Some(text.trim().to_string());
        }
    }

    None
}

/// Find the first child of a node with the given kind.
fn find_first_child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let child_count = node.child_count();
    for i in 0..child_count {
        if let Some(child) = node.child(i) {
            if child.kind() == kind {
                return Some(child);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Function extraction
// ---------------------------------------------------------------------------

/// Build the full function name, including `set.` or `get.` prefix if present.
///
/// In tree-sitter-matlab, `function set.Value(...)` produces:
/// - a child node with kind `set.`
/// - the `name` field pointing to the `identifier` node `Value`
///
/// This function checks for that prefix and prepends it to the base name.
fn build_function_name(func_node: Node, source: &str) -> Option<String> {
    let name_node = func_node.child_by_field_name("name")?;
    let base_name = node_text(name_node, source);

    // Check for `set.` or `get.` prefix child.
    let child_count = func_node.child_count();
    for i in 0..child_count {
        if let Some(child) = func_node.child(i) {
            let kind = child.kind();
            if kind == "set." || kind == "get." {
                return Some(format!("{}{}", kind, base_name));
            }
        }
    }

    Some(base_name.to_string())
}

/// Extract metadata from a `function_definition` node.
fn extract_function_meta(
    node: Node,
    source: &str,
    class_name: Option<&str>,
    is_method: bool,
    method_attributes: &[AttributeMeta],
    is_abstract: bool,
) -> Option<FunctionMeta> {
    let name = build_function_name(node, source)?;

    let inputs = extract_function_inputs(node, source);
    let outputs = extract_function_outputs(node, source);

    let is_constructor = class_name.is_some_and(|cn| cn == name);
    let is_setter = name.starts_with("set.");
    let is_getter = name.starts_with("get.");

    let (has_arguments_block, argument_validations) = extract_arguments_blocks(node, source);

    Some(FunctionMeta {
        name,
        inputs,
        outputs,
        is_constructor,
        is_setter,
        is_getter,
        has_arguments_block,
        argument_validations,
        byte_range: node.start_byte()..node.end_byte(),
        line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        is_method,
        is_abstract,
        method_attributes: method_attributes.to_vec(),
    })
}

/// Extract a `function_signature` (abstract method stub with no body).
fn extract_function_signature(
    node: Node,
    source: &str,
    class_name: Option<&str>,
    method_attributes: &[AttributeMeta],
) -> Option<FunctionMeta> {
    let name = build_function_name(node, source)?;

    let inputs = extract_function_inputs(node, source);
    let outputs = extract_function_outputs(node, source);

    let is_constructor = class_name.is_some_and(|cn| cn == name);
    let is_setter = name.starts_with("set.");
    let is_getter = name.starts_with("get.");

    Some(FunctionMeta {
        name,
        inputs,
        outputs,
        is_constructor,
        is_setter,
        is_getter,
        has_arguments_block: false,
        argument_validations: Vec::new(),
        byte_range: node.start_byte()..node.end_byte(),
        line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        is_method: true,
        is_abstract: true,
        method_attributes: method_attributes.to_vec(),
    })
}

/// Extract input argument names from a function definition.
fn extract_function_inputs(func_node: Node, source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut cursor = func_node.walk();

    for child in func_node.children(&mut cursor) {
        if child.kind() == "function_arguments" {
            let mut inner = child.walk();
            for param in child.children(&mut inner) {
                match param.kind() {
                    "identifier" => {
                        names.push(node_text(param, source).to_string());
                    }
                    "ignored_argument" => {
                        names.push("~".to_string());
                    }
                    _ => {}
                }
            }
            break;
        }
    }

    names
}

/// Extract output argument names from a function definition.
fn extract_function_outputs(func_node: Node, source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut cursor = func_node.walk();

    for child in func_node.children(&mut cursor) {
        if child.kind() == "function_output" {
            let mut inner = child.walk();
            for out_child in child.children(&mut inner) {
                match out_child.kind() {
                    "identifier" => {
                        names.push(node_text(out_child, source).to_string());
                    }
                    "ignored_argument" => {
                        names.push("~".to_string());
                    }
                    "multioutput_variable" => {
                        let mut mv_cursor = out_child.walk();
                        for mv_child in out_child.children(&mut mv_cursor) {
                            match mv_child.kind() {
                                "identifier" => {
                                    names.push(node_text(mv_child, source).to_string());
                                }
                                "ignored_argument" => {
                                    names.push("~".to_string());
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            break;
        }
    }

    names
}

/// Extract `arguments` validation blocks from within a function definition.
///
/// Returns `(has_arguments_block, validations)`.
fn extract_arguments_blocks(func_node: Node, source: &str) -> (bool, Vec<ArgValidation>) {
    let mut validations = Vec::new();
    let mut found = false;

    let mut cursor = func_node.walk();
    for child in func_node.children(&mut cursor) {
        if child.kind() == "arguments_statement" {
            found = true;
            // Each `property` child inside the arguments block describes one argument.
            let mut inner = child.walk();
            for arg_child in child.children(&mut inner) {
                if arg_child.kind() == "property" {
                    if let Some(validation) = extract_arg_validation(arg_child, source) {
                        validations.push(validation);
                    }
                }
            }
        }
    }

    (found, validations)
}

/// Extract a single argument validation entry from a `property` node inside
/// an `arguments` block.
fn extract_arg_validation(prop_node: Node, source: &str) -> Option<ArgValidation> {
    let name = prop_node
        .child_by_field_name("name")
        .or_else(|| find_first_child_of_kind(prop_node, "identifier"))
        .map(|n| node_text(n, source).to_string())?;

    let dimensions =
        find_first_child_of_kind(prop_node, "dimensions").map(|n| node_text(n, source).to_string());

    // Type constraint: bare identifier after dimensions (not the name).
    let type_constraint = extract_type_constraint(prop_node, source, &name);

    let validators = find_first_child_of_kind(prop_node, "validation_functions")
        .map(|vf| {
            let mut v = Vec::new();
            let mut vc = vf.walk();
            for vchild in vf.children(&mut vc) {
                if vchild.kind() == "identifier" {
                    v.push(node_text(vchild, source).to_string());
                }
            }
            v
        })
        .unwrap_or_default();

    let default_value = find_first_child_of_kind(prop_node, "default_value").map(|dv| {
        // The actual expression is the child(ren) after `=`.
        // Use the full text, trimming the leading `=` if present.
        let text = node_text(dv, source).trim().to_string();
        let text = text.strip_prefix('=').unwrap_or(&text).trim().to_string();
        text
    });

    Some(ArgValidation {
        name,
        dimensions,
        type_constraint,
        validators,
        default_value,
    })
}

/// Extract the type constraint from a property node.
///
/// In MATLAB, the property syntax is: `name (dims) type {validators} = default`.
/// The type is a bare identifier (or `property_name` for dotted names)
/// that appears after dimensions but before validation_functions/default_value.
fn extract_type_constraint(prop_node: Node, source: &str, prop_name: &str) -> Option<String> {
    let mut cursor = prop_node.walk();
    let mut past_name = false;
    let mut past_dims = false;

    for child in prop_node.children(&mut cursor) {
        let kind = child.kind();

        // Skip the name identifier.
        if !past_name {
            if kind == "identifier" || kind == "property_name" {
                let text = node_text(child, source);
                if text == prop_name {
                    past_name = true;
                    continue;
                }
            }
            continue;
        }

        // Skip dimensions.
        if kind == "dimensions" {
            past_dims = true;
            continue;
        }

        // If we haven't seen dimensions yet and we encounter an identifier,
        // it could still be the type (dimensions are optional).
        if !past_dims && kind == "dimensions" {
            continue;
        }

        // The type constraint is the first identifier/property_name after
        // the property name (and optional dimensions).
        if kind == "identifier" || kind == "property_name" {
            return Some(node_text(child, source).to_string());
        }

        // If we hit validation_functions or default_value, no type was specified.
        if kind == "validation_functions" || kind == "default_value" || kind == "=" {
            return None;
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Property extraction
// ---------------------------------------------------------------------------

/// Extract property metadata from a `property` node inside a `properties` block.
fn extract_property_meta(prop_node: Node, source: &str) -> Option<PropertyMeta> {
    let name = prop_node
        .child_by_field_name("name")
        .or_else(|| find_first_child_of_kind(prop_node, "identifier"))
        .map(|n| node_text(n, source).to_string())?;

    let dimensions =
        find_first_child_of_kind(prop_node, "dimensions").map(|n| node_text(n, source).to_string());

    let type_constraint = extract_type_constraint(prop_node, source, &name);

    let validators = find_first_child_of_kind(prop_node, "validation_functions")
        .map(|vf| {
            let mut v = Vec::new();
            let mut vc = vf.walk();
            for vchild in vf.children(&mut vc) {
                if vchild.kind() == "identifier" {
                    v.push(node_text(vchild, source).to_string());
                }
            }
            v
        })
        .unwrap_or_default();

    let default_value_node = find_first_child_of_kind(prop_node, "default_value");
    let has_default = default_value_node.is_some();
    let default_value = default_value_node.map(|dv| {
        let text = node_text(dv, source).trim().to_string();
        let text = text.strip_prefix('=').unwrap_or(&text).trim().to_string();
        text
    });

    Some(PropertyMeta {
        name,
        dimensions,
        type_constraint,
        validators,
        default_value,
        has_default,
        byte_range: prop_node.start_byte()..prop_node.end_byte(),
        line: prop_node.start_position().row + 1,
    })
}

// ---------------------------------------------------------------------------
// Class extraction
// ---------------------------------------------------------------------------

/// Extract class metadata from a `class_definition` node.
fn extract_class_meta(class_node: Node, source: &str) -> Option<ClassMeta> {
    let name_node = class_node.child_by_field_name("name")?;
    let class_name = node_text(name_node, source).to_string();

    // Superclasses.
    let superclasses = find_first_child_of_kind(class_node, "superclasses")
        .map(|sc| {
            let mut names = Vec::new();
            let mut cursor = sc.walk();
            for child in sc.children(&mut cursor) {
                match child.kind() {
                    "identifier" | "property_name" => {
                        names.push(node_text(child, source).to_string());
                    }
                    _ => {}
                }
            }
            names
        })
        .unwrap_or_default();

    // Class-level attributes.
    let attributes = find_first_child_of_kind(class_node, "attributes")
        .map(|a| extract_attributes(a, source))
        .unwrap_or_default();

    // Collect section blocks.
    let mut properties_blocks = Vec::new();
    let mut methods_blocks = Vec::new();
    let mut events_blocks = Vec::new();
    let mut enumeration_blocks = Vec::new();

    let mut cursor = class_node.walk();
    for child in class_node.children(&mut cursor) {
        match child.kind() {
            "properties" => {
                properties_blocks.push(extract_properties_block(child, source));
            }
            "methods" => {
                methods_blocks.push(extract_methods_block(child, source, &class_name));
            }
            "events" => {
                events_blocks.push(extract_events_block(child, source));
            }
            "enumeration" => {
                enumeration_blocks.push(extract_enumeration_block(child, source));
            }
            _ => {}
        }
    }

    Some(ClassMeta {
        name: class_name,
        superclasses,
        attributes,
        properties_blocks,
        methods_blocks,
        events_blocks,
        enumeration_blocks,
        byte_range: class_node.start_byte()..class_node.end_byte(),
        line: class_node.start_position().row + 1,
    })
}

/// Extract a `properties ... end` block.
fn extract_properties_block(props_node: Node, source: &str) -> PropertiesBlockMeta {
    let attributes = find_first_child_of_kind(props_node, "attributes")
        .map(|a| extract_attributes(a, source))
        .unwrap_or_default();

    let mut properties = Vec::new();
    let mut cursor = props_node.walk();
    for child in props_node.children(&mut cursor) {
        if child.kind() == "property" {
            if let Some(pm) = extract_property_meta(child, source) {
                properties.push(pm);
            }
        }
    }

    PropertiesBlockMeta {
        attributes,
        properties,
    }
}

/// Extract a `methods ... end` block.
fn extract_methods_block(methods_node: Node, source: &str, class_name: &str) -> MethodsBlockMeta {
    let block_attributes = find_first_child_of_kind(methods_node, "attributes")
        .map(|a| extract_attributes(a, source))
        .unwrap_or_default();

    let mut methods = Vec::new();
    let mut cursor = methods_node.walk();
    for child in methods_node.children(&mut cursor) {
        match child.kind() {
            "function_definition" => {
                if let Some(fm) = extract_function_meta(
                    child,
                    source,
                    Some(class_name),
                    true,
                    &block_attributes,
                    false,
                ) {
                    methods.push(fm);
                }
            }
            "function_signature" => {
                if let Some(fm) =
                    extract_function_signature(child, source, Some(class_name), &block_attributes)
                {
                    methods.push(fm);
                }
            }
            _ => {}
        }
    }

    MethodsBlockMeta {
        attributes: block_attributes,
        methods,
    }
}

/// Extract an `events ... end` block.
fn extract_events_block(events_node: Node, source: &str) -> EventsBlockMeta {
    let attributes = find_first_child_of_kind(events_node, "attributes")
        .map(|a| extract_attributes(a, source))
        .unwrap_or_default();

    let mut events = Vec::new();
    let mut cursor = events_node.walk();
    for child in events_node.children(&mut cursor) {
        if child.kind() == "identifier" {
            events.push(node_text(child, source).to_string());
        }
    }

    EventsBlockMeta { attributes, events }
}

/// Extract an `enumeration ... end` block.
fn extract_enumeration_block(enum_node: Node, source: &str) -> EnumerationBlockMeta {
    let mut members = Vec::new();
    let mut cursor = enum_node.walk();
    for child in enum_node.children(&mut cursor) {
        if child.kind() == "enum" {
            // The enum member name is the first identifier child or "name" field.
            let name = child
                .child_by_field_name("name")
                .or_else(|| find_first_child_of_kind(child, "identifier"))
                .map(|n| node_text(n, source).to_string());
            if let Some(n) = name {
                members.push(n);
            }
        }
    }

    EnumerationBlockMeta { members }
}

// ---------------------------------------------------------------------------
// FileMeta construction
// ---------------------------------------------------------------------------

impl FileMeta {
    /// Extract metadata from a parsed tree and source text.
    ///
    /// Determines file type, extracts class/function/property metadata in a
    /// single walk of the top-level children.
    pub fn build(tree: &Tree, source: &str) -> Self {
        let root = tree.root_node();

        // ---- Determine file type -----------------------------------------
        let mut class_node: Option<Node> = None;
        let mut first_non_comment: Option<&str> = None;

        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            let kind = child.kind();

            // Skip comments, line continuations, and whitespace nodes.
            if kind == "comment" || kind == "line_continuation" {
                continue;
            }

            if kind == "class_definition" {
                class_node = Some(child);
                break; // class_definition found → ClassFile
            }

            if first_non_comment.is_none() {
                first_non_comment = Some(kind);
            }
        }

        let file_type = if class_node.is_some() {
            FileType::ClassFile
        } else if first_non_comment == Some("function_definition") {
            FileType::FunctionFile
        } else {
            FileType::Script
        };

        // ---- Extract class metadata (if class file) ----------------------
        let class_meta = class_node.and_then(|cn| extract_class_meta(cn, source));
        let class_name = class_meta.as_ref().map(|c| c.name.as_str());

        // ---- Collect all functions from methods blocks (class file) ------
        let mut all_methods: Vec<FunctionMeta> = Vec::new();
        if let Some(ref cm) = class_meta {
            for mb in &cm.methods_blocks {
                all_methods.extend(mb.methods.iter().cloned());
            }
        }

        // ---- Collect top-level functions ---------------------------------
        let mut functions: Vec<FunctionMeta> = Vec::new();
        let mut local_functions: Vec<FunctionMeta> = Vec::new();
        let mut main_function_seen = false;

        let mut cursor2 = root.walk();
        for child in root.children(&mut cursor2) {
            if child.kind() != "function_definition" {
                continue;
            }

            // In a class file, top-level function_definitions after the class
            // are local/helper functions.
            if file_type == FileType::ClassFile {
                if let Some(fm) =
                    extract_function_meta(child, source, class_name, false, &[], false)
                {
                    local_functions.push(fm);
                }
                continue;
            }

            // Function file: first is main, rest are local.
            if let Some(fm) = extract_function_meta(child, source, class_name, false, &[], false) {
                if !main_function_seen {
                    main_function_seen = true;
                    functions.push(fm);
                } else {
                    local_functions.push(fm);
                }
            }
        }

        // Combine class methods + top-level functions.
        let mut all_functions = all_methods;
        all_functions.extend(functions);

        FileMeta {
            file_type,
            class: class_meta,
            functions: all_functions,
            local_functions,
        }
    }

    // ---- Helper methods --------------------------------------------------

    /// Get the main function (first function in a function file).
    ///
    /// Returns `None` for class files and scripts.
    pub fn main_function(&self) -> Option<&FunctionMeta> {
        if self.file_type == FileType::FunctionFile {
            self.functions.first()
        } else {
            None
        }
    }

    /// Get all methods (from all methods blocks in a class).
    ///
    /// Returns an empty vec for non-class files.
    pub fn all_methods(&self) -> Vec<&FunctionMeta> {
        self.functions.iter().filter(|f| f.is_method).collect()
    }

    /// Get all properties (from all properties blocks in a class).
    ///
    /// Returns an empty vec for non-class files.
    pub fn all_properties(&self) -> Vec<&PropertyMeta> {
        self.class
            .as_ref()
            .map(|c| {
                c.properties_blocks
                    .iter()
                    .flat_map(|pb| pb.properties.iter())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a method name exists in any methods block.
    pub fn has_method(&self, name: &str) -> bool {
        self.functions.iter().any(|f| f.is_method && f.name == name)
    }

    /// Check if a property name exists in any properties block.
    pub fn has_property(&self, name: &str) -> bool {
        self.class.as_ref().is_some_and(|c| {
            c.properties_blocks
                .iter()
                .flat_map(|pb| pb.properties.iter())
                .any(|p| p.name == name)
        })
    }

    /// Get the class name (if class file).
    pub fn class_name(&self) -> Option<&str> {
        self.class.as_ref().map(|c| c.name.as_str())
    }
}

// ---------------------------------------------------------------------------
// ClassMeta helper methods
// ---------------------------------------------------------------------------

impl ClassMeta {
    /// Check if the class has a specific attribute (by name, case-sensitive).
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|a| a.name == name)
    }

    /// Get the value of a specific class-level attribute.
    ///
    /// Returns `None` if the attribute is not present or has no value.
    pub fn attribute_value(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|a| a.name == name)
            .and_then(|a| a.value.as_deref())
    }

    /// Check if the class is `Sealed`.
    ///
    /// Returns `true` if the `Sealed` attribute is present and not negated.
    pub fn is_sealed(&self) -> bool {
        self.attributes
            .iter()
            .any(|a| a.name == "Sealed" && !a.negated)
    }

    /// Check if the class is `Abstract`.
    ///
    /// Returns `true` if the `Abstract` attribute is present and not negated.
    pub fn is_abstract(&self) -> bool {
        self.attributes
            .iter()
            .any(|a| a.name == "Abstract" && !a.negated)
    }

    /// Check if the class inherits from `handle`.
    pub fn is_handle(&self) -> bool {
        self.superclasses.iter().any(|s| s == "handle")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    /// Parse MATLAB source and return the tree.
    fn parse_matlab(source: &str) -> Tree {
        let mut parser = Parser::new();
        let language = tree_sitter_matlab::LANGUAGE;
        parser
            .set_language(&language.into())
            .expect("failed to set MATLAB language");
        parser.parse(source, None).expect("failed to parse")
    }

    // ---- File type detection ---------------------------------------------

    #[test]
    fn test_script_file() {
        let source = "x = 1;\ny = x + 2;\ndisp(y);\n";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);

        assert_eq!(meta.file_type, FileType::Script);
        assert!(meta.class.is_none());
        assert!(meta.functions.is_empty());
    }

    #[test]
    fn test_function_file() {
        let source = "\
function y = add(a, b)
    y = a + b;
end
";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);

        assert_eq!(meta.file_type, FileType::FunctionFile);
        assert!(meta.class.is_none());
        assert_eq!(meta.functions.len(), 1);

        let f = &meta.functions[0];
        assert_eq!(f.name, "add");
        assert_eq!(f.inputs, vec!["a", "b"]);
        assert_eq!(f.outputs, vec!["y"]);
        assert!(!f.is_method);
        assert!(!f.is_constructor);
    }

    #[test]
    fn test_function_file_with_locals() {
        let source = "\
function y = main(x)
    y = helper(x);
end

function z = helper(x)
    z = x * 2;
end
";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);

        assert_eq!(meta.file_type, FileType::FunctionFile);
        assert_eq!(meta.functions.len(), 1);
        assert_eq!(meta.functions[0].name, "main");
        assert_eq!(meta.local_functions.len(), 1);
        assert_eq!(meta.local_functions[0].name, "helper");
    }

    #[test]
    fn test_main_function_helper() {
        let source = "\
function y = foo(x)
    y = x;
end
";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);
        let main = meta.main_function().unwrap();
        assert_eq!(main.name, "foo");
    }

    // ---- Class file detection --------------------------------------------

    #[test]
    fn test_class_file() {
        let source = "\
classdef MyClass
    properties
        Value
    end

    methods
        function obj = MyClass(val)
            obj.Value = val;
        end

        function disp(obj)
            fprintf('Value: %d\\n', obj.Value);
        end
    end
end
";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);

        assert_eq!(meta.file_type, FileType::ClassFile);
        assert!(meta.class.is_some());

        let class = meta.class.as_ref().unwrap();
        assert_eq!(class.name, "MyClass");
        assert_eq!(class.properties_blocks.len(), 1);
        assert_eq!(class.methods_blocks.len(), 1);

        // Properties
        let props = meta.all_properties();
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].name, "Value");

        // Methods
        let methods = meta.all_methods();
        assert_eq!(methods.len(), 2);

        // Constructor detection
        let constructor = methods.iter().find(|m| m.name == "MyClass").unwrap();
        assert!(constructor.is_constructor);
        assert!(constructor.is_method);
    }

    // ---- Superclasses ----------------------------------------------------

    #[test]
    fn test_superclasses() {
        let source = "\
classdef MyClass < handle & matlab.mixin.Copyable
    methods
    end
end
";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);
        let class = meta.class.as_ref().unwrap();

        assert!(!class.superclasses.is_empty());
        assert!(class.is_handle());
    }

    // ---- Setter/Getter detection -----------------------------------------

    #[test]
    fn test_setter_getter() {
        let source = "\
classdef MyClass
    properties
        Value
    end

    methods
        function set.Value(obj, val)
            obj.Value = val;
        end

        function val = get.Value(obj)
            val = obj.Value;
        end
    end
end
";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);

        let methods = meta.all_methods();
        let setter = methods.iter().find(|m| m.name == "set.Value").unwrap();
        assert!(setter.is_setter);
        assert!(!setter.is_getter);

        let getter = methods.iter().find(|m| m.name == "get.Value").unwrap();
        assert!(getter.is_getter);
        assert!(!getter.is_setter);
    }

    // ---- Multi-output function -------------------------------------------

    #[test]
    fn test_multi_output() {
        let source = "\
function [a, b, ~] = split(x)
    a = x(1);
    b = x(2);
end
";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);

        let f = &meta.functions[0];
        assert_eq!(f.name, "split");
        assert_eq!(f.outputs, vec!["a", "b", "~"]);
    }

    // ---- has_method / has_property helpers --------------------------------

    #[test]
    fn test_has_method_and_property() {
        let source = "\
classdef Foo
    properties
        Bar
    end

    methods
        function obj = Foo()
        end

        function doWork(obj)
        end
    end
end
";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);

        assert!(meta.has_method("doWork"));
        assert!(meta.has_method("Foo"));
        assert!(!meta.has_method("nonexistent"));

        assert!(meta.has_property("Bar"));
        assert!(!meta.has_property("Baz"));
    }

    // ---- ClassMeta helpers -----------------------------------------------

    #[test]
    fn test_class_sealed_abstract() {
        let source = "\
classdef (Sealed) FinalClass
    methods
    end
end
";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);
        let class = meta.class.as_ref().unwrap();

        assert!(class.is_sealed());
        assert!(!class.is_abstract());
    }

    // ---- Events and enumerations -----------------------------------------

    #[test]
    fn test_events_and_enums() {
        let source = "\
classdef Color
    enumeration
        Red
        Green
        Blue
    end

    events
        ColorChanged
    end
end
";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);
        let class = meta.class.as_ref().unwrap();

        assert_eq!(class.enumeration_blocks.len(), 1);
        let members = &class.enumeration_blocks[0].members;
        assert_eq!(members.len(), 3);
        assert!(members.contains(&"Red".to_string()));
        assert!(members.contains(&"Green".to_string()));
        assert!(members.contains(&"Blue".to_string()));

        assert_eq!(class.events_blocks.len(), 1);
        assert!(class.events_blocks[0]
            .events
            .contains(&"ColorChanged".to_string()));
    }

    // ---- class_name helper -----------------------------------------------

    #[test]
    fn test_class_name_helper() {
        let source = "\
classdef Widget
end
";
        let tree = parse_matlab(source);
        let meta = FileMeta::build(&tree, source);
        assert_eq!(meta.class_name(), Some("Widget"));

        let script = "x = 1;\n";
        let tree2 = parse_matlab(script);
        let meta2 = FileMeta::build(&tree2, script);
        assert_eq!(meta2.class_name(), None);
    }
}
