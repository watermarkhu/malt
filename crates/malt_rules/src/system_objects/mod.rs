//! # SYSTEM_OBJECTS_ENGINE: System Object Validation Checks
//!
//! ```malt
//! id = "SYSTEM_OBJECTS_ENGINE"
//! title = "System Object Validation Checks"
//! category = "system-objects"
//! severity = "warning"
//! fix = false
//! icon = "lucide/boxes"
//! slug = "system-objects"
//! ```
//!
//! ## Rule
//!
//! Validates MATLAB System object usage. System objects (classes inheriting
//! from `matlab.System`) have specific lifecycle constraints that static
//! analysis can partially verify. Since full type information is not available
//! to a static linter, these checks use pattern-based heuristics — detecting
//! System object patterns by class inheritance, lifecycle method calls
//! (`step`, `setup`, `release`, `reset`), and property access conventions.
//! All 9 checks share a single `SystemObjectsEngine` that dispatches node-level
//! checks on `function_call` nodes plus file-level class analysis; each
//! diagnostic carries the specific check ID (e.g. `SONUMIN`, `SOTUNPROP3`).
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | SONUMIN | error | no | If 'stepImpl' accepts variable number of inputs, then you must define a 'getNumInputsImpl' method. |
//! | SONUMOUT | error | no | If 'stepImpl' returns variable number of outputs, then you must define a 'getNumOutputsImpl' method. |
//! | SODEPPROP | warning | no | Dependent properties are not supported for MATLAB System blocks. VAR_NAME property is not included on System block. |
//! | SOINITPROP | warning | no | Initialize DiscreteState property VAR_NAME within a 'resetImpl' method. |
//! | SODFLTVAL | error | no | Invalid initialization of DiscreteState property VAR_NAME. Initialize property within a 'resetImpl' method. |
//! | SORSRVDNM | warning | no | VAR_NAME property is a reserved name. |
//! | SOTUNPROP1 | warning | no | Logical attribute not supported for tunable properties on MATLAB System blocks. VAR_NAME property is made Nontunable on System block. |
//! | SOTUNPROP3 | warning | no | Tunable properties on MATLAB System blocks must be numeric. VAR_NAME property is made Nontunable on System block because it is a char. |
//! | SOTUNPROP4 | warning | no | Tunable properties on MATLAB System blocks must be numeric. VAR_NAME property is made Nontunable on System block because it is a string. |
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! classdef MySystem < matlab.System
//!     properties
//!         Gain = rand();      % SODFLTVAL: function-call default value
//!         Flag logical = false % SOTUNPROP1: logical tunable property
//!     end
//!     methods
//!         function step(obj)   % SORSRVDNM: reserved method name
//!         end
//!     end
//! end
//! step(); % SONUMIN: step() called without inputs
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! classdef MySystem < matlab.System
//!     properties
//!         Gain = 1;
//!         Flag = false;
//!     end
//!     methods
//!         function stepImpl(obj) % use the *Impl override, not 'step'
//!         end
//!     end
//! end
//! step(obj, input); % pass the object and the input signal
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.SYSTEM_OBJECTS_ENGINE]
//! severity = "warning"
//! skip_checks = ["SONUMIN"]
//! ```

mod check_sodeprop;
mod check_sodfltval;
mod check_soinitprop;
mod check_sonumin;
mod check_sonumout;
mod check_sorsrvdnm;
mod check_sotunprop;

use malt_core::{Category, Config, Diagnostic, FileContext, NodeContext, Rule, Severity};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for the system objects engine.
///
/// Deserialized from the `[lint.rules.SYSTEM_OBJECTS_ENGINE]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SystemObjectsEngineConfig {
    /// Check IDs to skip (e.g., `["SONUMIN"]`).
    #[serde(default)]
    pub skip_checks: Vec<String>,
}

// ---------------------------------------------------------------------------
// Check metadata
// ---------------------------------------------------------------------------

/// Static metadata for a single system object check.
pub(crate) struct CheckMeta {
    id: &'static str,
    severity: Severity,
    description: &'static str,
}

