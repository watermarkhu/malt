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
//! - Entities: class, function, localFunction, method, nestedFunction,
//!   property, event, enumeration, variable
//! - Check types: maxLength, minLength, regularExpression, requiredPrefix,
//!   disallowedPrefix, disallowedPhrase, requiredSuffix, disallowedSuffix,
//!   casing
//!
//! Rule IDs follow the pattern `naming.<entity>.<checkType>` (e.g.
//! `naming.class.casing`). The `NamingEngine` registers once with inventory
//! as `"NAMING_ENGINE"` and uses `has_file_check() = true`: it walks the tree
//! once to extract all named entities, then applies every enabled check to
//! each matching entity, emitting diagnostics with the specific rule ID.
//!
//! MATLAB's factory configuration has no naming limits, so no check reports
//! anything until it is configured: a check without its option is silent.
//! A variable is reported once per function (or script) at its first
//! occurrence there, as MATLAB does, and a regular expression must match the
//! whole name.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! myvar1 = 1;       % naming.variable.casing — style configured, e.g. lowerCamelCase
//! classdef myClass  % naming.class.casing — style configured, e.g. UpperCamelCase
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
//! style = "UpperCamelCase"
//!
//! [lint.rules."naming.function.maxLength"]
//! severity = "warn"
//! max = 32
//!
//! [lint.rules."naming.variable.disallowedPrefix"]
//! prefix = ["temp", "tmp"]
//! ```
//!
//! `style` takes MATLAB's names: `lowerCamelCase`, `UpperCamelCase`,
//! `lowercase_with_underscores`, `UPPERCASE_WITH_UNDERSCORES`, `lowercase` and
//! `UPPERCASE`. The older names `camelCase`, `PascalCase`, `snake_case` and
//! `UPPER_CASE` are accepted as aliases. `prefix`, `suffix` and `phrase` take a
//! string or an array of strings.

use std::collections::HashSet;
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
pub(crate) fn all_naming_combinations() -> Vec<(&'static str, NamingEntity, NamingCheckType)> {
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

/// One string, or an array of strings, as MATLAB accepts for affixes.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany {
    /// A single string.
    One(String),
    /// An array of strings: the check accepts any of them.
    Many(Vec<String>),
}

impl OneOrMany {
    /// The strings this value stands for.
    fn values(&self) -> Vec<&str> {
        match self {
            OneOrMany::One(value) => vec![value.as_str()],
            OneOrMany::Many(values) => values.iter().map(String::as_str).collect(),
        }
    }
}

/// Configuration for a single naming check, read from `.malt.toml`.
///
/// Each field applies to a specific check type:
/// - `max`: maxLength check threshold (inclusive); no check without it
/// - `min`: minLength check threshold (inclusive); no check without it
/// - `pattern`: regularExpression pattern, matched against the whole name
/// - `prefix`: requiredPrefix / disallowedPrefix value(s)
/// - `phrase`: disallowedPhrase value(s)
/// - `suffix`: requiredSuffix / disallowedSuffix value(s)
/// - `style`: casing style (see the module documentation)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct NamingCheckConfig {
    /// Maximum allowed name length (for maxLength checks).
    #[serde(default)]
    pub max: Option<usize>,
    /// Minimum required name length (for minLength checks).
    #[serde(default)]
    pub min: Option<usize>,
    /// Regular expression the whole name must match (for regularExpression checks).
    #[serde(default)]
    pub pattern: Option<String>,
    /// Required or disallowed prefix(es).
    #[serde(default)]
    pub prefix: Option<OneOrMany>,
    /// Disallowed phrase(s), matched as substrings.
    #[serde(default)]
    pub phrase: Option<OneOrMany>,
    /// Required or disallowed suffix(es).
    #[serde(default)]
    pub suffix: Option<OneOrMany>,
    /// Casing style (for casing checks).
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
    /// Compiled regex, anchored to the whole name.
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
                // Anchored, so that the pattern has to match the whole name.
                let compiled_regex = check_config
                    .pattern
                    .as_ref()
                    .and_then(|p| Regex::new(&format!("^(?:{p})$")).ok());
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

