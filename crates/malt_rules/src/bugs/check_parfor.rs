use super::*;
use std::collections::HashMap;

/// Functions that access or modify the workspace non-transparently, which is
/// invalid inside a `parfor` body.
/// Probed: `eval`, `evalc` and `clear` give PFBFN. `save`, `load`, `inputname`,
/// `evalin` and `assignin` get their own checks (PFSV, PFLD, PFINPT, PFEVB).
const NON_TRANSPARENT: &[&str] = &["eval", "evalc", "clear", "clearvars"];

/// Call `f` for `node` and every descendant, skipping nothing.
fn visit<'a>(node: Node<'a>, f: &mut impl FnMut(Node<'a>)) {
    f(node);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, f);
    }
}

/// Whether `node` is the field name of a field expression (`s.name`).
fn is_field_name(node: Node) -> bool {
    node.parent().is_some_and(|p| {
        p.kind() == "field_expression"
            && p.child_by_field_name("field")
                .is_some_and(|f| f.id() == node.id())
    })
}

/// First read of variable `name` in the statements following `parfor`, in the
/// same block, up to the point where `name` is assigned again.
fn later_use<'a>(parfor: Node<'a>, name: &str, source: &str) -> Option<Node<'a>> {
    let mut sibling = parfor.next_sibling();
    while let Some(stmt) = sibling {
        sibling = stmt.next_sibling();
        if stmt.kind() == "comment" || !stmt.is_named() {
            continue;
        }
        if stmt.kind() == "for_statement"
            && extract_for_variable(stmt, source).as_deref() == Some(name)
        {
            break;
        }
        // A bare `t` is a command with one command word in this grammar: a read of
        // `t`. `clear t` ends the value (probed: no PFTUSE, the later read is NODEF).
        if stmt.kind() == "command" {
            if crate::analysis::liveness::clear_names(node_text(stmt, source))
                .is_some_and(|names| names.iter().any(|n| n == name))
            {
                return None;
            }
            if stmt.named_child_count() == 1 {
                if let Some(word) = stmt
                    .named_child(0)
                    .filter(|w| w.kind() == "command_name" && node_text(*w, source).trim() == name)
                {
                    return Some(word);
                }
            }
        }
        let assigned = stmt.kind() == "assignment"
            && stmt.child_by_field_name("left").is_some_and(|l| {
                // `name = ...`, or `[a, name] = ...`: a whole target is a write (probed:
                // `[i, z] = deal(1, 2)` after the loop is silent for PFUIXE).
                (l.kind() == "identifier" && node_text(l, source).trim() == name)
                    || (!matches!(
                        l.kind(),
                        "function_call" | "field_expression" | "cell_index"
                    ) && {
                        let mut stack = vec![l];
                        let mut found = false;
                        while let Some(n) = stack.pop() {
                            if n.kind() == "identifier" && node_text(n, source).trim() == name {
                                found = true;
                                break;
                            }
                            let mut c = n.walk();
                            stack.extend(n.children(&mut c));
                        }
                        found
                    })
            });
        // Only the right-hand side of an assignment reads a variable: `name(2) = 5` is not
        // a read, but a field store `name.f = 1` is (probed: PFTUSE for the latter only).
        let scope = match stmt.kind() {
            "assignment"
                if stmt
                    .child_by_field_name("left")
                    .is_some_and(|l| l.kind() == "field_expression") =>
            {
                stmt
            }
            "assignment" => stmt.child_by_field_name("right").unwrap_or(stmt),
            _ => stmt,
        };
        let mut found = None;
        visit(scope, &mut |n| {
            if found.is_none()
                && n.kind() == "identifier"
                && node_text(n, source).trim() == name
                && !is_field_name(n)
            {
                found = Some(n);
            }
        });
        if found.is_some() {
            return found;
        }
        if assigned {
            break;
        }
    }
    None
}