/// All 9 system object check definitions.
pub(crate) const CHECKS: &[CheckMeta] = &[
    CheckMeta { id: "SONUMIN", severity: Severity::Error, description: "If 'stepImpl' accepts variable number of inputs, then you must define a 'getNumInputsImpl' method." },
    CheckMeta { id: "SONUMOUT", severity: Severity::Error, description: "If 'stepImpl' returns variable number of outputs, then you must define a 'getNumOutputsImpl' method." },
    CheckMeta { id: "SODEPPROP", severity: Severity::Warning, description: "Dependent properties are not supported for MATLAB System blocks. VAR_NAME property is not included on System block." },
    CheckMeta { id: "SOINITPROP", severity: Severity::Warning, description: "Initialize DiscreteState property VAR_NAME within a 'resetImpl' method." },
    CheckMeta { id: "SODFLTVAL", severity: Severity::Error, description: "Invalid initialization of DiscreteState property VAR_NAME. Initialize property within a 'resetImpl' method." },
    CheckMeta { id: "SORSRVDNM", severity: Severity::Warning, description: "VAR_NAME property is a reserved name." },
    CheckMeta { id: "SOINITPROP", severity: Severity::Warning, description: "Initialize DiscreteState property VAR_NAME within a 'resetImpl' method." },
    CheckMeta { id: "SOTUNPROP1", severity: Severity::Warning, description: "Logical attribute not supported for tunable properties on MATLAB System blocks. VAR_NAME property is made Nontunable on System block." },
    CheckMeta { id: "SOTUNPROP3", severity: Severity::Warning, description: "Tunable properties on MATLAB System blocks must be numeric. VAR_NAME property is made Nontunable on System block because it is a char." },
    CheckMeta { id: "SOTUNPROP4", severity: Severity::Warning, description: "Tunable properties on MATLAB System blocks must be numeric. VAR_NAME property is made Nontunable on System block because it is a string." },
];

/// System object lifecycle methods.
pub(crate) const SYSTEM_OBJECT_METHODS: &[&str] = &[
    "step",
    "setup",
    "release",
    "reset",
    "isDone",
    "isLocked",
    "getNumInputs",
    "getNumOutputs",
    "clone",
];

/// Reserved System object method names that should not be overridden.
pub(crate) const RESERVED_NAMES: &[&str] = &[
    "step",
    "setup",
    "release",
    "reset",
    "isDone",
    "isLocked",
    "getNumInputs",
    "getNumOutputs",
    "clone",
    "processTunedPropertiesImpl",
    "infoImpl",
];

/// Deprecated System object properties and their replacements.
pub(crate) const DEPRECATED_PROPERTIES: &[(&str, &str)] = &[
    ("SampleRate", "use getSampleRate method"),
    ("FrameLength", "set via input signal"),
    ("NumChannels", "determined automatically from input"),
];

/// Look up check metadata by ID.
pub(crate) fn check_meta(id: &str) -> Option<&'static CheckMeta> {
    CHECKS.iter().find(|c| c.id == id)
}

/// Look up check description by ID.
pub(crate) fn check_description(id: &str) -> &'static str {
    check_meta(id)
        .map(|c| c.description)
        .unwrap_or("System object validation issue")
}

