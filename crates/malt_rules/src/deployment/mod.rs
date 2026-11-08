//! # DEPLOYMENT_ENGINE: Deployment Constraint Checks
//!
//! ```malt
//! id = "DEPLOYMENT_ENGINE"
//! title = "MATLAB Compiler Deployment Constraint Checks"
//! category = "deployment"
//! severity = "warning"
//! fix = false
//! icon = "lucide/package"
//! slug = "deployment"
//! ```
//!
//! ## Rule
//!
//! Detects MATLAB constructs that are problematic in compiled, deployed
//! applications built with `mcc`. Deployed MATLAB applications run in a
//! restricted MCR environment where certain functions are unavailable or
//! behave differently. All 10 checks share a single `DeploymentEngine` that
//! dispatches node-level checks on `function_call` and `command` nodes; each
//! diagnostic carries the specific check ID (e.g. `MCCD`, `MCTBX`) and its
//! own severity.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | MCCD | error | no | MCC use of the CD function is problematic. |
//! | MCPRD | error | no | MCC allows only one argument in the PRINTDLG function. |
//! | MCHLP | warning | no | MCC does not permit the HELP function. |
//! | MCKBD | warning | no | MCC does not permit the KEYBOARD function. |
//! | MCSVP | warning | no | MCC does not permit the SAVEPATH function. |
//! | MCMLR | warning | no | MCC use of the MATLABROOT function is problematic. |
//! | MCABF | error | no | MCC use of absolute file names is likely to fail. |
//! | MCMFL | warning | no | MCC allows writing .m files, but they cannot be executed by the deployed application. |
//! | MCTBX | warning | no | MCC use of toolbox folder file names is likely to fail. |
//! | MCLL | error | no | MCC does not allow C++ files to be read directly using LOADLIBRARY. |
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! cd /tmp;              % MCCD
//! addpath('/abs/path'); % MCABF
//! doc plot;             % MCHLP
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! % Bundle resources with the deployed app and use relative paths.
//! result = processFile('data.bin');
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.DEPLOYMENT_ENGINE]
//! severity = "warning"
//! skip_checks = ["MCTBX"]
//! ```

mod check_command;
mod check_function_call;

use malt_core::{Category, Config, Diagnostic, NodeContext, Rule, Severity};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for the deployment engine.
///
/// Deserialized from the `[lint.rules.DEPLOYMENT_ENGINE]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DeploymentEngineConfig {
    /// Check IDs to skip (e.g., `["MCTBX"]`).
    #[serde(default)]
    pub skip_checks: Vec<String>,
}

// ---------------------------------------------------------------------------
// Check metadata
// ---------------------------------------------------------------------------

/// Static metadata for a single deployment check.
struct CheckMeta {
    id: &'static str,
    severity: Severity,
    description: &'static str,
}

/// All 10 deployment check definitions.
const CHECKS: &[CheckMeta] = &[
    CheckMeta {
        id: "MCCD",
        severity: Severity::Error,
        description: "MCC use of the CD function is problematic.",
    },
    CheckMeta {
        id: "MCPRD",
        severity: Severity::Error,
        description: "MCC allows only one argument in the PRINTDLG function.",
    },
    CheckMeta {
        id: "MCHLP",
        severity: Severity::Warning,
        description: "MCC does not permit the HELP function.",
    },
    CheckMeta {
        id: "MCKBD",
        severity: Severity::Warning,
        description: "MCC does not permit the KEYBOARD function.",
    },
    CheckMeta {
        id: "MCSVP",
        severity: Severity::Warning,
        description: "MCC does not permit the SAVEPATH function.",
    },
    CheckMeta {
        id: "MCMLR",
        severity: Severity::Warning,
        description: "MCC use of the MATLABROOT function is problematic.",
    },
    CheckMeta {
        id: "MCABF",
        severity: Severity::Error,
        description: "MCC use of absolute file names is likely to fail.",
    },
    CheckMeta {
        id: "MCMFL",
        severity: Severity::Warning,
        description:
            "MCC allows writing .m files, but they cannot be executed by the deployed application.",
    },
    CheckMeta {
        id: "MCTBX",
        severity: Severity::Warning,
        description: "MCC use of toolbox folder file names is likely to fail.",
    },
    CheckMeta {
        id: "MCLL",
        severity: Severity::Error,
        description: "MCC does not allow C++ files to be read directly using LOADLIBRARY.",
    },
];

/// Common toolbox-specific functions that require special compilation support.
const TOOLBOX_FUNCTIONS: &[&str] = &[
    // Signal Processing Toolbox
    "fft2",
    "ifft2",
    "fftshift",
    "butter",
    "cheby1",
    "filter",
    "filtfilt",
    "spectrogram",
    "pwelch",
    // Image Processing Toolbox
    "imread",
    "imshow",
    "imresize",
    "imfilter",
    "edge",
    "regionprops",
    "bwlabel",
    "rgb2gray",
    // Statistics and Machine Learning
    "fitlm",
    "predict",
    "kmeans",
    "pca",
    "corr",
    "ttest",
    "anova1",
    "normpdf",
    "normcdf",
    // Optimization Toolbox
    "fmincon",
    "linprog",
    "quadprog",
    "fsolve",
    "lsqnonlin",
    // Control System Toolbox
    "tf",
    "ss",
    "zpk",
    "bode",
    "step",
    "nyquist",
    "margin",
    // Curve Fitting Toolbox
    "fit",
    "cfit",
    "fittype",
    // Symbolic Math Toolbox (never deployable)
    "sym",
    "syms",
    "simplify",
    "expand",
    "solve",
    "diff",
    "int",
    "limit",
    "taylor",
];

