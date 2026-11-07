//! Configuration system for mlt.
//!
//! Supports `.mlt.toml` config files with the following schema:
//!
//! ```toml
//! [lint]
//! exclude = ["vendor/**", "test/fixtures/**"]
//!
//! [lint.rules]
//! M001 = "warn"           # shorthand: set severity
//! M002 = "off"            # disable a rule
//!
//! [lint.rules.M003]       # full table: severity + rule-specific params
//! severity = "error"
//! some_param = true
//! ```
//!
//! Rules access their typed parameters via [`Config::rule_params`], which
//! deserializes the rule's TOML table section into a `#[derive(Deserialize)]`
//! struct.

use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::diagnostic::Severity;
use crate::preset::{self, PresetDef};
use crate::rule::Category;

// ---------------------------------------------------------------------------
// Config errors
// ---------------------------------------------------------------------------

/// Errors that can occur during config parsing.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to parse config TOML: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("invalid severity value '{0}' (expected: error, warn, info, off)")]
    InvalidSeverity(String),

    #[error("unknown preset '{0}' (expected one of: {1})")]
    UnknownPreset(String, String),
}

// ---------------------------------------------------------------------------
// Raw TOML file representation (what serde deserializes directly)
// ---------------------------------------------------------------------------

/// The raw `.mlt.toml` file structure.
#[derive(Debug, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub lint: LintSection,
}

/// The `[lint]` section of the config file.
#[derive(Debug, Deserialize)]
pub struct LintSection {
    /// Glob patterns for files/directories to exclude from linting.
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Whether `%#ok<...>` inline suppression directives are honored.
    /// Defaults to `true` (mirrors MATLAB's inline suppression).
    #[serde(default = "default_true")]
    pub inline_suppression: bool,

    /// Named preset to base the configuration on (e.g. "all", "mathworks",
    /// "recommended"). Defaults to the built-in [`crate::preset::DEFAULT_PRESET`]
    /// when absent.
    #[serde(default)]
    pub preset: Option<String>,

    /// Per-rule configuration. Keys are rule IDs (e.g., "NOSEMI").
    /// Values are either a severity string or a full config table.
    #[serde(default)]
    pub rules: HashMap<String, RuleEntry>,

    /// Per-category configuration. Keys are category slugs (e.g., "performance").
    /// Values are severity strings ("off", "error", "warn", "info").
    #[serde(default)]
    pub categories: HashMap<String, String>,
}

impl Default for LintSection {
    fn default() -> Self {
        Self {
            exclude: Vec::new(),
            inline_suppression: true,
            preset: None,
            rules: HashMap::new(),
            categories: HashMap::new(),
        }
    }
}

/// A single rule's config entry — either a severity shorthand or a full table.
///
/// Supports both:
/// - `M001 = "warn"` (shorthand)
/// - `[lint.rules.M001] severity = "error" ...` (full table)
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum RuleEntry {
    /// Shorthand: just a severity string ("error", "warn", "info", "off").
    Severity(String),
    /// Full config table with optional severity + rule-specific parameters.
    Full(toml::Table),
}

// ---------------------------------------------------------------------------
// Normalized config (what rules and the engine consume)
// ---------------------------------------------------------------------------

/// Resolved per-rule configuration.
#[derive(Debug, Clone)]
pub struct RuleConfig {
    /// Whether the rule is enabled (false if set to "off").
    pub enabled: bool,
    /// Severity override (None = use rule's default).
    pub severity: Option<Severity>,
    /// Rule-specific parameters as a TOML table (deserialized by the rule).
    pub params: toml::Table,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: None,
            params: toml::Table::new(),
        }
    }
}

/// The resolved configuration passed to rule factories and the lint engine.
///
/// Constructed from a [`ConfigFile`] (parsed `.mlt.toml`) via [`Config::from_toml`],
/// or created with [`Config::default()`] for unconfigured usage.
#[derive(Debug, Clone)]
pub struct Config {
    /// Glob patterns for files/directories to exclude.
    pub exclude: Vec<String>,
    /// Per-rule configuration, keyed by rule ID.
    pub rules: HashMap<String, RuleConfig>,
    /// Per-category severity overrides, keyed by [`Category`].
    /// A value of `None` means the category is disabled ("off").
    pub categories: HashMap<Category, Option<Severity>>,
    /// The active configuration preset.
    pub preset: &'static PresetDef,
    /// Whether `%#ok<...>` inline suppression directives are honored.
    /// Defaults to `true`, mirroring MATLAB's inline suppression behavior.
    pub inline_suppression: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            exclude: Vec::new(),
            rules: HashMap::new(),
            categories: HashMap::new(),
            preset: preset::default_preset(),
            inline_suppression: true,
        }
    }
}

