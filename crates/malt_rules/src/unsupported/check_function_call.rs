//! Node-level `function_call` dispatch for the unsupported features engine.
//!
//! Fires checks for ADE functions (MCADE), ActiveX/COM automation (AXCHUD),
//! `feature` (FEATUD), `findprop` (FNDPUD), Handle Graphics containers
//! (HGCNUD), `isMember` (ISMBUD), `meta.package` (MIPKG), legacy `serial`
//! (SEPTUD), `System.Data` interop (SYDEUD), `uiresume` context (UIRSUD),
//! deprecated UI setup patterns (UISUUD), and chained `parfeval` awaits
//! (AWTIUD).

use super::*;

impl UnsupportedEngine {
    /// Check a `function_call` node for unsupported function usage.
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

        // MCADE: ADE functions
        if self.is_enabled("MCADE") && ADE_FUNCTIONS.contains(&func_name) {
            diags.push(make_diag("MCADE", node));
        }

        // AXCHUD: ActiveX/COM functions
        if self.is_enabled("AXCHUD") && ACTIVEX_FUNCTIONS.contains(&func_name) {
            diags.push(make_diag("AXCHUD", node));
        }

        // FEATUD: feature() function
        if self.is_enabled("FEATUD") && func_name == "feature" {
            diags.push(make_diag("FEATUD", node));
        }

        // FNDPUD: findprop
        if self.is_enabled("FNDPUD") && func_name == "findprop" {
            diags.push(make_diag("FNDPUD", node));
        }

        // HGCNUD: Handle Graphics containers
        if self.is_enabled("HGCNUD") && HG_CONTAINER_FUNCTIONS.contains(&func_name) {
            diags.push(make_diag("HGCNUD", node));
        }

        // ISMBUD: isMember (camelCase form)
        if self.is_enabled("ISMBUD") && func_name == "isMember" {
            diags.push(make_diag("ISMBUD", node));
        }

        // MIPKG: meta.package
        if self.is_enabled("MIPKG") && func_name == "meta.package" {
            diags.push(make_diag("MIPKG", node));
        }

        // SEPTUD: serial (legacy)
        if self.is_enabled("SEPTUD") && func_name == "serial" {
            diags.push(make_diag("SEPTUD", node));
        }

        // SYDEUD: System.Data .NET interop patterns
        if self.is_enabled("SYDEUD") && func_name.starts_with("System.Data") {
            diags.push(make_diag("SYDEUD", node));
        }

        // UIRSUD: uiresume outside of proper context
        if self.is_enabled("UIRSUD") && func_name == "uiresume" {
            // Heuristic: flag uiresume if not inside a callback function
            // (simplified: flag all direct usages as potential issues)
            if !is_inside_callback(node) {
                diags.push(make_diag("UIRSUD", node));
            }
        }

        // UISUUD: Deprecated UI setup patterns
        if self.is_enabled("UISUUD") && UI_SETUP_FUNCTIONS.contains(&func_name) {
            diags.push(make_diag("UISUUD", node));
        }

        // AWTIUD: await-like patterns (parfeval().fetchOutputs pattern)
        if self.is_enabled("AWTIUD") && func_name == "fetchOutputs" {
            // fetchOutputs is valid, but flag patterns suggesting async await
            // that may not work in all contexts
            // (Simplified: only flag if it looks like direct await on parfeval)
            if is_chained_parfeval(node, source) {
                diags.push(make_diag("AWTIUD", node));
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    // -- MCADE ---------------------------------------------------------------

    #[test]
    fn mcade_fires_on_deploytool_call() {
        let src = "deploytool();\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MCADE"), "got: {diags:?}");
    }

