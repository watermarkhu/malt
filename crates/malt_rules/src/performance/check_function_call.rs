//! Node-level checks on `function_call` nodes (GFLD, SFLD, EXIST, ST2NM, FLPST, MXFND,
//! EFIND, FNDSB, UDIM, FREAD, N2UNI, LAXES, MMTC, MRPBW, TRSRT, GRIDD, RGXP1, RGXPI,
//! TRIM1, TRIM2, STTOK, STNCI, STCCS) and on `cell` nodes (CCAT, CCAT1).

use super::*;

type Node<'a> = tree_sitter::Node<'a>;

/// Named argument nodes of a `function_call`.
fn call_args<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    match find_arguments(node) {
        Some(a) => {
            let mut c = a.walk();
            a.named_children(&mut c).collect()
        }
        None => Vec::new(),
    }
}

/// Function name when `node` is a `function_call`.
fn call_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    extract_func_name(node, source)
}

/// If `node` is a call to `name`, return its arguments.
fn call_to<'a>(node: Node<'a>, source: &'a str, name: &str) -> Option<Vec<Node<'a>>> {
    (node.kind() == "function_call" && call_name(node, source) == Some(name))
        .then(|| call_args(node))
}

/// Whether the call is written with braces (`A{...}`) rather than parentheses.
fn is_brace_index(node: Node, source: &str) -> bool {
    node.child_by_field_name("name")
        .is_some_and(|n| source[n.end_byte()..].trim_start().starts_with('{'))
}

/// Whether the node is a transpose (`'` or `.'`) expression.
fn is_transpose(node: Node, source: &str) -> bool {
    node.kind() == "postfix_operator" && node_text(node, source).trim_end().ends_with('\'')
}

/// The content of a plain string literal (`'abc'` or `"abc"`).
fn string_literal<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    let t = node_text(node, source);
    let b = t.as_bytes();
    (t.len() >= 2 && (b[0] == b'\'' || b[0] == b'"') && b[b.len() - 1] == b[0])
        .then(|| &t[1..t.len() - 1])
}

/// Whether `node` is inside a `while` loop (within the current function).
fn is_inside_while(node: Node) -> bool {
    let mut cur = node.parent();
    while let Some(p) = cur {
        match p.kind() {
            "while_statement" => return true,
            "function_definition" => return false,
            _ => {}
        }
        cur = p.parent();
    }
    false
}

/// Whether `name` is a variable assigned before `node` in the file.
fn is_known_variable(node: Node, source: &str, name: &str) -> bool {
    let mut root = node;
    while let Some(p) = root.parent() {
        root = p;
    }
    defined_before(root, source, name, node.start_byte())
}

/// Reductions taking a DIM argument.
const DIM_FUNCS: &[&str] = &[
    "sum", "prod", "max", "min", "mean", "median", "mode", "any", "all", "cumsum", "cumprod",
    "std", "var",
];

impl PerformanceEngine {
    /// Check a `function_call` node for various performance patterns.
    pub(crate) fn check_function_call<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let func_name = match call_name(node, source) {
            Some(n) => n,
            None => return diags,
        };
        let args = call_args(node);
        let arg0 = args.first().copied();
        let on = |id: &str| self.is_enabled(id);

        match func_name {
            // GFLD / SFLD
            "getfield" if on("GFLD") => diags.push(make_diag("GFLD", node)),
            "setfield" if on("SFLD") => diags.push(make_diag("SFLD", node)),
            // EXIST: single-argument form
            "exist" if on("EXIST") && args.len() == 1 => diags.push(make_diag("EXIST", node)),
            // ST2NM
            "str2num" if on("ST2NM") => diags.push(make_diag_with_fix(
                "ST2NM",
                node,
                // str2num evaluates expressions and returns arrays, str2double
                // returns NaN for them: a suggestion, not a safe rewrite.
                Some(fix_rename_func(node, source, "str2double").unsafe_fix()),
            )),
            // GRIDD
            "griddata" if on("GRIDD") => diags.push(make_diag("GRIDD", node)),
            _ => {}
        }

        // FLPST: fliplr(sort(x)) / flipud(sort(x)) -> sort(x, 'descend')
        if on("FLPST") && matches!(func_name, "fliplr" | "flipud") && args.len() == 1 {
            if let Some(sort_args) = call_to(args[0], source, "sort") {
                if sort_args.len() == 1 {
                    let fix = Fix::new(
                        node.start_byte()..node.end_byte(),
                        format!("sort({}, 'descend')", node_text(sort_args[0], source)),
                    )
                    // The flip direction and the sort dimension may differ for matrices.
                    .unsafe_fix();
                    diags.push(make_diag_with_fix("FLPST", node, Some(fix)));
                }
            }
        }

