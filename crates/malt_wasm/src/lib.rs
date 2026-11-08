//! Browser WebAssembly bindings for malt, the MATLAB linter.
//!
//! The `wasm` feature exposes a [`wasm_bindgen`] `Linter` class plus free
//! functions so the same linting engine that powers the CLI can run in the
//! browser. The npm package is built with `wasm-pack` (see `mise run
//! build-wasm`) and consumed as `malt-wasm`.
//!
//! Everything in this crate is gated behind the `wasm` feature (the crate has
//! no default features), mirroring how rumdl isolates its `src/wasm.rs`.
//! Host-side tests exercise the bindings under `cargo test --features wasm`
//! (`mise run test-wasm`); a plain workspace build compiles this crate to an
//! empty library.

#[cfg(feature = "wasm")]
mod wasm {
    use std::collections::HashMap;
    use std::path::Path;

    use malt_core::{fix_source, position_of, Config, Diagnostic, FixMode, RuleRegistry, Severity};
    use wasm_bindgen::prelude::*;

    /// Deepest syntax tree analyzed in the browser (about 80 nested statements).
    const WASM_MAX_TREE_DEPTH: usize = 200;

    /// A single diagnostic serialized for JavaScript consumers.
    #[derive(serde::Serialize)]
    struct JsWarning {
        rule_id: &'static str,
        message: String,
        severity: String,
        line: usize,
        column: usize,
        end_line: usize,
        end_column: usize,
        fix: Option<JsFix>,
    }

    /// A fix serialized for JavaScript consumers (character offsets).
    #[derive(serde::Serialize)]
    struct JsFix {
        start: usize,
        end: usize,
        replacement: String,
    }

    impl JsWarning {
        fn from_diagnostic(diag: &Diagnostic, source: &str) -> Self {
            let (end_line, end_column) = position_of(source, diag.byte_range.end);
            JsWarning {
                rule_id: diag.rule_id,
                message: diag.message.clone(),
                severity: match diag.severity {
                    Severity::Error => "Error".to_string(),
                    Severity::Warning => "Warning".to_string(),
                    Severity::Info => "Info".to_string(),
                },
                line: diag.line,
                column: diag.column,
                end_line,
                end_column,
                fix: diag.fix.as_ref().map(|fix| js_fix(fix, source)),
            }
        }
    }

    /// Convert a `Fix` (byte offsets) to JS-facing character offsets.
    fn js_fix(fix: &malt_core::Fix, source: &str) -> JsFix {
        let start = byte_offset_to_char_offset(source, fix.byte_range.start);
        let end = byte_offset_to_char_offset(source, fix.byte_range.end);
        JsFix {
            start,
            end,
            replacement: fix.replacement.clone(),
        }
    }

    /// Convert a byte offset to a character (UTF-8 scalar) offset.
    fn byte_offset_to_char_offset(source: &str, byte: usize) -> usize {
        source[..byte].chars().count()
    }

    /// JavaScript-facing configuration object mirroring rumdl's `LinterConfig`.
    #[derive(Debug, Default, serde::Deserialize)]
    pub struct LinterConfig {
        /// Rule IDs to disable (e.g., `["AGROW", "NOSEMI"]`).
        #[serde(default)]
        pub disable: Vec<String>,
        /// If set, only these rule IDs are enabled (all others disabled).
        #[serde(default)]
        pub enable: Vec<String>,
        /// Additional rule IDs to enable beyond the default set.
        #[serde(default)]
        pub extend_enable: Vec<String>,
        /// Additional rule IDs to disable beyond `disable`.
        #[serde(default)]
        pub extend_disable: Vec<String>,
        /// Rule-specific configurations, keyed by engine or rule ID.
        ///
        /// Reserved for future per-rule parameter support (e.g. severity
        /// overrides or engine thresholds). Deserialized to keep the JS-facing
        /// surface stable with rumdl's `LinterConfig`.
        #[serde(flatten)]
        #[allow(dead_code)]
        pub rules: HashMap<String, serde_json::Value>,
        /// File path patterns to exclude (matched against the `path` argument).
        #[serde(default)]
        pub exclude: Vec<String>,
        /// Whether `%#ok<...>` inline suppression directives are honored.
        #[serde(default = "default_true")]
        pub inline_suppression: bool,
        /// Named preset to base the configuration on ("all", "mathworks",
        /// "recommended"). Defaults to "mathworks" when absent.
        #[serde(default)]
        pub preset: Option<String>,
    }

