//! # NAMING_ENGINE: Naming Checks
//!
//! ```malt
//! id = "NAMING_ENGINE"
//! title = "Naming Checks"
//! category = "naming"
//! severity = "info"
//! fix = false
//! icon = "lucide/type"
//! slug = "naming"
//! generated = "naming"
//! ```
//!
//! ## Rule
//!
//! A single generic engine that implements all 81 naming-convention checks
//! from MATLAB's Code Analyzer. The check IDs are the Cartesian product of
//! 9 entity types × 9 check types:
//!
//! - **Entities**: class, function, localFunction, method, nestedFunction,
//!   property, event, enumeration, variable
//! - **Check types**: maxLength, minLength, regularExpression, requiredPrefix,
//!   disallowedPrefix, disallowedPhrase, requiredSuffix, disallowedSuffix,
//!   casing
//!
//! Rule IDs follow the pattern `naming.<entity>.<checkType>` (e.g.
//! `naming.class.casing`). The `NamingEngine` registers once with inventory
//! as `"NAMING_ENGINE"` and uses `has_file_check() = true`: it walks the tree
//! once to extract all named entities, then applies every enabled check to
//! each matching entity, emitting diagnostics with the specific rule ID.
//!
//! Only `maxLength` (default 63, MATLAB's `namelengthmax`) and `minLength`
//! (default 2) produce diagnostics without user configuration; the other
//! seven check types require configuration.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! myvar1 = 1;       % naming.variable.casing — style configured, e.g. camelCase
//! classdef myClass  % naming.class.casing — style configured, e.g. PascalCase
//! end
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! myVar1 = 1;
//! classdef MyClass
//! end
//! ```
//!
//! ## Configuration
//!
//! Each of the 81 checks can be configured individually through its rule ID:
//!
//! ```toml
//! [lint.rules."naming.class.casing"]
//! severity = "error"
//! style = "PascalCase"
//!
//! [lint.rules."naming.function.maxLength"]
//! severity = "warn"
//! max = 32
//!
//! [lint.rules."naming.variable.disallowedPrefix"]
//! prefix = "temp"
//! ```

use std::sync::LazyLock;

use malt_core::{Category, Config, Diagnostic, FileContext, Rule, Severity};
use regex::Regex;
use serde::Deserialize;
use tree_sitter::Node;

// ---------------------------------------------------------------------------
// Entity and check type enums
// ---------------------------------------------------------------------------

/// Entity types that naming rules apply to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamingEntity {
    Class,
    Function,
    LocalFunction,
    Method,
    NestedFunction,
    Property,
    Event,
    Enumeration,
    Variable,
}

impl NamingEntity {
    /// All entity types in the canonical order.
    pub const ALL: &'static [NamingEntity] = &[
        NamingEntity::Class,
        NamingEntity::Function,
        NamingEntity::LocalFunction,
        NamingEntity::Method,
        NamingEntity::NestedFunction,
        NamingEntity::Property,
        NamingEntity::Event,
        NamingEntity::Enumeration,
        NamingEntity::Variable,
    ];

    /// String key used in rule IDs.
    pub fn as_str(self) -> &'static str {
        match self {
            NamingEntity::Class => "class",
            NamingEntity::Function => "function",
            NamingEntity::LocalFunction => "localFunction",
            NamingEntity::Method => "method",
            NamingEntity::NestedFunction => "nestedFunction",
            NamingEntity::Property => "property",
            NamingEntity::Event => "event",
            NamingEntity::Enumeration => "enumeration",
            NamingEntity::Variable => "variable",
        }
    }
}

/// Check types for naming rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamingCheckType {
    MaxLength,
    MinLength,
    RegularExpression,
    RequiredPrefix,
    DisallowedPrefix,
    DisallowedPhrase,
    RequiredSuffix,
    DisallowedSuffix,
    Casing,
}

impl NamingCheckType {
    /// All check types in the canonical order.
    pub const ALL: &'static [NamingCheckType] = &[
        NamingCheckType::MaxLength,
        NamingCheckType::MinLength,
        NamingCheckType::RegularExpression,
        NamingCheckType::RequiredPrefix,
        NamingCheckType::DisallowedPrefix,
        NamingCheckType::DisallowedPhrase,
        NamingCheckType::RequiredSuffix,
        NamingCheckType::DisallowedSuffix,
        NamingCheckType::Casing,
    ];

    /// String key used in rule IDs.
    pub fn as_str(self) -> &'static str {
        match self {
            NamingCheckType::MaxLength => "maxLength",
            NamingCheckType::MinLength => "minLength",
            NamingCheckType::RegularExpression => "regularExpression",
            NamingCheckType::RequiredPrefix => "requiredPrefix",
            NamingCheckType::DisallowedPrefix => "disallowedPrefix",
            NamingCheckType::DisallowedPhrase => "disallowedPhrase",
            NamingCheckType::RequiredSuffix => "requiredSuffix",
            NamingCheckType::DisallowedSuffix => "disallowedSuffix",
            NamingCheckType::Casing => "casing",
        }
    }
}

