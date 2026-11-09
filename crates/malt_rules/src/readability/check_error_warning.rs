//! # SPERR / SPWRN / NCHKE: redundant message construction in `error`/`warning`
//!
//! * SPERR / SPWRN: `error(sprintf(...))` / `warning(sprintf(...))`; both
//!   functions take SPRINTF-like arguments directly.
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
        if args.len() != 1 || args[0].kind() != "function_call" {
            return;
        }
        let inner = args[0];
        match call_name(inner, source) {
            Some("sprintf") => {
                if !self.is_check_enabled(check_id) {
                    return;
                }
                let message = if check_id == "SPERR" {
                    "ERROR takes SPRINTF-like arguments directly."
                } else {
                    "WARNING takes SPRINTF-like arguments directly."
                };
                // `error(sprintf(f, a))` -> `error(f, a)` is not exact when the
                // text holds `%` or `\`: error(msg) does not process them but
                // error(fmt, ...) does, so the rewrite is left to the user.
                results.push(self.diag(check_id, message, node, None));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    #[test]
    fn sperr_error_of_sprintf_fires() {
        let diags = lint_nodes(&*engine(), "error(sprintf('bad %d', n));\n");
        assert!(has_id(&diags, "SPERR"), "got: {diags:?}");
    }

    #[test]
    fn sperr_plain_error_calls_do_not_fire() {
        for src in [
            "error('my message');\n",
            "error('MyComp:myID', 'message');\n",
            "error('MyComp:myID', sprintf('bad %d', n));\n",
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
    fn spwrn_plain_warning_calls_do_not_fire() {
        for src in ["warning('my message');\n", "warning('A:b', 'message');\n"] {
            assert!(!has_id(&lint_nodes(&*engine(), src), "SPWRN"), "{src}");
        }
    }

    #[test]
    fn nchke_error_of_nargchk_on_nargout_fires() {
        let diags = lint_nodes(&*engine(), "error(nargchk(1, 2, nargout));\n");
        assert!(has_id(&diags, "NCHKE"), "got: {diags:?}");
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
