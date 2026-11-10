//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// NCHKOS: `narginchk` does not return a value, so using it as a value (an
    /// assignment, a bracketed element, an argument or a default) is an error.
    /// MATLAB never reports `nargoutchk` here.
    pub(crate) fn check_nchkos_output_use(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(name) = callee_name(node, source) else {
            return;
        };
        if name != "narginchk" {
            return;
        }
        let is_statement = node
            .parent()
            .is_some_and(|p| ["source_file", "block"].contains(&p.kind()));
        if !is_statement {
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
    fn test_nchkos_fires_narginchk_used_as_value() {
        // Verified with MATLAB R2026b: assignment, bracketed, argument and
        // arguments-block default positions are all reported.
        let source = "\
function foo(n)
    arguments
        n (1,1) = narginchk(1, 2)
    end
    x = narginchk(1, 2);
    z = [narginchk(1, 2)];
    disp(narginchk(1, 2));
end
";
        let diags = check_source(source, "foo.m");
        let nchkos = filter_by_id(&diags, "NCHKOS");
        assert_eq!(
            nchkos.len(),
            4,
            "NCHKOS should fire for narginchk used as a value"
        );
    }

    #[test]
    fn test_nchkos_no_fire_nargoutchk() {
        // MATLAB never reports nargoutchk as a value.
        let source = "\
function foo()
    y = nargoutchk(1, 2);
end
";
        let diags = check_source(source, "foo.m");
        assert!(filter_by_id(&diags, "NCHKOS").is_empty());
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
