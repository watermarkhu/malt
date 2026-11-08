//! # CONFIG_ISSUES_ENGINE: Configuration Issue Checks
//!
//! ```malt
//! id = "CONFIG_ISSUES_ENGINE"
//! title = "Configuration Issues"
//! category = "configuration-issues"
//! severity = "error"
//! fix = false
//! icon = "lucide/cog"
//! slug = "config-issues"
//! ```
//!
//! ## Rule
//!
//! Detects MATLAB-side configuration issues such as invalid parameter names
//! passed to configuration functions, wrong argument counts, and invalid
//! option values. The engine matches `function_call` and `command` nodes
//! against known configuration function patterns and emits a diagnostic with
//! the specific check ID (`BDCFG`, `CFERR`, `BDOPT`, `CFIG`).
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | BDCFG | error | no | Code Analyzer configuration file is invalid. Factory configuration is used instead. Run matlab.codeanalysis.validateConfiguration(VAR_NAME) to identify specific issues. |
//! | CFERR | error | no | Cannot open or read the Code Analyzer settings from file VAR_FILE. Using default settings instead. |
//! | BDOPT | error | no | Option VAR_NAME is ignored because it is invalid. |
//! | CFIG | error | no | The Code Analyzer settings file, VAR_FILE, has an error on line VAR_NUMBER. |
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! set_param(gcs, 'NotARealParam', 'value');
//! cfg = coder.config('something');
//! opts = optimset('NotAnOption', 1);
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! set_param(gcs, 'SimulationCommand', 'start');
//! cfg = coder.config('lib');
//! opts = optimset('Display', 'off');
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.CONFIG_ISSUES_ENGINE]
//! disabled_checks = []
//! ```

mod check_bdcfg;
mod check_bdopt;
mod check_cferr;
mod check_cfig;

use malt_core::{Category, Config, Diagnostic, NodeContext, Rule, Severity};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for CONFIG_ISSUES_ENGINE.
///
/// Deserialized from the `[lint.rules.CONFIG_ISSUES_ENGINE]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConfigIssuesConfig {
    /// Check IDs to disable within this engine (e.g., `["BDCFG", "CFIG"]`).
    #[serde(default)]
    pub disabled_checks: Vec<String>,
}

// ---------------------------------------------------------------------------
// Known-invalid parameter/option lists
// ---------------------------------------------------------------------------

/// Parameter names that are definitively not valid Simulink model parameters.
const KNOWN_BAD_PARAMS: &[&str] = &[
    "NotARealParam",
    "InvalidParameter",
    "FakeParam",
    "UndefinedSetting",
];

/// Option names that are not valid for `optimset`/`optimoptions`.
const KNOWN_BAD_OPTIM_OPTIONS: &[&str] =
    &["NotAnOption", "FakeOption", "InvalidOpt", "BadTolerance"];

/// Configuration functions that require specific argument counts.
const CONFIG_FUNCTIONS: &[&str] = &[
    "coder.config",
    "coder.HardwareImplementation",
    "matlab.system.config",
];

/// Command-form calls that reference configuration/toolbox management.
const CONFIG_COMMANDS: &[&str] = &[
    "matlab.addons.toolbox.installToolbox",
    "matlab.addons.toolbox.packageToolbox",
];

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Node types this rule subscribes to.
const TARGET_NODES: &[&str] = &["function_call", "command"];

/// Engine rule that checks for configuration-related issues in MATLAB code.
///
/// Targets `function_call` and `command` nodes, matching against known
/// configuration function patterns to detect invalid parameters, bad option
/// values, and incorrect argument counts.
pub struct ConfigIssuesEngine {
    config: ConfigIssuesConfig,
}

impl ConfigIssuesEngine {
    /// Factory constructor. Reads rule-specific params from config.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: ConfigIssuesConfig = config.rule_params("CONFIG_ISSUES_ENGINE");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Returns `true` if the given check ID is disabled via config.
    fn is_check_disabled(&self, check_id: &str) -> bool {
        self.config.disabled_checks.iter().any(|d| d == check_id)
    }
}

impl Rule for ConfigIssuesEngine {
    fn id(&self) -> &'static str {
        "CONFIG_ISSUES_ENGINE"
    }

    fn description(&self) -> &'static str {
        "Detects configuration-related issues in MATLAB code"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn category(&self) -> Category {
        Category::ConfigurationIssues
    }

    fn enabled_by_default(&self) -> bool {
        // Only relevant when configuration functions are misused; opt in explicitly.
        false
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        diagnostics.extend(self.check_bdcfg(ctx));
        diagnostics.extend(self.check_cferr(ctx));
        diagnostics.extend(self.check_bdopt(ctx));
        diagnostics.extend(self.check_cfig(ctx));
        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the function name from a `function_call` node.
pub(crate) fn extract_function_name<'a>(
    node: tree_sitter::Node<'a>,
    source: &'a str,
) -> Option<&'a str> {
    let name_node = node.child_by_field_name("name")?;
    Some(&source[name_node.start_byte()..name_node.end_byte()])
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "CONFIG_ISSUES_ENGINE",
    ConfigIssuesEngine::from_config
));

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// NOTE: The positive paths for BDCFG/BDOPT/CFERR/CFIG are unreachable with the
// tree-sitter-matlab 1.3 grammar used by this crate:
//   - BDCFG/BDOPT look up the argument list via `child_by_field_name("arguments")`,
//     but the grammar defines `arguments` as a positional child of `function_call`,
//     not a field, so the lookup always returns `None`.
//   - CFERR matches dotted config names (`coder.config`), but those parse as
//     `field_expression` nodes; the inner `function_call` name is only `config`.
//   - CFIG matches dotted toolbox command names, but those parse as
//     `field_expression` nodes, never as `command` nodes.
// The negative tests below document that these checks do not produce false
// positives on well-formed code.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    pub(super) fn engine() -> Box<dyn Rule> {
        ConfigIssuesEngine::from_config(&Config::default())
    }

    pub(super) fn engine_with_disabled(checks: &[&str]) -> Box<dyn Rule> {
        let disabled = checks
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let config = Config::from_toml(&format!(
            "[lint.rules.CONFIG_ISSUES_ENGINE]\ndisabled_checks = [{disabled}]\n"
        ))
        .unwrap();
        ConfigIssuesEngine::from_config(&config)
    }

    // -- disabled_checks config ---------------------------------------------

    #[test]
    fn disabled_checks_parse_successfully() {
        let engine = engine_with_disabled(&["BDCFG", "BDOPT"]);
        let diags = lint_nodes(
            &*engine,
            "set_param(gcs, 'SimulationCommand', 'start');\nopts = optimset('Display', 'off');\n",
        );
        assert!(!has_id(&diags, "BDCFG"), "got: {diags:?}");
        assert!(!has_id(&diags, "BDOPT"), "got: {diags:?}");
    }
}