/// Look up check metadata by ID.
fn check_meta(id: &str) -> Option<&'static CheckMeta> {
    CHECKS.iter().find(|c| c.id == id)
}

/// Look up check description by ID.
fn check_description(id: &str) -> &'static str {
    check_meta(id)
        .map(|c| c.description)
        .unwrap_or("Deployment constraint violation")
}

/// Look up check severity by ID.
fn check_severity(id: &str) -> Severity {
    check_meta(id)
        .map(|c| c.severity)
        .unwrap_or(Severity::Warning)
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Engine covering 10 deployment checks for MATLAB Compiler constraints.
///
/// Node-level checks fire on `function_call` and `command` nodes.
pub struct DeploymentEngine {
    config: DeploymentEngineConfig,
}

/// Node types for node-level dispatch.
const TARGET_NODES: &[&str] = &["function_call", "command"];

impl DeploymentEngine {
    /// Factory constructor called by the rule registry.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: DeploymentEngineConfig = config.rule_params("DEPLOYMENT_ENGINE");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Whether a check ID is enabled (not in `skip_checks`).
    fn is_enabled(&self, check_id: &str) -> bool {
        !self.config.skip_checks.iter().any(|s| s == check_id)
    }
}

impl Rule for DeploymentEngine {
    fn id(&self) -> &'static str {
        "DEPLOYMENT_ENGINE"
    }

    fn description(&self) -> &'static str {
        "MATLAB Compiler deployment constraint checks"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn category(&self) -> Category {
        Category::Deployment
    }

    fn enabled_by_default(&self) -> bool {
        // Only relevant for MATLAB Compiler targets; opt in explicitly.
        false
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        match ctx.node.kind() {
            "function_call" => self.check_function_call(ctx.node, ctx.source),
            "command" => self.check_command(ctx.node, ctx.source),
            _ => Vec::new(),
        }
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

// ---------------------------------------------------------------------------
// Helper: node text extraction
// ---------------------------------------------------------------------------

/// Extract the raw text of a node.
fn node_text<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
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

/// Extract command name from a `command` node.
pub(crate) fn extract_command_name<'a>(
    node: tree_sitter::Node<'a>,
    source: &'a str,
) -> Option<&'a str> {
    if node.kind() != "command" {
        return None;
    }
    let name_node = node.child(0)?;
    if name_node.kind() == "command_name" {
        Some(&source[name_node.start_byte()..name_node.end_byte()])
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Helper: pattern detection
// ---------------------------------------------------------------------------

/// Check if a function_call has an argument that looks like an absolute path.
/// Detects strings starting with '/' (Unix) or drive letters like 'C:\' (Windows).
pub(crate) fn has_absolute_path_arg(node: tree_sitter::Node, source: &str) -> bool {
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            if child.kind() == "string" {
                let text = node_text(child, source);
                let inner = text.trim_matches('\'').trim_matches('"');
                // Check for Unix absolute path or Windows drive letter
                if inner.starts_with('/')
                    || (inner.len() >= 3
                        && inner.as_bytes()[0].is_ascii_alphabetic()
                        && inner.as_bytes()[1] == b':'
                        && (inner.as_bytes()[2] == b'\\' || inner.as_bytes()[2] == b'/'))
                {
                    return true;
                }
            }
            // Recurse into arguments node
            if child.kind() == "arguments" && has_absolute_path_arg(child, source) {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "DEPLOYMENT_ENGINE",
    DeploymentEngine::from_config
));

// ---------------------------------------------------------------------------
// Shared test helpers
// ---------------------------------------------------------------------------

/// Build an engine with default configuration for tests.
#[cfg(test)]
pub(crate) fn engine() -> Box<dyn Rule> {
    DeploymentEngine::from_config(&Config::default())
}

// ---------------------------------------------------------------------------
// Tests (generic)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes, parse};
    use malt_core::Config;

    #[test]
    fn skip_checks_disables_check() {
        let config =
            Config::from_toml("[lint.rules.DEPLOYMENT_ENGINE]\nskip_checks = [\"MCTBX\"]\n")
                .expect("valid config");
        let rule = DeploymentEngine::from_config(&config);
        let src = "y = fft2(x);\n";
        let diags = lint_nodes(&*rule, src);
        assert!(!has_id(&diags, "MCTBX"), "got: {diags:?}");
    }

    /// Ensure the test source parses without syntax errors (sanity check).
    #[test]
    fn test_sources_parse() {
        let sources = [
            "cd /tmp\n",
            "addpath('src');\n",
            "doc plot\n",
            "r = matlabroot;\n",
        ];
        for src in sources {
            let tree = parse(src);
            assert!(!tree.root_node().has_error(), "parse error for: {src:?}");
        }
    }
}