    fn default_true() -> bool {
        true
    }

    impl LinterConfig {
        /// Convert the JS-facing config into an `malt_core::Config`.
        fn to_config(&self) -> Config {
            let mut config = Config {
                exclude: self.exclude.clone(),
                inline_suppression: self.inline_suppression,
                ..Config::default()
            };

            if let Some(name) = &self.preset {
                config.preset = malt_core::preset::resolve(name)
                    .unwrap_or_else(|| malt_core::preset::default_preset());
            }

            // If `enable` is provided, disable every rule we know about first.
            let all_rule_ids = malt_rules::all_rules(&Config::default())
                .into_iter()
                .map(|r| r.id().to_string())
                .collect::<Vec<_>>();

            if !self.enable.is_empty() {
                for id in &all_rule_ids {
                    if !self.enable.contains(id) && !self.extend_enable.contains(id) {
                        disable_rule(&mut config, id);
                    }
                }
            }

            for id in self.disable.iter().chain(self.extend_disable.iter()) {
                disable_rule(&mut config, id);
            }

            config
        }
    }

    /// Build a `Linter` from a `.malt.toml` string, returning a plain error
    /// string (host-testable; the `#[wasm_bindgen]` wrapper maps it to `JsValue`).
    fn from_toml_str(toml: &str) -> Result<Linter, String> {
        let malt_config = Config::from_toml(toml).map_err(|e| format!("Invalid config: {e}"))?;
        Ok(build_linter(malt_config))
    }

    /// Mark a rule ID as disabled in the config.
    fn disable_rule(config: &mut Config, rule_id: &str) {
        let entry = config.rules.entry(rule_id.to_string()).or_default();
        entry.enabled = false;
        entry.severity = None;
    }

    /// The linting engine exposed to JavaScript.
    #[wasm_bindgen]
    pub struct Linter {
        linter: malt_core::Linter,
        config_warnings: Vec<String>,
    }

    /// Build a `Linter` from a JS-facing config (shared with tests).
    #[cfg(test)]
    pub(crate) fn from_config(config: LinterConfig) -> Linter {
        build_linter(config.to_config())
    }

    /// Build a `Linter` from a resolved `malt_core::Config`.
    fn build_linter(malt_config: Config) -> Linter {
        let rules = malt_rules::active_rules(&malt_config);
        let registry = RuleRegistry::new(rules, &malt_config);
        let mut linter = malt_core::Linter::new(registry);
        linter.set_inline_suppression(malt_config.inline_suppression);
        // About 1 MiB of stack and no thread to enlarge it: stop earlier than natively.
        linter.set_max_tree_depth(WASM_MAX_TREE_DEPTH);
        Linter {
            linter,
            config_warnings: Vec::new(),
        }
    }

    #[wasm_bindgen]
    impl Linter {
        /// Create a new `Linter` with the given configuration.
        ///
        /// # Arguments
        /// * `options` - Configuration object (see `LinterConfig`).
        #[wasm_bindgen(constructor)]
        pub fn new(options: JsValue) -> Result<Linter, JsValue> {
            let linter_config: LinterConfig = if options.is_undefined() || options.is_null() {
                LinterConfig::default()
            } else {
                serde_wasm_bindgen::from_value(options)
                    .map_err(|e| JsValue::from_str(&format!("Invalid config: {e}")))?
            };
            Ok(build_linter(linter_config.to_config()))
        }

        /// Create a new `Linter` from a `.malt.toml` configuration string.
        ///
        /// Mirrors the CLI's config file with full fidelity (`[lint] preset`,
        /// per-rule severity, per-category overrides, rule params, `exclude`).
        /// Returns a `JsValue` error string when the TOML cannot be parsed, so
        /// callers can display it.
        ///
        /// # Arguments
        /// * `toml` - The contents of a `.malt.toml` file.
        pub fn from_toml(toml: &str) -> Result<Linter, JsValue> {
            from_toml_str(toml).map_err(|e| JsValue::from_str(&e))
        }