/// Positions (0 = first, 1 = second) of `var` among the operands of a
/// non-commutative reduction `rhs`, with a key naming the operation.
fn reduction_position(var: &str, rhs: Node, source: &str) -> Option<(&'static str, usize)> {
    let is_var = |n: Node| n.kind() == "identifier" && node_text(n, source).trim() == var;
    match rhs.kind() {
        "binary_operator" if find_operator_text(rhs, source) == "*" => {
            let (l, r) = (rhs.child(0)?, rhs.child(2)?);
            if is_var(l) {
                Some(("*", 0))
            } else if is_var(r) {
                Some(("*", 1))
            } else {
                None
            }
        }
        "matrix" => {
            let mut rows = Vec::new();
            let mut cursor = rhs.walk();
            for row in rhs.children(&mut cursor).filter(|c| c.kind() == "row") {
                let mut inner = row.walk();
                let elems: Vec<Node> = row
                    .children(&mut inner)
                    .filter(|c| c.is_named() && c.kind() != "comment")
                    .collect();
                rows.push(elems);
            }
            let (key, elems): (&'static str, Vec<Node>) = if rows.len() == 1 {
                ("[,]", rows.remove(0))
            } else {
                (
                    "[;]",
                    rows.into_iter()
                        .filter_map(|r| r.into_iter().next())
                        .collect(),
                )
            };
            if elems.len() != 2 {
                return None;
            }
            if is_var(elems[0]) {
                Some((key, 0))
            } else if is_var(elems[1]) {
                Some((key, 1))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Whether `rhs` reads `var` in an arithmetic reduction (`x = x + ...`).
fn is_self_update(var: &str, rhs: Node, source: &str) -> bool {
    let mut uses = false;
    visit(rhs, &mut |n| {
        if n.kind() == "identifier" && node_text(n, source).trim() == var && !is_field_name(n) {
            uses = true;
        }
    });
    uses
}

fn diag(rule_id: &'static str, node: Node, message: String) -> Diagnostic {
    let pos = node.start_position();
    Diagnostic {
        rule_id,
        message,
        severity: Severity::Error,
        byte_range: node.start_byte()..node.end_byte(),
        line: pos.row + 1,
        column: pos.column + 1,
        fix: None,
    }
}

impl BugsEngine {
    /// PFUIXE, PFBFN, PFWHOS, PFTUSE, PFRNC: problems with `parfor` loops.
    pub(crate) fn check_parfor_issues(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        visit(root, &mut |node| {
            if node.kind() == "for_statement" && node_text(node, source).starts_with("parfor") {
                Self::check_parfor(node, source, &mut diagnostics);
            }
        });
        diagnostics
    }

    fn check_parfor(parfor: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let loop_var = extract_for_variable(parfor, source);
        // An empty loop (`parfor k=1:n, end`) has no block but still has an index (probed).
        let body = find_block_child(parfor);

        // PFUIXE: the loop index is unavailable after the loop. MATLAB reports it at the
        // loop header, not at the later use (probed), and names the use's line.
        if let Some(lv) = loop_var.as_deref() {
            if let Some(use_node) = later_use(parfor, lv, source) {
                let header =
                    crate::good_practices::find_parfor_index_identifier(parfor).unwrap_or(parfor);
                diagnostics.push(diag(
                    "PFUIXE",
                    header,
                    format!("The index variable '{lv}' might be used after the PARFOR loop on line {}, but it is unavailable after the loop.", use_node.start_position().row + 1),
                ));
            }
        }

        let Some(body) = body else {
            return;
        };

        let mut temps: Vec<String> = Vec::new();
        let mut temp_sites: HashMap<String, Node> = HashMap::new();
        let mut reductions: Vec<String> = Vec::new();
        let mut positions: HashMap<(String, &'static str), usize> = HashMap::new();
        let mut rnc: Vec<(Node, String)> = Vec::new();
        let mut calls: Vec<Diagnostic> = Vec::new();

        visit(body, &mut |node| match node.kind() {
            // A nested `for j` sets `j` in the body; MATLAB reports PFTUSE at its header
            // (the iterator), not at the later use (probed).
            "for_statement" if !node_text(node, source).starts_with("parfor") => {
                let mut cursor = node.walk();
                let iterator = node
                    .children(&mut cursor)
                    .find(|c| c.kind() == "iterator")
                    .and_then(|it| it.named_child(0));
                if let (Some(id), Some(var)) = (iterator, extract_for_variable(node, source)) {
                    if !temps.contains(&var) {
                        temp_sites.insert(var.clone(), id);
                        temps.push(var);
                    }
                }
            }
            "function_call" => {
                let Some(name) = extract_call_name(node, source) else {
                    return;
                };
                if name == "who" || name == "whos" {
                    let has_file = collect_call_args(node, source)
                        .iter()
                        .any(|a| a.trim().trim_matches(|c| c == '\'' || c == '"') == "-file");
                    if !has_file {
                        calls.push(diag("PFWHOS", node, PFWHOS_MSG.to_string()));
                    }
                } else if NON_TRANSPARENT.contains(&name) {
                    calls.push(diag("PFBFN", node, PFBFN_MSG.to_string()));
                }
            }
            "identifier" => {
                let text = node_text(node, source);
                let statement_or_callee = node.parent().is_none_or(|p| {
                    matches!(
                        p.kind(),
                        "block" | "source_file" | "command" | "function_call"
                    )
                });
                if (text == "who" || text == "whos") && !statement_or_callee {
                    calls.push(diag("PFWHOS", node, PFWHOS_MSG.to_string()));
                }
            }
            "command" => {
                let text = node_text(node, source).trim();
                let first = text.split_whitespace().next().unwrap_or("");
                if first == "who" || first == "whos" {
                    if !text.contains("-file") {
                        calls.push(diag("PFWHOS", node, PFWHOS_MSG.to_string()));
                    }
                } else if NON_TRANSPARENT.contains(&first) {
                    calls.push(diag("PFBFN", node, PFBFN_MSG.to_string()));
                }
            }
            "assignment" => {
                let Some(lhs) = node.child_by_field_name("left") else {
                    return;
                };
                if lhs.kind() != "identifier" {
                    return;
                }
                let var = node_text(lhs, source).trim().to_string();
                if Some(var.as_str()) == loop_var.as_deref() {
                    return;
                }
                let Some(rhs) = node.child_by_field_name("right") else {
                    return;
                };
                if is_self_update(&var, rhs, source) {
                    if !reductions.contains(&var) {
                        reductions.push(var.clone());
                    }
                    if let Some((key, pos)) = reduction_position(&var, rhs, source) {
                        match positions.get(&(var.clone(), key)) {
                            Some(first) if *first != pos => rnc.push((node, var.clone())),
                            Some(_) => {}
                            None => {
                                positions.insert((var.clone(), key), pos);
                            }
                        }
                    }
                } else if !temps.contains(&var) {
                    temp_sites.insert(var.clone(), node);
                    temps.push(var);
                }
            }
            _ => {}
        });

        diagnostics.extend(calls);

        // PFTUSE: temporary variables assigned in the loop but read after it.
        let globals = crate::good_practices::collect_global_persistent_vars(
            crate::good_practices::root_node_of(parfor),
            source,
        );
        // Probed: a variable already set before the loop is PFTUSW instead, and MATLAB
        // reports PFTUSE at the temporary's assignment in the body (not at the use).
        let mut set_before: Vec<String> = Vec::new();
        visit(crate::good_practices::root_node_of(parfor), &mut |n| {
            if n.kind() == "assignment" && n.end_byte() <= parfor.start_byte() {
                if let Some(lhs) = n.child_by_field_name("left") {
                    if lhs.kind() == "identifier" {
                        set_before.push(node_text(lhs, source).trim().to_string());
                    }
                }
            }
        });
        for var in temps.iter().filter(|v| {
            !reductions.contains(v) && !globals.contains(v.as_str()) && !set_before.contains(v)
        }) {
            if let (Some(use_node), Some(set_node)) =
                (later_use(parfor, var, source), temp_sites.get(var))
            {
                diagnostics.push(diag(
                    "PFTUSE",
                    *set_node,
                    format!("The temporary variable '{var}' is used after the PARFOR loop on line {}, but its value is not available after the loop.", use_node.start_position().row + 1),
                ));
            }
        }

        // PFRNC: inconsistent operand position of a non-commutative reduction.
        for (node, var) in rnc {
            diagnostics.push(diag(
                "PFRNC",
                node,
                format!("Parfor reduction variable '{var}' must be used in the same position in each assignment statement when using non-commutative reduction operations '*', '[,]', or '[;]'."),
            ));
        }
    }
}

const PFWHOS_MSG: &str = "Using \"who\" or \"whos\" without \"-file\" is invalid inside a PARFOR loop because it accesses the workspace in a non-transparent way.";
const PFBFN_MSG: &str = "Use of this function is invalid inside a PARFOR loop because it accesses or modifies the workspace in a non-transparent way.";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    #[test]
    fn pfuixe_fires_on_index_used_after_loop() {
        let src = "parfor i = 1:10\n    a(i) = i;\nend\ndisp(i);\n";
        let diags = file_diags(src);
        let d = diags
            .iter()
            .find(|d| d.rule_id == "PFUIXE")
            .expect("PFUIXE");
        assert!(
            d.message.contains("index variable 'i'"),
            "got: {}",
            d.message
        );
        // Probed: MATLAB reports PFUIXE on the loop header (line 1), not at the use.
        assert_eq!(d.line, 1);
    }

    #[test]
    fn pfuixe_no_fire_when_index_reassigned_or_unused() {
        let src = "parfor i = 1:10\n    a(i) = i;\nend\ni = 5;\ndisp(i);\n";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "PFUIXE"), "got: {diags:?}");
        let diags = file_diags("for i = 1:3\n    eval('x');\nend\n");
        assert!(!has_id(&diags, "PFUIXE"), "got: {diags:?}");
    }

    #[test]
    fn pfbfn_fires_on_evalc_in_parfor() {
        let diags = file_diags("parfor i = 1:10\n    s = evalc('i');\nend\n");
        assert!(has_id(&diags, "PFBFN"), "got: {diags:?}");
    }

    #[test]
    fn pfbfn_silent_on_save_in_parfor() {
        // Probed: MATLAB reports `save` under PFSV, not PFBFN.
        let diags = file_diags("parfor i = 1:10\n    save('f.mat');\nend\n");
        assert!(!has_id(&diags, "PFBFN"), "got: {diags:?}");
    }

    #[test]
    fn pfbfn_fires_on_command_form_and_eval() {
        let diags = file_diags("parfor i = 1:10\n    clear x\n    eval('y = 1');\nend\n");
        assert_eq!(
            diags.iter().filter(|d| d.rule_id == "PFBFN").count(),
            2,
            "got: {diags:?}"
        );
    }

    #[test]
    fn pfbfn_no_fire_outside_parfor_or_on_load_with_output() {
        let diags = file_diags("for i = 1:10\n    save('f.mat');\nend\n");
        assert!(!has_id(&diags, "PFBFN"), "got: {diags:?}");
        let diags = file_diags("parfor i = 1:3\n    s = load('f.mat');\nend\n");
        assert!(!has_id(&diags, "PFBFN"), "got: {diags:?}");
    }

    #[test]
    fn pfwhos_fires_on_who_in_parfor() {
        let diags = file_diags("parfor i = 1:10\n    who();\nend\n");
        assert!(has_id(&diags, "PFWHOS"), "got: {diags:?}");
    }

    #[test]
    fn pfwhos_fires_on_whos_command_form() {
        let diags = file_diags("parfor i = 1:10\n    whos\nend\n");
        assert!(has_id(&diags, "PFWHOS"), "got: {diags:?}");
    }

    #[test]
    fn pfwhos_fires_on_who_used_as_value() {
        let diags = file_diags("parfor i = 1:10\n    s = who;\nend\n");
        assert!(has_id(&diags, "PFWHOS"), "got: {diags:?}");
    }

    #[test]
    fn pfwhos_no_fire_with_file_flag_or_outside_parfor() {
        let diags = file_diags("parfor i = 1:10\n    whos('-file', 'a.mat');\nend\n");
        assert!(!has_id(&diags, "PFWHOS"), "got: {diags:?}");
        let diags = file_diags("for i = 1:10\n    who();\nend\n");
        assert!(!has_id(&diags, "PFWHOS"), "got: {diags:?}");
    }

    #[test]
    fn pftuse_fires_on_temporary_used_after_loop() {
        let src = "parfor i = 1:10\n    t = i * 2;\n    a(i) = t;\nend\ndisp(t);\n";
        let diags = file_diags(src);
        let d = diags
            .iter()
            .find(|d| d.rule_id == "PFTUSE")
            .expect("PFTUSE");
        assert!(
            d.message.contains("temporary variable 't'"),
            "got: {}",
            d.message
        );
    }

    #[test]
    fn pftuse_reports_at_temporary_assignment_not_use() {
        // Probed (pftuse_01): MATLAB reports PFTUSE at the assignment in the loop.
        let src = "parfor i = 1:10\n    t = i * 2;\n    a(i) = t;\nend\ndisp(t);\n";
        let diags = file_diags(src);
        let d = diags
            .iter()
            .find(|d| d.rule_id == "PFTUSE")
            .expect("PFTUSE");
        assert_eq!(d.line, 2, "got: {diags:?}");
    }

    #[test]
    fn pftuse_silent_when_set_before_loop() {
        // Probed (pftuse_02): a variable with a value before the loop is PFTUSW, not PFTUSE.
        let src = "t = 0;\nparfor i = 1:10\n    t = i;\n    a(i) = i;\nend\ndisp(t);\n";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "PFTUSE"), "got: {diags:?}");
    }

    #[test]
    fn pftuse_no_fire_for_reduction_or_unused_temp() {
        let src = "s = 0;\nparfor i = 1:10\n    s = s + i;\nend\ndisp(s);\n";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "PFTUSE"), "got: {diags:?}");
        let diags = file_diags("parfor i = 1:10\n    t = i;\n    a(i) = t;\nend\n");
        assert!(!has_id(&diags, "PFTUSE"), "got: {diags:?}");
    }

    #[test]
    fn pfrnc_fires_on_inconsistent_multiplication_position() {
        let src = "p = 1;\nparfor i = 1:10\n    p = p * i;\n    p = i * p;\nend\n";
        let diags = file_diags(src);
        let d = diags.iter().find(|d| d.rule_id == "PFRNC").expect("PFRNC");
        assert!(
            d.message.contains("reduction variable 'p'"),
            "got: {}",
            d.message
        );
    }

    #[test]
    fn pfrnc_fires_on_inconsistent_concatenation_position() {
        let src = "r = [];\nparfor i = 1:10\n    r = [r, i];\n    r = [i, r];\nend\n";
        let diags = file_diags(src);
        assert!(has_id(&diags, "PFRNC"), "got: {diags:?}");
    }

    #[test]
    fn pfrnc_no_fire_on_consistent_position() {
        let src = "p = 1;\nparfor i = 1:10\n    p = p * i;\n    p = p * 2;\nend\n";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "PFRNC"), "got: {diags:?}");
    }

    #[test]
    fn pfuixe_message_quotes_index_and_gives_use_line() {
        // MATLAB: "The index variable 'i' might be used after the PARFOR loop on line 4, ..."
        let diags = file_diags("parfor i = 1:3\n    x = i;\nend\ndisp(i)\n");
        let d = diags
            .iter()
            .find(|d| d.rule_id == "PFUIXE")
            .expect("PFUIXE");
        assert_eq!(
            d.message,
            "The index variable 'i' might be used after the PARFOR loop on line 4, but it is unavailable after the loop."
        );
        // Probed: MATLAB reports PFUIXE on the loop header line (here line 1).
        assert_eq!((d.line, d.column), (1, 8));
    }

    #[test]
    fn pftuse_message_quotes_temporary_and_gives_use_line() {
        let diags = file_diags("parfor i = 1:3\n    t = i;\nend\ndisp(t)\n");
        let d = diags
            .iter()
            .find(|d| d.rule_id == "PFTUSE")
            .expect("PFTUSE");
        assert_eq!(
            d.message,
            "The temporary variable 't' is used after the PARFOR loop on line 4, but its value is not available after the loop."
        );
    }

    #[test]
    fn pfrnc_message_quotes_the_reduction_variable() {
        let diags = file_diags("x = [];\nparfor m = 1:3\n    x = [x m];\n    x = [m x];\nend\n");
        let d = diags.iter().find(|d| d.rule_id == "PFRNC").expect("PFRNC");
        assert_eq!(
            d.message,
            "Parfor reduction variable 'x' must be used in the same position in each assignment statement when using non-commutative reduction operations '*', '[,]', or '[;]'."
        );
    }

    #[test]
    fn pfuixe_fires_on_index_read_after_empty_loop() {
        // Probed (xcq01, xcx02): an empty `parfor` still has an index, and a later read is PFUIXE.
        let diags = file_diags("function r = f(n)\nparfor k = 1:n\nend\nr = k + 1;\nend\n");
        assert!(has_id(&diags, "PFUIXE"), "got: {diags:?}");
    }

    #[test]
    fn pfuixe_silent_when_index_is_reassigned_after_empty_loop() {
        // Probed (xcq10): `k = 3` before the read makes the read a new value.
        let diags = file_diags("function r = f(n)\nparfor k = 1:n\nend\nk = 3;\nr = k;\nend\n");
        assert!(!has_id(&diags, "PFUIXE"), "got: {diags:?}");
    }

    #[test]
    fn pftuse_fires_on_bare_temporary_read_after_loop() {
        // Probed (xcp11, xct01): a bare `t` statement is a command in this grammar, and it is a
        // read of `t`. MATLAB reports PFTUSE at the assignment and names the read's line.
        let diags = file_diags("function r = f(n)\nparfor k = 1:n\n    t = k;\nend\nt\nend\n");
        let d = diags
            .iter()
            .find(|d| d.rule_id == "PFTUSE")
            .expect("PFTUSE");
        assert_eq!(d.line, 3);
        assert!(d.message.contains("line 5"), "got: {}", d.message);
    }

    #[test]
    fn pftuse_silent_after_clear_or_indexed_store() {
        // Probed (xcp40, xct20): `clear t` ends the value; `t(2) = 5` is not a read of `t`.
        let diags = file_diags(
            "function r = f(n)\nparfor k = 1:n\n    t = k;\nend\nclear t\nr = t;\nend\n",
        );
        assert!(!has_id(&diags, "PFTUSE"), "got: {diags:?}");
        let diags =
            file_diags("function r = f(n)\nparfor k = 1:n\n    t = k;\nend\nt(2) = 5;\nend\n");
        assert!(!has_id(&diags, "PFTUSE"), "got: {diags:?}");
    }

    #[test]
    fn pftuse_fires_on_field_store_after_loop() {
        // Probed (xct19): `t.x = 1` reads `t`.
        let diags =
            file_diags("function r = f(n)\nparfor k = 1:n\n    t = k;\nend\nt.x = 1;\nend\n");
        assert!(has_id(&diags, "PFTUSE"), "got: {diags:?}");
    }

    #[test]
    fn pftuse_for_nested_loop_variable_at_iterator() {
        // Probed (xcp17, xct05): a nested `for j` sets `j` in the body. PFTUSE is reported at
        // the iterator (line 3, column 7), not at the later read.
        let diags = file_diags(
            "function r = f(n)\nparfor k = 1:n\n  for j = 1:2\n    a(k) = j;\n  end\nend\nr = j;\nend\n",
        );
        let d = diags
            .iter()
            .find(|d| d.rule_id == "PFTUSE")
            .expect("PFTUSE");
        assert_eq!((d.line, d.column), (3, 7));
    }
}
