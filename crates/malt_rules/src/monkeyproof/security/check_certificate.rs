//! MP_SECURITY_3: do not set an empty certificate file name.
//!
//! ## What it catches
//!
//! A `weboptions` or `matlab.net.http.HTTPOptions` (also `HTTPOptions` after an import)
//! call with a `CertificateFilename` name-value pair whose value is an empty string
//! literal, and an assignment of an empty string literal to a `.CertificateFilename`
//! property. The name is matched ignoring case. The finding is at the empty value.
//!
//! ## What it does not catch
//!
//! An empty value that reaches the option through a variable.

use malt_core::{Config, Diagnostic, FileContext};

use super::{args_of, for_each_node, is_empty_string, string_value};
use crate::analysis::callee::{callee_path, Dotted};
use crate::monkeyproof::finding;

const ID: &str = "MP_SECURITY_3";
const MESSAGE: &str = "Do not empty the certificate file name: it turns certificate checks off.";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    for_each_node(ctx.tree.root_node(), |node| match node.kind() {
        "function_call" => {
            let Some(name) = callee_path(node, ctx.source, Dotted::Whole) else {
                return;
            };
            if !matches!(
                name,
                "weboptions" | "matlab.net.http.HTTPOptions" | "HTTPOptions"
            ) {
                return;
            }
            let args = args_of(node);
            for pair in args.windows(2) {
                let is_name = string_value(pair[0], ctx.source)
                    .is_some_and(|n| n.eq_ignore_ascii_case("CertificateFilename"));
                if is_name && is_empty_string(pair[1], ctx.source) {
                    out.push(finding(
                        ctx.source,
                        ID,
                        pair[1].start_byte()..pair[1].end_byte(),
                        MESSAGE,
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
            let text = &ctx.source[field.start_byte()..field.end_byte()];
            if text.eq_ignore_ascii_case("CertificateFilename")
                && is_empty_string(right, ctx.source)
            {
                out.push(finding(
                    ctx.source,
                    ID,
                    right.start_byte()..right.end_byte(),
                    MESSAGE,
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
            .filter(|d| d.rule_id == "MP_SECURITY_3")
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn flags_construction() {
        assert_eq!(
            hits("o = weboptions(\"CertificateFilename\", \"\");\n"),
            vec![(1, 39)]
        );
        assert_eq!(
            hits("o = matlab.net.http.HTTPOptions('certificatefilename', '');\n"),
            vec![(1, 56)]
        );
        assert_eq!(
            hits("o = weboptions('Timeout', 5, 'CertificateFilename', '');\n"),
            vec![(1, 53)]
        );
    }

    #[test]
    fn flags_property_assignment() {
        assert_eq!(hits("o.CertificateFilename = \"\";\n"), vec![(1, 25)]);
    }

    #[test]
    fn valid_values_are_silent() {
        let src = "a = weboptions('CertificateFilename', 'default');\n\
                   b = weboptions('CertificateFilename', 'my.crt');\n\
                   c = weboptions();\n\
                   o.CertificateFilename = 'default';\n\
                   o.CertificateFilename = f;\n\
                   d = other('CertificateFilename', '');\n\
                   o.Other = '';\n";
        assert!(hits(src).is_empty());
    }

    #[test]
    fn survives_syntax_errors() {
        let _ = hits("weboptions('CertificateFilename',\no.CertificateFilename = ");
    }
}
