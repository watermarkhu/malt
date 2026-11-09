//! # UNSUPPORTED_ENGINE: Unsupported Feature Checks
//!
//! ```malt
//! id = "UNSUPPORTED_ENGINE"
//! title = "Unsupported Feature Checks"
//! category = "unsupported"
//! severity = "warning"
//! fix = false
//! icon = "lucide/ban"
//! slug = "unsupported"
//! ```
//!
//! ## Rule
//!
//! Detects usage of unsupported, deprecated, or platform-specific features
//! that are no longer available in modern MATLAB or restricted to specific
//! configurations. All 13 checks share a single `UnsupportedEngine` that
//! dispatches node-level checks on `function_call`, `command`, `attribute`
//! and `field_expression` nodes; each diagnostic carries the specific check ID
//! (e.g. `MCADE`, `FEATUD`). Function checks fire on both call syntax
//! (`feature('x')`) and command syntax (`feature accel on`); `IMPKG` and
//! `MIPKG` fire on references and imports of `internal.matlab.*` and
//! `matlab.<...>.internal.*` respectively; `MCADE` fires on a `Description`
//! class-member attribute.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | MCADE | warning | no | Using Description as an attribute is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//! | AWTIUD | warning | no | 'awtinvoke' is unsupported and might have been changed without notice or might be removed without notice. With appropriate code changes, use javaMethodEDT instead. |
//! | AXCHUD | warning | no | 'axescheck' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//! | FEATUD | warning | no | 'feature' and flags passed to it are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//! | FNDPUD | warning | no | 'findpackage' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//! | HGCNUD | warning | no | 'hgconvertunits' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//! | IMPKG | warning | no | Functions in internal.matlab namespace are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//! | ISMBUD | warning | no | 'ismembc' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//! | MIPKG | warning | no | Functions in MATLAB's internal namespaces are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//! | SEPTUD | warning | no | 'setptr' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//! | SYDEUD | warning | no | 'system_dependent' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//! | UIRSUD | warning | no | 'uirestore' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//! | UISUUD | warning | no | 'uisuspend' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this. |
//!
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! v = feature('version');       % FEATUD
//! setptr(gcf, 'hand');          % SEPTUD
//! x = ismembc(a, b);            % ISMBUD
//! import internal.matlab.*      % IMPKG
//! y = matlab.internal.foo.bar;  % MIPKG
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! tf = ismember(a, b);
//! v = version;
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.UNSUPPORTED_ENGINE]
//! severity = "warning"
//! skip_checks = ["IMPKG"]
//! ```

use malt_core::{Category, Config, Diagnostic, NodeContext, Rule, Severity};
use serde::Deserialize;

mod check_command;
mod check_function_call;

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for the unsupported features engine.
///
/// Deserialized from the `[lint.rules.UNSUPPORTED_ENGINE]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UnsupportedEngineConfig {
    /// Check IDs to skip (e.g., `["IMPKG"]`).
    #[serde(default)]
    pub skip_checks: Vec<String>,
}

// ---------------------------------------------------------------------------
// Check metadata
// ---------------------------------------------------------------------------

/// Static metadata for a single unsupported feature check.
struct CheckMeta {
    id: &'static str,
    description: &'static str,
}

