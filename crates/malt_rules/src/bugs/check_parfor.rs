use super::*;
use std::collections::HashMap;

/// Functions that access or modify the workspace non-transparently, which is
/// invalid inside a `parfor` body.
const NON_TRANSPARENT: &[&str] = &[
    "assignin",
    "evalin",
    "eval",
    "evalc",
    "save",
    "clear",
    "clearvars",
    "inputname",
];

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
        let assigned = stmt.kind() == "assignment"
            && stmt
                .child_by_field_name("left")
                .is_some_and(|l| l.kind() == "identifier" && node_text(l, source).trim() == name);
        // For `name = ...` only the right-hand side can read the variable.
        let scope = if assigned {
            stmt.child_by_field_name("right").unwrap_or(stmt)
        } else {
            stmt
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
        let line = parfor.start_position().row + 1;
        let Some(body) = find_block_child(parfor) else {
            return;
        };

        // PFUIXE: the loop index is unavailable after the loop.
        if let Some(lv) = loop_var.as_deref() {
            if let Some(use_node) = later_use(parfor, lv, source) {
                diagnostics.push(diag(
                    "PFUIXE",
                    use_node,
                    format!("The index variable {lv} might be used after the PARFOR loop on line {line}, but it is unavailable after the loop."),
                ));
            }
        }

        let mut temps: Vec<String> = Vec::new();
        let mut reductions: Vec<String> = Vec::new();
        let mut positions: HashMap<(String, &'static str), usize> = HashMap::new();
        let mut rnc: Vec<(Node, String)> = Vec::new();
        let mut calls: Vec<Diagnostic> = Vec::new();

        visit(body, &mut |node| match node.kind() {
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
                } else if NON_TRANSPARENT.contains(&name)
                    || (name == "load" && !node.parent().is_some_and(|p| p.kind() == "assignment"))
                {
                    calls.push(diag("PFBFN", node, PFBFN_MSG.to_string()));
                }
            }
            "command" => {
                let text = node_text(node, source).trim();
                let first = text.split_whitespace().next().unwrap_or("");
                if first == "who" || first == "whos" {
                    if !text.contains("-file") {
                        calls.push(diag("PFWHOS", node, PFWHOS_MSG.to_string()));
                    }
                } else if NON_TRANSPARENT.contains(&first) || first == "load" {
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
                    temps.push(var);
                }
            }
            _ => {}
        });

        diagnostics.extend(calls);

        // PFTUSE: temporary variables assigned in the loop but read after it.
        for var in temps.iter().filter(|v| !reductions.contains(v)) {
            if let Some(use_node) = later_use(parfor, var, source) {
                diagnostics.push(diag(
                    "PFTUSE",
                    use_node,
                    format!("The temporary variable {var} is used after the PARFOR loop on line {line}, but its value is not available after the loop."),
                ));
            }
        }

        // PFRNC: inconsistent operand position of a non-commutative reduction.
        for (node, var) in rnc {
            diagnostics.push(diag(
                "PFRNC",
                node,
                format!("Parfor reduction variable {var} must be used in the same position in each assignment statement when using non-commutative reduction operations '*', '[,]', or '[;]'."),
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
        assert!(d.message.contains("index variable i"), "got: {}", d.message);
        assert_eq!(d.line, 4);
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
    fn pfbfn_fires_on_save_in_parfor() {
        let diags = file_diags("parfor i = 1:10\n    save('f.mat');\nend\n");
        assert!(has_id(&diags, "PFBFN"), "got: {diags:?}");
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
            d.message.contains("temporary variable t"),
            "got: {}",
            d.message
        );
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
            d.message.contains("reduction variable p"),
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
}
