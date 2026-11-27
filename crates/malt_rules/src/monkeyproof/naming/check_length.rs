//! NAMING-20: length of variable and function names.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;

use super::decls::{declarations, Kind};
use crate::monkeyproof::finding;

const ID: &str = "MP_NAMING_20";

/// Parameters of `MP_NAMING_20`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    /// The longest variable name.
    max_variable_length: usize,
    /// The shortest function name.
    min_function_length: usize,
    /// The longest function name.
    max_function_length: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            max_variable_length: 63,
            min_function_length: 3,
            max_function_length: 63,
        }
    }
}

/// Report variable names that are too long and function names that are too short or
/// too long.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    for decl in declarations(ctx.tree.root_node(), ctx.source) {
        let length = decl.name.chars().count();
        let message = match decl.kind {
            Kind::Variable if length > params.max_variable_length => format!(
                "The variable name '{}' has {length} characters; use at most {}.",
                decl.name, params.max_variable_length
            ),
            Kind::Function if length > params.max_function_length => format!(
                "The function name '{}' has {length} characters; use at most {}.",
                decl.name, params.max_function_length
            ),
            Kind::Function if length < params.min_function_length => format!(
                "The function name '{}' has {length} characters; use at least {}.",
                decl.name, params.min_function_length
            ),
            _ => continue,
        };
        out.push(finding(ctx.source, ID, decl.range, message));
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
    fn long_variable_name_is_reported_at_the_limit_plus_one() {
        let ok = "a".repeat(63);
        let long = "a".repeat(64);
        assert!(run(&format!("{ok} = 1;\n"), "").is_empty());
        let found = run(&format!("x = 1;\n{long} = 1;\n"), "");
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].0, found[0].1), (2, 1));
    }

    #[test]
    fn short_variable_names_are_fine() {
        assert!(run("x = 1;\ny = 2;\n", "").is_empty());
    }

    #[test]
    fn function_name_bounds() {
        let long = "f".repeat(64);
        let src = format!("function ab\nend\nfunction abc\nend\nfunction {long}\nend\n");
        let found = run(&src, "");
        assert_eq!(found.len(), 2);
        assert_eq!((found[0].0, found[0].1), (1, 10));
        assert_eq!(found[1].0, 5);
    }

    #[test]
    fn the_constructor_is_a_class_name() {
        let src =
            "classdef Ab\n    methods\n        function obj = Ab\n        end\n    end\nend\n";
        assert!(run(src, "").is_empty());
    }

    #[test]
    fn parameters_are_respected() {
        let toml = "[lint.rules.MP_NAMING_20]\nmax_variable_length = 3\nmin_function_length = 1\nmax_function_length = 4\n";
        let found = run("abcd = 1;\nfunction a\nend\nfunction abcde\nend\n", toml);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].0, 1);
        assert_eq!(found[1].0, 4);
    }

    #[test]
    fn broken_code_does_not_panic() {
        run("function\nx = \n", "");
    }
}
