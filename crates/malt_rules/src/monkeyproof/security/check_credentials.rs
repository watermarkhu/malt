//! MP_SECURITY_6: no hard-coded `Username` and `Password` values.
//!
//! ## What it catches
//!
//! A non-empty string literal given as the value of an `option` name-value pair
//! (default `Username`, `Password`, matched ignoring case) in a call to one of `functions`
//! (default `weboptions`, `webread`, `webwrite`, `websave`, `matlab.net.http.Credentials`
//! and `Credentials`), a non-empty string literal in the user name or password position
//! (arguments 2 and 3) of a call to one of `positional` (default `database`, `ftp`,
//! `sftp`), and a non-empty string literal assigned to a `.Username` or `.Password`
//! property. The finding is at the literal. A plain variable that is named `username`
//! is `MP_SECURITY_5`.
//!
//! ## What it does not catch
//!
//! Credentials in a variable, in a struct built elsewhere, or in a function that is not in
//! the lists.

use serde::Deserialize;

use malt_core::{Config, Diagnostic, FileContext};

use super::{args_of, for_each_node, is_empty_string, string_value};
use crate::analysis::callee::{callee_path, Dotted};
use crate::monkeyproof::finding;

const ID: &str = "MP_SECURITY_6";

/// Parameters of `MP_SECURITY_6`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    /// Functions that take credentials as name-value pairs.
    functions: Vec<String>,
    /// Functions that take a user name and a password as arguments 2 and 3.
    positional: Vec<String>,
    /// The option and property names that hold credentials.
    options: Vec<String>,
}

impl Default for Params {
    fn default() -> Self {
        let list = |items: &[&str]| items.iter().map(|s| s.to_string()).collect();
        Self {
            functions: list(&[
                "weboptions",
                "webread",
                "webwrite",
                "websave",
                "matlab.net.http.Credentials",
                "Credentials",
            ]),
            positional: list(&["database", "ftp", "sftp"]),
            options: list(&["Username", "Password"]),
        }
    }
}

/// Run the check over a file.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    let is_option = |text: &str| params.options.iter().any(|o| o.eq_ignore_ascii_case(text));
    let message =
        "Do not hard-code a user name or password: ask for it or read it from a secured place.";
    let hard_coded =
        |node: tree_sitter::Node| node.kind() == "string" && !is_empty_string(node, ctx.source);
    for_each_node(ctx.tree.root_node(), |node| match node.kind() {
        "function_call" => {
            let Some(name) = callee_path(node, ctx.source, Dotted::Whole) else {
                return;
            };
            let args = args_of(node);
            if params.functions.iter().any(|f| f == name) {
                for pair in args.windows(2) {
                    let named = string_value(pair[0], ctx.source).is_some_and(&is_option);
                    if named && hard_coded(pair[1]) {
                        out.push(finding(
                            ctx.source,
                            ID,
                            pair[1].start_byte()..pair[1].end_byte(),
                            message,
                        ));
                    }
                }
            }
            if params.positional.iter().any(|f| f == name) {
                for arg in args.iter().skip(1).take(2).filter(|a| hard_coded(**a)) {
                    out.push(finding(
                        ctx.source,
                        ID,
                        arg.start_byte()..arg.end_byte(),
                        message,
                    ));
                }
            }
        }
        "assignment" => {
            let (Some(left), Some(right)) = (
                node.child_by_field_name("left"),
                node.child_by_field_name("right"),
            ) else {
                return;
            };
            if left.kind() != "field_expression" {
                return;
            }
            let Some(field) = super::last_field(left) else {
                return;
            };
            if is_option(&ctx.source[field.start_byte()..field.end_byte()]) && hard_coded(right) {
                out.push(finding(
                    ctx.source,
                    ID,
                    right.start_byte()..right.end_byte(),
                    message,
                ));
            }
        }
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::{lint, security::from_config};

    fn hits(source: &str, toml: &str) -> Vec<(usize, usize)> {
        lint(from_config, source, toml)
            .iter()
            .filter(|d| d.rule_id == "MP_SECURITY_6")
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn flags_name_value_pairs() {
        let src = "o = weboptions(\"Username\", \"John\", \"Password\", 'secret');\n";
        assert_eq!(hits(src, ""), vec![(1, 28), (1, 48)]);
        let src = "c = matlab.net.http.Credentials('Username', 'John');\n";
        assert_eq!(hits(src, ""), vec![(1, 45)]);
    }

    #[test]
    fn flags_positional_and_properties() {
        assert_eq!(
            hits("c = database('src', 'me', 'pw');\n", ""),
            vec![(1, 21), (1, 27)]
        );
        assert_eq!(hits("o.Password = 'pw';\n", ""), vec![(1, 14)]);
    }

    #[test]
    fn variables_and_empty_values_are_silent() {
        let src = "o = weboptions('Username', user, 'Password', pw);\n\
                   p = weboptions('Username', '');\n\
                   q = other('Username', 'John');\n\
                   r = database('src', user, pw);\n\
                   o.Password = pw;\n\
                   o.Timeout = 'x';\n";
        assert!(hits(src, "").is_empty());
    }

    #[test]
    fn parameters() {
        let toml = "[lint.rules.MP_SECURITY_6]\nfunctions = [\"other\"]\npositional = []\noptions = [\"Token\"]\n";
        let src = "q = other('Token', 'abc');\nw = weboptions('Token', 'abc');\nd = database('s','a','b');\n";
        assert_eq!(hits(src, toml), vec![(1, 20)]);
    }

    #[test]
    fn survives_syntax_errors() {
        let _ = hits("weboptions('Username',\no.Password = '", "");
    }
}