        /// Lint MATLAB content and return warnings as JSON.
        ///
        /// # Arguments
        /// * `content` - MATLAB source to lint.
        /// * `path` - Optional file path used for exclude matching and
        ///   file-name-dependent checks; defaults to `"untitled.m"`.
        ///
        /// Returns a JSON array of warnings.
        pub fn check(&mut self, content: &str, path: Option<String>) -> String {
            let path = Path::new(path.as_deref().unwrap_or("untitled.m"));
            match self.linter.lint(content, path) {
                Ok(diagnostics) => {
                    let warnings: Vec<JsWarning> = diagnostics
                        .iter()
                        .map(|d| JsWarning::from_diagnostic(d, content))
                        .collect();
                    serde_json::to_string(&warnings).unwrap_or_else(|_| "[]".to_string())
                }
                Err(e) => format!(r#"[{{"error": "{e}"}}]"#),
            }
        }

        /// Apply all auto-fixes to the content and return the fixed content.
        ///
        /// # Arguments
        /// * `content` - MATLAB source to fix.
        /// * `path` - Optional file path (see `check`).
        pub fn fix(&mut self, content: &str, path: Option<String>) -> String {
            let path = Path::new(path.as_deref().unwrap_or("untitled.m"));
            match fix_source(&mut self.linter, content, path, FixMode::Safe) {
                Ok(outcome) => outcome.source,
                Err(_) => content.to_string(),
            }
        }

        /// Get the malt version.
        pub fn get_version() -> String {
            env!("CARGO_PKG_VERSION").to_string()
        }

        /// Get the list of available rule IDs as a JSON array.
        pub fn get_available_rules() -> String {
            let ids: Vec<&str> = malt_rules::all_rules(&Config::default())
                .into_iter()
                .map(|r| r.id())
                .collect();
            serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string())
        }

        /// Get any warnings generated during configuration parsing (JSON array).
        pub fn get_config_warnings(&self) -> String {
            serde_json::to_string(&self.config_warnings).unwrap_or_else(|_| "[]".to_string())
        }
    }