/// Look up check severity by ID.
pub(crate) fn check_severity(id: &str) -> Severity {
    check_meta(id)
        .map(|c| c.severity)
        .unwrap_or(Severity::Warning)
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Engine covering 9 system object validation checks.
///
/// Uses pattern-based heuristics since full type information is not available.
/// Node-level checks fire on `function_call` nodes. File-level checks
/// analyze class definitions for property and method patterns.
pub struct SystemObjectsEngine {
    config: SystemObjectsEngineConfig,
}

/// Node types for node-level dispatch.
const TARGET_NODES: &[&str] = &["function_call"];

impl SystemObjectsEngine {
    /// Factory constructor called by the rule registry.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: SystemObjectsEngineConfig = config.rule_params("SYSTEM_OBJECTS_ENGINE");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Whether a check ID is enabled (not in `skip_checks`).
    pub(crate) fn is_enabled(&self, check_id: &str) -> bool {
        !self.config.skip_checks.iter().any(|s| s == check_id)
    }

    // -----------------------------------------------------------------------
    // Node-level check dispatchers
    // -----------------------------------------------------------------------

    /// Check a `function_call` node for system object method patterns.
    fn check_function_call<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let func_name = match extract_func_name(node, source) {
            Some(n) => n,
            None => return diags,
        };

        // Detect method calls on objects: obj.step(), obj.setup(), etc.
        // In tree-sitter-matlab, these may appear as field_expression function_calls
        // where the name is like "obj.step"
        if let Some(method_name) = func_name.rsplit('.').next() {
            if SYSTEM_OBJECT_METHODS.contains(&method_name) {
                diags.extend(self.check_sonumin(node, func_name, method_name));
                diags.extend(self.check_sonumout(node, method_name));
            }
        }

        diags.extend(self.check_sodeprop(node, func_name));

        diags
    }

    // -----------------------------------------------------------------------
    // File-level checks
    // -----------------------------------------------------------------------

    /// File-level checks for class definitions inheriting from matlab.System.
    fn check_file_level(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let root = tree.root_node();

        // Find class definitions that inherit from matlab.System
        self.find_system_classes(&root, source, &mut diags);

        diags
    }

    /// Find class definitions and check for system object patterns.
    fn find_system_classes(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                if child.kind() == "class_definition" {
                    // Check if this class inherits from matlab.System
                    if is_system_object_class(child, source) {
                        self.check_system_class(child, source, diags);
                    }
                }
                self.find_system_classes(&child, source, diags);
            }
        }
    }

    /// Check a System object class for various issues.
    fn check_system_class(
        &self,
        class_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        let count = class_node.child_count();
        for i in 0..count {
            if let Some(child) = class_node.child(i) {
                // SORSRVDNM: Check method names against reserved names
                if self.is_enabled("SORSRVDNM") && child.kind() == "methods" {
                    self.check_sorsrvdnm(child, source, diags);
                }

                // SODFLTVAL: Check properties for default value issues
                if self.is_enabled("SODFLTVAL") && child.kind() == "properties" {
                    self.check_sodfltval(child, source, diags);
                }

                // SOTUNPROP1/3/4: tunable property type constraints
                if child.kind() == "properties" {
                    self.check_sotunprop(child, source, diags);
                }
            }
        }

        // SOINITPROP: DiscreteState properties need a resetImpl method.
        self.check_soinitprop(class_node, source, diags);
    }
}

impl Rule for SystemObjectsEngine {
    fn id(&self) -> &'static str {
        "SYSTEM_OBJECTS_ENGINE"
    }

    fn description(&self) -> &'static str {
        "System object validation checks"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn category(&self) -> Category {
        Category::SystemObjects
    }

    fn enabled_by_default(&self) -> bool {
        // Only relevant for matlab.System classes; opt in explicitly.
        false
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        match ctx.node.kind() {
            "function_call" => self.check_function_call(ctx.node, ctx.source),
            _ => Vec::new(),
        }
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        self.check_file_level(ctx.tree, ctx.source)
    }
}

// ---------------------------------------------------------------------------
// Helper: diagnostic construction
// ---------------------------------------------------------------------------

/// Build a diagnostic from a check ID and node, using the check's own severity.
pub(crate) fn make_diag(check_id: &'static str, node: tree_sitter::Node) -> Diagnostic {
    let start = node.start_position();
    Diagnostic {
        rule_id: check_id,
        message: check_description(check_id).to_string(),
        severity: check_severity(check_id),
        byte_range: node.start_byte()..node.end_byte(),
        line: start.row + 1,
        column: start.column + 1,
        fix: None,
    }
}

/// Build a diagnostic whose message embeds a runtime name in place of the
/// `VAR_NAME` placeholder.
pub(crate) fn make_diag_named(
    check_id: &'static str,
    node: tree_sitter::Node,
    name: &str,
) -> Diagnostic {
    let start = node.start_position();
    Diagnostic {
        rule_id: check_id,
        message: check_description(check_id).replace("VAR_NAME", name),
        severity: check_severity(check_id),
        byte_range: node.start_byte()..node.end_byte(),
        line: start.row + 1,
        column: start.column + 1,
        fix: None,
    }
}

// ---------------------------------------------------------------------------
// Helper: node text extraction
// ---------------------------------------------------------------------------

/// Extract the raw text of a node.
pub(crate) fn node_text<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// The name of a `property` node — the first whitespace-delimited token of its
/// text (e.g. `Flag` for `Flag logical = false`).
pub(crate) fn property_name<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> &'a str {
    node_text(node, source)
        .split_whitespace()
        .next()
        .unwrap_or("")
}

