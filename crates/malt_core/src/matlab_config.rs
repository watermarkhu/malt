//! MATLAB's Code Analyzer configuration file, `codeAnalyzerConfiguration.json`.
//!
//! MATLAB reads it from a `resources` folder (for `codeIssues` and the editor) or
//! from the path given to `checkcode(file, '-config=<path>')`. [`parse`] reads the
//! part of it that malt supports into a [`Config`], and [`layer`] puts a
//! `.malt.toml` over that configuration. The mapping and what is left out are
//! described in `docs/configuration.md`, and the evidence in `docs/VERIFY.md`.
//!
//! What malt takes from the file:
//!
//! - `checks.<ID>.enabled` and `severity` become [`Config::check_settings`], keyed by
//!   the check ID, so they apply to whichever engine emits the check.
//! - `checks.<threshold check>.limit` sets the matching parameter of the custom
//!   checks engine. A threshold check that is enabled without a `limit` reports
//!   nothing, as it does in MATLAB.
//! - `checks.DAF*` and `checks.SYSBANG` switch on the matching `disallow_*` parameter.
//! - `naming.<entity>.<check>` sets the parameters of the naming check of that ID.
//! - `ignoredFiles` is returned to the caller, which skips those files.
//!
//! Custom checks (`rule`), `messageText` and `formatting` are not supported: they
//! are reported as warnings and ignored.

use std::collections::HashMap;

use serde_json::Value;

use crate::config::{CheckSetting, Config, RuleConfig};
use crate::diagnostic::Severity;
use crate::known_ids;

/// The entity groups of the `naming` section.
const ENTITIES: &[&str] = &[
    "class",
    "enumeration",
    "event",
    "function",
    "localFunction",
    "method",
    "nestedFunction",
    "property",
    "variable",
];

/// The casing styles MATLAB accepts in `casing`.
const CASINGS: &[&str] = &[
    "lowerCamelCase",
    "UpperCamelCase",
    "lowercase_with_underscores",
    "UPPERCASE_WITH_UNDERSCORES",
    "lowercase",
    "UPPERCASE",
];

/// The MATLAB checks that take a `limit`, with the CUSTOM_CHECKS parameter it sets.
const LIMITS: &[(&str, &str)] = &[
    ("FCNIL", "max_function_inputs"),
    ("FCNOL", "max_function_outputs"),
    ("FCNLL", "max_function_lines"),
    ("LLMNC", "max_line_length"),
    ("MNCSN", "max_nesting_depth"),
    ("CYCCOM", "max_cyclomatic_complexity"),
    ("MCYCCOM", "max_cyclomatic_complexity"),
    ("SCYCCOM", "max_script_cyclomatic_complexity"),
    ("MSCYCCOM", "max_script_cyclomatic_complexity"),
    ("ACYCCOM", "max_anonymous_cyclomatic_complexity"),
    ("MACYCCOM", "max_anonymous_cyclomatic_complexity"),
];

/// The MATLAB checks that `enabled` switches on, with the CUSTOM_CHECKS flag it sets.
const DISALLOW: &[(&str, &str)] = &[
    ("SYSBANG", "disallow_bang"),
    ("DAFTC", "disallow_try_catch"),
    ("DAFPV", "disallow_persistent"),
    ("DAFCO", "disallow_continue"),
    ("DAFBR", "disallow_break"),
    ("DAFRT", "disallow_return"),
    ("DAFSC", "disallow_script"),
    ("DAFNF", "disallow_nested_functions"),
    ("DAFCF", "disallow_command_syntax"),
    ("DAFAF", "disallow_anonymous_functions"),
    ("DAFCV", "disallow_char_vectors"),
    ("DAFCVC", "disallow_cellstr"),
    ("DAFVI", "disallow_varargin"),
    ("DAFVO", "disallow_varargout"),
];

/// The keys of a check object.
const CHECK_FIELDS: &[&str] = &["enabled", "severity", "limit", "rule", "messageText"];

/// The keys at the top of the file.
const TOP_FIELDS: &[&str] = &[
    "$schema",
    "schemaVersion",
    "baseConfiguration",
    "checks",
    "naming",
    "ignoredFiles",
    "formatting",
];

