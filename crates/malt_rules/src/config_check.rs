//! Validation of the rule keys and per-rule parameters of `.malt.toml`.
//!
//! [`malt_core::Config::from_toml`] accepts any key under `[lint.rules]`, so a
//! typo such as `NOSMI = "off"` used to be silently ignored. [`config_warnings`]
//! reports each key that names no rule, with a "did you mean" suggestion, and
//! each parameter that an engine does not read. Presets and category names are
//! not checked here: `Config::from_toml` already rejects them with an error.
//!
//! A rule key is valid when it is the ID of a rule that [`crate::all_rules`]
//! builds (engines and standalone rules, including those off by default), a
//! check ID in [`malt_core::known_ids`], or a `naming.<entity>.<check>` ID.
//! Parameter names come from [`ENGINE_PARAMS`] and [`NAMING_PARAMS`], the
//! fields of each engine's config struct: a new field must be listed there.
//! Keys of check IDs that are not engines have no parameters and are not checked.

use std::collections::HashSet;

use malt_core::{config::suggest, known_ids, limits, Config};

/// The parameters each engine reads with `rule_params`, by the engine's rule ID.
const ENGINE_PARAMS: &[(&str, &[&str])] = &[
    ("SYNTAX_ERRORS_ENGINE", &["disabled_checks"]),
    ("UNSET_VARIABLES_ENGINE", &["ignore"]),
    ("CODEGEN_ENGINE", &["skip_checks"]),
    ("UNSUPPORTED_ENGINE", &["skip_checks"]),
    ("READABILITY_ENGINE", &["disabled_checks"]),
    ("NOSEMI", &[]),
    (
        "BUGS_ENGINE",
        &[
            "debug_functions",
            "higher_order_functions",
            "disabled_checks",
        ],
    ),
    ("LANGUAGE_SPEC_ENGINE", &["disabled_checks"]),
    (
        "GOOD_PRACTICES_ENGINE",
        &["max_variable_name_length", "disabled_checks"],
    ),
    (
        "INCOMPLETE_ANALYSIS",
        &["max_parse_errors", "max_paren_depth", "max_function_depth"],
    ),
    (
        "CUSTOM_CHECKS",
        &[
            "max_function_inputs",
            "max_function_outputs",
            "max_function_lines",
            "max_line_length",
            "max_nesting_depth",
            "max_cyclomatic_complexity",
            "max_script_cyclomatic_complexity",
            "max_anonymous_cyclomatic_complexity",
            "disallow_bang",
            "disallow_try_catch",
            "disallow_persistent",
            "disallow_continue",
            "disallow_break",
            "disallow_return",
            "disallow_script",
            "disallow_nested_functions",
            "disallow_command_syntax",
            "disallow_anonymous_functions",
            "disallow_char_vectors",
            "disallow_cellstr",
            "disallow_varargin",
            "disallow_varargout",
        ],
    ),
    ("UNUSED_ENGINE", &["ignore_patterns", "disabled_checks"]),
    ("SYSTEM_OBJECTS_ENGINE", &["skip_checks"]),
    ("PERFORMANCE_ENGINE", &["skip_checks"]),
    ("DEPLOYMENT_ENGINE", &["skip_checks"]),
];

/// The parameters of each `naming.<entity>.<check>` rule.
const NAMING_PARAMS: &[&str] = &[
    "max", "min", "pattern", "prefix", "phrase", "suffix", "style",
];

/// The parameters an engine reads, or `None` when the rule's config is not known.
fn engine_params(rule_id: &str) -> Option<&'static [&'static str]> {
    if rule_id.starts_with("naming.") {
        return Some(NAMING_PARAMS);
    }
    ENGINE_PARAMS
        .iter()
        .find(|(id, _)| *id == rule_id)
        .map(|(_, params)| *params)
}