// ---------------------------------------------------------------------------
// Static rule ID table (81 entries)
// ---------------------------------------------------------------------------

/// All 81 naming rule IDs as `&'static str`, lazily generated.
///
/// Uses `Box::leak` to produce `'static` references since these live for the
/// entire process lifetime.
static NAMING_IDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let mut ids = Vec::with_capacity(81);
    for entity in NamingEntity::ALL {
        for check in NamingCheckType::ALL {
            let id = format!("naming.{}.{}", entity.as_str(), check.as_str());
            ids.push(Box::leak(id.into_boxed_str()) as &'static str);
        }
    }
    ids
});

/// Get the static rule ID for a given entity/check combination.
fn rule_id_for(entity: NamingEntity, check_type: NamingCheckType) -> &'static str {
    let entity_idx = NamingEntity::ALL.iter().position(|e| *e == entity).unwrap();
    let check_idx = NamingCheckType::ALL
        .iter()
        .position(|c| *c == check_type)
        .unwrap();
    let index = entity_idx * NamingCheckType::ALL.len() + check_idx;
    NAMING_IDS[index]
}

/// Iterator over all (id, entity, check_type) combinations.
fn all_naming_combinations() -> Vec<(&'static str, NamingEntity, NamingCheckType)> {
    let mut combos = Vec::with_capacity(81);
    for &entity in NamingEntity::ALL {
        for &check_type in NamingCheckType::ALL {
            combos.push((rule_id_for(entity, check_type), entity, check_type));
        }
    }
    combos
}

// ---------------------------------------------------------------------------
// Per-check configuration
// ---------------------------------------------------------------------------

/// Configuration for a single naming check, read from `.malt.toml`.
///
/// Each field applies to a specific check type:
/// - `max`: maxLength check threshold
/// - `min`: minLength check threshold
/// - `pattern`: regularExpression pattern
/// - `prefix`: requiredPrefix / disallowedPrefix value
/// - `phrase`: disallowedPhrase value
/// - `suffix`: requiredSuffix / disallowedSuffix value
/// - `style`: casing style ("camelCase", "PascalCase", "snake_case", "UPPER_CASE")
#[derive(Debug, Clone, Deserialize, Default)]
pub struct NamingCheckConfig {
    /// Maximum allowed name length (for maxLength checks).
    #[serde(default)]
    pub max: Option<usize>,
    /// Minimum required name length (for minLength checks).
    #[serde(default)]
    pub min: Option<usize>,
    /// Regular expression the name must match (for regularExpression checks).
    #[serde(default)]
    pub pattern: Option<String>,
    /// Required or disallowed prefix string.
    #[serde(default)]
    pub prefix: Option<String>,
    /// Disallowed phrase (substring).
    #[serde(default)]
    pub phrase: Option<String>,
    /// Required or disallowed suffix string.
    #[serde(default)]
    pub suffix: Option<String>,
    /// Casing style: "camelCase", "PascalCase", "snake_case", "UPPER_CASE".
    #[serde(default)]
    pub style: Option<String>,
}

// ---------------------------------------------------------------------------
// Named entity representation
// ---------------------------------------------------------------------------

/// A named entity extracted from the AST.
struct NamedEntity<'a> {
    /// The entity type (class, function, variable, etc.)
    entity: NamingEntity,
    /// The name text.
    name: &'a str,
    /// Byte range of the name in the source (for diagnostic positioning).
    byte_range: std::ops::Range<usize>,
    /// 1-indexed line number.
    line: usize,
    /// 1-indexed column number.
    column: usize,
}

// ---------------------------------------------------------------------------
// NamingEngine rule
// ---------------------------------------------------------------------------

/// A single active naming check instance.
struct NamingRuleInstance {
    /// The static rule ID (e.g., "naming.class.casing").
    id: &'static str,
    /// Which entity type this check applies to.
    entity: NamingEntity,
    /// What kind of check this is.
    check_type: NamingCheckType,
    /// User configuration for this check.
    config: NamingCheckConfig,
    /// Compiled regex (lazily built from config.pattern).
    compiled_regex: Option<Regex>,
}