impl Config {
    /// Parse a `.mlt.toml` file's content into a resolved [`Config`].
    pub fn from_toml(content: &str) -> Result<Self, ConfigError> {
        let file: ConfigFile = toml::from_str(content)?;
        Self::from_config_file(file)
    }

    /// Convert a parsed [`ConfigFile`] into a resolved [`Config`].
    fn from_config_file(file: ConfigFile) -> Result<Self, ConfigError> {
        let mut rules = HashMap::new();

        for (rule_id, entry) in file.lint.rules {
            let rule_config = match entry {
                RuleEntry::Severity(ref s) => {
                    let (enabled, severity) = parse_severity_string(s)?;
                    RuleConfig {
                        enabled,
                        severity,
                        params: toml::Table::new(),
                    }
                }
                RuleEntry::Full(table) => {
                    // Extract "severity" key if present, pass the rest as params.
                    let severity = if let Some(val) = table.get("severity") {
                        let s = val
                            .as_str()
                            .ok_or_else(|| ConfigError::InvalidSeverity(val.to_string()))?;
                        let (enabled, sev) = parse_severity_string(s)?;
                        if !enabled {
                            // Rule is disabled — record it and continue to the next rule.
                            rules.insert(
                                rule_id,
                                RuleConfig {
                                    enabled: false,
                                    severity: None,
                                    params: toml::Table::new(),
                                },
                            );
                            continue;
                        }
                        sev
                    } else {
                        None
                    };

                    // All keys except "severity" are rule-specific params.
                    let params: toml::Table =
                        table.into_iter().filter(|(k, _)| k != "severity").collect();

                    RuleConfig {
                        enabled: true,
                        severity,
                        params,
                    }
                }
            };

            rules.insert(rule_id, rule_config);
        }

        // Parse category overrides.
        let mut categories = HashMap::new();
        for (cat_name, severity_str) in file.lint.categories {
            let category: Category = cat_name.parse().map_err(|_| {
                ConfigError::InvalidSeverity(format!("unknown category: {cat_name}"))
            })?;
            let (enabled, severity) = parse_severity_string(&severity_str)?;
            if enabled {
                categories.insert(category, severity);
            } else {
                // "off" means the category is disabled — store None.
                categories.insert(category, None);
            }
        }

        // Resolve the preset (defaults to DEFAULT_PRESET when not specified).
        let preset = match &file.lint.preset {
            Some(name) => preset::resolve(name).ok_or_else(|| {
                let names = preset::PRESETS
                    .iter()
                    .map(|p| p.name)
                    .collect::<Vec<_>>()
                    .join(", ");
                ConfigError::UnknownPreset(name.clone(), names)
            })?,
            None => preset::default_preset(),
        };

        Ok(Self {
            exclude: file.lint.exclude,
            rules,
            categories,
            preset,
            inline_suppression: file.lint.inline_suppression,
        })
    }

    /// Load and deserialize rule-specific parameters into a typed struct.
    ///
    /// If the rule has no config entry or the params table is empty, returns
    /// `T::default()`. If deserialization fails, logs a warning to stderr and
    /// returns `T::default()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[derive(Deserialize, Default)]
    /// struct M013Config {
    ///     max_line_length: Option<usize>,
    /// }
    ///
    /// let params: M013Config = config.rule_params("M013");
    /// ```
    pub fn rule_params<T: DeserializeOwned + Default>(&self, rule_id: &str) -> T {
        self.rules
            .get(rule_id)
            .and_then(|rc| {
                if rc.params.is_empty() {
                    return None;
                }
                let value = toml::Value::Table(rc.params.clone());
                match value.try_into::<T>() {
                    Ok(v) => Some(v),
                    Err(e) => {
                        eprintln!(
                            "mlt: warning: invalid config for rule {rule_id}: {e}; using defaults"
                        );
                        None
                    }
                }
            })
            .unwrap_or_default()
    }