        // MXFND: min(find(A)) / max(find(A))
        if on("MXFND")
            && matches!(func_name, "max" | "min")
            && args.len() == 1
            && call_to(args[0], source, "find").is_some()
        {
            diags.push(make_diag("MXFND", node));
        }

        // EFIND: isempty(find(A))
        if on("EFIND") && func_name == "isempty" && args.len() == 1 {
            if let Some(find_args) = call_to(args[0], source, "find") {
                if find_args.len() == 1 {
                    diags.push(make_diag("EFIND", node));
                }
            }
        }

        // RGXP1 / RGXPI: isempty(regexp(...)) without the 'once' option
        if func_name == "isempty" && args.len() == 1 {
            for (inner, id) in [("regexp", "RGXP1"), ("regexpi", "RGXPI")] {
                if on(id)
                    && call_to(args[0], source, inner).is_some()
                    && !has_string_arg(args[0], source, "once")
                {
                    diags.push(make_diag(id, node));
                }
            }
        }

        // FNDSB: find(L) used directly as an index into a variable
        if on("FNDSB") && func_name == "find" && args.len() == 1 && is_subscript_context(node) {
            let parent_call = node.parent().and_then(|p| {
                if p.kind() == "arguments" {
                    p.parent()
                } else {
                    Some(p)
                }
            });
            if let Some(name) = parent_call.and_then(|c| call_name(c, source)) {
                if is_known_variable(node, source, name) {
                    diags.push(make_diag("FNDSB", node));
                }
            }
        }

        // UDIM: reduction applied to a transposed array
        if on("UDIM")
            && DIM_FUNCS.contains(&func_name)
            && args.len() == 1
            && is_transpose(args[0], source)
        {
            diags.push(make_diag_named("UDIM", node, func_name));
        }

        // FREAD: char(fread(...)) -> fread(..., '*char')
        if on("FREAD") && func_name == "char" && args.len() == 1 {
            if let Some(fread_args) = call_to(args[0], source, "fread") {
                let precision_ok = match fread_args.get(2) {
                    None => true,
                    Some(p) => string_literal(*p, source).is_some_and(|s| {
                        matches!(
                            s,
                            "uint8" | "uchar" | "char" | "schar" | "int8" | "uint8=>uint8"
                        )
                    }),
                };
                if precision_ok {
                    diags.push(make_diag("FREAD", node));
                }
            }
        }

        // N2UNI: native2unicode(fread(...), ...)
        if on("N2UNI")
            && func_name == "native2unicode"
            && arg0.is_some_and(|a| call_to(a, source, "fread").is_some())
        {
            diags.push(make_diag("N2UNI", node));
        }

        // LAXES: axes(h) inside a loop
        if on("LAXES") && func_name == "axes" && !args.is_empty() && is_inside_loop(node) {
            diags.push(make_diag("LAXES", node));
        }

        // MMTC: mat2cell(x, ones(..), ones(..)) -> num2cell
        if on("MMTC")
            && func_name == "mat2cell"
            && args.len() >= 3
            && args[1..]
                .iter()
                .all(|a| call_to(*a, source, "ones").is_some())
        {
            diags.push(make_diag("MMTC", node));
        }

        // MRPBW: regionprops(bwlabel(bw), ...)
        if on("MRPBW") && func_name == "regionprops" {
            if let Some(bw_args) = arg0.and_then(|a| call_to(a, source, "bwlabel")) {
                let fix = (bw_args.len() == 1).then(|| {
                    let a = arg0.expect("checked above");
                    // bwlabel also returns labels for non-logical input: unsafe.
                    Fix::new(
                        a.start_byte()..a.end_byte(),
                        format!("logical({})", node_text(bw_args[0], source)),
                    )
                    .unsafe_fix()
                });
                diags.push(make_diag_with_fix("MRPBW", node, fix));
            }
        }

        // TRSRT: sort(x')
        if on("TRSRT") && func_name == "sort" && arg0.is_some_and(|a| is_transpose(a, source)) {
            diags.push(make_diag("TRSRT", node));
        }

        // TRIM1: nested fliplr/deblank
        if on("TRIM1") && matches!(func_name, "deblank" | "fliplr") {
            self.check_trim1(node, source, &mut diags);
        }

