//! MP_STAT_18: `try` used for something other than exception handling.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;

use super::{callee_uses, name_list, statements_of, walk};
use crate::monkeyproof::finding;

/// Functions that raise an error again.
const DEFAULT: &[&str] = &["error", "rethrow", "throw", "throwAsCaller"];

/// Parameters of `MP_STAT_18`.
#[derive(Debug, Clone, Deserialize, Default)]
struct Params {
    /// Functions that count as raising an error in a `catch`. Replaces the default list
    /// when set.
    error_functions: Option<Vec<String>>,
}

/// Flag a `try` that swallows errors: it has no `catch`, an empty `catch`, or a `catch`
/// that never raises an error (it only assigns a default, logs or ignores). The finding
/// covers the `catch` keyword, or the `try` keyword when there is no `catch`.
///
/// This reads the standard strictly: a `catch` that only prints a warning still hides the
/// error from the caller. A call in a nested statement of the `catch` counts.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params("MP_STAT_18");
    let raising = name_list(params.error_functions, DEFAULT);
    walk(ctx.tree.root_node(), |node, _| {
        if node.kind() != "try_statement" {
            return;
        }
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        let catch = children.iter().find(|c| c.kind() == "catch_clause");
        let (anchor, message) = match catch {
            None => (
                children
                    .first()
                    .map_or(node.byte_range(), |c| c.byte_range()),
                "Add a catch that raises an error, or avoid the try.",
            ),
            Some(catch) => {
                let mut inner = catch.walk();
                let body = catch.children(&mut inner).find(|c| c.kind() == "block");
                let raises = body.is_some_and(|body| {
                    !statements_of(body).is_empty()
                        && callee_uses(body, ctx.source)
                            .iter()
                            .any(|usage| raising.contains(usage.name))
                });
                if raises {
                    return;
                }
                (
                    catch
                        .child(0)
                        .map_or(catch.byte_range(), |c| c.byte_range()),
                    "Raise an error in this catch, a try is for exception handling only.",
                )
            }
        };
        out.push(finding(ctx.source, "MP_STAT_18", anchor, message));
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_a_catch_that_swallows() {
        let source = "try\n  n = c.count;\ncatch\n  n = 0;\nend\n";
        assert_eq!(hits("MP_STAT_18", source, ""), vec![(3, 1)]);
    }

    #[test]
    fn flags_an_empty_catch_and_a_missing_catch() {
        let source = "try\n  a = 1;\ncatch ME\nend\ntry\n  b = 2;\nend\n";
        assert_eq!(hits("MP_STAT_18", source, ""), vec![(3, 1), (5, 1)]);
    }

    #[test]
    fn flags_a_catch_that_only_logs() {
        let source = "try\n  a = 1;\ncatch ME\n  disp(ME.message)\nend\n";
        assert_eq!(hits("MP_STAT_18", source, "").len(), 1);
    }

    #[test]
    fn a_catch_that_raises_is_exception_handling() {
        let source = "try\n  a = 1;\ncatch ME\n  rethrow(ME)\nend\ntry\n  b = 1;\ncatch\n  if x\n    error('A:b', 'c')\n  end\nend\n";
        assert!(hits("MP_STAT_18", source, "").is_empty());
    }

    #[test]
    fn the_error_functions_are_a_parameter() {
        let toml = "[lint.rules.MP_STAT_18]\nerror_functions = [\"fail\"]\n";
        let source =
            "try\n  a = 1;\ncatch\n  fail()\nend\ntry\n  a = 1;\ncatch\n  error('A:b', 'c')\nend\n";
        assert_eq!(hits("MP_STAT_18", source, toml), vec![(8, 1)]);
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_18", "try\ncatch (\n", "");
    }
}
