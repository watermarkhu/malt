//! NAMING-15: placeholder names such as `temp...`, `my...` or `testFunction`.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;

use super::decls::{declarations, Kind};
use crate::monkeyproof::finding;

const ID: &str = "MP_NAMING_15";

/// Parameters of `MP_NAMING_15`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    /// Prefixes that mark a placeholder name.
    prefixes: Vec<String>,
    /// Whole names that are placeholders (compared ignoring case).
    names: Vec<String>,
}

impl Default for Params {
    fn default() -> Self {
        let owned = |list: &[&str]| list.iter().map(|s| s.to_string()).collect();
        Self {
            prefixes: owned(&["temp", "my"]),
            names: owned(&[
                "testFunction",
                "testFcn",
                "testClass",
                "testVariable",
                "testVar",
                "testProperty",
                "testField",
                "testStruct",
                "testMethod",
            ]),
        }
    }
}

/// Whether `name` starts with `prefix` as a word: the prefix, ignoring case, is followed
/// by an uppercase letter, a digit, an underscore or the end of the name. `temperature`
/// and `myriad` start with the letters but not the word. In an all-uppercase name the
/// next character must be a digit, an underscore or the end.
fn starts_with_word(name: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return false;
    }
    let head: String = name.chars().take(prefix.chars().count()).collect();
    if !head.eq_ignore_ascii_case(prefix) {
        return false;
    }
    let rest = &name[head.len()..];
    let shouting = !name.chars().any(|c| c.is_ascii_lowercase());
    match rest.chars().next() {
        None => true,
        Some(c) if c.is_ascii_digit() || c == '_' => true,
        Some(c) => c.is_ascii_uppercase() && !shouting,
    }
}

/// Report class, function, variable, property and field names that are placeholders.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    for decl in declarations(ctx.tree.root_node(), ctx.source) {
        let listed = params
            .names
            .iter()
            .any(|n| n.eq_ignore_ascii_case(decl.name));
        let prefixed = params
            .prefixes
            .iter()
            .any(|p| starts_with_word(decl.name, p));
        if !(listed || prefixed) {
            continue;
        }
        let what = match decl.kind {
            Kind::Class => "class",
            Kind::Function => "function",
            Kind::Variable => "variable",
            Kind::Property => "property",
            Kind::Field => "field",
        };
        out.push(finding(
            ctx.source,
            ID,
            decl.range,
            format!(
                "The {what} name '{}' says nothing about its purpose; pick a descriptive name.",
                decl.name
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::lint;
    use crate::monkeyproof::naming::from_config;

    fn run(src: &str, toml: &str) -> Vec<(usize, usize, String)> {
        lint(from_config, src, toml)
            .into_iter()
            .filter(|d| d.rule_id == super::ID)
            .map(|d| (d.line, d.column, d.message))
            .collect()
    }

    #[test]
    fn prefixed_variables_are_reported() {
        let found = run("tempData = 1;\nmyValue = 2;\ntemp = 3;\ntemp1 = 4;\n", "");
        assert_eq!(found.len(), 4);
        assert_eq!((found[1].0, found[1].1), (2, 1));
    }

    #[test]
    fn words_that_merely_start_with_the_letters_are_silent() {
        assert!(run(
            "temperature = 1;\nmyriad = 2;\nmystery = 3;\nTEMPERATURE = 4;\n",
            ""
        )
        .is_empty());
    }

    #[test]
    fn upper_case_constants_use_underscores() {
        assert_eq!(run("TEMP_LIMIT = 1;\n", "").len(), 1);
    }

    #[test]
    fn listed_names_are_reported_for_every_kind() {
        let src = "classdef MyClass\n    properties\n        tempProp\n    end\nend\n";
        assert_eq!(run(src, "").len(), 2);
        let src = "function testFunction\n    s.myField = 1;\nend\n";
        let found = run(src, "");
        assert_eq!(found.len(), 2);
        assert_eq!((found[0].0, found[0].1), (1, 10));
    }

    #[test]
    fn a_variable_is_reported_once_per_scope() {
        assert_eq!(run("tmp = 1;\ntempA = 1;\ntempA = 2;\n", "").len(), 1);
    }

    #[test]
    fn parameters_are_respected() {
        let toml = "[lint.rules.MP_NAMING_15]\nprefixes = [\"foo\"]\nnames = [\"bar\"]\n";
        assert!(run("tempData = 1;\n", toml).is_empty());
        assert_eq!(run("fooData = 1;\nbar = 2;\n", toml).len(), 2);
    }

    #[test]
    fn broken_code_does_not_panic() {
        run("tempX = \nfunction\n", "");
    }
}