/// `true` for a script: the file's first statement (comments aside) is neither a
/// function nor a class definition.
fn is_script_file(root: tree_sitter::Node) -> bool {
    let mut cursor = root.walk();
    let first = root
        .named_children(&mut cursor)
        .find(|c| !matches!(c.kind(), "comment" | "line_continuation"));
    first.is_some_and(|c| !matches!(c.kind(), "function_definition" | "class_definition"))
}

/// Walk the tree-sitter tree and extract all named entities.
///
/// Determines entity type based on AST context:
/// - `class_definition` → Class
/// - `function_definition` at file level (first one) → Function
/// - `function_definition` at file level (subsequent) → LocalFunction
/// - `function_definition` inside `methods` block → Method (not for the
///   constructor, which is named after the class)
/// - `function_definition` inside another function → NestedFunction
/// - `property` node → Property (not inside an `arguments` block)
/// - node inside `events` block → Event
/// - `enum` node inside `enumeration` block → Enumeration
/// - variables → see [`collect_scope_variables`]
fn collect_named_entities<'a>(
    tree: &'a tree_sitter::Tree,
    source: &'a str,
) -> Vec<NamedEntity<'a>> {
    let mut entities = Vec::new();
    // In a script every function is a local function (probed: MATLAB gives
    // `localFunction` to a function after a script's statements), so the flag starts set.
    let mut first_function_seen = is_script_file(tree.root_node());

    collect_recursive(
        tree.root_node(),
        source,
        &mut entities,
        &mut first_function_seen,
        EntityContext::TopLevel,
    );
    collect_scope_variables(tree.root_node(), source, &mut entities);

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

/// The text of a node.
fn node_text<'a>(node: Node<'a>, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// The name of the class that encloses a node, if any.
fn enclosing_class_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    let mut current = node.parent();
    while let Some(ancestor) = current {
        if ancestor.kind() == "class_definition" {
            return ancestor
                .child_by_field_name("name")
                .map(|name| node_text(name, source));
        }
        current = ancestor.parent();
    }
    None
}

/// Recursively collect named entities from the AST (everything but variables).
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
                let name = node_text(name_node, source);
                let pos = name_node.start_position();
                // The constructor is named after its class: it is the class, not a method.
                let is_constructor = context == EntityContext::Methods
                    && enclosing_class_name(node, source) == Some(name);
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
                if !is_constructor {
                    entities.push(NamedEntity {
                        entity: entity_type,
                        name,
                        byte_range: name_node.start_byte()..name_node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                    });
                }
            }

            // Recurse into function body with Function context.
            for child in crate::analysis::symbols::children_of(node) {
                collect_recursive(
                    child,
                    source,
                    entities,
                    first_function_seen,
                    EntityContext::Function,
                );
            }
            return; // Don't recurse again below.
        }

        "class_definition" => {
            // Extract class name from the "name" field.
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, source);
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
            for child in crate::analysis::symbols::children_of(node) {
                collect_recursive(child, source, entities, first_function_seen, context);
            }
            return;
        }

        "methods" => {
            // Children are function_definitions → Method.
            for child in crate::analysis::symbols::children_of(node) {
                collect_recursive(
                    child,
                    source,
                    entities,
                    first_function_seen,
                    EntityContext::Methods,
                );
            }
            return;
        }

        "events" => {
            // Children are event identifiers.
            for child in crate::analysis::symbols::children_of(node) {
                collect_recursive(
                    child,
                    source,
                    entities,
                    first_function_seen,
                    EntityContext::Events,
                );
            }
            return;
        }

        "enumeration" => {
            // Children are enum nodes.
            for child in crate::analysis::symbols::children_of(node) {
                collect_recursive(
                    child,
                    source,
                    entities,
                    first_function_seen,
                    EntityContext::Enumeration,
                );
            }
            return;
        }

        "arguments_statement" => {
            // Names in an arguments block are the function's inputs and outputs:
            // they are variables, reported by the scope pass, not properties.
            return;
        }

        "property" => {
            // Extract property name (first identifier child or "name" field).
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = node_text(name_node, source);
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
                for child in crate::analysis::symbols::children_of(node) {
                    if child.kind() == "identifier" {
                        let name = node_text(child, source);
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
            return;
        }

        "enum" => {
            if context == EntityContext::Enumeration {
                // Extract enum member name (first identifier child or "name" field).
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = node_text(name_node, source);
                    let pos = name_node.start_position();
                    entities.push(NamedEntity {
                        entity: NamingEntity::Enumeration,
                        name,
                        byte_range: name_node.start_byte()..name_node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                    });
                } else {
                    for child in crate::analysis::symbols::children_of(node) {
                        if child.kind() == "identifier" {
                            let name = node_text(child, source);
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
            return;
        }

        "identifier" if context == EntityContext::Events => {
            // Identifier inside events block is an event name.
            let name = node_text(node, source);
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

        _ => {}
    }

    // Default: recurse into children with the same context.
    for child in crate::analysis::symbols::children_of(node) {
        collect_recursive(child, source, entities, first_function_seen, context);
    }
}

// ---------------------------------------------------------------------------
// Variable extraction
// ---------------------------------------------------------------------------

/// Report the variables of every scope: the top level of the file and each
/// function, including nested functions and methods.
///
/// A name is a variable of a scope when the scope declares it: an input or
/// output, an assignment target (including the base of `s.f = …` and `c{i} = …`),
/// a `for` loop variable, a `catch` identifier, a `global` or `persistent`
/// name, or a lambda parameter. MATLAB reports such a name once per scope, at
/// its first occurrence there, which may be a read before the assignment.
fn collect_scope_variables<'a>(
    root: Node<'a>,
    source: &'a str,
    entities: &mut Vec<NamedEntity<'a>>,
) {
    let mut scopes = vec![root];
    find_function_scopes(root, &mut scopes);
    for scope in scopes {
        variables_of_scope(scope, source, entities);
    }
}

/// Collect every function definition below `node`, nested ones included.
fn find_function_scopes<'a>(node: Node<'a>, scopes: &mut Vec<Node<'a>>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "function_definition" {
            scopes.push(child);
        }
        find_function_scopes(child, scopes);
    }
}