/// All 13 unsupported feature check definitions.
const CHECKS: &[CheckMeta] = &[
    CheckMeta {
        id: "MCADE",
        description: "Using Description as an attribute is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
    CheckMeta {
        id: "AWTIUD",
        description: "'awtinvoke' is unsupported and might have been changed without notice or might be removed without notice. With appropriate code changes, use javaMethodEDT instead.",
    },
    CheckMeta {
        id: "AXCHUD",
        description: "'axescheck' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
    CheckMeta {
        id: "FEATUD",
        description: "'feature' and flags passed to it are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
    CheckMeta {
        id: "FNDPUD",
        description: "'findpackage' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
    CheckMeta {
        id: "HGCNUD",
        description: "'hgconvertunits' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
    CheckMeta {
        id: "IMPKG",
        description: "Functions in internal.matlab namespace are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
    CheckMeta {
        id: "ISMBUD",
        description: "'ismembc' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
    CheckMeta {
        id: "MIPKG",
        description: "Functions in MATLAB's internal namespaces are unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
    CheckMeta {
        id: "SEPTUD",
        description: "'setptr' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
    CheckMeta {
        id: "SYDEUD",
        description: "'system_dependent' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
    CheckMeta {
        id: "UIRSUD",
        description: "'uirestore' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
    CheckMeta {
        id: "UISUUD",
        description: "'uisuspend' is unsupported and might have been changed without notice or might be removed without notice. There is no simple replacement for this.",
    },
];

/// Unsupported functions: `(function name, check ID)`.
pub(crate) const UNSUPPORTED_FUNCTIONS: &[(&str, &str)] = &[
    ("awtinvoke", "AWTIUD"),
    ("axescheck", "AXCHUD"),
    ("feature", "FEATUD"),
    ("findpackage", "FNDPUD"),
    ("hgconvertunits", "HGCNUD"),
    ("ismembc", "ISMBUD"),
    ("setptr", "SEPTUD"),
    ("system_dependent", "SYDEUD"),
    ("uirestore", "UIRSUD"),
    ("uisuspend", "UISUUD"),
];

/// Classify a (dotted) name against the unsupported namespaces.
///
/// Returns `IMPKG` for `internal.matlab.*` and `MIPKG` for `matlab.<...>.internal.*`.
pub(crate) fn namespace_check(name: &str) -> Option<&'static str> {
    // A wildcard import (`internal.matlab.*`) names every member of the namespace.
    let mut segs: Vec<&str> = name.split('.').collect();
    if segs.last() == Some(&"*") {
        segs.pop();
        segs.push("member");
    }
    if segs.len() >= 3 && segs[0] == "internal" && segs[1] == "matlab" {
        return Some("IMPKG");
    }
    if segs.len() >= 3 && segs[0] == "matlab" && segs[1..segs.len() - 1].contains(&"internal") {
        return Some("MIPKG");
    }
    None
}

/// Look up check description by ID.
fn check_description(id: &str) -> &'static str {
    CHECKS
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.description)
        .unwrap_or("Unsupported feature detected")
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Engine covering 13 unsupported feature checks.
///
/// Node-level checks fire on `function_call`, `command`, `attribute` and
/// `field_expression` nodes.
pub struct UnsupportedEngine {
    config: UnsupportedEngineConfig,
}

/// Node types for node-level dispatch.
const TARGET_NODES: &[&str] = &["function_call", "command", "attribute", "field_expression"];

impl UnsupportedEngine {
    /// Factory constructor called by the rule registry.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: UnsupportedEngineConfig = config.rule_params("UNSUPPORTED_ENGINE");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Whether a check ID is enabled (not in `skip_checks`).
    fn is_enabled(&self, check_id: &str) -> bool {
        !self.config.skip_checks.iter().any(|s| s == check_id)
    }
}

impl Rule for UnsupportedEngine {
    fn id(&self) -> &'static str {
        "UNSUPPORTED_ENGINE"
    }

    fn description(&self) -> &'static str {
        "Unsupported or deprecated feature checks"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn category(&self) -> Category {
        Category::Unsupported
    }

    fn enabled_by_default(&self) -> bool {
        // Platform-specific feature checks are noisy for general runs; opt in explicitly.
        false
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        match ctx.node.kind() {
            "function_call" => self.check_function_call(ctx.node, ctx.source),
            "command" => self.check_command(ctx.node, ctx.source),
            "attribute" => self.check_attribute(ctx.node, ctx.source),
            "field_expression" => self.check_field_expression(ctx.node, ctx.source),
            _ => Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: diagnostic construction
// ---------------------------------------------------------------------------

/// Build a diagnostic from a check ID and node.
pub(crate) fn make_diag(check_id: &'static str, node: tree_sitter::Node) -> Diagnostic {
    let start = node.start_position();
    Diagnostic {
        rule_id: check_id,
        message: check_description(check_id).to_string(),
        severity: Severity::Warning,
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
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "UNSUPPORTED_ENGINE",
    UnsupportedEngine::from_config
));

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Shared test helper: construct the engine with the default configuration.
#[cfg(test)]
pub(crate) fn engine() -> Box<dyn Rule> {
    UnsupportedEngine::from_config(&Config::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes, parse};
    use malt_core::Config;

    // -- skip_checks configuration -------------------------------------------

    #[test]
    fn skip_checks_disables_check() {
        let config =
            Config::from_toml("[lint.rules.UNSUPPORTED_ENGINE]\nskip_checks = [\"FEATUD\"]\n")
                .expect("valid config");
        let rule = UnsupportedEngine::from_config(&config);
        let src = "v = feature('version');\n";
        let diags = lint_nodes(&*rule, src);
        assert!(!has_id(&diags, "FEATUD"), "got: {diags:?}");
    }

    /// Ensure the test sources parse without syntax errors (sanity check).
    #[test]
    fn test_sources_parse() {
        let sources = [
            "internal.matlab.foo(1);\n",
            "feature accel on\n",
            "import pkg.sub.*;\n",
            "y = matlab.internal.a.b;\n",
            "classdef A\n properties (Description = 'x')\n a\n end\nend\n",
        ];
        for src in sources {
            let tree = parse(src);
            assert!(!tree.root_node().has_error(), "parse error for: {src:?}");
        }
    }
}