    /// Initialize the WASM module with better panic messages.
    #[wasm_bindgen(start)]
    pub fn init() {
        console_error_panic_hook::set_once();
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn linter() -> Linter {
            from_config(LinterConfig::default())
        }

        #[test]
        fn version_matches_cargo() {
            assert_eq!(Linter::get_version(), env!("CARGO_PKG_VERSION"));
        }

        #[test]
        fn available_rules_is_nonempty_json() {
            let json = Linter::get_available_rules();
            let ids: Vec<String> = serde_json::from_str(&json).unwrap();
            assert!(!ids.is_empty());
        }

        #[test]
        fn check_returns_warning_for_missing_semicolon() {
            let mut l = linter();
            let warnings = l.check("x = 1\n", None);
            let parsed: Vec<serde_json::Value> = serde_json::from_str(&warnings).unwrap();
            assert!(
                parsed.iter().any(|w| w["rule_id"] == "NOPTS"),
                "expected NOPTS warning, got: {warnings}"
            );
        }

        #[test]
        fn check_clean_code_has_no_nosemi() {
            let mut l = linter();
            // A simple script statement with a trailing semicolon is clean from
            // the formatting perspective; no NOPTS (and no parse error).
            let warnings = l.check("result = 1;\n", None);
            let parsed: Vec<serde_json::Value> = serde_json::from_str(&warnings).unwrap();
            assert!(
                !parsed.iter().any(|w| w["rule_id"] == "NOPTS"),
                "unexpected NOPTS, got: {warnings}"
            );
            assert!(
                parsed.iter().all(|w| w.get("error").is_none()),
                "unexpected parse error, got: {warnings}"
            );
        }

        #[test]
        fn warning_shape_includes_positions_and_fix() {
            let mut l = linter();
            let warnings = l.check("x = 1\n", None);
            let parsed: Vec<serde_json::Value> = serde_json::from_str(&warnings).unwrap();
            let nosemi = parsed.iter().find(|w| w["rule_id"] == "NOPTS").unwrap();
            assert!(nosemi["line"].as_u64().is_some());
            assert!(nosemi["column"].as_u64().is_some());
            assert!(nosemi["end_line"].as_u64().is_some());
            assert!(nosemi["end_column"].as_u64().is_some());
            assert!(nosemi["severity"].as_str().is_some());
            assert!(nosemi["fix"]["start"].as_u64().is_some());
        }

        #[test]
        fn fix_applies_semicolon() {
            let mut l = linter();
            let fixed = l.fix("x = 1\n", None);
            assert!(
                fixed.contains(';'),
                "expected a semicolon added, got: {fixed:?}"
            );
        }

        #[test]
        fn disable_rule_suppresses_diagnostic() {
            let config = LinterConfig {
                disable: vec!["NOPTS".to_string()],
                ..Default::default()
            };
            let mut l = from_config(config);
            let warnings = l.check("x = 1\n", None);
            let parsed: Vec<serde_json::Value> = serde_json::from_str(&warnings).unwrap();
            assert!(
                !parsed.iter().any(|w| w["rule_id"] == "NOPTS"),
                "NOPTS should be disabled, got: {warnings}"
            );
        }

        #[test]
        fn enable_only_specific_rules() {
            let config = LinterConfig {
                enable: vec!["AGROW".to_string()],
                ..Default::default()
            };
            let mut l = from_config(config);
            let warnings = l.check("x = 1\n", None);
            let parsed: Vec<serde_json::Value> = serde_json::from_str(&warnings).unwrap();
            assert!(
                !parsed.iter().any(|w| w["rule_id"] == "NOPTS"),
                "NOPTS should be disabled when only AGROW is enabled"
            );
        }

        #[test]
        fn multi_byte_char_offsets_are_characters() {
            let content = "% ééé\nx = 1\n";
            let mut l = linter();
            let warnings = l.check(content, None);
            let parsed: Vec<serde_json::Value> = serde_json::from_str(&warnings).unwrap();
            for w in &parsed {
                if let Some(fix) = w["fix"].as_object() {
                    let start = fix["start"].as_u64().unwrap() as usize;
                    let end = fix["end"].as_u64().unwrap() as usize;
                    let _ = &content[start..end];
                }
            }
            assert!(serde_json::from_str::<Vec<serde_json::Value>>(&warnings).is_ok());
        }

        #[test]
        fn from_toml_disables_rule() {
            let mut l = from_toml_str("[lint.rules]\nNOPTS = \"off\"\n").unwrap();
            let warnings = l.check("x = 1\n", None);
            let parsed: Vec<serde_json::Value> = serde_json::from_str(&warnings).unwrap();
            assert!(
                !parsed.iter().any(|w| w["rule_id"] == "NOPTS"),
                "NOPTS should be disabled, got: {warnings}"
            );
        }

        #[test]
        fn from_toml_reenables_default_off_engine() {
            // A bare script triggers EMSCR (codegen) only when CODEGEN_ENGINE is on.
            let mut default_linter = linter();
            let default_warnings = default_linter.check("x = 1\n", None);
            let default_parsed: Vec<serde_json::Value> =
                serde_json::from_str(&default_warnings).unwrap();
            assert!(
                !default_parsed.iter().any(|w| w["rule_id"] == "EMSCR"),
                "EMSCR should be off by default, got: {default_warnings}"
            );

            let mut l =
                from_toml_str("[lint.rules.CODEGEN_ENGINE]\nseverity = \"error\"\n").unwrap();
            let warnings = l.check("x = 1\n", None);
            let parsed: Vec<serde_json::Value> = serde_json::from_str(&warnings).unwrap();
            assert!(
                parsed.iter().any(|w| w["rule_id"] == "EMSCR"),
                "EMSCR should fire once CODEGEN_ENGINE is re-enabled, got: {warnings}"
            );
        }

        #[test]
        fn from_toml_rejects_invalid_toml() {
            assert!(from_toml_str("[lint.rules\nNOPTS = ").is_err());
        }
    }
}

#[cfg(all(test, feature = "wasm"))]
mod preset_tests {
    use crate::wasm::*;

    fn rule_ids(l: &mut Linter, src: &str) -> Vec<String> {
        let warnings = l.check(src, None);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&warnings).unwrap();
        parsed
            .iter()
            .map(|w| w["rule_id"].as_str().unwrap_or_default().to_string())
            .collect()
    }

    #[test]
    fn from_toml_reads_preset() {
        // `recommended` disables formatting (which includes NOPTS).
        let mut l = Linter::from_toml("[lint]\npreset = \"recommended\"\n").unwrap();
        assert!(!rule_ids(&mut l, "x = 1\n").contains(&"NOPTS".to_string()));
    }

    #[test]
    fn from_toml_mathworks_keeps_formatting() {
        let mut l = Linter::from_toml("").unwrap();
        assert!(rule_ids(&mut l, "x = 1\n").contains(&"NOPTS".to_string()));
    }

    #[test]
    fn linter_config_preset_field() {
        let config = LinterConfig {
            preset: Some("recommended".to_string()),
            ..Default::default()
        };
        let mut l = from_config(config);
        assert!(!rule_ids(&mut l, "x = 1\n").contains(&"NOPTS".to_string()));
    }
}
