//! MP_SECURITY_5: avoid constant variable names such as `username` and `password`.
//!
//! ## What it catches
//!
//! An assignment to a plain variable whose name is in `names` (default `username`,
//! `loginname`, `password`, `passwd`, `credentials`) when the value is a literal: a
//! non-empty string or a number. The match ignores case and underscores, so `UserName`
//! and `user_name` match `username`. An empty string is an initialisation and is silent,
//! as is a variable that gets its value from a call, which is how the credentials should
//! be obtained. Struct fields are not covered here; `Username` and `Password` properties
//! are covered by `MP_SECURITY_6`.
//!
//! ## What it does not catch
//!
//! A credential that is assigned through a variable or built from several literals.

use serde::Deserialize;

use malt_core::{Config, Diagnostic, FileContext};

use super::{for_each_node, is_empty_string};
use crate::monkeyproof::finding;

const ID: &str = "MP_SECURITY_5";

/// Parameters of `MP_SECURITY_5`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    /// The variable names to flag, compared ignoring case and underscores.
    names: Vec<String>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            names: ["username", "loginname", "password", "passwd", "credentials"]
                .map(String::from)
                .to_vec(),
        }
    }
}

fn normalise(name: &str) -> String {
    name.chars()
        .filter(|c| *c != '_')
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Run the check over a file.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    let names: Vec<String> = params.names.iter().map(|n| normalise(n)).collect();
    for_each_node(ctx.tree.root_node(), |node| {
        if node.kind() != "assignment" {
            return;
        }
        let (Some(left), Some(right)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("right"),
        ) else {
            return;
        };
        if left.kind() != "identifier" {
            return;
        }
        let is_literal = (right.kind() == "string" && !is_empty_string(right, ctx.source))
            || right.kind() == "number";
        if !is_literal {
            return;
        }
        let name = &ctx.source[left.start_byte()..left.end_byte()];
        if names.contains(&normalise(name)) {
            out.push(finding(
                ctx.source,
                ID,
                left.start_byte()..left.end_byte(),
                format!("Do not give `{name}` a constant value: ask for it or read it from a secured place."),
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::{lint, security::from_config};

    fn hits(source: &str, toml: &str) -> Vec<(usize, usize)> {
        lint(from_config, source, toml)
            .iter()
            .filter(|d| d.rule_id == "MP_SECURITY_5")
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn flags_literal_assignments() {
        let src = "username = 'me';\nPassword = \"pw\";\nuser_name = 'x';\npin = 1;\n";
        assert_eq!(hits(src, ""), vec![(1, 1), (2, 1), (3, 1)]);
    }

    #[test]
    fn obtained_values_are_silent() {
        let src = "[username, password] = showLoginDialog();\n\
                   password = input('pw');\n\
                   username = '';\n\
                   password = other;\n\
                   s.password = 'x';\n\
                   usernames = 'x';\n";
        assert!(hits(src, "").is_empty());
    }

    #[test]
    fn names_parameter() {
        let toml = "[lint.rules.MP_SECURITY_5]\nnames = [\"apikey\"]\n";
        assert_eq!(
            hits("apiKey = 'abc';\npassword = 'x';\n", toml),
            vec![(1, 1)]
        );
    }

    #[test]
    fn survives_syntax_errors() {
        let _ = hits("password = \nusername = '", "");
    }
}