/// The warnings for the rule keys and parameters of `config`, sorted, one per
/// problem. Empty when the configuration is clean.
pub fn config_warnings(config: &Config) -> Vec<String> {
    let rules = crate::all_rules(config);
    let rule_ids: HashSet<&str> = rules.iter().map(|rule| rule.id()).collect();

    // Suggestions are drawn from every ID a rule key may take.
    let mut candidates: Vec<String> = rule_ids.iter().map(|id| id.to_string()).collect();
    candidates.extend(known_ids::KNOWN_CHECK_IDS.iter().map(|id| id.to_string()));
    candidates.extend(
        crate::naming::all_naming_combinations()
            .into_iter()
            .map(|(id, _, _)| id.to_string()),
    );
    candidates.sort();
    candidates.dedup();

    let mut warnings = Vec::new();
    for (id, rule) in &config.rules {
        if !rule_ids.contains(id.as_str()) && !known_ids::is_known_check_id(id) {
            let hint = if id == "naming" {
                // `[lint.rules.naming.class.casing]` nests the keys: the rule is `naming`.
                " (a dotted rule ID must be quoted: [lint.rules.\"naming.<entity>.<check>\"])"
                    .to_string()
            } else {
                match suggest(id, candidates.iter().map(String::as_str)) {
                    Some(s) => format!(" (did you mean '{s}'?)"),
                    None => String::new(),
                }
            };
            warnings.push(format!("unknown rule ID '{id}' in [lint.rules]{hint}"));
            continue;
        }
        let Some(params) = engine_params(id) else {
            continue;
        };
        let mut keys: Vec<&String> = rule.params.keys().collect();
        keys.sort();
        for key in keys {
            if params.contains(&key.as_str()) {
                continue;
            }
            let hint = match suggest(key, params.iter().copied()) {
                Some(s) => format!(" (did you mean '{s}'?)"),
                None if params.is_empty() => " (this rule takes no parameters)".to_string(),
                None => format!(" (valid parameters: {})", params.join(", ")),
            };
            warnings.push(format!("unknown parameter '{key}' for rule '{id}'{hint}"));
        }
    }
    for key in &config.limits.unknown {
        let hint = match suggest(key, limits::LIMIT_KEYS.iter().copied()) {
            Some(s) => format!(" (did you mean '{s}'?)"),
            None => format!(" (valid keys: {})", limits::LIMIT_KEYS.join(", ")),
        };
        warnings.push(format!("unknown key '{key}' in [lint.limits]{hint}"));
    }
    warnings.sort();
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn warnings(toml: &str) -> Vec<String> {
        config_warnings(&Config::from_toml(toml).expect("valid TOML"))
    }

    #[test]
    fn misspelled_rule_key_gets_a_suggestion() {
        let w = warnings("[lint.rules]\nNOSMI = \"off\"\n");
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(
            w[0].contains("'NOSMI'") && w[0].contains("'NOSEMI'"),
            "{w:?}"
        );
    }

    #[test]
    fn misspelled_naming_key_gets_a_suggestion() {
        let w = warnings("[lint.rules]\n\"naming.clas.casing\" = \"off\"\n");
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].contains("'naming.class.casing'"), "{w:?}");
    }

    #[test]
    fn valid_keys_are_not_reported() {
        let toml = "[lint.rules]\n\
            NOSEMI = \"warn\"\n\
            FVINR = \"off\"\n\
            MSNE = \"info\"\n\
            \"naming.class.casing\" = \"off\"\n\
            NAMING_ENGINE = \"off\"\n\
            GOOD_PRACTICES_ENGINE = { max_variable_name_length = 40, disabled_checks = [\"MCHDP\"] }\n\
            [lint.rules.CUSTOM_CHECKS]\n\
            max_line_length = 120\n";
        assert!(warnings(toml).is_empty(), "{:?}", warnings(toml));
    }

    #[test]
    fn valid_naming_entry_with_style_and_pattern_is_clean() {
        let toml = "[lint.rules.\"naming.class.casing\"]\nstyle = \"camelCase\"\nmax = 20\n";
        assert!(warnings(toml).is_empty(), "{:?}", warnings(toml));
    }

    #[test]
    fn unquoted_naming_key_is_reported_with_the_quoting_hint() {
        let w = warnings("[lint.rules.naming.class.casing]\nstyle = \"camelCase\"\n");
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].contains("must be quoted"), "{w:?}");
    }

    #[test]
    fn rule_keys_are_case_sensitive() {
        let w = warnings("[lint.rules]\nnosemi = \"off\"\n");
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].contains("'NOSEMI'"), "{w:?}");
    }

    #[test]
    fn misspelled_parameter_gets_a_suggestion() {
        let w = warnings("[lint.rules.GOOD_PRACTICES_ENGINE]\nmax_variable_nam_length = 40\n");
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(
            w[0].contains("'max_variable_nam_length'")
                && w[0].contains("'max_variable_name_length'"),
            "{w:?}"
        );
    }

    #[test]
    fn parameter_of_a_disabled_engine_is_checked() {
        let w =
            warnings("[lint.rules.BUGS_ENGINE]\nseverity = \"off\"\ndisabled_check = [\"X\"]\n");
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].contains("'disabled_check'"), "{w:?}");
    }

    #[test]
    fn engine_without_parameters_reports_every_key() {
        let w = warnings("[lint.rules.NOSEMI]\nfoo = 1\n");
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].contains("takes no parameters"), "{w:?}");
    }

    #[test]
    fn severity_key_of_a_table_is_not_a_parameter() {
        assert!(warnings("[lint.rules.GOOD_PRACTICES_ENGINE]\nseverity = \"warn\"\n").is_empty());
    }

    #[test]
    fn severity_shorthand_of_a_standalone_check_is_clean() {
        assert!(warnings("[lint.rules]\nNOPTS = \"error\"\n").is_empty());
    }

    #[test]
    fn unknown_limit_key_gets_a_suggestion() {
        let w = warnings("[lint.limits]\nmax_file_sise = \"1 MiB\"\nfoo = 1\nmax_threads = 2\n");
        assert_eq!(w.len(), 2, "{w:?}");
        assert!(
            w[0].contains("'foo'") && w[0].contains("valid keys"),
            "{w:?}"
        );
        assert!(
            w[1].contains("'max_file_sise'") && w[1].contains("'max_file_size'"),
            "{w:?}"
        );
    }

    #[test]
    fn valid_limits_are_clean() {
        let toml = "[lint.limits]\nmax_file_size = \"8 MiB\"\nmax_tree_depth = 500\n\
                    max_nesting = 5000\nmax_threads = 2\n";
        assert!(warnings(toml).is_empty(), "{:?}", warnings(toml));
    }

    #[test]
    fn suggest_finds_close_names_only() {
        let names = ["NOSEMI", "NOPTS", "FVINR"];
        assert_eq!(suggest("NOSMI", names), Some("NOSEMI"));
        assert_eq!(suggest("nosemi", names), Some("NOSEMI"));
        assert_eq!(suggest("ZZZZZZ", names), None);
    }
}