/// Extract function name from a `function_call` node.
pub(crate) fn extract_func_name<'a>(
    node: tree_sitter::Node<'a>,
    source: &'a str,
) -> Option<&'a str> {
    if node.kind() != "function_call" {
        return None;
    }
    let name_node = node.child_by_field_name("name")?;
    Some(&source[name_node.start_byte()..name_node.end_byte()])
}

// ---------------------------------------------------------------------------
// Helper: pattern detection
// ---------------------------------------------------------------------------

/// Count the number of arguments in a `function_call` node.
pub(crate) fn count_args(node: tree_sitter::Node) -> usize {
    // In tree-sitter-matlab call arguments are a child node of kind
    // `arguments` rather than a named field, so `child_by_field_name` never
    // matches. Locate them by child kind instead.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "arguments" {
            return child.named_child_count();
        }
    }
    0
}

/// Check if a function_call is the RHS of an assignment (has output).
pub(crate) fn has_output_assignment(node: tree_sitter::Node) -> bool {
    if let Some(parent) = node.parent() {
        // Check if parent is an assignment and this node is on the right side
        if parent.kind() == "assignment" {
            if let Some(rhs) = parent
                .child_by_field_name("right")
                .or_else(|| parent.child(2))
            {
                return node.start_byte() >= rhs.start_byte() && node.end_byte() <= rhs.end_byte();
            }
        }
    }
    false
}

/// Check if a class definition inherits from matlab.System.
pub(crate) fn is_system_object_class(class_node: tree_sitter::Node, source: &str) -> bool {
    // Look for superclasses node containing "matlab.System"
    let count = class_node.child_count();
    for i in 0..count {
        if let Some(child) = class_node.child(i) {
            if child.kind() == "superclasses" {
                let text = node_text(child, source);
                if text.contains("matlab.System") {
                    return true;
                }
            }
        }
    }
    false
}

/// The `property` child nodes of a class definition, across all its
/// `properties` blocks.
pub(crate) fn class_properties(class_node: tree_sitter::Node) -> Vec<tree_sitter::Node> {
    let mut result = Vec::new();
    let mut cursor = class_node.walk();
    for child in class_node.children(&mut cursor) {
        if child.kind() == "properties" {
            let mut inner = child.walk();
            for prop in child.children(&mut inner) {
                if prop.kind() == "property" {
                    result.push(prop);
                }
            }
        }
    }
    result
}

/// True when the class defines a method with the given name.
pub(crate) fn class_has_method(
    class_node: tree_sitter::Node,
    method_name: &str,
    source: &str,
) -> bool {
    let mut stack = vec![class_node];
    while let Some(node) = stack.pop() {
        if node.kind() == "function_definition" {
            // Function name is the first identifier child (or after a
            // function_output like `function y = resetImpl(...)`).
            if let Some(name) = function_name(node, source) {
                if name == method_name {
                    return true;
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }
    false
}

/// The name of a `function_definition` node.
fn function_name<'a>(func: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    let mut cursor = func.walk();
    for child in func.children(&mut cursor) {
        match child.kind() {
            "function_output" => {
                // `function y = name(...)`: the name is the next identifier
                // sibling of the function_output.
                if let Some(next) = child.next_named_sibling() {
                    if next.kind() == "identifier" {
                        return Some(&source[next.start_byte()..next.end_byte()]);
                    }
                }
            }
            "identifier" => {
                // `function name(...)`: first plain identifier after `function`.
                return Some(&source[child.start_byte()..child.end_byte()]);
            }
            _ => {}
        }
    }
    None
}

/// True when the raw text of a node contains `needle`.
pub(crate) fn block_text_has(node: tree_sitter::Node, needle: &str, source: &str) -> bool {
    node_text(node, source).contains(needle)
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "SYSTEM_OBJECTS_ENGINE",
    SystemObjectsEngine::from_config
));

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use malt_core::Config;

    /// Build a `SystemObjectsEngine` with default configuration for tests.
    pub(crate) fn engine() -> Box<dyn Rule> {
        SystemObjectsEngine::from_config(&Config::default())
    }

    /// Wrap a class body in a `matlab.System` class definition.
    pub(crate) fn system_class(source: &str) -> String {
        format!("classdef mySystem < matlab.System\n{}\nend\n", source)
    }
}
