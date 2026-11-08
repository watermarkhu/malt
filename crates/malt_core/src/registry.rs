use std::collections::HashMap;

use crate::config::Config;
use crate::diagnostic::Severity;
use crate::rule::Rule;

// ---------------------------------------------------------------------------
// RuleRegistry
// ---------------------------------------------------------------------------

/// A registry that holds all active lint rules and provides O(1) lookup
/// of which rules are interested in a given tree-sitter node type.
///
/// Built once at startup from the list of rules provided by `malt_rules`.
/// The registry owns the rules, indexes them by their declared
/// `target_node_types()`, and stores resolved effective severity per rule
/// (config override if present, else rule's default).
pub struct RuleRegistry {
    /// All registered rules, owned.
    rules: Vec<Box<dyn Rule>>,
    /// Effective severity per rule (index-aligned with `rules`).
    /// Resolved at construction: config override takes precedence over rule default.
    severities: Vec<Severity>,
    /// Maps tree-sitter node type string → indices into `rules`.
    /// Multiple rules may subscribe to the same node type.
    node_type_index: HashMap<&'static str, Vec<usize>>,
    /// Indices of rules that implement file-level checking.
    file_check_indices: Vec<usize>,
}

impl RuleRegistry {
    /// Build a new registry from a list of rules and the resolved config.
    ///
    /// Constructs the internal index mapping each declared node type to the
    /// rules that subscribe to it. Resolves effective severity per rule:
    /// config override (per-rule or per-category) takes precedence over rule default.
    pub fn new(rules: Vec<Box<dyn Rule>>, config: &Config) -> Self {
        let mut node_type_index: HashMap<&'static str, Vec<usize>> = HashMap::new();
        let mut severities = Vec::with_capacity(rules.len());
        let mut file_check_indices = Vec::new();

        for (idx, rule) in rules.iter().enumerate() {
            // Resolve effective severity: per-rule > per-category > rule default.
            let effective_severity = config
                .effective_severity(rule.id(), rule.category())
                .unwrap_or_else(|| rule.severity());
            severities.push(effective_severity);

            for &node_type in rule.target_node_types() {
                node_type_index.entry(node_type).or_default().push(idx);
            }

            if rule.has_file_check() {
                file_check_indices.push(idx);
            }
        }

        Self {
            rules,
            severities,
            node_type_index,
            file_check_indices,
        }
    }

    /// Get indices of rules interested in the given node type.
    ///
    /// Returns an empty slice if no rules subscribe to this node type,
    /// enabling a fast skip during traversal.
    #[inline]
    pub fn rules_for_node_type(&self, node_type: &str) -> &[usize] {
        self.node_type_index
            .get(node_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get indices of rules that implement file-level checking.
    #[inline]
    pub fn file_check_rules(&self) -> &[usize] {
        &self.file_check_indices
    }

    /// Get a rule by its index.
    #[inline]
    pub fn get_rule(&self, index: usize) -> &dyn Rule {
        &*self.rules[index]
    }

    /// Get the effective severity for a rule by its index.
    ///
    /// This is the config-resolved severity (override if present, else rule default).
    #[inline]
    pub fn effective_severity(&self, index: usize) -> Severity {
        self.severities[index]
    }

    /// Iterate over all registered rules.
    pub fn all_rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    /// Iterate over all rules with their indices (for severity lookup).
    pub fn rules_with_indices(&self) -> impl Iterator<Item = (usize, &Box<dyn Rule>)> {
        self.rules.iter().enumerate()
    }

    /// Number of registered rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Whether the registry contains no rules.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}