/// The variables of one scope, each at its first occurrence.
fn variables_of_scope<'a>(scope: Node<'a>, source: &'a str, entities: &mut Vec<NamedEntity<'a>>) {
    let mut occurrences = Vec::new();
    let mut declared = HashSet::new();
    walk_scope(scope, scope, source, &mut occurrences, &mut declared);

    let mut reported = HashSet::new();
    for node in occurrences {
        let name = node_text(node, source);
        if declared.contains(name) && reported.insert(name) {
            let pos = node.start_position();
            entities.push(NamedEntity {
                entity: NamingEntity::Variable,
                name,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
            });
        }
    }
}

/// Walk one scope in source order: record its identifier occurrences and the
/// names it declares. Nested functions and classes are skipped (they are scopes
/// of their own).
fn walk_scope<'a>(
    node: Node<'a>,
    scope: Node<'a>,
    source: &'a str,
    occurrences: &mut Vec<Node<'a>>,
    declared: &mut HashSet<&'a str>,
) {
    if matches!(node.kind(), "function_definition" | "class_definition") && node.id() != scope.id()
    {
        return;
    }

    match node.kind() {
        "identifier" => {
            // Field names (s.name) are not variables, nor is the scope's own name.
            let is_field = node.parent().is_some_and(|parent| {
                parent.kind() == "field_expression"
                    && parent
                        .child_by_field_name("field")
                        .is_some_and(|field| field.id() == node.id())
            });
            let is_scope_name = scope.kind() == "function_definition"
                && scope
                    .child_by_field_name("name")
                    .is_some_and(|name| name.id() == node.id());
            if !is_field && !is_scope_name {
                occurrences.push(node);
            }
            return;
        }
        "assignment" => {
            if let Some(left) = node.child_by_field_name("left") {
                declare_target(left, source, declared);
            }
        }
        "function_arguments"
        | "function_output"
        | "global_operator"
        | "persistent_operator"
        | "catch_clause" => {
            declare_identifiers(node, source, declared);
        }
        "lambda" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "arguments" {
                    declare_identifiers(child, source, declared);
                }
            }
        }
        "for_statement" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "iterator" {
                    let mut inner = child.walk();
                    let variable = child
                        .children(&mut inner)
                        .find(|grandchild| grandchild.kind() == "identifier");
                    if let Some(variable) = variable {
                        declared.insert(node_text(variable, source));
                    }
                }
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_scope(child, scope, source, occurrences, declared);
    }
}

