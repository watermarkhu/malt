//! Node-level check for `function_call` nodes (GFLD, SFLD, EXIST, ST2NM, FLPST, CCAT, MXFND, EFIND, UDIM, FREAD, N2UNI, TNMLP, LAXES, MRPBW, SPRIX, TRSRT, GRIDD, RGXP1, RGXPI, TRIM1, TRIM2, STTOK, STNCI, FNDSB).

use super::*;

impl PerformanceEngine {
    /// Check a `function_call` node for various performance patterns.
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

        // GFLD: getfield usage
        if self.is_enabled("GFLD") && func_name == "getfield" {
            diags.push(make_diag("GFLD", node));
        }

        // SFLD: setfield usage
        if self.is_enabled("SFLD") && func_name == "setfield" {
            diags.push(make_diag("SFLD", node));
        }

        // EXIST: exist(..., 'file') or exist(..., 'dir')
        if self.is_enabled("EXIST") && func_name == "exist" && has_exist_type_arg(node, source) {
            diags.push(make_diag("EXIST", node));
        }

        // ST2NM: str2num
        if self.is_enabled("ST2NM") && func_name == "str2num" {
            diags.push(make_diag_with_fix(
                "ST2NM",
                node,
                Some(fix_rename_func(node, source, "str2double")),
            ));
        }

        // FLPST: flipud / fliplr
        if self.is_enabled("FLPST") && (func_name == "flipud" || func_name == "fliplr") {
            diags.push(make_diag_with_fix(
                "FLPST",
                node,
                Some(fix_rename_func(node, source, "flip")),
            ));
        }

        // CCAT: strcat usage
        if self.is_enabled("CCAT") && func_name == "strcat" {
            diags.push(make_diag("CCAT", node));
        }

        // CCAT1: cellstr concatenation via strjoin candidate
        if self.is_enabled("CCAT1") && func_name == "strjoin" {
            // strjoin is already good, but flag if this is inside a loop
            // (in practice, CCAT1 flags patterns that should use join/strjoin)
            // We flag cellstr usage patterns — keep as simple function_call check
        }

        // MXFND: nested max(max(x)) or min(min(x))
        if self.is_enabled("MXFND")
            && (func_name == "max" || func_name == "min")
            && is_nested_same_call(node, source, func_name)
        {
            diags.push(make_diag("MXFND", node));
        }

        // EFIND: find() used as subscript index
        if self.is_enabled("EFIND") && func_name == "find" && is_subscript_context(node) {
            diags.push(make_diag("EFIND", node));
        }

        // UDIM: sum/max/min/prod/mean with 1 arg (no dimension)
        if self.is_enabled("UDIM")
            && matches!(func_name, "sum" | "max" | "min" | "prod" | "mean")
            && count_args(node) == 1
        {
            diags.push(make_diag_named("UDIM", node, func_name));
        }

        // FREAD: fread without precision
        if self.is_enabled("FREAD") && func_name == "fread" && count_args(node) < 3 {
            diags.push(make_diag("FREAD", node));
        }

        // N2UNI: setdiff / union patterns
        if self.is_enabled("N2UNI") && (func_name == "setdiff" || func_name == "union") {
            diags.push(make_diag("N2UNI", node));
        }

        // TNMLP: tic/toc inside loop
        if self.is_enabled("TNMLP")
            && (func_name == "tic" || func_name == "toc")
            && is_inside_loop(node)
        {
            diags.push(make_diag("TNMLP", node));
        }

        // LAXES: gca/gcf calls (suggest caching)
        if self.is_enabled("LAXES") && (func_name == "gca" || func_name == "gcf") {
            diags.push(make_diag("LAXES", node));
        }

        // MRPBW: im2bw
        if self.is_enabled("MRPBW") && func_name == "im2bw" {
            diags.push(make_diag_with_fix(
                "MRPBW",
                node,
                Some(fix_rename_func(node, source, "imbinarize")),
            ));
        }

        // TRSRT: sort then index — detect sort() call inside subscript or
        // assigned then immediately indexed. Simplified: flag sort() when
        // used as a subscript argument.
        if self.is_enabled("TRSRT") && func_name == "sort" && is_subscript_context(node) {
            diags.push(make_diag("TRSRT", node));
        }

