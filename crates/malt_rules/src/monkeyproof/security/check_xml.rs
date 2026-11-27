//! MP_SECURITY_4: set `AllowDoctype` to false when reading XML.
//!
//! ## What it catches
//!
//! An `xmlread` call that has no `AllowDoctype` name-value pair, or has one whose value
//! is the literal `true` or `1`, and an assignment of `true` or `1` to a
//! `.AllowDoctype` property (a `matlab.io.xml.dom.Parser` configuration, which is false
//! unless set). The name is matched ignoring case. A call whose arguments include a
//! comma-separated list (`varargin{:}`) is not reported, because the options may be in it.
//!
//! ## What it does not catch
//!
//! An `AllowDoctype` value that is not a literal.

use malt_core::{Config, Diagnostic, FileContext};

use super::{args_of, for_each_node, is_literal_bool, string_value};
use crate::analysis::callee::{callee_path, Dotted};
use crate::monkeyproof::finding;

const ID: &str = "MP_SECURITY_4";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    for_each_node(ctx.tree.root_node(), |node| match node.kind() {
        "function_call" => {
            if callee_path(node, ctx.source, Dotted::Skip) != Some("xmlread") {
                return;
            }
            let args = args_of(node);
            if args
                .iter()
                .any(|a| ctx.source[a.start_byte()..a.end_byte()].contains("{:}"))
            {
                return;
            }
            let option = args.windows(2).find(|pair| {
                string_value(pair[0], ctx.source)
                    .is_some_and(|n| n.eq_ignore_ascii_case("AllowDoctype"))
            });
            let message = "Pass `\"AllowDoctype\", false` when reading XML.";
            match option {
                None => out.push(finding(
                    ctx.source,
                    ID,
                    node.start_byte()..node.end_byte(),
                    message,
                )),
                Some(pair) if is_literal_bool(pair[1], ctx.source) == Some(true) => {
                    out.push(finding(
                        ctx.source,
                        ID,
                        pair[1].start_byte()..pair[1].end_byte(),
                        message,
                    ));
                }
                Some(_) => {}
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
            if ctx.source[field.start_byte()..field.end_byte()].eq_ignore_ascii_case("AllowDoctype")
                && is_literal_bool(right, ctx.source) == Some(true)
            {
                out.push(finding(
                    ctx.source,
                    ID,
                    right.start_byte()..right.end_byte(),
                    "Set `AllowDoctype` to false.",
                ));
            }
        }
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::{lint, security::from_config};

    fn hits(source: &str) -> Vec<(usize, usize)> {
        lint(from_config, source, "")
            .iter()
            .filter(|d| d.rule_id == "MP_SECURITY_4")
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn flags_a_missing_option() {
        assert_eq!(hits("c = xmlread(f);\n"), vec![(1, 5)]);
    }

    #[test]
    fn flags_true() {
        assert_eq!(
            hits("c = xmlread(f, 'AllowDoctype', true);\n"),
            vec![(1, 32)]
        );
        assert_eq!(
            hits("p.Configuration.AllowDoctype = true;\n"),
            vec![(1, 32)]
        );
    }

    #[test]
    fn false_and_unknown_are_silent() {
        let src = "a = xmlread(f, \"AllowDoctype\", false);\n\
                   b = xmlread(f, 'allowdoctype', 0);\n\
                   c = xmlread(f, 'AllowDoctype', flag);\n\
                   d = xmlread(f, varargin{:});\n\
                   p.Configuration.AllowDoctype = false;\n\
                   e = xmlwrite(f);\n\
                   obj.xmlread(f);\n";
        assert!(hits(src).is_empty());
    }

    #[test]
    fn survives_syntax_errors() {
        let _ = hits("xmlread(\np.AllowDoctype = ");
    }
}