/// The generic naming engine that handles all 81 naming convention checks.
///
/// Registered as `"NAMING_ENGINE"` with inventory. Uses file-level analysis
/// to walk the tree once and apply all enabled naming checks.
pub struct NamingEngine {
    /// All enabled naming check instances.
    checks: Vec<NamingRuleInstance>,
}

impl NamingEngine {
    /// Factory constructor invoked by inventory.
    ///
    /// Reads configuration for all 81 naming rules and constructs instances
    /// only for those that are enabled.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let mut checks = Vec::with_capacity(81);
        for (id, entity, check_type) in all_naming_combinations() {
            if config.is_rule_enabled(id) {
                let check_config: NamingCheckConfig = config.rule_params(id);
                let compiled_regex = check_config
                    .pattern
                    .as_ref()
                    .and_then(|p| Regex::new(p).ok());
                checks.push(NamingRuleInstance {
                    id,
                    entity,
                    check_type,
                    config: check_config,
                    compiled_regex,
                });
            }
        }
        Box::new(Self { checks })
    }
}

impl Rule for NamingEngine {
    fn id(&self) -> &'static str {
        "NAMING_ENGINE"
    }

    fn description(&self) -> &'static str {
        "Naming convention checks (81 sub-rules for 9 entity types x 9 check types)"
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn category(&self) -> Category {
        Category::Naming
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        // File-level only — no node subscriptions.
        &[]
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        if self.checks.is_empty() {
            return Vec::new();
        }

        // Step 1: Extract all named entities from the tree.
        let entities = collect_named_entities(ctx.tree, ctx.source);

        // Step 2: Apply each enabled check to matching entities.
        let mut diagnostics = Vec::new();
        for entity in &entities {
            for check in &self.checks {
                if check.entity != entity.entity {
                    continue;
                }
                if let Some(diag) = run_check(check, entity) {
                    diagnostics.push(diag);
                }
            }
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Entity extraction
// ---------------------------------------------------------------------------

/// Walk the tree-sitter tree and extract all named entities.
///
/// Determines entity type based on AST context:
/// - `class_definition` → Class
/// - `function_definition` at file level (first one) → Function
/// - `function_definition` at file level (subsequent) → LocalFunction
/// - `function_definition` inside `methods` block → Method
/// - `function_definition` inside another function → NestedFunction
/// - `property` node → Property
/// - node inside `events` block → Event
/// - `enum` node inside `enumeration` block → Enumeration
/// - `assignment` LHS identifier → Variable
fn collect_named_entities<'a>(
    tree: &'a tree_sitter::Tree,
    source: &'a str,
) -> Vec<NamedEntity<'a>> {
    let mut entities = Vec::new();
    let mut first_function_seen = false;

    collect_recursive(
        tree.root_node(),
        source,
        &mut entities,
        &mut first_function_seen,
        EntityContext::TopLevel,
    );

    entities
}

/// Context for determining what kind of entity a function_definition represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntityContext {
    /// At the top level of the file (source_file or block).
    TopLevel,
    /// Inside a methods block of a class.
    Methods,
    /// Inside another function (nested).
    Function,
    /// Inside an events block.
    Events,
    /// Inside an enumeration block.
    Enumeration,
}