/// A configuration read from a MATLAB configuration file.
#[derive(Debug, Clone)]
pub struct MatlabConfig {
    /// The malt configuration: check settings, custom-check and naming parameters.
    pub config: Config,
    /// The `ignoredFiles` patterns, as written (relative to the folder that holds the
    /// `resources` folder of the file).
    pub ignored_files: Vec<String>,
    /// Problems that MATLAB reports as warnings, each `path: problem`. The
    /// settings they name are ignored.
    pub warnings: Vec<String>,
}

/// Read a MATLAB Code Analyzer configuration.
///
/// Returns the problems MATLAB reports as errors (the file is invalid: `codeIssues`
/// refuses it) as `Err`, one message per problem. Warnings are in
/// [`MatlabConfig::warnings`].
pub fn parse(text: &str) -> Result<MatlabConfig, Vec<String>> {
    let root: Value =
        serde_json::from_str(text).map_err(|e| vec![format!("not valid JSON: {e}")])?;
    let Some(root) = root.as_object() else {
        return Err(vec!["must be a JSON object".to_string()]);
    };

    let mut problems = Problems::default();
    let mut built = Built::default();

    schema_version(root.get("schemaVersion"), &mut problems);
    if let Some(base) = root.get("baseConfiguration") {
        match base.as_str() {
            Some("factory" | "closestParentFolder") => {}
            _ => problems.warn(
                "baseConfiguration",
                "must be \"factory\" or \"closestParentFolder\"; ignored",
            ),
        }
    }
    if let Some(checks) = root.get("checks") {
        match checks.as_object() {
            Some(checks) => {
                for (name, value) in checks {
                    check(name, value, &mut problems, &mut built);
                }
            }
            None => problems.error("checks", "must be a JSON object"),
        }
    }
    if let Some(naming) = root.get("naming") {
        naming_section(naming, &mut problems, &mut built);
    }
    if let Some(ignored) = root.get("ignoredFiles") {
        ignored_files(ignored, &mut problems, &mut built);
    }
    if root.contains_key("formatting") {
        problems.warn("formatting", "is not used by malt; ignored");
    }
    for key in root.keys() {
        if !TOP_FIELDS.contains(&key.as_str()) {
            problems.warn(key, "unknown field; ignored");
        }
    }

    if !problems.errors.is_empty() {
        return Err(problems.errors);
    }
    let ignored_files = std::mem::take(&mut built.ignored);
    Ok(MatlabConfig {
        config: built.into_config(),
        ignored_files,
        warnings: problems.warnings,
    })
}

/// Put a `.malt.toml` configuration over a MATLAB one: the TOML rule and category
/// entries replace the JSON ones key by key, and the TOML's own settings (preset,
/// exclude, inline suppression, `matlab_config`) apply. Check settings come from the
/// JSON file only.
pub fn layer(json: Config, toml: Config) -> Config {
    let mut rules: HashMap<String, RuleConfig> = json.rules;
    rules.extend(toml.rules);
    let mut categories = json.categories;
    categories.extend(toml.categories);
    Config {
        exclude: toml.exclude,
        rules,
        categories,
        preset: toml.preset,
        inline_suppression: toml.inline_suppression,
        check_settings: json.check_settings,
        matlab_config: toml.matlab_config,
        limits: toml.limits,
    }
}

/// The problems found while reading a file: errors make it invalid, warnings are
/// ignored settings.
#[derive(Default)]
struct Problems {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl Problems {
    fn error(&mut self, at: &str, what: &str) {
        self.errors.push(format!("{at}: {what}"));
    }

