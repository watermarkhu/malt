//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// NCHKOS: `narginchk` does not return a value, so using it as a value (an
    /// assignment, a bracketed element, an argument or a default) is an error.
    /// MATLAB never reports `nargoutchk` here, nor a call that is the whole body of
    /// an anonymous function (`f = @() narginchk(1, 2)` is silent).
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
        let parent = node.parent();
        let is_statement = parent.is_some_and(|p| ["source_file", "block"].contains(&p.kind()));
        let is_lambda_body = parent.is_some_and(|p| p.kind() == "lambda");
        if !is_statement && !is_lambda_body {
            self.push_diag(
                node,
                "NCHKOS",
                "NARGINCHK does not return any values.",
                diagnostics,
            );
        }
    }

    /// NCHKOS for a bare `narginchk` identifier used as a value, such as `x = narginchk;`,
    /// `disp(narginchk)` or `[narginchk]` (probed). A statement, a lambda body, a call
    /// name and an assignment target are not values.
    pub(crate) fn check_nchkos_bare(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node_text(node, source) != "narginchk" {
            return;
        }
        let Some(parent) = node.parent() else {
            return;
        };
        let is_target = parent.kind() == "assignment"
            && parent
                .child_by_field_name("left")
                .is_some_and(|left| left.id() == node.id());
        let is_not_value = matches!(
            parent.kind(),
            "source_file"
                | "block"
                | "lambda"
                | "function_call"
                | "field_expression"
                | "function_arguments"
                | "command"
        );
        if !is_target && !is_not_value {
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

    #[test]
    fn test_nchkos_silent_lambda_body() {
        // Probed (n09, n25): MATLAB is silent on `@() narginchk(1,2)` and `@() narginchk`.
        let source = "function nc(n)\nf = @() narginchk(1,2);\ng = @() narginchk;\nend\n";
        let diags = check_source(source, "nc.m");
        assert!(filter_by_id(&diags, "NCHKOS").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_nchkos_fires_call_in_lambda_argument() {
        // Probed (n21): the call is an argument inside the lambda body, so it is a value.
        let source = "function nc(n)\nf = @() disp(narginchk(1,2));\nend\n";
        let diags = check_source(source, "nc.m");
        assert_eq!(filter_by_id(&diags, "NCHKOS").len(), 1, "got: {diags:?}");
    }

    #[test]
    fn test_nchkos_fires_bare_narginchk_as_value() {
        // Probed (n11, n23, n24): a bare `narginchk` used as a value is NCHKOS.
        let source = "function nc(n)\nx = narginchk;\ny = [narginchk];\ndisp(narginchk);\nend\n";
        let diags = check_source(source, "nc.m");
        let nchkos = filter_by_id(&diags, "NCHKOS");
        assert_eq!(nchkos.len(), 3, "got: {diags:?}");
    }

    #[test]
    fn test_nchkos_silent_bare_narginchk_statement() {
        // Probed (n22): a statement `narginchk;` is not a value.
        let source = "function nc(n)\nnarginchk;\nend\n";
        let diags = check_source(source, "nc.m");
        assert!(filter_by_id(&diags, "NCHKOS").is_empty(), "got: {diags:?}");
    }
}
