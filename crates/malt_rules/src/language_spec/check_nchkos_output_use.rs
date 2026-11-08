//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// NCHKOS: `narginchk`/`nargoutchk` do not return values, so using them on the
    /// right-hand side of an assignment is an error.
    pub(crate) fn check_nchkos_output_use(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(name) = callee_name(node, source) else {
            return;
        };
        if name != "narginchk" && name != "nargoutchk" {
            return;
        }
        if is_assignment_rhs(node) {
            self.push_diag(
                node,
                "NCHKOS",
                "NARGINCHK does not return any values.",
                diagnostics,
            );
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_nchkos_fires_assignment_rhs() {
        let source = "\
function foo()
    x = narginchk(1, 2);
    y = nargoutchk(1, 2);
end
";
        let diags = check_source(source, "foo.m");
        let nchkos = filter_by_id(&diags, "NCHKOS");
        assert_eq!(
            nchkos.len(),
            2,
            "NCHKOS should fire for narginchk/nargoutchk on RHS"
        );
    }

    #[test]
    fn test_nchkos_no_fire_standalone() {
        let source = "\
function foo()
    narginchk(1, 2);
    nargoutchk(1, 2);
end
";
        let diags = check_source(source, "foo.m");
        let nchkos = filter_by_id(&diags, "NCHKOS");
        assert!(
            nchkos.is_empty(),
            "NCHKOS should NOT fire for standalone calls"
        );
    }
}