    /// Check whether a rule is enabled in this config.
    ///
    /// Returns `true` if the rule has no config entry (rules are enabled by default).
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        self.rules.get(rule_id).map(|rc| rc.enabled).unwrap_or(true)
    }

    /// Check whether a rule is enabled, considering both per-rule and per-category config.
    ///
    /// Resolution order:
    /// 1. Per-rule config takes precedence (if rule is explicitly configured).
    /// 2. Per-category config applies if the rule has no explicit config.
    /// 3. The active preset combined with `default_enabled` (the rule's
    ///    built-in default); see [`PresetDef::default_enabled`].
    pub fn is_rule_enabled_for_category(
        &self,
        rule_id: &str,
        category: Category,
        default_enabled: bool,
    ) -> bool {
        // Per-rule override takes precedence.
        if let Some(rc) = self.rules.get(rule_id) {
            return rc.enabled;
        }
        // Check category-level config.
        if let Some(cat_severity) = self.categories.get(&category) {
            // None means category is disabled ("off").
            return cat_severity.is_some();
        }
        // Fall back to the preset applied to the rule's built-in default.
        self.preset.default_enabled(category, default_enabled)
    }

    /// Get the configured severity override for a rule, if any.
    pub fn rule_severity(&self, rule_id: &str) -> Option<Severity> {
        self.rules.get(rule_id).and_then(|rc| rc.severity)
    }

    /// Get the effective severity for a rule, considering category overrides.
    ///
    /// Resolution order:
    /// 1. Per-rule severity override.
    /// 2. Per-category severity override.
    /// 3. None (use rule's default).
    pub fn effective_severity(&self, rule_id: &str, category: Category) -> Option<Severity> {
        // Per-rule override takes precedence.
        if let Some(rc) = self.rules.get(rule_id) {
            if rc.severity.is_some() {
                return rc.severity;
            }
        }
        // Category-level override (flatten Option<Option<Severity>> → Option<Severity>).
        if let Some(cat_severity) = self.categories.get(&category) {
            return *cat_severity;
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Serde default for opt-out flags that default to enabled (`true`).
fn default_true() -> bool {
    true
}

/// Parse a severity string into (enabled, Option<Severity>).
///
/// "off" → (false, None)
/// "error" → (true, Some(Error))
/// "warn" → (true, Some(Warning))
/// "info" → (true, Some(Info))
fn parse_severity_string(s: &str) -> Result<(bool, Option<Severity>), ConfigError> {
    match s.to_lowercase().as_str() {
        "off" | "false" | "disabled" => Ok((false, None)),
        "error" | "err" => Ok((true, Some(Severity::Error))),
        "warn" | "warning" => Ok((true, Some(Severity::Warning))),
        "info" | "note" => Ok((true, Some(Severity::Info))),
        other => Err(ConfigError::InvalidSeverity(other.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let config = Config::from_toml("").unwrap();
        assert!(config.exclude.is_empty());
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_severity_shorthand() {
        let toml = r#"
[lint.rules]
M001 = "warn"
M002 = "error"
M003 = "off"
"#;
        let config = Config::from_toml(toml).unwrap();

        let m001 = &config.rules["M001"];
        assert!(m001.enabled);
        assert_eq!(m001.severity, Some(Severity::Warning));

        let m002 = &config.rules["M002"];
        assert!(m002.enabled);
        assert_eq!(m002.severity, Some(Severity::Error));

        let m003 = &config.rules["M003"];
        assert!(!m003.enabled);
    }

    #[test]
    fn test_full_table_config() {
        let toml = r#"
[lint.rules.M001]
severity = "error"
ignore_functions = ["disp", "fprintf"]
"#;
        let config = Config::from_toml(toml).unwrap();

        let m001 = &config.rules["M001"];
        assert!(m001.enabled);
        assert_eq!(m001.severity, Some(Severity::Error));
        assert!(m001.params.contains_key("ignore_functions"));
        assert!(!m001.params.contains_key("severity")); // severity stripped from params
    }

    #[test]
    fn test_rule_params_deserialization() {
        #[derive(Deserialize, Default, Debug, PartialEq)]
        struct TestParams {
            #[serde(default)]
            max_length: usize,
            #[serde(default)]
            enabled_checks: Vec<String>,
        }

        let toml = r#"
[lint.rules.M010]
severity = "warn"
max_length = 120
enabled_checks = ["foo", "bar"]
"#;
        let config = Config::from_toml(toml).unwrap();
        let params: TestParams = config.rule_params("M010");

        assert_eq!(params.max_length, 120);
        assert_eq!(params.enabled_checks, vec!["foo", "bar"]);
    }

    #[test]
    fn test_rule_params_missing_returns_default() {
        #[derive(Deserialize, Default, Debug, PartialEq)]
        struct TestParams {
            #[serde(default)]
            max_length: usize,
        }

        let config = Config::default();
        let params: TestParams = config.rule_params("M999");
        assert_eq!(params.max_length, 0);
    }

    #[test]
    fn test_exclude_patterns() {
        let toml = r#"
[lint]
exclude = ["vendor/**", "test/fixtures/**"]
"#;
        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.exclude, vec!["vendor/**", "test/fixtures/**"]);
    }

    #[test]
    fn test_inline_suppression_defaults_to_true() {
        // Empty config: field absent entirely.
        assert!(Config::default().inline_suppression);
        let config = Config::from_toml("").unwrap();
        assert!(config.inline_suppression);
        // [lint] section present but flag omitted.
        let config = Config::from_toml("[lint]\nexclude = []\n").unwrap();
        assert!(config.inline_suppression);
    }

    #[test]
    fn test_inline_suppression_can_be_disabled() {
        let config = Config::from_toml("[lint]\ninline_suppression = false\n").unwrap();
        assert!(!config.inline_suppression);
    }

    #[test]
    fn test_inline_suppression_explicit_true() {
        let config = Config::from_toml("[lint]\ninline_suppression = true\n").unwrap();
        assert!(config.inline_suppression);
    }

    #[test]
    fn test_invalid_severity() {
        let toml = r#"
[lint.rules]
M001 = "banana"
"#;
        let result = Config::from_toml(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("banana"));
    }

    #[test]
    fn test_full_table_off_does_not_drop_subsequent_rules() {
        // Regression test: a rule with severity = "off" in full-table form
        // must not prevent subsequent rules from being parsed.
        let toml = r#"
[lint.rules.M001]
severity = "off"

[lint.rules.M002]
severity = "error"

[lint.rules]
M003 = "warn"
"#;
        let config = Config::from_toml(toml).unwrap();

        let m001 = &config.rules["M001"];
        assert!(!m001.enabled);

        let m002 = &config.rules["M002"];
        assert!(m002.enabled);
        assert_eq!(m002.severity, Some(Severity::Error));

        let m003 = &config.rules["M003"];
        assert!(m003.enabled);
        assert_eq!(m003.severity, Some(Severity::Warning));
    }
}

#[cfg(test)]
mod preset_config_tests {
    use super::*;

    #[test]
    fn default_config_uses_mathworks_preset() {
        let config = Config::default();
        assert_eq!(config.preset.name, "mathworks");
    }

    #[test]
    fn empty_toml_uses_mathworks_preset() {
        let config = Config::from_toml("").unwrap();
        assert_eq!(config.preset.name, "mathworks");
    }

    #[test]
    fn explicit_preset_is_resolved() {
        let config = Config::from_toml("[lint]\npreset = \"recommended\"\n").unwrap();
        assert_eq!(config.preset.name, "recommended");
    }

    #[test]
    fn unknown_preset_errors() {
        let err = Config::from_toml("[lint]\npreset = \"bogus\"\n").unwrap_err();
        assert!(err.to_string().contains("bogus"));
        assert!(err.to_string().contains("all"));
        assert!(err.to_string().contains("mathworks"));
    }

    #[test]
    fn default_enabled_false_keeps_rule_off() {
        // A rule whose `enabled_by_default()` returns false stays off unless it
        // is explicitly enabled per-rule or per-category.
        let config = Config::from_toml("").unwrap();
        assert!(!config.is_rule_enabled_for_category(
            "CODEGEN_ENGINE",
            Category::CodeGeneration,
            false,
        ));
        assert!(config.is_rule_enabled_for_category(
            "CODEGEN_ENGINE",
            Category::CodeGeneration,
            true,
        ));
    }

    #[test]
    fn explicit_category_reenables_default_off_rule() {
        // A default-off rule can be re-enabled per category.
        let config = Config::from_toml("[lint.categories]\ncode-generation = \"warn\"\n").unwrap();
        assert!(config.is_rule_enabled_for_category(
            "CODEGEN_ENGINE",
            Category::CodeGeneration,
            false,
        ));
    }

    #[test]
    fn mathworks_preset_disables_naming_by_default() {
        let config = Config::from_toml("").unwrap();
        assert!(!config.is_rule_enabled_for_category("NAMING_ENGINE", Category::Naming, true));
        assert!(!config.is_rule_enabled_for_category(
            "CUSTOM_CHECKS",
            Category::CustomChecks,
            true,
        ));
        assert!(config.is_rule_enabled_for_category("BUGS_ENGINE", Category::Bugs, true));
    }

    #[test]
    fn explicit_category_reenables_preset_disabled() {
        let config = Config::from_toml(
            "[lint]\npreset = \"mathworks\"\n\n[lint.categories]\nnaming = \"warn\"\n",
        )
        .unwrap();
        assert!(config.is_rule_enabled_for_category("NAMING_ENGINE", Category::Naming, true));
    }

    #[test]
    fn all_preset_enables_everything() {
        let config = Config::from_toml("[lint]\npreset = \"all\"\n").unwrap();
        assert!(config.is_rule_enabled_for_category("NAMING_ENGINE", Category::Naming, true));
        assert!(config.is_rule_enabled_for_category(
            "CODEGEN_ENGINE",
            Category::CodeGeneration,
            false,
        ));
    }
}
