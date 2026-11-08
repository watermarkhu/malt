use super::*;

impl CodegenEngine {
    /// Check a `function_call` node for unsupported functions.
    pub(crate) fn check_function_call<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let func_name = match extract_func_name(node, source) {
            Some(n) => n,
            None => return diags,
        };

        // EMFCN: Unsupported function for code generation
        if self.is_enabled("EMFCN") && UNSUPPORTED_FUNCTIONS.contains(&func_name) {
            diags.push(make_diag("EMFCN", node));
        }

        // EMLOAD: load function
        if self.is_enabled("EMLOAD") && func_name == "load" {
            diags.push(make_diag("EMLOAD", node));
        }

        // EMS2N: str2num
        if self.is_enabled("EMS2N") && func_name == "str2num" {
            diags.push(make_diag("EMS2N", node));
        }

        // PRMNOIN: validateattributes / inputParser in codegen context
        if self.is_enabled("PRMNOIN")
            && (func_name == "validateattributes"
                || func_name == "inputParser"
                || func_name == "addRequired"
                || func_name == "addParameter")
        {
            diags.push(make_diag("PRMNOIN", node));
        }

        // LOOPPRAGMAWITHOUTFOR: coder.loop without following for
        if self.is_enabled("LOOPPRAGMAWITHOUTFOR") && func_name == "coder.loop" {
            // Check if next sibling is a for_statement
            if !is_followed_by_for(node) {
                diags.push(make_diag("LOOPPRAGMAWITHOUTFOR", node));
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    // -- EMFCN ---------------------------------------------------------------

    #[test]
    fn emfcn_fires_on_unsupported_function_call() {
        let src = "eval('x');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMFCN"), "got: {diags:?}");
    }

    #[test]
    fn emfcn_fires_on_unsupported_command() {
        let src = "clear all;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMFCN"), "got: {diags:?}");
    }

    #[test]
    fn emfcn_fires_on_plot() {
        let src = "plot(x, y);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMFCN"), "got: {diags:?}");
    }

    #[test]
    fn emfcn_not_fire_on_supported_function() {
        let src = "y = sum(x);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMFCN"), "got: {diags:?}");
    }

    // -- EMLOAD ---------------------------------------------------------------

    #[test]
    fn emload_fires_on_load_function() {
        let src = "load('file.mat');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMLOAD"), "got: {diags:?}");
    }

    #[test]
    fn emload_not_fire_on_save() {
        let src = "save('file.mat');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMLOAD"), "got: {diags:?}");
    }

    // -- EMS2N ----------------------------------------------------------------

    #[test]
    fn ems2n_fires_on_str2num() {
        let src = "x = str2num('1 2');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMS2N"), "got: {diags:?}");
    }

    #[test]
    fn ems2n_not_fire_on_str2double() {
        let src = "x = str2double('1 2');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMS2N"), "got: {diags:?}");
    }

    // -- PRMNOIN --------------------------------------------------------------

    #[test]
    fn prmnoin_fires_on_validateattributes() {
        let src = "validateattributes(x, {'numeric'}, {'scalar'});\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "PRMNOIN"), "got: {diags:?}");
    }

    #[test]
    fn prmnoin_fires_on_inputparser() {
        let src = "p = inputParser();\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "PRMNOIN"), "got: {diags:?}");
    }

    #[test]
    fn prmnoin_not_fire_on_plain_call() {
        let src = "x = fcn(1, 2);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "PRMNOIN"), "got: {diags:?}");
    }
}