        // RGXPI: regexp/regexpi with 'ignorecase' option
        if self.is_enabled("RGXPI")
            && func_name == "regexp"
            && has_string_arg(node, source, "ignorecase")
        {
            diags.push(make_diag("RGXPI", node));
        }

        // RGXP1: regexp/regexpi with overly simple patterns
        if self.is_enabled("RGXP1")
            && (func_name == "regexp" || func_name == "regexpi")
            && has_simple_regex_pattern(node, source)
        {
            diags.push(make_diag("RGXP1", node));
        }

        // TRIM1: deblank → strtrim
        if self.is_enabled("TRIM1") && func_name == "deblank" {
            diags.push(make_diag_with_fix(
                "TRIM1",
                node,
                Some(fix_rename_func(node, source, "strtrim")),
            ));
        }

        // TRIM2: strtrim → strip
        if self.is_enabled("TRIM2") && func_name == "strtrim" {
            diags.push(make_diag_with_fix(
                "TRIM2",
                node,
                Some(fix_rename_func(node, source, "strip")),
            ));
        }

        // STTOK: strtok inside loop
        if self.is_enabled("STTOK") && func_name == "strtok" && is_inside_loop(node) {
            diags.push(make_diag("STTOK", node));
        }

        // FNDSB: findstr
        if self.is_enabled("FNDSB") && func_name == "findstr" {
            diags.push(make_diag_with_fix(
                "FNDSB",
                node,
                Some(fix_rename_func(node, source, "contains")),
            ));
        }

        // SPRIX: sparse indexing with logical — detect sparse() inside subscript
        if self.is_enabled("SPRIX") && func_name == "sparse" && is_subscript_context(node) {
            diags.push(make_diag("SPRIX", node));
        }

        // GRIDD: repmat for grid generation
        if self.is_enabled("GRIDD") && func_name == "repmat" {
            diags.push(make_diag("GRIDD", node));
        }

        // STNCI: lower(strcmp(...)) or upper(strcmp(...)) pattern
        if self.is_enabled("STNCI")
            && (func_name == "lower" || func_name == "upper")
            && has_strcmp_arg(node, source)
        {
            diags.push(make_diag("STNCI", node));
        }

        // STCCS: ~isempty(strfind(...)) → contains
        // This requires checking unary_operator parent, handled in check_unary

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- GFLD / SFLD ----------------------------------------------------------

    #[test]
    fn gfld_fires_on_getfield() {
        let src = "v = getfield(s, 'field');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "GFLD"), "got: {diags:?}");
    }

    #[test]
    fn gfld_not_fire_on_dot_access() {
        let src = "v = s.field;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "GFLD"), "got: {diags:?}");
    }

    #[test]
    fn sfld_fires_on_setfield() {
        let src = "setfield(s, 'field', v);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "SFLD"), "got: {diags:?}");
    }

    #[test]
    fn sfld_not_fire_on_dot_assignment() {
        let src = "s.field = v;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "SFLD"), "got: {diags:?}");
    }

    // -- EXIST ----------------------------------------------------------------