    fn warn(&mut self, at: &str, what: &str) {
        self.warnings.push(format!("{at}: {what}"));
    }
}

/// What the file says, in malt's terms, before it becomes a [`Config`].
#[derive(Default)]
struct Built {
    settings: HashMap<String, CheckSetting>,
    custom_params: toml::Table,
    custom_on: bool,
    naming: HashMap<String, toml::Table>,
    ignored: Vec<String>,
}

impl Built {
    fn into_config(self) -> Config {
        let mut config = Config::default();
        if self.custom_on {
            config.rules.insert(
                "CUSTOM_CHECKS".to_string(),
                RuleConfig {
                    enabled: true,
                    severity: None,
                    params: self.custom_params,
                },
            );
        }
        if !self.naming.is_empty() {
            config.rules.insert(
                "NAMING_ENGINE".to_string(),
                RuleConfig {
                    enabled: true,
                    severity: None,
                    params: toml::Table::new(),
                },
            );
        }
        for (id, params) in self.naming {
            config.rules.insert(
                id,
                RuleConfig {
                    enabled: true,
                    severity: None,
                    params,
                },
            );
        }
        let mut settings = self.settings;
        // Custom checks are off unless the file enables them.
        for (id, _) in LIMITS.iter().chain(DISALLOW) {
            settings.entry((*id).to_string()).or_insert(CheckSetting {
                enabled: Some(false),
                severity: None,
            });
        }
        config.check_settings = settings;
        config
    }
}

fn schema_version(value: Option<&Value>, problems: &mut Problems) {
    let Some(value) = value else { return };
    let Some(text) = value.as_str() else {
        problems.error("schemaVersion", "must be a string such as \"1.1.0\"");
        return;
    };
    let parts: Option<Vec<u64>> = text.split('.').map(|p| p.parse().ok()).collect();
    let Some(parts) = parts.filter(|parts| parts.len() == 3) else {
        problems.error(
            "schemaVersion",
            "must be \"<major>.<minor>.<patch>\" with numbers",
        );
        return;
    };
    let (major, minor) = (parts[0], parts[1]);
    if major != 1 {
        problems.error(
            "schemaVersion",
            &format!("{text} is not supported (malt reads 1.x)"),
        );
    } else if minor > 1 {
        problems.warn(
            "schemaVersion",
            &format!("{text} is newer than 1.1.0; fields malt does not know are ignored"),
        );
    }
}

fn check(name: &str, value: &Value, problems: &mut Problems, built: &mut Built) {
    let at = format!("checks.{name}");
    let Some(fields) = value.as_object() else {
        problems.error(&at, "must be a JSON object");
        return;
    };
    let builtin = known_ids::is_known_check_id(name) && !name.starts_with("naming.");
    if !builtin && !fields.contains_key("rule") {
        problems.warn(
            &at,
            "unknown check: not a MATLAB check ID and no rule; ignored",
        );
        return;
    }
    if !builtin && !is_matlab_name(name) {
        problems.error(&at, "a custom check name must be a MATLAB identifier");
        return;
    }

    let mut enabled = None;
    let mut severity = None;
    let mut limit = None;
    for (key, field) in fields {
        match key.as_str() {
            "enabled" => match field.as_bool() {
                Some(on) => enabled = Some(on),
                None => problems.error(&format!("{at}.enabled"), "must be true or false"),
            },
            "severity" => match field.as_str() {
                Some("error") => severity = Some(Severity::Error),
                Some("warning") => severity = Some(Severity::Warning),
                Some("info") => severity = Some(Severity::Info),
                Some(_) => problems.warn(
                    &format!("{at}.severity"),
                    "must be \"error\", \"warning\" or \"info\"; ignored",
                ),
                None => problems.error(&format!("{at}.severity"), "must be a string"),
            },
            "limit" => match field.as_u64() {
                Some(n) => limit = Some(n),
                None => problems.error(&format!("{at}.limit"), "must be a nonnegative integer"),
            },
            "rule" => {}
            "messageText" => problems.warn(
                &format!("{at}.messageText"),
                "is not supported; malt keeps its own message",
            ),
            _ if CHECK_FIELDS.contains(&key.as_str()) => {}
            _ => problems.warn(&format!("{at}.{key}"), "unknown field; ignored"),
        }
    }

    if let Some(rule) = fields.get("rule") {
        if builtin {
            problems.error(&at, "a built-in check cannot be redefined with a rule");
            return;
        }
        custom_rule(&at, rule, problems);
        problems.warn(
            &at,
            "custom checks are not supported by malt; this check is ignored",
        );
        return;
    }

    let threshold = LIMITS.iter().find(|(id, _)| *id == name);
    let disallow = DISALLOW.iter().find(|(id, _)| *id == name);
    if limit.is_some() && threshold.is_none() {
        problems.warn(
            &format!("{at}.limit"),
            "is not a field of this check; ignored",
        );
    }
    if threshold.is_some() || disallow.is_some() {
        let on = enabled == Some(true) && (disallow.is_some() || limit.is_some());
        if let Some((_, param)) = threshold.filter(|_| on) {
            let value = i64::try_from(limit.unwrap_or_default()).unwrap_or(i64::MAX);
            match built
                .custom_params
                .get(*param)
                .and_then(toml::Value::as_integer)
            {
                Some(first) if first != value => problems.warn(
                    &at,
                    &format!(
                        "shares malt's {param} with another check, which has {first}; using {first}"
                    ),
                ),
                Some(_) => {}
                None => {
                    built
                        .custom_params
                        .insert((*param).to_string(), toml::Value::Integer(value));
                }
            }
        }
        if let Some((_, param)) = disallow.filter(|_| on) {
            built
                .custom_params
                .insert((*param).to_string(), toml::Value::Boolean(true));
        }
        built.custom_on |= on;
        built.settings.insert(
            name.to_string(),
            CheckSetting {
                enabled: Some(on),
                severity,
            },
        );
        return;
    }

    if enabled.is_some() || severity.is_some() {
        built
            .settings
            .insert(name.to_string(), CheckSetting { enabled, severity });
    }
}

/// The problems of a custom check's `rule`: it must name a template with the field
/// that template needs. Any template is unsupported, so nothing else is kept.
fn custom_rule(at: &str, rule: &Value, problems: &mut Problems) {
    let at = format!("{at}.rule");
    let Some(fields) = rule.as_object() else {
        problems.error(&at, "must be a JSON object");
        return;
    };
    let Some(template) = fields.get("template") else {
        problems.error(&at, "must have a template field");
        return;
    };
    let Some(template) = template.as_str() else {
        problems.error(&format!("{at}.template"), "must be a string");
        return;
    };
    let (needed, allowed) = match template {
        "functionCall" => ("functionNames", ["template", "functionNames"]),
        "variableName" => ("variableNames", ["template", "variableNames"]),
        _ => {
            problems.warn(
                &format!("{at}.template"),
                &format!("unknown template '{template}'"),
            );
            return;
        }
    };
    match fields.get(needed) {
        None => problems.error(
            &at,
            &format!("template '{template}' needs the field '{needed}'"),
        ),
        Some(value) if !is_text_or_text_list(value) => problems.error(
            &format!("{at}.{needed}"),
            "must be a non-empty string or array of strings",
        ),
        Some(_) => {}
    }
    for key in fields.keys() {
        if !allowed.contains(&key.as_str()) {
            problems.warn(&format!("{at}.{key}"), "unknown field; ignored");
        }
    }
}

fn naming_section(naming: &Value, problems: &mut Problems, built: &mut Built) {
    let Some(groups) = naming.as_object() else {
        problems.error("naming", "must be a JSON object");
        return;
    };
    for (group, fields) in groups {
        let at = format!("naming.{group}");
        if !ENTITIES.contains(&group.as_str()) {
            problems.warn(&at, "unknown group; ignored");
            continue;
        }
        let Some(fields) = fields.as_object() else {
            problems.error(&at, "must be a JSON object");
            continue;
        };
        for (check, value) in fields {
            let at = format!("{at}.{check}");
            let id = format!("naming.{group}.{check}");
            let mut params = toml::Table::new();
            match check.as_str() {
                "maxLength" | "minLength" => match value.as_u64() {
                    Some(n) if (1..=2048).contains(&n) => {
                        let key = if check == "maxLength" { "max" } else { "min" };
                        params.insert(key.to_string(), toml::Value::Integer(n as i64));
                    }
                    Some(_) => problems.warn(&at, "must be between 1 and 2048; ignored"),
                    None => problems.error(&at, "must be a nonnegative integer"),
                },
                "regularExpression" => match value.as_str() {
                    Some(pattern) if !pattern.is_empty() => {
                        if regex::Regex::new(&format!("^(?:{pattern})$")).is_ok() {
                            params.insert(
                                "pattern".to_string(),
                                toml::Value::String(pattern.to_string()),
                            );
                        } else {
                            problems.warn(&at, "is not a valid regular expression; ignored");
                        }
                    }
                    Some(_) => problems.error(&at, "must be a non-empty string"),
                    None => problems.error(&at, "must be a string"),
                },
                "casing" => match value.as_str() {
                    Some(style) if CASINGS.contains(&style) => {
                        params.insert("style".to_string(), toml::Value::String(style.to_string()));
                    }
                    Some(_) => problems.warn(&at, "must be one of the six casing styles; ignored"),
                    None => problems.error(&at, "must be a string"),
                },
                "requiredPrefix" | "disallowedPrefix" | "disallowedPhrase" | "requiredSuffix"
                | "disallowedSuffix" => {
                    let key = match check.as_str() {
                        "disallowedPhrase" => "phrase",
                        "requiredPrefix" | "disallowedPrefix" => "prefix",
                        _ => "suffix",
                    };
                    if is_text_or_text_list(value) {
                        params.insert(key.to_string(), text_or_list(value));
                    } else {
                        problems.error(&at, "must be a non-empty string or array of strings");
                    }
                }
                _ => {
                    problems.warn(&at, "unknown check; ignored");
                    continue;
                }
            }
            if !params.is_empty() {
                built.naming.entry(id).or_default().extend(params);
            }
        }
    }
}

fn ignored_files(value: &Value, problems: &mut Problems, built: &mut Built) {
    let patterns: Vec<&Value> = match value {
        Value::Array(items) => items.iter().collect(),
        Value::String(_) => vec![value],
        _ => {
            problems.error("ignoredFiles", "must be a string or an array of strings");
            return;
        }
    };
    for pattern in patterns {
        let Some(pattern) = pattern.as_str() else {
            problems.error("ignoredFiles", "must be a string or an array of strings");
            continue;
        };
        if pattern.contains("..") {
            problems.error("ignoredFiles", "patterns containing '..' are not supported");
            continue;
        }
        built
            .ignored
            .push(pattern.trim_start_matches('/').to_string());
    }
}

/// A non-empty string, or a non-empty array of non-empty strings.
fn is_text_or_text_list(value: &Value) -> bool {
    match value {
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => {
            !items.is_empty()
                && items
                    .iter()
                    .all(|item| item.as_str().is_some_and(|text| !text.is_empty()))
        }
        _ => false,
    }
}

/// The value of [`is_text_or_text_list`] as a TOML value: a string or an array.
fn text_or_list(value: &Value) -> toml::Value {
    match value {
        Value::String(text) => toml::Value::String(text.clone()),
        Value::Array(items) => toml::Value::Array(
            items
                .iter()
                .filter_map(|item| item.as_str().map(|s| toml::Value::String(s.to_string())))
                .collect(),
        ),
        _ => toml::Value::String(String::new()),
    }
}

/// A MATLAB identifier, or identifiers separated by periods.
fn is_matlab_name(name: &str) -> bool {
    !name.is_empty()
        && name.split('.').all(|part| {
            let mut chars = part.chars();
            matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(text: &str) -> MatlabConfig {
        parse(text).unwrap_or_else(|e| panic!("expected a valid file, got {e:?}"))
    }

    fn errors(text: &str) -> Vec<String> {
        parse(text).expect_err("expected errors")
    }

    #[test]
    fn empty_object_is_the_default_configuration() {
        let config = ok(r#"{"schemaVersion":"1.1.0"}"#);
        assert!(config
            .config
            .check_settings
            .values()
            .all(|s| s.enabled == Some(false)));
        assert!(config.config.rules.is_empty());
        assert!(config.warnings.is_empty());
    }

    #[test]
    fn disable_and_severity_are_check_settings() {
        let config = ok(r#"{"checks":{"NOPRT":{"enabled":false},"NASGU":{"severity":"error"}}}"#);
        let settings = &config.config.check_settings;
        assert_eq!(settings["NOPRT"].enabled, Some(false));
        assert_eq!(settings["NASGU"].severity, Some(Severity::Error));
        assert_eq!(settings["NASGU"].enabled, None);
    }

    #[test]
    fn threshold_needs_enabled_and_a_limit() {
        let config =
            ok(r#"{"checks":{"FCNLL":{"enabled":true,"limit":3},"CYCCOM":{"enabled":true}}}"#);
        let rule = &config.config.rules["CUSTOM_CHECKS"];
        assert_eq!(rule.params["max_function_lines"].as_integer(), Some(3));
        assert!(!rule.params.contains_key("max_cyclomatic_complexity"));
        assert_eq!(config.config.check_settings["CYCCOM"].enabled, Some(false));
    }

    #[test]
    fn disallow_checks_set_their_flag() {
        let config = ok(r#"{"checks":{"DAFSC":{"enabled":true}}}"#);
        let rule = &config.config.rules["CUSTOM_CHECKS"];
        assert_eq!(rule.params["disallow_script"].as_bool(), Some(true));
        assert_eq!(config.config.check_settings["DAFSC"].enabled, Some(true));
    }

    #[test]
    fn custom_checks_are_unsupported_warnings() {
        let config = ok(
            r#"{"checks":{"noDisp":{"severity":"error","rule":{"template":"functionCall","functionNames":["disp"]}}}}"#,
        );
        assert!(config.warnings.iter().any(|w| w.contains("checks.noDisp")));
        assert!(!config.config.check_settings.contains_key("noDisp"));
    }

    #[test]
    fn naming_values_map_to_check_parameters() {
        let config = ok(
            r#"{"naming":{"variable":{"maxLength":6,"casing":"lowerCamelCase","disallowedSuffix":"x","requiredPrefix":["a","b"]}}}"#,
        );
        let max = &config.config.rules["naming.variable.maxLength"];
        assert_eq!(max.params["max"].as_integer(), Some(6));
        let casing = &config.config.rules["naming.variable.casing"];
        assert_eq!(casing.params["style"].as_str(), Some("lowerCamelCase"));
        let suffix = &config.config.rules["naming.variable.disallowedSuffix"];
        assert_eq!(suffix.params["suffix"].as_str(), Some("x"));
        assert!(config.config.rules["NAMING_ENGINE"].enabled);
    }

    #[test]
    fn invalid_values_are_errors_and_unknown_keys_warnings() {
        assert!(errors(r#"{"checks":{"NOPRT":{"enabled":"no"}}}"#)
            .iter()
            .any(|e| e.contains("checks.NOPRT.enabled")));
        assert!(errors("{").iter().any(|e| e.contains("not valid JSON")));
        assert!(errors("[]").iter().any(|e| e.contains("JSON object")));
        assert!(errors(r#"{"checks":{"CYCCOM":{"limit":"x"}}}"#)
            .iter()
            .any(|e| e.contains("nonnegative integer")));
        let config = ok(r#"{"checks":{"NOSMI":{"enabled":false}},"bogus":1}"#);
        assert_eq!(config.warnings.len(), 2);
    }

    #[test]
    fn severity_spelling_is_checked() {
        let config = ok(r#"{"checks":{"NOPRT":{"severity":"warn"}}}"#);
        assert!(!config.config.check_settings.contains_key("NOPRT"));
        assert_eq!(config.warnings.len(), 1);
        assert!(errors(r#"{"schemaVersion":"2.0.0"}"#)
            .iter()
            .any(|e| e.contains("not supported")));
    }

    #[test]
    fn ignored_files_are_returned() {
        let config = ok(r#"{"ignoredFiles":["sub/*.m","/a.m"]}"#);
        assert_eq!(config.ignored_files, vec!["sub/*.m", "a.m"]);
        assert!(errors(r#"{"ignoredFiles":["../a.m"]}"#)
            .iter()
            .any(|e| e.contains("ignoredFiles")));
    }

    #[test]
    fn toml_layers_over_json() {
        let json = ok(r#"{"checks":{"NOPRT":{"enabled":false}}}"#).config;
        let toml = Config::from_toml("[lint.rules]\nNOPRT = \"warn\"\n").unwrap();
        let merged = layer(json, toml);
        assert_eq!(merged.check_settings["NOPRT"].enabled, Some(false));
        assert_eq!(merged.rules["NOPRT"].severity, Some(Severity::Warning));
    }
}