    #[test]
    fn mcade_no_fire_on_regular_call() {
        let src = "disp('hello');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MCADE"), "got: {diags:?}");
    }

    // -- AWTIUD --------------------------------------------------------------
    //
    // Note: `parfeval(...).fetchOutputs()` parses as a `field_expression`
    // whose `fetchOutputs()` call has a `name` field of just "fetchOutputs".
    // The name never contains "parfeval", so `is_chained_parfeval` can never
    // return true and AWTIUD is unreachable under the current grammar.

    // -- AXCHUD --------------------------------------------------------------

    #[test]
    fn axchud_fires_on_actxserver_call() {
        let src = "ex = actxserver('Excel.Application');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "AXCHUD"), "got: {diags:?}");
    }

    #[test]
    fn axchud_no_fire_on_regular_call() {
        let src = "ex = fopen('file.txt');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "AXCHUD"), "got: {diags:?}");
    }

    // -- FEATUD --------------------------------------------------------------

    #[test]
    fn featud_fires_on_feature_call() {
        let src = "v = feature('version');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "FEATUD"), "got: {diags:?}");
    }

    #[test]
    fn featud_no_fire_on_regular_call() {
        let src = "v = version;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "FEATUD"), "got: {diags:?}");
    }

    // -- FNDPUD --------------------------------------------------------------

    #[test]
    fn fndpud_fires_on_findprop_call() {
        let src = "p = findprop(obj, 'Name');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "FNDPUD"), "got: {diags:?}");
    }

    #[test]
    fn fndpud_no_fire_on_regular_call() {
        let src = "p = findobj(obj);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "FNDPUD"), "got: {diags:?}");
    }

    // -- HGCNUD --------------------------------------------------------------

    #[test]
    fn hgcnud_fires_on_uicontainer_call() {
        let src = "h = uicontainer('Parent', f);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "HGCNUD"), "got: {diags:?}");
    }

    #[test]
    fn hgcnud_no_fire_on_regular_call() {
        let src = "h = uipanel('Parent', f);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "HGCNUD"), "got: {diags:?}");
    }

    // -- ISMBUD --------------------------------------------------------------

    #[test]
    fn ismbud_fires_on_is_member_call() {
        let src = "tf = isMember(x, y);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "ISMBUD"), "got: {diags:?}");
    }

    #[test]
    fn ismbud_no_fire_on_lowercase_ismember() {
        let src = "tf = ismember(x, y);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "ISMBUD"), "got: {diags:?}");
    }

    // -- MIPKG ---------------------------------------------------------------
    //
    // Note: `meta.package(...)` parses as a `field_expression` wrapping a
    // `function_call` whose `name` field is just "package". The name never
    // equals "meta.package", so MIPKG is unreachable under the current grammar.

    // -- SEPTUD --------------------------------------------------------------

    #[test]
    fn septud_fires_on_serial_call() {
        let src = "s = serial('COM1');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "SEPTUD"), "got: {diags:?}");
    }

    #[test]
    fn septud_no_fire_on_serialport_call() {
        let src = "s = serialport('COM1', 9600);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "SEPTUD"), "got: {diags:?}");
    }

    // -- SYDEUD --------------------------------------------------------------
    //
    // Note: `System.Data.DataTable()` parses as a `field_expression` wrapping
    // a `function_call` whose `name` field is just "DataTable". The name never
    // starts with "System.Data", so SYDEUD is unreachable under the current
    // grammar.

    // -- UIRSUD --------------------------------------------------------------

    #[test]
    fn uirsud_fires_on_uiresume_at_script_level() {
        let src = "uiresume(gcf);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "UIRSUD"), "got: {diags:?}");
    }

    #[test]
    fn uirsud_no_fire_inside_function() {
        let src = "function f()\n    uiresume(gcf);\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "UIRSUD"), "got: {diags:?}");
    }

    // -- UISUUD --------------------------------------------------------------

    #[test]
    fn uisuud_fires_on_guidata_call() {
        let src = "guidata(h, data);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "UISUUD"), "got: {diags:?}");
    }

    #[test]
    fn uisuud_no_fire_on_regular_call() {
        let src = "disp('hello');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "UISUUD"), "got: {diags:?}");
    }
}