        // TRIM2: deblank(strjust(s, 'left'))
        if on("TRIM2") && func_name == "deblank" && args.len() == 1 {
            if let Some(sj) = call_to(args[0], source, "strjust") {
                if sj.len() == 2 && string_literal(sj[1], source) == Some("left") {
                    let fix = Fix::new(
                        node.start_byte()..node.end_byte(),
                        format!("strtrim({})", node_text(sj[0], source)),
                    )
                    .unsafe_fix();
                    diags.push(make_diag_with_fix("TRIM2", node, Some(fix)));
                }
            }
        }

        // STTOK: strtok inside a while loop
        if on("STTOK") && func_name == "strtok" && is_inside_while(node) {
            diags.push(make_diag("STTOK", node));
        }

        // STNCI: strncmp(lower(a), lower(b), n)
        if on("STNCI")
            && func_name == "strncmp"
            && args.iter().take(2).any(|a| {
                matches!(call_name(*a, source), Some("lower" | "upper"))
                    && a.kind() == "function_call"
            })
        {
            diags.push(make_diag("STNCI", node));
        }

        // STCCS: strcmpi/strncmpi against a literal that has no letters
        if on("STCCS")
            && matches!(func_name, "strcmpi" | "strncmpi")
            && args.iter().take(2).any(|a| {
                string_literal(*a, source).is_some_and(|s| !s.chars().any(|c| c.is_alphabetic()))
            })
        {
            diags.push(make_diag("STCCS", node));
        }