    #[test]
    fn exist_fires_with_type_argument() {
        let src = "if exist('foo', 'file')\n    y = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EXIST"), "got: {diags:?}");
    }

    #[test]
    fn exist_not_fire_without_type_argument() {
        let src = "if exist('foo')\n    y = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EXIST"), "got: {diags:?}");
    }

    // -- ST2NM ----------------------------------------------------------------

    #[test]
    fn st2nm_fires_on_str2num() {
        let src = "x = str2num('1 2');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "ST2NM"), "got: {diags:?}");
    }

    #[test]
    fn st2nm_not_fire_on_str2double() {
        let src = "x = str2double('1 2');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "ST2NM"), "got: {diags:?}");
    }

    // -- FLPST ----------------------------------------------------------------

    #[test]
    fn flpst_fires_on_flipud() {
        let src = "x = flipud(y);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "FLPST"), "got: {diags:?}");
    }

    #[test]
    fn flpst_fires_on_fliplr() {
        let src = "x = fliplr(y);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "FLPST"), "got: {diags:?}");
    }

    #[test]
    fn flpst_not_fire_on_flip() {
        let src = "x = flip(y);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "FLPST"), "got: {diags:?}");
    }

    // -- CCAT -----------------------------------------------------------------

    #[test]
    fn ccat_fires_on_strcat() {
        let src = "s = strcat('a', 'b');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "CCAT"), "got: {diags:?}");
    }

    #[test]
    fn ccat_not_fire_on_bracket_concatenation() {
        let src = "s = ['a', 'b'];\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "CCAT"), "got: {diags:?}");
    }

    // -- MXFND ----------------------------------------------------------------

    #[test]
    fn mxfnd_fires_on_nested_max() {
        let src = "x = max(max(y));\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MXFND"), "got: {diags:?}");
    }

    #[test]
    fn mxfnd_fires_on_nested_min() {
        let src = "x = min(min(y));\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MXFND"), "got: {diags:?}");
    }

    #[test]
    fn mxfnd_not_fire_on_single_max() {
        let src = "x = max(y, z);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MXFND"), "got: {diags:?}");
    }

    // -- EFIND ----------------------------------------------------------------

    #[test]
    fn efind_fires_on_find_as_subscript() {
        let src = "x = y(find(z));\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EFIND"), "got: {diags:?}");
    }

    #[test]
    fn efind_not_fire_on_standalone_find() {
        let src = "idx = find(z);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EFIND"), "got: {diags:?}");
    }

    // -- UDIM -----------------------------------------------------------------

    #[test]
    fn udim_fires_on_single_argument_reduction() {
        let src = "s = sum(x);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "UDIM"), "got: {diags:?}");
    }

    #[test]
    fn udim_fires_on_single_argument_mean() {
        let src = "m = mean(x);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "UDIM"), "got: {diags:?}");
    }

    #[test]
    fn udim_not_fire_with_dimension_argument() {
        let src = "s = sum(x, 2);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "UDIM"), "got: {diags:?}");
    }

    // -- FREAD ----------------------------------------------------------------

    #[test]
    fn fread_fires_without_precision() {
        let src = "d = fread(fid, 100);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "FREAD"), "got: {diags:?}");
    }

    #[test]
    fn fread_not_fire_with_precision() {
        let src = "d = fread(fid, 100, 'uint8');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "FREAD"), "got: {diags:?}");
    }

    // -- N2UNI ----------------------------------------------------------------

    #[test]
    fn n2uni_fires_on_setdiff() {
        let src = "c = setdiff(a, b);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "N2UNI"), "got: {diags:?}");
    }

    #[test]
    fn n2uni_fires_on_union() {
        let src = "c = union(a, b);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "N2UNI"), "got: {diags:?}");
    }

    #[test]
    fn n2uni_not_fire_on_unique() {
        let src = "c = unique(a);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "N2UNI"), "got: {diags:?}");
    }

    // -- TNMLP ----------------------------------------------------------------

    #[test]
    fn tnmlp_fires_on_tic_inside_loop() {
        let src = "for i = 1:10\n    tic();\n    y = i;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "TNMLP"), "got: {diags:?}");
    }

    #[test]
    fn tnmlp_fires_on_toc_inside_loop() {
        let src = "for i = 1:10\n    y = i;\n    toc();\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "TNMLP"), "got: {diags:?}");
    }

    #[test]
    fn tnmlp_not_fire_outside_loop() {
        let src = "tic();\ny = 1;\ntoc();\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "TNMLP"), "got: {diags:?}");
    }

    // -- LAXES ----------------------------------------------------------------

    #[test]
    fn laxes_fires_on_gca() {
        let src = "ax = gca();\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "LAXES"), "got: {diags:?}");
    }

    #[test]
    fn laxes_fires_on_gcf() {
        let src = "f = gcf();\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "LAXES"), "got: {diags:?}");
    }

    #[test]
    fn laxes_not_fire_on_other_calls() {
        let src = "ax = axes();\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "LAXES"), "got: {diags:?}");
    }

    // -- MRPBW ----------------------------------------------------------------

    #[test]
    fn mrp_bw_fires_on_im2bw() {
        let src = "bw = im2bw(I, 0.5);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "MRPBW"), "got: {diags:?}");
    }

    #[test]
    fn mrp_bw_not_fire_on_imbinarize() {
        let src = "bw = imbinarize(I);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "MRPBW"), "got: {diags:?}");
    }

    // -- SPRIX ----------------------------------------------------------------

    #[test]
    fn sprix_fires_on_sparse_in_subscript() {
        let src = "idx = A(sparse(1));\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "SPRIX"), "got: {diags:?}");
    }

    #[test]
    fn sprix_not_fire_on_standalone_sparse() {
        let src = "S = sparse(1);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "SPRIX"), "got: {diags:?}");
    }

    // -- TRSRT ----------------------------------------------------------------

    #[test]
    fn trsrt_fires_on_sort_in_subscript() {
        let src = "x = y(sort(z));\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "TRSRT"), "got: {diags:?}");
    }

    #[test]
    fn trsrt_not_fire_on_standalone_sort() {
        let src = "s = sort(z);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "TRSRT"), "got: {diags:?}");
    }

    // -- GRIDD ----------------------------------------------------------------

    #[test]
    fn gridd_fires_on_repmat() {
        let src = "M = repmat(x, 1, 3);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "GRIDD"), "got: {diags:?}");
    }

    #[test]
    fn gridd_not_fire_on_ones() {
        let src = "M = ones(3);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "GRIDD"), "got: {diags:?}");
    }

    // -- RGXP1 / RGXPI ----------------------------------------------------------

    #[test]
    fn rgxp1_fires_on_simple_regex_pattern() {
        let src = "m = regexp(s, 'foo');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "RGXP1"), "got: {diags:?}");
    }

    #[test]
    fn rgxp1_not_fire_on_metacharacter_pattern() {
        let src = "m = regexp(s, 'a+b');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "RGXP1"), "got: {diags:?}");
    }

    #[test]
    fn rgxpi_fires_on_ignorecase_option() {
        let src = "m = regexp(s, 'foo', 'ignorecase');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "RGXPI"), "got: {diags:?}");
    }

    #[test]
    fn rgxpi_not_fire_without_ignorecase() {
        let src = "m = regexp(s, 'a+b');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "RGXPI"), "got: {diags:?}");
    }

    // -- TRIM1 / TRIM2 ----------------------------------------------------------

    #[test]
    fn trim1_fires_on_deblank() {
        let src = "x = deblank(s);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "TRIM1"), "got: {diags:?}");
    }

    #[test]
    fn trim1_not_fire_on_strtrim() {
        let src = "x = strtrim(s);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "TRIM1"), "got: {diags:?}");
    }

    #[test]
    fn trim2_fires_on_strtrim() {
        let src = "x = strtrim(s);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "TRIM2"), "got: {diags:?}");
    }

    #[test]
    fn trim2_not_fire_on_strip() {
        let src = "x = strip(s);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "TRIM2"), "got: {diags:?}");
    }

    // -- STTOK ----------------------------------------------------------------

    #[test]
    fn sttok_fires_on_strtok_in_loop() {
        let src = "for i = 1:10\n    [t, r] = strtok(s);\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "STTOK"), "got: {diags:?}");
    }

    #[test]
    fn sttok_not_fire_outside_loop() {
        let src = "[t, r] = strtok(s);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "STTOK"), "got: {diags:?}");
    }

    // -- STNCI ----------------------------------------------------------------

    #[test]
    fn stnci_fires_on_lower_strcmp() {
        let src = "x = lower(strcmp(a, b));\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "STNCI"), "got: {diags:?}");
    }

    #[test]
    fn stnci_not_fire_on_bare_strcmp() {
        let src = "x = strcmp(a, b);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "STNCI"), "got: {diags:?}");
    }

    // -- FNDSB ----------------------------------------------------------------

    #[test]
    fn fndsb_fires_on_findstr() {
        let src = "k = findstr(s, 'x');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "FNDSB"), "got: {diags:?}");
    }

    #[test]
    fn fndsb_not_fire_on_contains() {
        let src = "k = contains(s, 'x');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "FNDSB"), "got: {diags:?}");
    }
}