/// Recursively collect named entities from the AST.
fn collect_recursive<'a>(
    node: Node<'a>,
    source: &'a str,
    entities: &mut Vec<NamedEntity<'a>>,
    first_function_seen: &mut bool,
    context: EntityContext,
) {
    match node.kind() {
        "function_definition" => {
            // Extract function name from the "name" field.
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = &source[name_node.start_byte()..name_node.end_byte()];
                let pos = name_node.start_position();
                let entity_type = match context {
                    EntityContext::Methods => NamingEntity::Method,
                    EntityContext::Function => NamingEntity::NestedFunction,
                    EntityContext::TopLevel => {
                        if !*first_function_seen {
                            *first_function_seen = true;
                            NamingEntity::Function
                        } else {
                            NamingEntity::LocalFunction
                        }
                    }
                    _ => NamingEntity::Function,
                };
                entities.push(NamedEntity {
                    entity: entity_type,
                    name,
                    byte_range: name_node.start_byte()..name_node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                });
            }

            // Recurse into function body with Function context.
            let child_count = node.child_count();
            for i in 0..child_count {
                if let Some(child) = node.child(i) {
                    collect_recursive(
                        child,
                        source,
                        entities,
                        first_function_seen,
                        EntityContext::Function,
                    );
                }
            }
            return; // Don't recurse again below.
        }

        "class_definition" => {
            // Extract class name from the "name" field.
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = &source[name_node.start_byte()..name_node.end_byte()];
                let pos = name_node.start_position();
                entities.push(NamedEntity {
                    entity: NamingEntity::Class,
                    name,
                    byte_range: name_node.start_byte()..name_node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                });
            }

            // Recurse into class children (methods, properties, events, enumeration).
            let child_count = node.child_count();
            for i in 0..child_count {
                if let Some(child) = node.child(i) {
                    collect_recursive(child, source, entities, first_function_seen, context);
                }
            }
            return;
        }

        "methods" => {
            // Children are function_definitions → Method.
            let child_count = node.child_count();
            for i in 0..child_count {
                if let Some(child) = node.child(i) {
                    collect_recursive(
                        child,
                        source,
                        entities,
                        first_function_seen,
                        EntityContext::Methods,
                    );
                }
            }
            return;
        }

        "events" => {
            // Children are event identifiers.
            let child_count = node.child_count();
            for i in 0..child_count {
                if let Some(child) = node.child(i) {
                    collect_recursive(
                        child,
                        source,
                        entities,
                        first_function_seen,
                        EntityContext::Events,
                    );
                }
            }
            return;
        }

        "enumeration" => {
            // Children are enum nodes.
            let child_count = node.child_count();
            for i in 0..child_count {
                if let Some(child) = node.child(i) {
                    collect_recursive(
                        child,
                        source,
                        entities,
                        first_function_seen,
                        EntityContext::Enumeration,
                    );
                }
            }
            return;
        }

        "property" => {
            // Extract property name (first identifier child or "name" field).
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = &source[name_node.start_byte()..name_node.end_byte()];
                let pos = name_node.start_position();
                entities.push(NamedEntity {
                    entity: NamingEntity::Property,
                    name,
                    byte_range: name_node.start_byte()..name_node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                });
            } else {
                // Fallback: find the first identifier child.
                let child_count = node.child_count();
                for i in 0..child_count {
                    if let Some(child) = node.child(i) {
                        if child.kind() == "identifier" {
                            let name = &source[child.start_byte()..child.end_byte()];
                            let pos = child.start_position();
                            entities.push(NamedEntity {
                                entity: NamingEntity::Property,
                                name,
                                byte_range: child.start_byte()..child.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                            });
                            break;
                        }
                    }
                }
            }
            return;
        }

        "enum" => {
            if context == EntityContext::Enumeration {
                // Extract enum member name (first identifier child or "name" field).
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = &source[name_node.start_byte()..name_node.end_byte()];
                    let pos = name_node.start_position();
                    entities.push(NamedEntity {
                        entity: NamingEntity::Enumeration,
                        name,
                        byte_range: name_node.start_byte()..name_node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                    });
                } else {
                    let child_count = node.child_count();
                    for i in 0..child_count {
                        if let Some(child) = node.child(i) {
                            if child.kind() == "identifier" {
                                let name = &source[child.start_byte()..child.end_byte()];
                                let pos = child.start_position();
                                entities.push(NamedEntity {
                                    entity: NamingEntity::Enumeration,
                                    name,
                                    byte_range: child.start_byte()..child.end_byte(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                });
                                break;
                            }
                        }
                    }
                }
            }
            return;
        }

        "identifier" if context == EntityContext::Events => {
            // Identifier inside events block is an event name.
            let name = &source[node.start_byte()..node.end_byte()];
            let pos = node.start_position();
            entities.push(NamedEntity {
                entity: NamingEntity::Event,
                name,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
            });
            return;
        }

        "assignment" => {
            // Extract LHS variable name. Only simple identifiers, not field
            // expressions or indexed assignments.
            if let Some(lhs) = node.child_by_field_name("left") {
                if lhs.kind() == "identifier" {
                    let name = &source[lhs.start_byte()..lhs.end_byte()];
                    let pos = lhs.start_position();
                    entities.push(NamedEntity {
                        entity: NamingEntity::Variable,
                        name,
                        byte_range: lhs.start_byte()..lhs.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                    });
                } else if lhs.kind() == "multioutput_variable" {
                    // Handle [a, b] = ... — extract each identifier.
                    let lhs_count = lhs.child_count();
                    for i in 0..lhs_count {
                        if let Some(child) = lhs.child(i) {
                            if child.kind() == "identifier" {
                                let name = &source[child.start_byte()..child.end_byte()];
                                let pos = child.start_position();
                                entities.push(NamedEntity {
                                    entity: NamingEntity::Variable,
                                    name,
                                    byte_range: child.start_byte()..child.end_byte(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                });
                            }
                        }
                    }
                }
            }
            // Don't return — recurse into RHS for nested assignments.
        }

        _ => {}
    }

    // Default: recurse into children with the same context.
    let child_count = node.child_count();
    for i in 0..child_count {
        if let Some(child) = node.child(i) {
            collect_recursive(child, source, entities, first_function_seen, context);
        }
    }
}

