//! `malt_rules` — Lint rule implementations for malt.
//!
//! This crate houses all concrete lint rules. It exposes two entry points:
//! - [`all_rules`]: Constructs all available rules from config.
//! - [`active_rules`]: Constructs only enabled rules (respects config filtering).
//!
//! ## Adding a New Rule
//!
//! Rules self-register using the [`inventory`] crate. To add a new rule:
//!
//! 1. Create a new module (e.g., `nosemi.rs`).
//! 2. Implement the `Rule` trait with a `from_config` factory.
//! 3. Register the factory at the bottom of the module:
//!    ```ignore
//!    inventory::submit!(crate::RuleRegistration::new("NOSEMI", Nosemi::from_config));
//!    ```
//!
//! No manual edits to `lib.rs` are needed — the rule is automatically discovered
//! at link time via the `inventory` crate.

#[cfg(test)]
pub mod test_util;

pub mod analysis;
pub mod bugs;
pub mod codegen;
pub mod compatibility;
pub mod config_issues;
pub mod custom_checks;
pub mod deployment;
pub mod formatting;
pub mod good_practices;
pub mod incomplete_analysis;
pub mod language_spec;
pub mod naming;
pub mod nosemi;
pub mod performance;
pub mod readability;
pub mod suggested_improvements;
pub mod syntax_errors;
pub mod system_objects;
pub mod unset_variables;
pub mod unsupported;
pub mod unused;

use malt_core::{Config, Rule};

/// Type alias for a rule factory function.
pub type RuleFactory = fn(&Config) -> Box<dyn Rule>;

/// A self-registering rule entry collected via `inventory`.
///
/// Each rule module submits one of these at link time using
/// `inventory::submit!`.
pub struct RuleRegistration {
    /// The rule ID (e.g., "NOSEMI").
    pub id: &'static str,
    /// Factory function that constructs the rule from config.
    pub factory: RuleFactory,
}

impl RuleRegistration {
    /// Create a new registration entry.
    pub const fn new(id: &'static str, factory: RuleFactory) -> Self {
        Self { id, factory }
    }
}

inventory::collect!(RuleRegistration);

/// Construct all available lint rules from the given configuration.
///
/// Returns all rules regardless of whether they are enabled in config.
/// Use [`active_rules`] to get only enabled rules.
pub fn all_rules(config: &Config) -> Vec<Box<dyn Rule>> {
    inventory::iter::<RuleRegistration>
        .into_iter()
        .map(|reg| (reg.factory)(config))
        .collect()
}

/// Construct only the enabled lint rules from the given configuration.
///
/// Filters out rules that are set to "off" in the config (either per-rule
/// or per-category). This is the primary entry point used by the CLI.
pub fn active_rules(config: &Config) -> Vec<Box<dyn Rule>> {
    inventory::iter::<RuleRegistration>
        .into_iter()
        .filter_map(|reg| {
            // Construct the rule to check its category for category-level filtering.
            let rule = (reg.factory)(config);
            if config.is_rule_enabled_for_category(
                reg.id,
                rule.category(),
                rule.enabled_by_default(),
            ) {
                Some(rule)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_ids(config: &Config) -> Vec<String> {
        active_rules(config)
            .iter()
            .map(|r| r.id().to_string())
            .collect()
    }

    #[test]
    fn specialized_engines_are_disabled_by_default() {
        let ids = active_ids(&Config::default());
        for off in [
            "CODEGEN_ENGINE",
            "DEPLOYMENT_ENGINE",
            "SYSTEM_OBJECTS_ENGINE",
            "CONFIG_ISSUES_ENGINE",
            "SUGGESTED_IMPROVEMENTS",
            "UNSUPPORTED_ENGINE",
        ] {
            assert!(
                !ids.contains(&off.to_string()),
                "{off} should be off by default"
            );
        }
    }

    #[test]
    fn general_engines_are_enabled_by_default() {
        let ids = active_ids(&Config::default());
        for on in [
            "INCOMPLETE_ANALYSIS",
            "SYNTAX_ERRORS_ENGINE",
            "LANGUAGE_SPEC_ENGINE",
            "BUGS_ENGINE",
            "UNSET_VARIABLES_ENGINE",
            "UNUSED_ENGINE",
            "GOOD_PRACTICES_ENGINE",
            "READABILITY_ENGINE",
            "FORMATTING_ENGINE",
            "NOSEMI",
            "PERFORMANCE_ENGINE",
            "COMPAT",
        ] {
            assert!(
                ids.contains(&on.to_string()),
                "{on} should be on by default"
            );
        }
    }

    #[test]
    fn mathworks_preset_disables_naming_and_custom_checks() {
        let ids = active_ids(&Config::default());
        assert!(!ids.contains(&"NAMING_ENGINE".to_string()));
        assert!(!ids.contains(&"CUSTOM_CHECKS".to_string()));
    }

    #[test]
    fn all_preset_enables_every_engine() {
        let config = Config::from_toml("[lint]\npreset = \"all\"\n").unwrap();
        let active = active_ids(&config);
        for id in all_rules(&config).iter().map(|r| r.id()) {
            assert!(
                active.contains(&id.to_string()),
                "{id} should be on under 'all'"
            );
        }
    }

    #[test]
    fn recommended_preset_disables_formatting() {
        let config = Config::from_toml("[lint]\npreset = \"recommended\"\n").unwrap();
        let ids = active_ids(&config);
        assert!(!ids.contains(&"NOSEMI".to_string()));
        assert!(ids.contains(&"BUGS_ENGINE".to_string()));
    }

    #[test]
    fn disabled_by_default_engine_can_be_reenabled_per_rule() {
        let config =
            Config::from_toml("[lint.rules.CODEGEN_ENGINE]\nseverity = \"error\"\n").unwrap();
        assert!(active_ids(&config).contains(&"CODEGEN_ENGINE".to_string()));
    }

    #[test]
    fn disabled_by_default_engine_can_be_reenabled_per_category() {
        let config = Config::from_toml("[lint.categories]\ncode-generation = \"error\"\n").unwrap();
        assert!(active_ids(&config).contains(&"CODEGEN_ENGINE".to_string()));
    }

    #[test]
    fn enabled_by_default_engine_can_be_disabled_per_category() {
        let config = Config::from_toml("[lint.categories]\nperformance = \"off\"\n").unwrap();
        assert!(!active_ids(&config).contains(&"PERFORMANCE_ENGINE".to_string()));
    }
}