        diags
    }

    /// TRIM1: `fliplr(deblank(fliplr(deblank(s))))` and the mirrored form.
    fn check_trim1(&self, node: Node, source: &str, diags: &mut Vec<Diagnostic>) {
        // Only report the outermost call of a chain.
        if let Some(parent) = node.parent().and_then(|p| {
            if p.kind() == "arguments" {
                p.parent()
            } else {
                None
            }
        }) {
            if matches!(call_name(parent, source), Some("deblank" | "fliplr")) {
                return;
            }
        }
        let mut names = Vec::new();
        let mut cur = node;
        let mut innermost = None;
        while let Some(name) = call_name(cur, source).filter(|n| matches!(*n, "deblank" | "fliplr"))
        {
            let a = call_args(cur);
            if a.len() != 1 {
                break;
            }
            names.push(name);
            innermost = Some(a[0]);
            cur = a[0];
            if cur.kind() != "function_call" {
                break;
            }
        }
        if names.len() >= 4 && names.len() % 2 == 0 && names.windows(2).all(|w| w[0] != w[1]) {
            if let Some(inner) = innermost {
                let fix = Fix::new(
                    node.start_byte()..node.end_byte(),
                    format!("strtrim({})", node_text(inner, source)),
                )
                .unsafe_fix();
                diags.push(make_diag_with_fix("TRIM1", node, Some(fix)));
            }
        }
    }

    /// Check a `cell` literal for CCAT (`{A{:} X}`) and CCAT1 (`{A{I}}`).
    pub(crate) fn check_cell<'a>(&self, node: Node<'a>, source: &'a str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut c = node.walk();
        let rows: Vec<Node> = node.named_children(&mut c).collect();
        if rows.len() != 1 || rows[0].kind() != "row" {
            return diags;
        }
        let mut c2 = rows[0].walk();
        let elems: Vec<Node> = rows[0].named_children(&mut c2).collect();
        let brace_call = |e: &Node| e.kind() == "function_call" && is_brace_index(*e, source);
        let is_colon = |e: &Node| {
            let a = call_args(*e);
            a.len() == 1 && a[0].kind() == "spread_operator"
        };

        if self.is_enabled("CCAT")
            && elems.len() >= 2
            && elems.iter().any(|e| brace_call(e) && is_colon(e))
        {
            diags.push(make_diag("CCAT", node));
        }
        if self.is_enabled("CCAT1")
            && elems.len() == 1
            && brace_call(&elems[0])
            && !is_colon(&elems[0])
        {
            diags.push(make_diag("CCAT1", node));
        }
        diags
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    fn fires(src: &str, id: &str) -> bool {
        has_id(&lint_nodes(&*engine(), src), id)
    }

    #[test]
    fn gfld_sfld() {
        assert!(fires("v = getfield(s, 'field');\n", "GFLD"));
        assert!(!fires("v = s.field;\n", "GFLD"));
        assert!(fires("setfield(s, 'field', v);\n", "SFLD"));
        assert!(!fires("s.field = v;\n", "SFLD"));
    }

    #[test]
    fn exist_fires_on_single_argument_only() {
        assert!(fires("if exist('foo')\n    y = 1;\nend\n", "EXIST"));
        assert!(!fires(
            "if exist('foo', 'file')\n    y = 1;\nend\n",
            "EXIST"
        ));
    }

    #[test]
    fn st2nm() {
        assert!(fires("x = str2num('1 2');\n", "ST2NM"));
        assert!(!fires("x = str2double('1 2');\n", "ST2NM"));
    }

    #[test]
    fn flpst_only_for_flipped_sort() {
        let d = lint_nodes(&*engine(), "a = fliplr(sort(a));\n");
        let f = d.iter().find(|d| d.rule_id == "FLPST").expect("FLPST");
        assert!(f.fix.is_some());
        assert!(!fires("x = flipud(y);\n", "FLPST"));
        assert!(!fires("x = flip(sort(y));\n", "FLPST"));
        assert!(!fires("x = fliplr(sort(y, 'descend'));\n", "FLPST"));
    }

    #[test]
    fn ccat_fires_on_extract_and_rebuild() {
        assert!(fires("c = {A{:} x};\n", "CCAT"));
        assert!(!fires("c = {A{:}};\n", "CCAT"));
        assert!(!fires("c = {f(x) y};\n", "CCAT"));
        assert!(!fires("s = strcat('a', 'b');\n", "CCAT"));
    }

    #[test]
    fn ccat1_fires_on_single_brace_index() {
        assert!(fires("c = { A{I} };\n", "CCAT1"));
        assert!(!fires("c = { A(I) };\n", "CCAT1"));
        assert!(!fires("c = { f(x), g(y) };\n", "CCAT1"));
    }

    #[test]
    fn mxfnd_fires_on_min_max_of_find() {
        assert!(fires("x = max(find(y));\n", "MXFND"));
        assert!(fires("x = min(find(y));\n", "MXFND"));
        assert!(!fires("x = max(max(y));\n", "MXFND"));
        assert!(!fires("x = max(y, z);\n", "MXFND"));
    }

    #[test]
    fn efind_fires_on_isempty_find() {
        assert!(fires("if isempty(find(z))\nend\n", "EFIND"));
        assert!(!fires("if isempty(find(z, 1))\nend\n", "EFIND"));
        assert!(!fires("idx = find(z);\n", "EFIND"));
    }

    #[test]
    fn fndsb_fires_on_find_index_into_variable() {
        assert!(fires("y = 1:3;\nx = y(find(z));\n", "FNDSB"));
        assert!(!fires("x = y(find(z));\n", "FNDSB")); // y may be a function
        assert!(!fires("y = 1;\nidx = find(z);\n", "FNDSB"));
        assert!(!fires("if isempty(find(z))\nend\n", "FNDSB"));
    }

    #[test]
    fn udim_fires_on_reduction_of_transpose() {
        let d = lint_nodes(&*engine(), "s = sum(x');\n");
        let u = d.iter().find(|d| d.rule_id == "UDIM").expect("UDIM");
        assert!(u.message.contains("argument to sum"), "{}", u.message);
        assert!(!u.message.contains("VAR_NAME"));
        assert!(fires("m = mean(x.');\n", "UDIM"));
        assert!(!fires("s = sum(x);\n", "UDIM"));
        assert!(!fires("s = sum(x, 2);\n", "UDIM"));
    }

    #[test]
    fn fread_fires_on_char_of_fread() {
        assert!(fires("d = char(fread(fid));\n", "FREAD"));
        assert!(fires("d = char(fread(fid, inf, 'uint8'));\n", "FREAD"));
        assert!(!fires("d = char(fread(fid, inf, 'uint16'));\n", "FREAD"));
        assert!(!fires("d = fread(fid, 100);\n", "FREAD"));
    }

    #[test]
    fn n2uni_fires_on_native2unicode_of_fread() {
        assert!(fires("s = native2unicode(fread(fid), 'UTF-8');\n", "N2UNI"));
        assert!(!fires("s = native2unicode(b, 'UTF-8');\n", "N2UNI"));
        assert!(!fires("c = setdiff(a, b);\n", "N2UNI"));
    }

    #[test]
    fn laxes_fires_on_axes_h_in_loop() {
        assert!(fires("for i = 1:3\n    axes(h);\nend\n", "LAXES"));
        assert!(!fires("axes(h);\n", "LAXES"));
        assert!(!fires("for i = 1:3\n    axes();\nend\n", "LAXES"));
        assert!(!fires("ax = gca();\n", "LAXES"));
    }

    #[test]
    fn mmtc_fires_on_mat2cell_with_ones() {
        assert!(fires("m = mat2cell(x, ones(3,1), ones(1,3));\n", "MMTC"));
        assert!(!fires("m = mat2cell(x, [1 2], 3);\n", "MMTC"));
        assert!(!fires("z = x .* x;\n", "MMTC"));
    }

    #[test]
    fn mrpbw_fires_on_regionprops_of_bwlabel() {
        let d = lint_nodes(&*engine(), "s = regionprops(bwlabel(bw), 'Area');\n");
        let m = d.iter().find(|d| d.rule_id == "MRPBW").expect("MRPBW");
        assert!(m.fix.is_some());
        assert!(!fires("s = regionprops(bw, 'Area');\n", "MRPBW"));
        assert!(!fires("bw = im2bw(I, 0.5);\n", "MRPBW"));
    }

    #[test]
    fn trsrt_fires_on_sort_of_transpose() {
        assert!(fires("y = sort(x')';\n", "TRSRT"));
        assert!(!fires("y = sort(x, 2);\n", "TRSRT"));
        assert!(!fires("x = y(sort(z));\n", "TRSRT"));
    }

    #[test]
    fn gridd_fires_on_griddata() {
        assert!(fires("v = griddata(x, y, z, xi, yi);\n", "GRIDD"));
        assert!(!fires("M = repmat(x, 1, 3);\n", "GRIDD"));
    }

    #[test]
    fn rgxp_fires_on_isempty_of_regexp() {
        assert!(fires("e = isempty(regexp(s, p));\n", "RGXP1"));
        assert!(fires("e = ~isempty(regexp(s, p));\n", "RGXP1"));
        assert!(!fires("e = isempty(regexp(s, p, 'once'));\n", "RGXP1"));
        assert!(!fires("m = regexp(s, 'foo');\n", "RGXP1"));
        assert!(fires("e = isempty(regexpi(s, p));\n", "RGXPI"));
        assert!(!fires("e = isempty(regexp(s, p));\n", "RGXPI"));
    }

    #[test]
    fn trim1_fires_on_nested_fliplr_deblank() {
        let d = lint_nodes(&*engine(), "t = fliplr(deblank(fliplr(deblank(s))));\n");
        let n = d.iter().filter(|d| d.rule_id == "TRIM1").count();
        assert_eq!(n, 1, "got: {d:?}");
        assert!(d
            .iter()
            .find(|d| d.rule_id == "TRIM1")
            .unwrap()
            .fix
            .is_some());
        assert!(fires("t = deblank(fliplr(deblank(fliplr(s))));\n", "TRIM1"));
        assert!(!fires("x = deblank(s);\n", "TRIM1"));
        assert!(!fires("x = fliplr(deblank(s));\n", "TRIM1"));
    }

    #[test]
    fn trim2_fires_on_deblank_strjust_left() {
        assert!(fires("t = deblank(strjust(s, 'left'));\n", "TRIM2"));
        assert!(!fires("t = deblank(strjust(s, 'right'));\n", "TRIM2"));
        assert!(!fires("t = strtrim(s);\n", "TRIM2"));
    }

    #[test]
    fn sttok_fires_only_in_while_loop() {
        assert!(fires(
            "while ~isempty(r)\n    [t, r] = strtok(r);\nend\n",
            "STTOK"
        ));
        assert!(!fires(
            "for i = 1:10\n    [t, r] = strtok(s);\nend\n",
            "STTOK"
        ));
        assert!(!fires("[t, r] = strtok(s);\n", "STTOK"));
    }

    #[test]
    fn stnci_fires_on_strncmp_of_lower() {
        assert!(fires("x = strncmp(lower(a), lower(b), 3);\n", "STNCI"));
        assert!(fires("x = strncmp(upper(a), b, 3);\n", "STNCI"));
        assert!(!fires("x = strncmp(a, b, 3);\n", "STNCI"));
        assert!(!fires("x = lower(strcmp(a, b));\n", "STNCI"));
    }

    #[test]
    fn stccs_fires_on_case_insensitive_compare_without_letters() {
        assert!(fires("x = strcmpi(a, '123');\n", "STCCS"));
        assert!(!fires("x = strcmpi(a, 'abc');\n", "STCCS"));
        assert!(!fires("x = ~isempty(strfind(a, 'b'));\n", "STCCS"));
    }
}
