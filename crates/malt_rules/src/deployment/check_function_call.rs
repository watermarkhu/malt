//! Function-call deployment checks: MCPRD, MCABF, MCHLP, MCKBD, MCSVP, MCMLR,
//! MCMFL, MCTBX, MCLL.

use super::*;

impl DeploymentEngine {
    /// Check a `function_call` node for deployment-restricted functions.
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

        // MCPRD: addpath / rmpath / path modification
        if self.is_enabled("MCPRD")
            && (func_name == "addpath"
                || func_name == "rmpath"
                || func_name == "path"
                || func_name == "pathtool"
                || func_name == "restoredefaultpath")
        {
            diags.push(make_diag("MCPRD", node));

            // MCABF: addpath with absolute path (additional check)
            if self.is_enabled("MCABF")
                && func_name == "addpath"
                && has_absolute_path_arg(node, source)
            {
                diags.push(make_diag("MCABF", node));
            }
        }

        // MCHLP: help / doc
        if self.is_enabled("MCHLP") && (func_name == "help" || func_name == "doc") {
            diags.push(make_diag("MCHLP", node));
        }

        // MCKBD: keyboard
        if self.is_enabled("MCKBD") && func_name == "keyboard" {
            diags.push(make_diag("MCKBD", node));
        }

        // MCSVP: savepath
        if self.is_enabled("MCSVP") && func_name == "savepath" {
            diags.push(make_diag("MCSVP", node));
        }

        // MCMLR: matlabroot
        if self.is_enabled("MCMLR") && func_name == "matlabroot" {
            diags.push(make_diag("MCMLR", node));
        }

        // MCMFL: mfilename
        if self.is_enabled("MCMFL") && func_name == "mfilename" {
            diags.push(make_diag("MCMFL", node));
        }

        // MCTBX: Toolbox functions
        if self.is_enabled("MCTBX") && TOOLBOX_FUNCTIONS.contains(&func_name) {
            diags.push(make_diag("MCTBX", node));
        }

        // MCLL: license check
        if self.is_enabled("MCLL") && func_name == "license" {
            diags.push(make_diag("MCLL", node));
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::super::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- MCPRD ---------------------------------------------------------------

    #[test]
    fn mcprd_fires_on_addpath_call() {
        let src = "addpath('src');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCPRD"), "got: {diags:?}");
    }

    #[test]
    fn mcprd_no_fire_on_regular_call() {
        let src = "disp('hello');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MCPRD"), "got: {diags:?}");
    }

    // -- MCHLP ---------------------------------------------------------------

    #[test]
    fn mchlp_fires_on_help_call() {
        let src = "help('plot');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCHLP"), "got: {diags:?}");
    }

    #[test]
    fn mchlp_no_fire_on_regular_call() {
        let src = "plot(x, y);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MCHLP"), "got: {diags:?}");
    }

    // -- MCKBD ---------------------------------------------------------------

    #[test]
    fn mckbd_fires_on_keyboard_call() {
        let src = "keyboard();\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCKBD"), "got: {diags:?}");
    }

    #[test]
    fn mckbd_no_fire_on_regular_call() {
        let src = "disp('x');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MCKBD"), "got: {diags:?}");
    }

    // -- MCSVP ---------------------------------------------------------------

    #[test]
    fn mcsvp_fires_on_savepath_call() {
        let src = "savepath();\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCSVP"), "got: {diags:?}");
    }

    #[test]
    fn mcsvp_no_fire_on_regular_call() {
        let src = "save('data.mat');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MCSVP"), "got: {diags:?}");
    }

    // -- MCMLR ---------------------------------------------------------------

    #[test]
    fn mcmlr_fires_on_matlabroot_call() {
        let src = "r = matlabroot();\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCMLR"), "got: {diags:?}");
    }

    #[test]
    fn mcmlr_no_fire_on_regular_call() {
        let src = "r = pwd;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MCMLR"), "got: {diags:?}");
    }

    // -- MCABF ---------------------------------------------------------------

    #[test]
    fn mcabf_fires_on_absolute_path_addpath() {
        let src = "addpath('/home/user/mytools');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCABF"), "got: {diags:?}");
    }

    #[test]
    fn mcabf_fires_on_windows_drive_addpath() {
        let src = "addpath('C:\\mytools');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCABF"), "got: {diags:?}");
    }

    #[test]
    fn mcabf_no_fire_on_relative_path_addpath() {
        let src = "addpath('mytools');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MCABF"), "got: {diags:?}");
    }

    // -- MCMFL ---------------------------------------------------------------

    #[test]
    fn mcmfl_fires_on_mfilename_call() {
        let src = "name = mfilename();\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCMFL"), "got: {diags:?}");
    }

    #[test]
    fn mcmfl_no_fire_on_regular_call() {
        let src = "name = mfile;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MCMFL"), "got: {diags:?}");
    }

    // -- MCTBX ---------------------------------------------------------------

    #[test]
    fn mctbx_fires_on_toolbox_function() {
        let src = "y = fft2(x);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCTBX"), "got: {diags:?}");
    }

    #[test]
    fn mctbx_no_fire_on_builtin_function() {
        let src = "y = sin(x);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MCTBX"), "got: {diags:?}");
    }

    // -- MCLL ----------------------------------------------------------------

    #[test]
    fn mcll_fires_on_license_call() {
        let src = "license('checkout', 'Signal_Toolbox');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCLL"), "got: {diags:?}");
    }

    #[test]
    fn mcll_no_fire_on_regular_call() {
        let src = "lic = 'abc';\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MCLL"), "got: {diags:?}");
    }
}
