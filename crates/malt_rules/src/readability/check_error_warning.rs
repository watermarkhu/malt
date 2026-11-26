//! # SPERR / SPWRN / NCHKE: redundant message construction in `error`/`warning`
//!
//! * SPERR / SPWRN: `error(sprintf(...))` / `warning(sprintf(...))`, and
//!   `error('a:b', sprintf(...))` / `warning('a:b', sprintf(...))` with a literal
//!   identifier first; both functions take SPRINTF-like arguments directly.
//! * NCHKE: `error(nargchk(min, max, nargout))`; use `nargoutchk` instead.

use super::*;

impl ReadabilityEngine {
    /// SPERR / SPWRN / NCHKE: a single-argument `error`/`warning` call whose
    /// argument is itself a call to `sprintf` (or `nargchk`).
    pub(crate) fn check_error_warning(
        &self,
        node: tree_sitter::Node,
        source: &str,
        check_id: &'static str,
        results: &mut Vec<Diagnostic>,
    ) {
        let args = call_args(node);
        let message = if check_id == "SPERR" {
            "ERROR takes SPRINTF-like arguments directly."
        } else {
            "WARNING takes SPRINTF-like arguments directly."
        };
        // `error('a:b', sprintf(...))`: with an identifier first, the second argument is the
        // message, whatever the sprintf arguments (probed with error and warning).
        if args.len() >= 2
            && is_message_id(args[0], source)
            && call_path(args[1], source, Dotted::Skip) == Some("sprintf")
        {
            if self.is_check_enabled(check_id) {
                // `error('a:b', sprintf(f, x))` becomes `error('a:b', f, x)` when the
                // sprintf is the last argument.
                let fix = (args.len() == 2)
                    .then(|| inline_sprintf_fix(args[1], source))
                    .flatten();
                results.push(self.diag(check_id, message, node, fix));
            }
            return;
        }
        if args.len() != 1 || args[0].kind() != "function_call" {
            return;
        }
        let inner = args[0];
        match call_path(inner, source, Dotted::Skip) {
            Some("sprintf") => {
                if !self.is_check_enabled(check_id) {
                    return;
                }
                // Probed: MATLAB reports SPERR only when sprintf takes arguments;
                // `error(sprintf('a:b'))` (format only) is silent (gp-b__t1:29).
                if check_id == "SPERR" && call_args(inner).len() < 2 {
                    return;
                }
                // `error(sprintf(f, a))` -> `error(f, a)` is not exact when the
                // text holds `%` or `\`: error(msg) does not process them but
                // error(fmt, ...) does, so the rewrite is unsafe.
                let fix = inline_sprintf_fix(inner, source);
                results.push(self.diag(check_id, message, node, fix));
            }
            Some("nargchk") if check_id == "SPERR" && self.is_check_enabled("NCHKE") => {
                let nargchk_args = call_args(inner);
                if nargchk_args.len() == 3
                    && &source[nargchk_args[2].start_byte()..nargchk_args[2].end_byte()]
                        == "nargout"
                {
                    results.push(self.diag("NCHKE", "Use NARGOUTCHK without ERROR.", node, None));
                }
            }
            _ => {}
        }
    }
}

/// The fix that replaces a `sprintf(f, a, ...)` call with its arguments `f, a, ...`.
fn inline_sprintf_fix(call: tree_sitter::Node, source: &str) -> Option<Fix> {
    let args = call_args(call);
    let (first, last) = (args.first()?, args.last()?);
    Some(Fix::new(
        call.byte_range(),
        &source[first.start_byte()..last.end_byte()],
    ))
}