// ---------------------------------------------------------------------------
// Check execution
// ---------------------------------------------------------------------------

/// Run a single naming check against a named entity.
/// Returns `Some(Diagnostic)` if the check fails.
fn run_check(check: &NamingRuleInstance, entity: &NamedEntity) -> Option<Diagnostic> {
    let name = entity.name;
    let message = match check.check_type {
        NamingCheckType::MaxLength => check_max_length(name, &check.config)?,
        NamingCheckType::MinLength => check_min_length(name, &check.config)?,
        NamingCheckType::RegularExpression => check_regex(name, check)?,
        NamingCheckType::RequiredPrefix => check_required_prefix(name, &check.config)?,
        NamingCheckType::DisallowedPrefix => check_disallowed_prefix(name, &check.config)?,
        NamingCheckType::DisallowedPhrase => check_disallowed_phrase(name, &check.config)?,
        NamingCheckType::RequiredSuffix => check_required_suffix(name, &check.config)?,
        NamingCheckType::DisallowedSuffix => check_disallowed_suffix(name, &check.config)?,
        NamingCheckType::Casing => check_casing(name, &check.config)?,
    };

    Some(Diagnostic {
        rule_id: check.id,
        message,
        severity: Severity::Info, // Will be overridden by effective severity.
        byte_range: entity.byte_range.clone(),
        line: entity.line,
        column: entity.column,
        fix: None,
    })
}

// ---------------------------------------------------------------------------
// Individual check implementations
// ---------------------------------------------------------------------------

/// Check if name exceeds maximum length.
fn check_max_length(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let max = config.max.unwrap_or(63); // MATLAB default namelengthmax
    if name.len() > max {
        Some(format!(
            "Name '{}' exceeds maximum length of {} (has {} characters)",
            name,
            max,
            name.len()
        ))
    } else {
        None
    }
}

/// Check if name is shorter than minimum length.
fn check_min_length(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let min = config.min.unwrap_or(2);
    // Don't flag single-char loop vars like i, j, k by default unless min > 1.
    if name.len() < min {
        Some(format!(
            "Name '{}' is shorter than minimum length of {} (has {} characters)",
            name,
            min,
            name.len()
        ))
    } else {
        None
    }
}

/// Check if name matches the required regular expression pattern.
fn check_regex(name: &str, check: &NamingRuleInstance) -> Option<String> {
    let re = check.compiled_regex.as_ref()?;
    if !re.is_match(name) {
        Some(format!(
            "Name '{}' does not match required pattern '{}'",
            name,
            check.config.pattern.as_deref().unwrap_or("")
        ))
    } else {
        None
    }
}

/// Check if name has the required prefix.
fn check_required_prefix(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let prefix = config.prefix.as_deref()?;
    if !name.starts_with(prefix) {
        Some(format!(
            "Name '{}' does not have the required prefix '{}'",
            name, prefix
        ))
    } else {
        None
    }
}

/// Check if name has a disallowed prefix.
fn check_disallowed_prefix(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let prefix = config.prefix.as_deref()?;
    if name.starts_with(prefix) {
        Some(format!(
            "Name '{}' has the disallowed prefix '{}'",
            name, prefix
        ))
    } else {
        None
    }
}

/// Check if name contains a disallowed phrase.
fn check_disallowed_phrase(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let phrase = config.phrase.as_deref()?;
    if name.contains(phrase) {
        Some(format!(
            "Name '{}' contains the disallowed phrase '{}'",
            name, phrase
        ))
    } else {
        None
    }
}

/// Check if name has the required suffix.
fn check_required_suffix(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let suffix = config.suffix.as_deref()?;
    if !name.ends_with(suffix) {
        Some(format!(
            "Name '{}' does not have the required suffix '{}'",
            name, suffix
        ))
    } else {
        None
    }
}

/// Check if name has a disallowed suffix.
fn check_disallowed_suffix(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let suffix = config.suffix.as_deref()?;
    if name.ends_with(suffix) {
        Some(format!(
            "Name '{}' has the disallowed suffix '{}'",
            name, suffix
        ))
    } else {
        None
    }
}

