//! MP_SECURITY_1: do not use `eval`, minimize `feval`, `evalin` and `evalc`.
//!
//! ## What it catches
//!
//! A call or command-syntax use of a function from `functions` (default `eval`, `evalin`,
//! `evalc`), and of a function from `literal_only` (default `feval`) when its first
//! argument is a string literal. `feval('sin', x)` is a call written the long way, while
//! `feval(fn, x)` and `feval(@sin, x)` have a reason to exist and stay silent.
//!
//! ## What it does not catch
//!
//! A variable or local function that shadows one of the names, and an alias made with a
//! function handle (`f = @eval; f(s)`).

use serde::Deserialize;

use malt_core::{Config, Diagnostic, FileContext};

use super::{args_of, for_each_node, is_string};
use crate::analysis::callee::{callee_path, Dotted};
use crate::monkeyproof::finding;

const ID: &str = "MP_SECURITY_1";

/// Parameters of `MP_SECURITY_1`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    /// Functions flagged whenever they are used.
    functions: Vec<String>,
    /// Functions flagged only when the first argument is a string literal.
    literal_only: Vec<String>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            functions: vec!["eval".into(), "evalin".into(), "evalc".into()],
            literal_only: vec!["feval".into()],
        }
    }
}

/// Run the check over a file.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    for_each_node(ctx.tree.root_node(), |node| {
        let Some(name) = callee_path(node, ctx.source, Dotted::Skip) else {
            return;
        };
        if node.kind() == "handle_operator" {
            return;
        }
        let flagged = if params.functions.iter().any(|f| f == name) {
            true
        } else if params.literal_only.iter().any(|f| f == name) {
            args_of(node).first().is_some_and(|first| is_string(*first))
                || (node.kind() == "command")
        } else {
            false
        };
        if flagged {
            out.push(finding(
                ctx.source,
                ID,
                node.start_byte()..node.end_byte(),
                format!("Avoid `{name}`: call the function directly or restructure the code."),
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::{lint, security::from_config};

    fn ids(source: &str, toml: &str) -> Vec<(usize, usize)> {
        lint(from_config, source, toml)
            .iter()
            .filter(|d| d.rule_id == "MP_SECURITY_1")
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn flags_eval_evalin_evalc() {
        let src = "eval('a = 1');\nx = evalc('disp(1)');\nevalin('base', 'a');\n";
        assert_eq!(ids(src, ""), vec![(1, 1), (2, 5), (3, 1)]);
    }

    #[test]
    fn feval_with_literal_name_only() {
        let src = "feval('sin', 1);\nfeval(@sin, 1);\nfeval(fn, 1);\n";
        assert_eq!(ids(src, ""), vec![(1, 1)]);
    }

    #[test]
    fn lookalikes_are_silent() {
        let src = "obj.eval(1);\nevaluate(1);\nEval('x');\ny = 3;\n";
        assert!(ids(src, "").is_empty());
    }

    #[test]
    fn functions_parameter() {
        let toml = "[lint.rules.MP_SECURITY_1]\nfunctions = [\"assignin\"]\nliteral_only = []\n";
        let src = "eval('a');\nassignin('base', 'a', 1);\nfeval('sin', 1);\n";
        assert_eq!(ids(src, toml), vec![(2, 1)]);
    }

    #[test]
    fn literal_only_parameter() {
        let toml = "[lint.rules.MP_SECURITY_1]\nliteral_only = [\"feval\", \"cellfun\"]\n";
        let src = "cellfun('isempty', c);\ncellfun(@isempty, c);\n";
        assert_eq!(ids(src, toml), vec![(1, 1)]);
    }

    #[test]
    fn survives_syntax_errors() {
        let _ = ids("eval(\nfunction ( end ((", "");
    }
}
