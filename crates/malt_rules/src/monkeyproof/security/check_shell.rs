//! MP_SECURITY_2: do not use the shell escape.
//!
//! ## What it catches
//!
//! A line that starts with `!`, which runs the rest of the line as an operating system
//! command. `system` and its relatives are the recommended replacement, so they are not
//! flagged.
//!
//! ## What it does not catch
//!
//! A `!` inside a matrix, a string or a comment is not a shell escape and is not reported.

use malt_core::{Config, Diagnostic, FileContext};

use super::for_each_node;
use crate::monkeyproof::finding;

const ID: &str = "MP_SECURITY_2";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    for_each_node(ctx.tree.root_node(), |node| {
        if node.kind() == "command" && ctx.source[node.start_byte()..].starts_with('!') {
            out.push(finding(
                ctx.source,
                ID,
                node.start_byte()..node.end_byte(),
                "Avoid the `!` shell escape: call `system` instead.",
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::{lint, security::from_config};

    fn hits(source: &str) -> Vec<(usize, usize)> {
        lint(from_config, source, "")
            .iter()
            .filter(|d| d.rule_id == "MP_SECURITY_2")
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn flags_the_escape() {
        assert_eq!(hits("x = 1;\n!ls -l\n"), vec![(2, 1)]);
        assert_eq!(
            hits("function f()\n    !mycommand arg\nend\n"),
            vec![(2, 5)]
        );
    }

    #[test]
    fn system_and_not_equal_are_silent() {
        assert!(hits("system('ls');\nb = a ~= 1;\ns = '!ls';\n% !ls\n").is_empty());
    }

    #[test]
    fn survives_syntax_errors() {
        let _ = hits("!\n!(((\nfunction (");
    }
}