/// Check if name follows the required casing style.
///
/// Supported styles:
/// - `"camelCase"`: starts with lowercase, no underscores (except leading `_`), contains uppercase
/// - `"PascalCase"`: starts with uppercase, no underscores
/// - `"snake_case"`: all lowercase with underscores
/// - `"UPPER_CASE"` / `"CONSTANT_CASE"`: all uppercase with underscores
fn check_casing(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let style = config.style.as_deref()?;

    // Strip leading underscores for style checking (preserving convention for private members).
    let stripped = name.trim_start_matches('_');
    if stripped.is_empty() {
        return None; // All underscores — skip.
    }

    let violation = match style {
        "camelCase" => !is_camel_case(stripped),
        "PascalCase" => !is_pascal_case(stripped),
        "snake_case" => !is_snake_case(stripped),
        "UPPER_CASE" | "CONSTANT_CASE" => !is_upper_case(stripped),
        _ => false, // Unknown style — don't flag.
    };

    if violation {
        Some(format!(
            "Name '{}' does not follow {} naming convention",
            name, style
        ))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Casing helpers
// ---------------------------------------------------------------------------

/// Check if a name follows camelCase: starts lowercase, no underscores, has uppercase after first char.
fn is_camel_case(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    // Must not contain underscores (except it's acceptable if single-word lowercase).
    if name.contains('_') {
        return false;
    }
    true
}

/// Check if a name follows PascalCase: starts uppercase, no underscores.
fn is_pascal_case(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() => {}
        _ => return false,
    }
    !name.contains('_')
}

/// Check if a name follows snake_case: all lowercase and underscores, no uppercase.
fn is_snake_case(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Check if a name follows UPPER_CASE: all uppercase and underscores, no lowercase.
fn is_upper_case(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "NAMING_ENGINE",
    NamingEngine::from_config
));

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        NamingEngine::from_config(&Config::default())
    }

    /// Build an engine with a single naming check configured via TOML.
    fn engine_with(rule_id: &str, params: &str) -> Box<dyn Rule> {
        let config = Config::from_toml(&format!("[lint.rules.\"{rule_id}\"]\n{params}\n")).unwrap();
        NamingEngine::from_config(&config)
    }

    // -- minLength (default) --------------------------------------------------

    #[test]
    fn variable_min_length_fires_on_single_char() {
        let diags = lint_file(&*engine(), "a = 1;\n");
        assert!(
            has_id(&diags, "naming.variable.minLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn variable_min_length_ok_on_two_chars() {
        let diags = lint_file(&*engine(), "ab = 1;\n");
        assert!(
            !has_id(&diags, "naming.variable.minLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn function_min_length_fires_on_single_char() {
        let diags = lint_file(&*engine(), "function f()\nend\n");
        assert!(
            has_id(&diags, "naming.function.minLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn function_min_length_ok_on_two_chars() {
        let diags = lint_file(&*engine(), "function fo()\nend\n");
        assert!(
            !has_id(&diags, "naming.function.minLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn multioutput_assignment_extracts_each_variable() {
        let diags = lint_file(&*engine(), "[a, b] = deal(1, 2);\n");
        assert!(
            has_id(&diags, "naming.variable.minLength"),
            "got: {diags:?}"
        );
    }

    // -- maxLength (default 63) ------------------------------------------------

    #[test]
    fn variable_max_length_fires_on_long_name() {
        let long_name = "a".repeat(64);
        let source = format!("{long_name} = 1;\n");
        let diags = lint_file(&*engine(), &source);
        assert!(
            has_id(&diags, "naming.variable.maxLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn variable_max_length_ok_at_exactly_63_chars() {
        let name = "b".repeat(63);
        let source = format!("{name} = 1;\n");
        let diags = lint_file(&*engine(), &source);
        assert!(
            !has_id(&diags, "naming.variable.maxLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn variable_max_length_ok_on_short_name() {
        let diags = lint_file(&*engine(), "someVariable = 1;\n");
        assert!(
            !has_id(&diags, "naming.variable.maxLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn default_config_no_diagnostics_for_normal_names() {
        let diags = lint_file(&*engine(), "someVariable = 1;\nanotherVariable = 2;\n");
        assert!(diags.is_empty(), "got: {diags:?}");
    }

    // -- Configured thresholds ------------------------------------------------

    #[test]
    fn max_length_fires_with_lowered_threshold() {
        let engine = engine_with("naming.variable.maxLength", "max = 5");
        let diags = lint_file(&*engine, "myVariable = 1;\n");
        assert!(
            has_id(&diags, "naming.variable.maxLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn max_length_ok_with_lowered_threshold() {
        let engine = engine_with("naming.variable.maxLength", "max = 5");
        let diags = lint_file(&*engine, "myVar = 1;\n");
        assert!(
            !has_id(&diags, "naming.variable.maxLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn min_length_fires_with_raised_threshold() {
        let engine = engine_with("naming.variable.minLength", "min = 5");
        let diags = lint_file(&*engine, "ab = 1;\n");
        assert!(
            has_id(&diags, "naming.variable.minLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn min_length_ok_with_raised_threshold() {
        let engine = engine_with("naming.variable.minLength", "min = 5");
        let diags = lint_file(&*engine, "abcdef = 1;\n");
        assert!(
            !has_id(&diags, "naming.variable.minLength"),
            "got: {diags:?}"
        );
    }

    // -- requiredPrefix --------------------------------------------------------

    #[test]
    fn required_prefix_fires_when_configured() {
        let engine = engine_with("naming.variable.requiredPrefix", "prefix = \"temp\"");
        let diags = lint_file(&*engine, "myVar = 1;\n");
        assert!(
            has_id(&diags, "naming.variable.requiredPrefix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn required_prefix_ok_when_satisfied() {
        let engine = engine_with("naming.variable.requiredPrefix", "prefix = \"temp\"");
        let diags = lint_file(&*engine, "tempVar = 1;\n");
        assert!(
            !has_id(&diags, "naming.variable.requiredPrefix"),
            "got: {diags:?}"
        );
    }

    // -- disallowedPrefix ------------------------------------------------------

    #[test]
    fn disallowed_prefix_fires_when_configured() {
        let engine = engine_with("naming.variable.disallowedPrefix", "prefix = \"tmp\"");
        let diags = lint_file(&*engine, "tmpVar = 1;\n");
        assert!(
            has_id(&diags, "naming.variable.disallowedPrefix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn disallowed_prefix_ok_when_not_matching() {
        let engine = engine_with("naming.variable.disallowedPrefix", "prefix = \"tmp\"");
        let diags = lint_file(&*engine, "myVar = 1;\n");
        assert!(
            !has_id(&diags, "naming.variable.disallowedPrefix"),
            "got: {diags:?}"
        );
    }

    // -- requiredSuffix --------------------------------------------------------

    #[test]
    fn required_suffix_fires_when_configured() {
        let engine = engine_with("naming.function.requiredSuffix", "suffix = \"_fn\"");
        let diags = lint_file(&*engine, "function doStuff()\nend\n");
        assert!(
            has_id(&diags, "naming.function.requiredSuffix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn required_suffix_ok_when_satisfied() {
        let engine = engine_with("naming.function.requiredSuffix", "suffix = \"_fn\"");
        let diags = lint_file(&*engine, "function doStuff_fn()\nend\n");
        assert!(
            !has_id(&diags, "naming.function.requiredSuffix"),
            "got: {diags:?}"
        );
    }

    // -- disallowedSuffix ------------------------------------------------------

    #[test]
    fn disallowed_suffix_fires_when_configured() {
        let engine = engine_with("naming.variable.disallowedSuffix", "suffix = \"_count\"");
        let diags = lint_file(&*engine, "loop_count = 0;\n");
        assert!(
            has_id(&diags, "naming.variable.disallowedSuffix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn disallowed_suffix_ok_when_not_matching() {
        let engine = engine_with("naming.variable.disallowedSuffix", "suffix = \"_count\"");
        let diags = lint_file(&*engine, "loopTotal = 0;\n");
        assert!(
            !has_id(&diags, "naming.variable.disallowedSuffix"),
            "got: {diags:?}"
        );
    }

    // -- disallowedPhrase ------------------------------------------------------

    #[test]
    fn disallowed_phrase_fires_when_configured() {
        let engine = engine_with("naming.variable.disallowedPhrase", "phrase = \"old\"");
        let diags = lint_file(&*engine, "oldValue = 1;\n");
        assert!(
            has_id(&diags, "naming.variable.disallowedPhrase"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn disallowed_phrase_ok_when_not_matching() {
        let engine = engine_with("naming.variable.disallowedPhrase", "phrase = \"old\"");
        let diags = lint_file(&*engine, "freshValue = 1;\n");
        assert!(
            !has_id(&diags, "naming.variable.disallowedPhrase"),
            "got: {diags:?}"
        );
    }

    // -- regularExpression -----------------------------------------------------

    #[test]
    fn regular_expression_fires_when_pattern_not_matched() {
        let engine = engine_with(
            "naming.variable.regularExpression",
            "pattern = \"^[a-z]+[A-Z][a-z]+$\"",
        );
        let diags = lint_file(&*engine, "my_var = 1;\n");
        assert!(
            has_id(&diags, "naming.variable.regularExpression"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn regular_expression_ok_when_pattern_matched() {
        let engine = engine_with(
            "naming.variable.regularExpression",
            "pattern = \"^[a-z]+[A-Z][a-z]+$\"",
        );
        let diags = lint_file(&*engine, "myVar = 1;\n");
        assert!(
            !has_id(&diags, "naming.variable.regularExpression"),
            "got: {diags:?}"
        );
    }

    // -- casing ----------------------------------------------------------------

    #[test]
    fn casing_camel_fires_when_configured() {
        let engine = engine_with("naming.variable.casing", "style = \"camelCase\"");
        let diags = lint_file(&*engine, "MyVar = 1;\n");
        assert!(has_id(&diags, "naming.variable.casing"), "got: {diags:?}");
    }

    #[test]
    fn casing_camel_ok_when_satisfied() {
        let engine = engine_with("naming.variable.casing", "style = \"camelCase\"");
        let diags = lint_file(&*engine, "myVar = 1;\n");
        assert!(!has_id(&diags, "naming.variable.casing"), "got: {diags:?}");
    }

    #[test]
    fn casing_pascal_fires_when_configured() {
        let engine = engine_with("naming.variable.casing", "style = \"PascalCase\"");
        let diags = lint_file(&*engine, "myVar = 1;\n");
        assert!(has_id(&diags, "naming.variable.casing"), "got: {diags:?}");
    }

    #[test]
    fn casing_snake_fires_when_configured() {
        let engine = engine_with("naming.variable.casing", "style = \"snake_case\"");
        let diags = lint_file(&*engine, "MyVar = 1;\n");
        assert!(has_id(&diags, "naming.variable.casing"), "got: {diags:?}");
    }

    #[test]
    fn casing_upper_fires_when_configured() {
        let engine = engine_with("naming.variable.casing", "style = \"UPPER_CASE\"");
        let diags = lint_file(&*engine, "myVar = 1;\n");
        assert!(has_id(&diags, "naming.variable.casing"), "got: {diags:?}");
    }

    // -- Entity extraction -----------------------------------------------------

    #[test]
    fn class_entity_detected() {
        let engine = engine_with("naming.class.requiredPrefix", "prefix = \"Cls\"");
        let diags = lint_file(&*engine, "classdef MyClass\nend\n");
        assert!(
            has_id(&diags, "naming.class.requiredPrefix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn local_function_entity_detected() {
        let engine = engine_with("naming.localFunction.requiredPrefix", "prefix = \"lf_\"");
        let diags = lint_file(&*engine, "function main()\nend\nfunction helper()\nend\n");
        assert!(
            has_id(&diags, "naming.localFunction.requiredPrefix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn nested_function_entity_detected() {
        let engine = engine_with("naming.nestedFunction.requiredPrefix", "prefix = \"nf_\"");
        let diags = lint_file(
            &*engine,
            "function main()\n    function helper()\n    end\nend\n",
        );
        assert!(
            has_id(&diags, "naming.nestedFunction.requiredPrefix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn method_entity_detected() {
        let engine = engine_with("naming.method.requiredPrefix", "prefix = \"m_\"");
        let diags = lint_file(
            &*engine,
            "classdef MyClass\n    methods\n        function helper()\n        end\n    end\nend\n",
        );
        assert!(
            has_id(&diags, "naming.method.requiredPrefix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn property_entity_detected() {
        let engine = engine_with("naming.property.requiredPrefix", "prefix = \"prop_\"");
        let diags = lint_file(
            &*engine,
            "classdef MyClass\n    properties\n        Value\n    end\nend\n",
        );
        assert!(
            has_id(&diags, "naming.property.requiredPrefix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn event_entity_detected() {
        let engine = engine_with("naming.event.requiredPrefix", "prefix = \"ev_\"");
        let diags = lint_file(
            &*engine,
            "classdef MyClass\n    events\n        ValueChanged\n    end\nend\n",
        );
        assert!(
            has_id(&diags, "naming.event.requiredPrefix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn enumeration_entity_detected() {
        let engine = engine_with("naming.enumeration.requiredPrefix", "prefix = \"ENUM_\"");
        let diags = lint_file(
            &*engine,
            "classdef MyClass\n    enumeration\n        Red\n        Green\n    end\nend\n",
        );
        assert!(
            has_id(&diags, "naming.enumeration.requiredPrefix"),
            "got: {diags:?}"
        );
    }
}
