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

        // EMLOAD: `load` used as a bare statement discards its output.
        let is_stmt = node
            .parent()
            .is_some_and(|p| matches!(p.kind(), "block" | "source_file"));
        if self.is_enabled("EMLOAD") && func_name == "load" && is_stmt {
            diags.push(make_diag("EMLOAD", node));
        }

        // PRMNOIN: mobileRobotPRM constructed without a map argument.
        if self.is_enabled("PRMNOIN") && func_name == "mobileRobotPRM" && call_arg_count(node) == 0
        {
            diags.push(make_diag("PRMNOIN", node));
        }

        // LOOPPRAGMAWITHOUTFOR: coder.loop.* transform without a following for.
        // The grammar parses `coder.loop.Control(...)` as a `field_expression`
        // whose last field is the call.
        if self.is_enabled("LOOPPRAGMAWITHOUTFOR") {
            if let Some(fe) = node.parent() {
                let at_stmt = fe
                    .parent()
                    .is_some_and(|p| matches!(p.kind(), "block" | "source_file"));
                if fe.kind() == "field_expression"
                    && node_text(fe, source).starts_with("coder.loop.")
                    && at_stmt
                    && !is_followed_by_for(fe)
                {
                    diags.push(make_diag("LOOPPRAGMAWITHOUTFOR", fe));
                }
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
    fn emfcn_fires_on_save() {
        let src = "save('f.mat');\n";
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
    fn emload_fires_on_load_statement() {
        let diags = lint_nodes(&*engine(), "load('file.mat');\n");
        assert!(has_id(&diags, "EMLOAD"), "got: {diags:?}");
    }

    #[test]
    fn emload_not_fire_when_assigned() {
        let diags = lint_nodes(&*engine(), "s = load('file.mat');\n");
        assert!(!has_id(&diags, "EMLOAD"), "got: {diags:?}");
    }

    // -- EMFCN str2num ----------------------------------------------------------

    #[test]
    fn emfcn_fires_on_str2num() {
        let diags = lint_nodes(&*engine(), "x = str2num('1 2');\n");
        assert!(has_id(&diags, "EMFCN"), "got: {diags:?}");
    }

    #[test]
    fn emfcn_not_fire_on_str2double() {
        let diags = lint_nodes(&*engine(), "x = str2double('1 2');\n");
        assert!(!has_id(&diags, "EMFCN"), "got: {diags:?}");
    }

    // -- PRMNOIN --------------------------------------------------------------

    #[test]
    fn prmnoin_fires_on_prm_without_map() {
        let diags = lint_nodes(&*engine(), "p = mobileRobotPRM();\n");
        assert!(has_id(&diags, "PRMNOIN"), "got: {diags:?}");
    }

    #[test]
    fn prmnoin_not_fire_with_map() {
        let diags = lint_nodes(&*engine(), "p = mobileRobotPRM(map);\n");
        assert!(!has_id(&diags, "PRMNOIN"), "got: {diags:?}");
    }

    #[test]
    fn prmnoin_not_fire_on_inputparser() {
        let diags = lint_nodes(&*engine(), "p = inputParser();\n");
        assert!(!has_id(&diags, "PRMNOIN"), "got: {diags:?}");
    }

    // -- LOOPPRAGMAWITHOUTFOR -----------------------------------------------------

    #[test]
    fn looppragma_fires_without_for() {
        let diags = lint_nodes(&*engine(), "coder.loop.Control(1);\nx = 1;\n");
        assert!(has_id(&diags, "LOOPPRAGMAWITHOUTFOR"), "got: {diags:?}");
    }

    #[test]
    fn looppragma_not_fire_before_for() {
        let diags = lint_nodes(
            &*engine(),
            "coder.loop.Control(1);\nfor i = 1:3\n    x = i;\nend\n",
        );
        assert!(!has_id(&diags, "LOOPPRAGMAWITHOUTFOR"), "got: {diags:?}");
    }
}