/// Declare the identifiers that are direct children of `node`, and those in its
/// multi-output groups (`[a, b]`).
fn declare_identifiers<'a>(node: Node<'a>, source: &'a str, declared: &mut HashSet<&'a str>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                declared.insert(node_text(child, source));
            }
            "multioutput_variable" => declare_identifiers(child, source, declared),
            _ => {}
        }
    }
}

/// Declare the variable assigned by an assignment target: a name, an entry of a
/// multi-output group, or the base of an indexed or field target.
fn declare_target<'a>(target: Node<'a>, source: &'a str, declared: &mut HashSet<&'a str>) {
    match target.kind() {
        "identifier" => {
            declared.insert(node_text(target, source));
        }
        "multioutput_variable" => declare_identifiers(target, source, declared),
        "field_expression" => {
            if let Some(object) = target.child_by_field_name("object") {
                declare_target(object, source, declared);
            }
        }
        "function_call" => {
            if let Some(name) = target.child_by_field_name("name") {
                declare_target(name, source, declared);
            }
        }
        _ => {}
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

/// Check if name is longer than the configured maximum (inclusive).
fn check_max_length(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let max = config.max?;
    let length = name.chars().count();
    if length > max {
        Some(format!(
            "Name '{name}' should contain less than or equal to {max} character(s)"
        ))
    } else {
        None
    }
}

/// Check if name is shorter than the configured minimum (inclusive).
fn check_min_length(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let min = config.min?;
    let length = name.chars().count();
    if length < min {
        Some(format!(
            "Name '{name}' should contain more than or equal to {min} character(s)"
        ))
    } else {
        None
    }
}

/// Check if name matches the configured regular expression, over the whole name.
fn check_regex(name: &str, check: &NamingRuleInstance) -> Option<String> {
    let re = check.compiled_regex.as_ref()?;
    if !re.is_match(name) {
        Some(format!(
            "Name '{}' should conform to the regular expression: '{}'",
            name,
            check.config.pattern.as_deref().unwrap_or("")
        ))
    } else {
        None
    }
}

/// Check if name starts with one of the required prefixes (case-sensitive).
fn check_required_prefix(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let prefixes = config.prefix.as_ref()?.values();
    if prefixes
        .iter()
        .copied()
        .any(|prefix| name.starts_with(prefix))
    {
        None
    } else {
        Some(format!(
            "Name '{}' should start with one of the required prefix(es): {}",
            name,
            prefixes.join(", ")
        ))
    }
}

/// Check if name starts with a disallowed prefix (case-sensitive).
fn check_disallowed_prefix(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let prefixes = config.prefix.as_ref()?.values();
    if prefixes
        .iter()
        .copied()
        .any(|prefix| name.starts_with(prefix))
    {
        Some(format!(
            "Name '{}' should not start with any of the disallowed prefix(es): {}",
            name,
            prefixes.join(", ")
        ))
    } else {
        None
    }
}

/// Check if name contains a disallowed phrase (case-sensitive).
fn check_disallowed_phrase(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let phrases = config.phrase.as_ref()?.values();
    if phrases.iter().copied().any(|phrase| name.contains(phrase)) {
        Some(format!(
            "Name '{}' should not contain any of the disallowed phrase(s): {}",
            name,
            phrases.join(", ")
        ))
    } else {
        None
    }
}

/// Check if name ends with one of the required suffixes (case-sensitive).
fn check_required_suffix(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let suffixes = config.suffix.as_ref()?.values();
    if suffixes
        .iter()
        .copied()
        .any(|suffix| name.ends_with(suffix))
    {
        None
    } else {
        Some(format!(
            "Name '{}' should end with one of the required suffix(es): {}",
            name,
            suffixes.join(", ")
        ))
    }
}

/// Check if name ends with a disallowed suffix (case-sensitive).
fn check_disallowed_suffix(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let suffixes = config.suffix.as_ref()?.values();
    if suffixes
        .iter()
        .copied()
        .any(|suffix| name.ends_with(suffix))
    {
        Some(format!(
            "Name '{}' should not end with any of the disallowed suffix(es): {}",
            name,
            suffixes.join(", ")
        ))
    } else {
        None
    }
}

/// Check if name follows the configured casing style.
fn check_casing(name: &str, config: &NamingCheckConfig) -> Option<String> {
    let style = config.style.as_deref()?;
    let casing = parse_casing(style)?;
    if follows_casing(name, casing) {
        None
    } else {
        Some(format!("Name '{name}' does not follow the {style} casing"))
    }
}

/// A casing style that a name can be checked against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Casing {
    LowerCamel,
    UpperCamel,
    LowerWithUnderscores,
    UpperWithUnderscores,
    Lower,
    Upper,
}

/// The casing style for a `style` value: MATLAB's names, or an older alias.
fn parse_casing(style: &str) -> Option<Casing> {
    match style {
        "lowerCamelCase" | "camelCase" => Some(Casing::LowerCamel),
        "UpperCamelCase" | "PascalCase" => Some(Casing::UpperCamel),
        "lowercase_with_underscores" | "snake_case" => Some(Casing::LowerWithUnderscores),
        "UPPERCASE_WITH_UNDERSCORES" | "UPPER_CASE" | "CONSTANT_CASE" => {
            Some(Casing::UpperWithUnderscores)
        }
        "lowercase" => Some(Casing::Lower),
        "UPPERCASE" => Some(Casing::Upper),
        _ => None,
    }
}

/// Whether a name follows a casing style, as MATLAB defines them: letters and
/// digits only, with underscores allowed only where the style says so.
fn follows_casing(name: &str, casing: Casing) -> bool {
    let first = name.chars().next();
    let starts_lower = first.is_some_and(|c| c.is_ascii_lowercase());
    let starts_upper = first.is_some_and(|c| c.is_ascii_uppercase());
    let has_lower = name.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = name.chars().any(|c| c.is_ascii_uppercase());
    let has_underscore = name.contains('_');
    match casing {
        Casing::LowerCamel => starts_lower && !has_underscore,
        Casing::UpperCamel => starts_upper && !has_underscore,
        Casing::LowerWithUnderscores => !has_upper,
        Casing::UpperWithUnderscores => !has_lower,
        Casing::Lower => !has_upper && !has_underscore,
        Casing::Upper => !has_lower && !has_underscore,
    }
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

    fn count(diags: &[Diagnostic], rule_id: &str) -> usize {
        diags.iter().filter(|d| d.rule_id == rule_id).count()
    }

    // -- No configuration: MATLAB's factory setting reports nothing ----------

    #[test]
    fn unconfigured_engine_reports_no_naming_check() {
        let source = "a = 1;\naVeryLongVariableNameThatIsLong = 2;\nfunction f()\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(
            diags.iter().all(|d| !d.rule_id.starts_with("naming.")),
            "got: {diags:?}"
        );
    }

    // -- maxLength / minLength (inclusive bounds) -----------------------------

    #[test]
    fn max_length_accepts_name_at_the_limit() {
        let engine = engine_with("naming.variable.maxLength", "max = 5");
        let diags = lint_file(&*engine, "abcde = 1;\n");
        assert!(
            !has_id(&diags, "naming.variable.maxLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn max_length_flags_name_over_the_limit() {
        let engine = engine_with("naming.variable.maxLength", "max = 5");
        let diags = lint_file(&*engine, "abcdef = 1;\n");
        assert!(
            has_id(&diags, "naming.variable.maxLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn min_length_accepts_name_at_the_limit() {
        let engine = engine_with("naming.variable.minLength", "min = 2");
        let diags = lint_file(&*engine, "ab = 1;\n");
        assert!(
            !has_id(&diags, "naming.variable.minLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn min_length_flags_name_under_the_limit() {
        let engine = engine_with("naming.variable.minLength", "min = 2");
        let diags = lint_file(&*engine, "a = 1;\n");
        assert!(
            has_id(&diags, "naming.variable.minLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn function_max_length_flags_long_function_name() {
        let engine = engine_with("naming.function.maxLength", "max = 4");
        let diags = lint_file(&*engine, "function longName()\nend\n");
        assert!(
            has_id(&diags, "naming.function.maxLength"),
            "got: {diags:?}"
        );
    }

    // -- regularExpression: the whole name must match -------------------------

    #[test]
    fn regex_must_match_the_whole_name() {
        let engine = engine_with("naming.variable.regularExpression", "pattern = \"a\"");
        let diags = lint_file(&*engine, "abc = 1;\n");
        assert!(
            has_id(&diags, "naming.variable.regularExpression"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn regex_accepts_a_name_matching_the_whole_pattern() {
        let engine = engine_with("naming.variable.regularExpression", "pattern = \"a\"");
        let diags = lint_file(&*engine, "a = 1;\n");
        assert!(
            !has_id(&diags, "naming.variable.regularExpression"),
            "got: {diags:?}"
        );
    }

    // -- prefix / suffix / phrase: case-sensitive, one of a list ---------------

    #[test]
    fn required_prefix_accepts_any_listed_prefix() {
        let engine = engine_with(
            "naming.variable.requiredPrefix",
            "prefix = [\"ab\", \"cd\"]",
        );
        let ok = lint_file(&*engine, "cdx = 1;\n");
        let bad = lint_file(&*engine, "xyz = 1;\n");
        assert!(
            !has_id(&ok, "naming.variable.requiredPrefix"),
            "got: {ok:?}"
        );
        assert!(
            has_id(&bad, "naming.variable.requiredPrefix"),
            "got: {bad:?}"
        );
    }

    #[test]
    fn required_prefix_is_case_sensitive() {
        let engine = engine_with("naming.variable.requiredPrefix", "prefix = \"Ab\"");
        let diags = lint_file(&*engine, "ab1 = 1;\n");
        assert!(
            has_id(&diags, "naming.variable.requiredPrefix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn disallowed_prefix_is_case_sensitive() {
        let engine = engine_with("naming.variable.disallowedPrefix", "prefix = \"my\"");
        let flagged = lint_file(&*engine, "myVar = 1;\n");
        let allowed = lint_file(&*engine, "MyVar = 1;\n");
        assert!(
            has_id(&flagged, "naming.variable.disallowedPrefix"),
            "got: {flagged:?}"
        );
        assert!(
            !has_id(&allowed, "naming.variable.disallowedPrefix"),
            "got: {allowed:?}"
        );
    }

    #[test]
    fn disallowed_prefix_accepts_a_list() {
        let engine = engine_with(
            "naming.variable.disallowedPrefix",
            "prefix = [\"zz\", \"x\"]",
        );
        let flagged = lint_file(&*engine, "xyz = 1;\n");
        let allowed = lint_file(&*engine, "abc = 1;\n");
        assert!(
            has_id(&flagged, "naming.variable.disallowedPrefix"),
            "got: {flagged:?}"
        );
        assert!(
            !has_id(&allowed, "naming.variable.disallowedPrefix"),
            "got: {allowed:?}"
        );
    }

    #[test]
    fn disallowed_phrase_is_case_sensitive() {
        let engine = engine_with("naming.variable.disallowedPhrase", "phrase = \"ab\"");
        let flagged = lint_file(&*engine, "xab = 1;\n");
        let allowed = lint_file(&*engine, "xAB = 1;\n");
        assert!(
            has_id(&flagged, "naming.variable.disallowedPhrase"),
            "got: {flagged:?}"
        );
        assert!(
            !has_id(&allowed, "naming.variable.disallowedPhrase"),
            "got: {allowed:?}"
        );
    }

    #[test]
    fn required_suffix_is_case_sensitive() {
        let engine = engine_with("naming.variable.requiredSuffix", "suffix = \"ar\"");
        let ok = lint_file(&*engine, "myVar = 1;\n");
        let bad = lint_file(&*engine, "myVAR = 1;\n");
        assert!(
            !has_id(&ok, "naming.variable.requiredSuffix"),
            "got: {ok:?}"
        );
        assert!(
            has_id(&bad, "naming.variable.requiredSuffix"),
            "got: {bad:?}"
        );
    }

    #[test]
    fn disallowed_suffix_is_case_sensitive() {
        let engine = engine_with("naming.variable.disallowedSuffix", "suffix = \"ar\"");
        let flagged = lint_file(&*engine, "myVar = 1;\n");
        let allowed = lint_file(&*engine, "myVAR = 1;\n");
        assert!(
            has_id(&flagged, "naming.variable.disallowedSuffix"),
            "got: {flagged:?}"
        );
        assert!(
            !has_id(&allowed, "naming.variable.disallowedSuffix"),
            "got: {allowed:?}"
        );
    }

    // -- casing: MATLAB's six styles ------------------------------------------

    #[test]
    fn casing_styles_match_matlab() {
        let cases = [
            ("lowerCamelCase", "myVar", false),
            ("lowerCamelCase", "x1", false),
            ("lowerCamelCase", "MyVar", true),
            ("lowerCamelCase", "my_var", true),
            ("UpperCamelCase", "MyVar", false),
            ("UpperCamelCase", "V", false),
            ("UpperCamelCase", "myVar", true),
            ("UpperCamelCase", "MY_VAR", true),
            ("lowercase_with_underscores", "my_var2", false),
            ("lowercase_with_underscores", "my2Var", true),
            ("UPPERCASE_WITH_UNDERSCORES", "MY_VAR", false),
            ("UPPERCASE_WITH_UNDERSCORES", "x1", true),
            ("lowercase", "my2var", false),
            ("lowercase", "my_var", true),
            ("UPPERCASE", "MYVAR", false),
            ("UPPERCASE", "MY_VAR", true),
        ];
        for (style, name, violation) in cases {
            let engine = engine_with("naming.variable.casing", &format!("style = \"{style}\""));
            let diags = lint_file(&*engine, &format!("{name} = 1;\n"));
            assert_eq!(
                has_id(&diags, "naming.variable.casing"),
                violation,
                "{name} with {style}: {diags:?}"
            );
        }
    }

    #[test]
    fn older_casing_names_are_aliases() {
        let engine = engine_with("naming.variable.casing", "style = \"PascalCase\"");
        let diags = lint_file(&*engine, "myVar = 1;\n");
        assert!(has_id(&diags, "naming.variable.casing"), "got: {diags:?}");
    }

    // -- variables: one report per scope, at the first occurrence -------------

    #[test]
    fn variable_is_reported_once_per_scope_at_its_first_occurrence() {
        let engine = engine_with("naming.variable.maxLength", "max = 2");
        let source = "function out = fn(in_arg)\nout = 1;\nout = 2;\nend\n";
        let diags = lint_file(&*engine, source);
        assert_eq!(
            count(&diags, "naming.variable.maxLength"),
            2,
            "got: {diags:?}"
        );
        assert!(
            diags.iter().any(|d| d.line == 1 && d.column == 10),
            "got: {diags:?}"
        );
    }

    #[test]
    fn variable_read_before_assignment_is_reported_at_the_read() {
        let engine = engine_with("naming.variable.maxLength", "max = 2");
        let diags = lint_file(&*engine, "function f()\ndisp(zzz)\nzzz = 1;\nend\n");
        let zzz: Vec<_> = diags
            .iter()
            .filter(|d| d.rule_id == "naming.variable.maxLength")
            .collect();
        assert_eq!(zzz.len(), 1, "got: {diags:?}");
        assert_eq!((zzz[0].line, zzz[0].column), (2, 6), "got: {diags:?}");
    }

    #[test]
    fn for_loop_variable_is_a_variable() {
        let engine = engine_with("naming.variable.maxLength", "max = 3");
        let diags = lint_file(&*engine, "for loopVar = 1:2\nend\n");
        assert!(
            has_id(&diags, "naming.variable.maxLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn catch_identifier_is_a_variable() {
        let engine = engine_with("naming.variable.maxLength", "max = 3");
        let diags = lint_file(&*engine, "try\nerror('x')\ncatch errObj\nend\n");
        assert!(
            has_id(&diags, "naming.variable.maxLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn function_after_script_statements_is_a_local_function() {
        // MATLAB probe (casing config): `Bad_helper` after a script's statements is
        // reported as naming.localFunction.casing, in a function file too after the main one.
        let script = "x = 1;\ndisp(x)\nBad_helper();\nfunction Bad_helper()\ndisp(1)\nend\n";
        let local = engine_with("naming.localFunction.casing", "style = \"lowerCamelCase\"");
        assert!(has_id(
            &lint_file(&*local, script),
            "naming.localFunction.casing"
        ));
        let function = engine_with("naming.function.casing", "style = \"lowerCamelCase\"");
        assert!(!has_id(
            &lint_file(&*function, script),
            "naming.function.casing"
        ));

        let file = "function mainF()\ndisp(1)\nend\nfunction Bad_helper()\ndisp(2)\nend\n";
        assert!(has_id(
            &lint_file(&*local, file),
            "naming.localFunction.casing"
        ));
        assert!(!has_id(
            &lint_file(&*function, file),
            "naming.function.casing"
        ));
    }

    #[test]
    fn required_prefix_array_is_any_of() {
        // MATLAB probe (requiredPrefix ["ab", "xy"]): `abVal` and `xyVal` pass, `zzVal` is
        // reported. The suffix array ["Max", "Min"] works the same way (valMax passes).
        let source = "function p()\nabVal = 1;\nxyVal = 2;\nzzVal = 3;\nvalMax = 4;\n\
                      disp(abVal + xyVal + zzVal + valMax)\nend\n";
        let engine = engine_with(
            "naming.variable.requiredPrefix",
            "prefix = [\"ab\", \"xy\"]",
        );
        let diags = lint_file(&*engine, source);
        let reported: Vec<&str> = diags
            .iter()
            .filter(|d| d.rule_id == "naming.variable.requiredPrefix")
            .map(|d| d.message.as_str())
            .collect();
        assert_eq!(reported.len(), 2, "got: {diags:?}");
        assert!(reported[0].contains("'zzVal'") && reported[1].contains("'valMax'"));

        let engine = engine_with(
            "naming.variable.requiredSuffix",
            "suffix = [\"Max\", \"Min\"]",
        );
        let diags = lint_file(&*engine, source);
        assert!(
            diags.iter().all(|d| !d.message.contains("'valMax'")),
            "got: {diags:?}"
        );
        assert!(
            diags.iter().any(|d| d.message.contains("'abVal'")),
            "got: {diags:?}"
        );
    }

    #[test]
    fn global_declaration_is_a_variable() {
        let engine = engine_with("naming.variable.maxLength", "max = 3");
        let diags = lint_file(&*engine, "function f()\nglobal gVar\nend\n");
        assert!(
            has_id(&diags, "naming.variable.maxLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn lambda_parameter_is_a_variable() {
        let engine = engine_with("naming.variable.maxLength", "max = 3");
        let diags = lint_file(&*engine, "f = @(lamArg) lamArg + 1;\n");
        assert!(
            has_id(&diags, "naming.variable.maxLength"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn struct_field_is_not_a_variable() {
        let engine = engine_with("naming.variable.minLength", "min = 2");
        let diags = lint_file(&*engine, "s.fieldName = 1;\n");
        assert_eq!(
            count(&diags, "naming.variable.minLength"),
            1,
            "got: {diags:?}"
        );
    }

    #[test]
    fn arguments_block_names_are_variables_not_properties() {
        let engine = engine_with("naming.property.requiredPrefix", "prefix = \"p_\"");
        let source = "function r = f(a)\narguments\n  a (1,1) double\nend\nr = a;\nend\n";
        let diags = lint_file(&*engine, source);
        assert!(
            !has_id(&diags, "naming.property.requiredPrefix"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn constructor_is_not_a_method() {
        let engine = engine_with("naming.method.maxLength", "max = 3");
        let source = "classdef Thing < handle\n  methods\n    function obj = Thing()\n    end\n    function r = longName(obj)\n      r = obj;\n    end\n  end\nend\n";
        let diags = lint_file(&*engine, source);
        let methods: Vec<_> = diags
            .iter()
            .filter(|d| d.rule_id == "naming.method.maxLength")
            .collect();
        assert_eq!(methods.len(), 1, "got: {diags:?}");
        assert!(methods[0].message.contains("longName"), "got: {diags:?}");
    }
}
