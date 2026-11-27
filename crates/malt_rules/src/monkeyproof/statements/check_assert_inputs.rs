//! MP_STAT_10: `assert` without an error identifier and a message.

use malt_core::{Config, Diagnostic, FileContext};

use super::{call_arguments, call_identifier, string_value, variables, walk};
use crate::monkeyproof::finding;

/// Whether `text` has the shape of an error identifier: two or more parts separated by
/// colons, each made of letters, digits and underscores, the first starting with a letter.
fn is_error_id(text: &str) -> bool {
    let parts: Vec<&str> = text.split(':').collect();
    parts.len() >= 2
        && parts.iter().all(|part| {
            !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
        && parts[0].starts_with(|c: char| c.is_ascii_alphabetic())
}

/// Flag a call of `assert` that has fewer than three arguments, or whose second argument
/// is a string that is not an error identifier. The finding covers the call.
///
/// A second argument that is not a string literal is trusted to be an identifier.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let root = ctx.tree.root_node();
    let vars = variables(root, ctx.source);
    walk(root, |node, parent| {
        if node.kind() != "function_call"
            || parent.is_some_and(|p| super::is_member(node, p))
            || call_identifier(node, ctx.source) != Some("assert")
            || vars.contains("assert")
        {
            return;
        }
        let args = call_arguments(node);
        let message = if args.len() < 3 {
            "Give assert an error identifier and a message as its second and third inputs."
        } else if string_value(args[1], ctx.source).is_some_and(|text| !is_error_id(text)) {
            "The second input of assert shall be an error identifier."
        } else {
            return;
        };
        out.push(finding(
            ctx.source,
            "MP_STAT_10",
            node.byte_range(),
            message,
        ));
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_too_few_inputs() {
        let source = "assert(x > 0)\nassert(x > 0, 'Too small')\n";
        assert_eq!(hits("MP_STAT_10", source, ""), vec![(1, 1), (2, 1)]);
    }

    #[test]
    fn flags_a_message_in_the_id_position() {
        let source = "assert(x > 0, 'Value %d is too small', x)\n";
        assert_eq!(hits("MP_STAT_10", source, ""), vec![(1, 1)]);
    }

    #[test]
    fn accepts_id_and_message() {
        let source = "assert(x > 0, 'Pkg:Fcn:TooSmall', 'Too small')\nassert(ok, id, \"msg\")\n";
        assert!(hits("MP_STAT_10", source, "").is_empty());
    }

    #[test]
    fn leaves_other_calls_alone() {
        let source = "obj.assert(x)\nmyassert(x)\nx = assert;\n";
        assert!(hits("MP_STAT_10", source, "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_10", "assert(\nassert(x,\n", "");
    }
}