/// A literal message identifier such as `'a:b'` or `"Mod:Sub:Id"`: colon-separated
/// components, each starting with a letter and holding letters, digits or underscores
/// (probed: `a:1b`, `_a:b`, `a:_b`, `a.b:c`, `a:` and `a:b c` are not identifiers). A
/// variable is not read: MATLAB also reports a parameter used as the identifier, but the
/// rule for it is not established.
fn is_message_id(node: tree_sitter::Node, source: &str) -> bool {
    if node.kind() != "string" {
        return false;
    }
    let text = &source[node.byte_range()];
    let quoted = text.len() >= 2
        && (text.starts_with('\'') || text.starts_with('"'))
        && text.ends_with(text.get(..1).unwrap_or(""));
    if !quoted {
        return false;
    }
    text[1..text.len() - 1].split(':').count() >= 2
        && text[1..text.len() - 1].split(':').all(|part| {
            part.starts_with(|c: char| c.is_ascii_alphabetic())
                && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{fix_of, has_id, lint_all, lint_nodes};
    use malt_core::Config;

    #[test]
    fn sperr_and_spwrn_fix_inline_the_sprintf_arguments() {
        for (src, want, id) in [
            (
                "error(sprintf('bad %d', n));\n",
                "error('bad %d', n);\n",
                "SPERR",
            ),
            (
                "error('A:b', sprintf('bad %d', n));\n",
                "error('A:b', 'bad %d', n);\n",
                "SPERR",
            ),
            (
                "warning(sprintf('w %s', s));\n",
                "warning('w %s', s);\n",
                "SPWRN",
            ),
        ] {
            let (safe, fixed) = fix_of(&*engine(), src, id)
                .unwrap_or_else(|| panic!("{src}: {:?}", lint_all(&*engine(), src)));
            assert!(!safe, "{src}");
            assert_eq!(fixed, want);
            let still = lint_all(&*engine(), &fixed);
            assert!(still.iter().all(|d| d.rule_id != id), "{still:?}");
        }
    }

    #[test]
    fn sperr_no_fix_when_more_arguments_follow_the_sprintf() {
        let src = "error('a:b', sprintf('Value %d', 42), 3);\n";
        assert!(crate::test_util::reported_without_fix(
            &*engine(),
            src,
            "SPERR"
        ));
    }

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    #[test]
    fn sperr_error_of_sprintf_fires() {
        let diags = lint_nodes(&*engine(), "error(sprintf('bad %d', n));\n");
        assert!(has_id(&diags, "SPERR"), "got: {diags:?}");
    }

    #[test]
    fn sperr_identifier_then_sprintf_fires() {
        // Probed (k1, k2, k3, s1, s3, s5, s9, i10, j1, j2, j6, j7, j9, j10): `error(ID, sprintf(...))`
        // is SPERR at the call, with or without arguments to sprintf, with an extra argument.
        for src in [
            "error('MyComp:myID', sprintf('bad %d', n));\n",
            "error('a:b', sprintf('x'));\n",
            "error('a:b', sprintf('Value %d', 42), 3);\n",
            "error(\"a:b\", sprintf('x %d', 1));\n",
            "error('a_b:c', sprintf('x %d', 1));\n",
            "error('a:b:c', sprintf('x %d', 1));\n",
            "error('Mod:Sub:Id', sprintf('x %s', 'y'));\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, "SPERR"), "{src} got: {diags:?}");
        }
    }

    #[test]
    fn sperr_non_identifier_first_argument_is_silent() {
        // Probed (i2, i4, i11, i12, i15, j3, j4, j5, s2, s6, s11): with a format or a
        // string that is not an identifier, the second argument is not the message.
        for src in [
            "error('Value %d', sprintf('x'));\n",
            "error('Value: %d', sprintf('x %d', 1));\n",
            "error('a:b c', sprintf('x %d', 1));\n",
            "error('1a:b', sprintf('x %d', 1));\n",
            "error('a:1b', sprintf('x %d', 1));\n",
            "error('a:_b', sprintf('x %d', 1));\n",
            "error('a.b:c', sprintf('x %d', 1));\n",
            "error('a:', sprintf('x %d', 1));\n",
        ] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "SPERR"), "{src}");
        }
    }

    #[test]
    fn sperr_plain_error_calls_do_not_fire() {
        for src in [
            "error('my message');\n",
            "error('MyComp:myID', 'message');\n",
            "error(sprintf('a:b'));\n",
        ] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "SPERR"), "{src}");
        }
    }

    #[test]
    fn spwrn_warning_of_sprintf_fires() {
        let diags = lint_nodes(&*engine(), "warning(sprintf('bad %d', n));\n");
        assert!(has_id(&diags, "SPWRN"), "got: {diags:?}");
    }

    #[test]
    fn spwrn_identifier_then_sprintf_fires() {
        // Probed (k4, k5): `warning('a:b', sprintf(...))` is SPWRN, format only or not.
        for src in [
            "warning('a:b', sprintf('x'));\n",
            "warning('a:b', sprintf('x %d', 2));\n",
        ] {
            assert!(has_id(&lint_nodes(&*engine(), src), "SPWRN"), "{src}");
        }
    }

    #[test]
    fn spwrn_is_not_reported_after_error_call() {
        let src = "error('x');\nwarning(sprintf('bad %d', n));\n";
        assert!(!has_id(&lint_nodes(&*engine(), src), "SPWRN"));
        let src = "warning(sprintf('bad %d', n));\nerror('x');\n";
        assert!(has_id(&lint_nodes(&*engine(), src), "SPWRN"));
    }

    #[test]
    fn spwrn_plain_warning_calls_do_not_fire() {
        for src in ["warning('my message');\n", "warning('A:b', 'message');\n"] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "SPWRN"), "{src}");
        }
    }

    #[test]
    fn nchke_error_of_nargchk_on_nargout_fires() {
        // Off under the mathworks preset: MATLAB reports NCHKM (suggested improvements) instead.
        let all = ReadabilityEngine::from_config(
            &malt_core::Config::from_toml("[lint]\npreset = \"all\"\n").unwrap(),
        );
        let diags = lint_nodes(&*all, "error(nargchk(1, 2, nargout));\n");
        assert!(has_id(&diags, "NCHKE"), "got: {diags:?}");
        let diags = lint_nodes(&*engine(), "error(nargchk(1, 2, nargout));\n");
        assert!(!has_id(&diags, "NCHKE"), "got: {diags:?}");
    }

    #[test]
    fn nchke_other_nargchk_forms_do_not_fire() {
        for src in [
            "x = nargchk(1, 2, nargout);\n",
            "error(nargchk(1, 2, nargin));\n",
        ] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "NCHKE"), "{src}");
        }
    }
}
