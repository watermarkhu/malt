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
mod action;

#[cfg(feature = "wasm")]
mod wasm {
    use std::collections::HashMap;
    use std::path::Path;

    use malt_core::report::Record;
    use malt_core::{fix_source, position_of, Config, Diagnostic, FixMode, RuleRegistry, Severity};
    use wasm_bindgen::prelude::*;

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
    #[derive(Debug, serde::Deserialize)]
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
        /// Run the experimental checks too (`[lint] experimental`). Defaults to false.
        #[serde(default)]
        pub experimental: bool,
    }

    fn default_true() -> bool {
        true
    }

    /// The configuration of `new Linter()`: the same as an empty object. A derived `Default`
    /// would turn `%#ok` suppression off (`bool::default()`), which only `new Linter(null)`
    /// and `new Linter()` would see, not `new Linter({})`.
    impl Default for LinterConfig {
        fn default() -> Self {
            Self {
                disable: Vec::new(),
                enable: Vec::new(),
                extend_disable: Vec::new(),
                extend_enable: Vec::new(),
                rules: HashMap::new(),
                exclude: Vec::new(),
                inline_suppression: default_true(),
                preset: None,
                experimental: false,
            }
        }
    }

    impl LinterConfig {
        /// Convert the JS-facing config into an `malt_core::Config`.
        fn to_config(&self) -> Config {
            let mut config = Config {
                exclude: self.exclude.clone(),
                inline_suppression: self.inline_suppression,
                experimental: self.experimental,
                ..Config::default()
            };

            let preset = match &self.preset {
                Some(name) => malt_core::preset::resolve(name)
                    .unwrap_or_else(|| malt_core::preset::default_preset()),
                None => malt_core::preset::default_preset(),
            };
            config.set_preset(preset);

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
        from_toml_with_str(toml, "{}")
    }

    /// Like [`from_toml_str`], with overrides given as a JSON object that win over the
    /// file: `{"preset": "monkeyproof"}`. An unknown preset or key is an error.
    pub(crate) fn from_toml_with_str(toml: &str, overrides: &str) -> Result<Linter, String> {
        let mut malt_config =
            Config::from_toml(toml).map_err(|e| format!("Invalid config: {e}"))?;
        let overrides: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(overrides).map_err(|e| format!("Invalid overrides: {e}"))?;
        for (key, value) in &overrides {
            match (key.as_str(), value) {
                ("preset", serde_json::Value::String(name)) if !name.is_empty() => {
                    let preset = malt_core::preset::resolve(name).ok_or_else(|| {
                        let names = malt_core::preset::PRESETS
                            .iter()
                            .map(|p| p.name)
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("Unknown preset '{name}' (expected one of: {names})")
                    })?;
                    malt_config.set_preset(preset);
                }
                ("preset", _) => {}
                _ => return Err(format!("Unknown override '{key}'")),
            }
        }
        let warnings = malt_rules::config_check::config_warnings(&malt_config);
        let mut linter = build_linter(malt_config);
        linter.config_warnings = warnings;
        Ok(linter)
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
        config: Config,
        config_warnings: Vec<String>,
    }

    impl Linter {
        /// The resolved configuration.
        pub(crate) fn config(&self) -> &Config {
            &self.config
        }

        /// The warnings found while reading the configuration.
        pub(crate) fn config_warnings(&self) -> &[String] {
            &self.config_warnings
        }
    }

    /// The findings of `content` as records for the file at `path`: what the outputs of
    /// `malt_core::report` are written from. A file that cannot be analyzed gives one
    /// error record.
    pub(crate) fn records_of(linter: &mut Linter, content: &str, path: &str) -> Vec<Record> {
        match linter.linter.lint(content, Path::new(path)) {
            Ok(diagnostics) => diagnostics
                .iter()
                .map(|d| Record::new(Path::new(path), d))
                .collect(),
            Err(e) => vec![Record {
                path: path.to_string(),
                line: 1,
                column: 1,
                rule_id: "MALT_ERROR".to_string(),
                severity: Severity::Error,
                message: e.to_string(),
                fix: None,
            }],
        }
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
        // About 1 MiB of stack, little memory and no thread to enlarge the stack:
        // stop earlier than natively (200 levels, 2 MiB, 1,000 nested brackets)
        // unless the config's `[lint.limits]` says otherwise.
        linter.set_limits(malt_config.limits.resolve(malt_core::Limits::wasm()));
        Linter {
            linter,
            config: malt_config,
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

        /// Create a new `Linter` from a `.malt.toml` string and overrides that win over
        /// it, as a JSON object: `{"preset": "monkeyproof"}`.
        pub fn from_toml_with(toml: &str, overrides: &str) -> Result<Linter, JsValue> {
            from_toml_with_str(toml, overrides).map_err(|e| JsValue::from_str(&e))
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

    /// The file system of a run, as three JavaScript callbacks: `stat(path)` gives
    /// `"file"`, `"dir"`, `"other"` or null; `list(dir)` an array of `[name, kind]` (and
    /// throws when the folder cannot be read); `read(path)` a `Uint8Array` or null.
    struct JsFs {
        stat: js_sys::Function,
        list: js_sys::Function,
        read: js_sys::Function,
    }

    fn kind_of(value: &JsValue) -> Option<malt_core::files::Kind> {
        use malt_core::files::Kind;
        match value.as_string()?.as_str() {
            "file" => Some(Kind::File),
            "dir" => Some(Kind::Dir),
            "other" => Some(Kind::Other),
            _ => None,
        }
    }

    impl malt_core::files::Fs for JsFs {
        fn kind(&self, path: &str) -> Option<malt_core::files::Kind> {
            let value = self
                .stat
                .call1(&JsValue::NULL, &JsValue::from_str(path))
                .ok()?;
            kind_of(&value)
        }

        fn list(&self, dir: &str) -> Result<Vec<(String, malt_core::files::Kind)>, String> {
            let value = self
                .list
                .call1(&JsValue::NULL, &JsValue::from_str(dir))
                .map_err(|e| {
                    e.as_string()
                        .unwrap_or_else(|| "cannot read the folder".into())
                })?;
            let mut entries = Vec::new();
            for entry in js_sys::Array::from(&value).iter() {
                let pair = js_sys::Array::from(&entry);
                let name = pair.get(0).as_string().unwrap_or_default();
                let kind = kind_of(&pair.get(1)).unwrap_or(malt_core::files::Kind::Other);
                entries.push((name, kind));
            }
            Ok(entries)
        }

        fn read(&self, path: &str) -> Option<Vec<u8>> {
            let value = self
                .read
                .call1(&JsValue::NULL, &JsValue::from_str(path))
                .ok()?;
            if value.is_null() || value.is_undefined() {
                return None;
            }
            Some(js_sys::Uint8Array::new(&value).to_vec())
        }
    }

    /// Run the GitHub Action: the raw inputs as a JSON object, and the file system as three
    /// callbacks (see `JsFs`). Returns a JSON object: `stdout`, `summary_markdown`,
    /// `results_json`, `findings`, `errors`, `warnings`, `files` and `failed`. An invalid
    /// input is an error string.
    #[wasm_bindgen]
    pub fn run_action(
        inputs: &str,
        stat: js_sys::Function,
        list: js_sys::Function,
        read: js_sys::Function,
    ) -> Result<String, JsValue> {
        let inputs: crate::action::Inputs = serde_json::from_str(inputs)
            .map_err(|e| JsValue::from_str(&format!("Invalid inputs: {e}")))?;
        let outcome = crate::action::run(&JsFs { stat, list, read }, &inputs)
            .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&outcome).map_err(|e| JsValue::from_str(&e.to_string()))
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
        fn overrides_win_over_the_toml() {
            let source = "function f(x)\n  y = 1;\nend\n";
            let ids = |l: &mut Linter| -> Vec<String> {
                records_of(l, source, "a.m")
                    .iter()
                    .map(|r| r.rule_id.clone())
                    .collect()
            };
            let mut plain = from_toml_with_str("[lint]\npreset = \"mathworks\"\n", "{}").unwrap();
            assert!(!ids(&mut plain).iter().any(|id| id.starts_with("MP_")));
            let mut monkeyproof = from_toml_with_str(
                "[lint]\npreset = \"mathworks\"\n",
                r#"{"preset": "monkeyproof"}"#,
            )
            .unwrap();
            assert!(ids(&mut monkeyproof).iter().any(|id| id.starts_with("MP_")));
        }

        #[test]
        fn an_empty_preset_override_keeps_the_file() {
            let linter =
                from_toml_with_str("[lint]\npreset = \"recommended\"\n", r#"{"preset": ""}"#)
                    .unwrap();
            assert_eq!(linter.config().preset.name, "recommended");
        }

        #[test]
        fn bad_overrides_are_errors() {
            let unknown_preset = from_toml_with_str("", r#"{"preset": "bogus"}"#)
                .err()
                .unwrap();
            assert!(
                unknown_preset.contains("Unknown preset 'bogus'"),
                "{unknown_preset}"
            );
            let unknown_key = from_toml_with_str("", r#"{"colour": "red"}"#)
                .err()
                .unwrap();
            assert!(
                unknown_key.contains("Unknown override 'colour'"),
                "{unknown_key}"
            );
            assert!(from_toml_with_str("", "not json").is_err());
        }

        /// Every conformance fixture (`matlab_files.txt`, with the findings MATLAB R2026b
        /// recorded in `matlab_expected.tsv`) through the same `check` the browser calls:
        /// the wasm package must report what the native build reports. Both lists are read
        /// from the CLI's test folder at test time.
        #[test]
        fn conformance_fixtures_match_the_recorded_matlab_findings() {
            use std::collections::BTreeSet;
            let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../malt_cli/tests/fixtures");
            let list = std::fs::read_to_string(fixtures.join("matlab_files.txt")).unwrap();
            let files: Vec<&str> = list
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();
            let recorded = std::fs::read_to_string(fixtures.join("matlab_expected.tsv")).unwrap();
            let expected: BTreeSet<(String, usize, String)> = recorded
                .lines()
                .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
                .map(|l| {
                    let c: Vec<&str> = l.split('\t').collect();
                    (c[0].to_string(), c[1].parse().unwrap(), c[2].to_string())
                })
                .collect();

            // The depth guard of the wasm build is lower than the native one, so a fixture
            // that nests deeper than it reports MALT_TOO_DEEP here: none may.
            let found = std::thread::Builder::new()
                .stack_size(256 * 1024 * 1024)
                .spawn({
                    let files: Vec<String> = files.iter().map(|f| f.to_string()).collect();
                    let fixtures = fixtures.clone();
                    move || {
                        let mut linter = linter();
                        let mut found = BTreeSet::new();
                        for file in &files {
                            let source = std::fs::read_to_string(fixtures.join(file)).unwrap();
                            let json = linter.check(&source, Some(file.clone()));
                            let warnings: Vec<serde_json::Value> =
                                serde_json::from_str(&json).unwrap();
                            for w in warnings {
                                let id = w["rule_id"].as_str().expect("a rule_id").to_string();
                                let line = w["line"].as_u64().expect("a line") as usize;
                                found.insert((file.clone(), line, id));
                            }
                        }
                        found
                    }
                })
                .unwrap()
                .join()
                .unwrap();

            // Known differences of the wasm build. `fix/incomplete__di200.m` nests 200 levels
            // deep: the wasm depth guard (200, see malt_core::limits::WASM_MAX_TREE_DEPTH) reports MALT_TOO_DEEP
            // where the native one (1,000) does not. A trap was measured at about 750
            // chained binary operators in V8, so the lower guard is deliberate.
            let found: BTreeSet<_> = found
                .into_iter()
                .filter(|(file, _, id)| {
                    !(file == "fix/incomplete__di200.m" && id == "MALT_TOO_DEEP")
                })
                .collect();
            let missing: Vec<_> = expected.difference(&found).take(10).collect();
            let extra: Vec<_> = found.difference(&expected).take(10).collect();
            assert!(
                missing.is_empty() && extra.is_empty(),
                "wasm differs from the recorded findings over {} files:\nrecorded but not reported: {missing:?}\nreported but not recorded: {extra:?}",
                files.len()
            );
        }

        #[test]
        fn a_default_config_honors_inline_suppression_like_an_empty_object() {
            // `new Linter()` and `new Linter(null)` use `LinterConfig::default()`.
            assert!(LinterConfig::default().inline_suppression);
            let empty: LinterConfig = serde_json::from_str("{}").unwrap();
            assert_eq!(
                empty.inline_suppression,
                LinterConfig::default().inline_suppression
            );
            let mut l = linter();
            let suppressed = l.check(
                "x = [];\nfor i=1:3\n    x(end+1) = i; %#ok<SAGROW>\nend\n",
                None,
            );
            assert!(!suppressed.contains("GROW"), "{suppressed}");
        }

        #[test]
        fn from_toml_keeps_rule_key_warnings() {
            let l = from_toml_str("[lint.rules]\nNOSMI = \"off\"\n").expect("valid TOML");
            let warnings: Vec<String> = serde_json::from_str(&l.get_config_warnings()).unwrap();
            assert_eq!(warnings.len(), 1, "{warnings:?}");
            assert!(
                warnings[0].contains("did you mean 'NOSEMI'?"),
                "{warnings:?}"
            );
            let clean = from_toml_str("[lint.rules]\nNOSEMI = \"warn\"\n").expect("valid TOML");
            assert_eq!(clean.get_config_warnings(), "[]");
        }

        #[test]
        fn wasm_defaults_are_lower_than_native() {
            // Over the wasm default of 2 MiB: refused without parsing.
            let mut l = linter();
            let big = "x = 1;\n".repeat(2 * 1024 * 1024 / 7 + 1);
            let json = l.check(&big, None);
            assert!(
                json.contains("MALT_TOO_BIG"),
                "{}",
                &json[..json.len().min(300)]
            );
            assert!(json.contains("not a MATLAB message"));
            // Nesting past the wasm default of 1,000 brackets: refused without parsing.
            let nested = format!("x = {}1{};\n", "(".repeat(1_001), ")".repeat(1_001));
            assert!(l.check(&nested, None).contains("MALT_TOO_DEEP"));
        }

        #[test]
        fn from_toml_limits_override_the_wasm_defaults() {
            let mut l = from_toml_str("[lint.limits]\nmax_file_size = \"1 KiB\"\n").unwrap();
            assert!(l.get_config_warnings() == "[]");
            let json = l.check(&"x = 1;\n".repeat(300), None);
            assert!(
                json.contains("MALT_TOO_BIG") && json.contains("1 KiB"),
                "{json}"
            );
            assert!(!l.check("x = 1;\n", None).contains("MALT_TOO_BIG"));
            // Raising the size leaves the other wasm defaults in place (the lint of a file of
            // megabytes would take minutes in a debug test, so the boundary is tested small).
            let mut l = from_toml_str("[lint.limits]\nmax_file_size = \"4 KiB\"\n").unwrap();
            let src = "x = 1;\n".repeat(400);
            assert!(!l.check(&src, None).contains("MALT_TOO_BIG"));
            assert!(l
                .check(&"x = 1;\n".repeat(700), None)
                .contains("MALT_TOO_BIG"));
            // A lower tree depth than 200.
            let mut l = from_toml_str("[lint.limits]\nmax_tree_depth = 5\n").unwrap();
            assert!(l
                .check("if a\nif b\nif c\nx = 1;\nend\nend\nend\n", None)
                .contains("MALT_TOO_DEEP"));
        }

        #[test]
        fn from_toml_rejects_invalid_limits_and_warns_about_unknown_keys() {
            for bad in [
                "[lint.limits]\nmax_file_size = 0\n",
                "[lint.limits]\nmax_tree_depth = -1\n",
                "[lint.limits]\nmax_file_size = \"lots\"\n",
            ] {
                match from_toml_str(bad) {
                    Err(e) => assert!(e.contains("[lint.limits]"), "{e}"),
                    Ok(_) => panic!("{bad} must be rejected"),
                }
            }
            let l = from_toml_str("[lint.limits]\nmax_file_sise = 1\n").unwrap();
            let warnings: Vec<String> = serde_json::from_str(&l.get_config_warnings()).unwrap();
            assert_eq!(warnings.len(), 1, "{warnings:?}");
            assert!(warnings[0].contains("max_file_size"), "{warnings:?}");
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
            // A script marked `%#codegen` triggers EMSCR (codegen checks fire only there)
            // when CODEGEN_ENGINE is on.
            let source = "%#codegen\nx = 1\n";
            let mut default_linter = linter();
            let default_warnings = default_linter.check(source, None);
            let default_parsed: Vec<serde_json::Value> =
                serde_json::from_str(&default_warnings).unwrap();
            assert!(
                !default_parsed.iter().any(|w| w["rule_id"] == "EMSCR"),
                "EMSCR should be off by default, got: {default_warnings}"
            );

            let mut l =
                from_toml_str("[lint.rules.CODEGEN_ENGINE]\nseverity = \"error\"\n").unwrap();
            let warnings = l.check(source, None);
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
    fn from_toml_honours_the_experimental_flag() {
        // CTCH is on the mathworks off-list (experimental): on only with the flag.
        let mut plain = Linter::from_toml("").unwrap();
        let mut flagged = Linter::from_toml("[lint]\nexperimental = true\n").unwrap();
        let code = "try\n  x = 1;\ncatch\nend\n";
        assert!(!rule_ids(&mut plain, code).contains(&"CTCH".to_string()));
        assert!(rule_ids(&mut flagged, code).contains(&"CTCH".to_string()));
        let mut all = Linter::from_toml("[lint]\npreset = \"all\"\n").unwrap();
        assert!(rule_ids(&mut all, code).contains(&"CTCH".to_string()));
    }

    #[test]
    fn linter_config_experimental_field() {
        let config = LinterConfig {
            experimental: true,
            ..Default::default()
        };
        let mut l = from_config(config);
        assert!(rule_ids(&mut l, "try\n  x = 1;\ncatch\nend\n").contains(&"CTCH".to_string()));
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
