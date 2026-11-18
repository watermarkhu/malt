//! Indexed growth inside loops: the target of `x(i) = v`, `x{i} = v` or
//! `x(i).f = v` is grown on every iteration when `x` is undefined, `[]`, `{}`
//! or a number before the loop and the index names a variable.
//!
//! Probed in MATLAB (AGROW in functions, SAGROW in scripts): reported for
//! `x(i)`, `x(i+1)`, `x(2*i)`, `x(k)` and `x(n)` with a counter, `x(i,j)`,
//! `x(i,:)`, `x(:,i)`, `c{i}` and `x(i).a`, in `for` and `while` loops, in nested
//! loops and inside `if`. Silent for `x(1)`, `x(end)`, `x(:)`, for a variable
//! that holds a call result (`zeros`, `cell`, `struct`, `foo()`), a literal
//! array or cell (`[1 2 3]`, `{1, 2}`), a character or logical value, a negated
//! number, a function parameter, and for a field of an undefined name
//! (`y.x(i)`).

use super::*;
use crate::analysis::reaching::{is_growable_initial_value, reaching, Binding};

/// Whether the assignment target `target` (`x(i)`, `x{i}`, `x(i).f`) is an
/// indexed write with a variable in its index into a variable that starts empty.
pub(crate) fn indexed_write_grows(target: tree_sitter::Node, source: &str) -> bool {
    let t = node_text(target, source);
    let Some(open) = t.find(['(', '{']) else {
        return false;
    };
    let base = t[..open].trim();
    if base.is_empty()
        || base.starts_with(|c: char| c.is_ascii_digit())
        || !base.chars().all(|c| c.is_alphanumeric() || c == '_')
    {
        return false;
    }
    // The index is the text up to the bracket that closes the first one.
    let mut depth = 0usize;
    let mut close = None;
    for (i, c) in t[open..].char_indices() {
        match c {
            '(' | '{' => depth += 1,
            ')' | '}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(close) = close else {
        return false;
    };
    let words: Vec<&str> = t[open + 1..close]
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty() && !w.starts_with(|c: char| c.is_ascii_digit()) && *w != "end")
        .collect();
    if !words.iter().any(|w| changes_in_loop(target, source, w)) || inside_parfor(target, source) {
        return false;
    }
    let r = reaching(target, source, base);
    // An indexed write before the loop with no `x = ...` before it (`x(n) = v;`
    // ahead of the loop) has already sized the array: probed silent in a parfeval
    // loop. A write inside the same loop does not count (probed: each repeat reports).
    if r.binding == Binding::Undefined && indexed_write_before_loop(target, source, base) {
        return false;
    }
    is_growable_initial_value(r.binding, source)
}

/// The first bracket group of an indexed target, brackets included: `(end+1)`
/// in `x(end+1).f`, `{i}` in `x{i}(end+1)`. Empty when the text has no bracket.
pub(crate) fn first_index_group(text: &str) -> &str {
    let Some(open) = text.find(['(', '{']) else {
        return "";
    };
    let mut depth = 0usize;
    for (i, c) in text[open..].char_indices() {
        match c {
            '(' | '{' => depth += 1,
            ')' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return &text[open..=open + i];
                }
            }
            _ => {}
        }
    }
    ""
}

/// Whether the first bracket group of a target grows its variable by one with
/// `end+1` (`x(end+1)`, `x{end+1}`, `x(end+1).f`). MATLAB reports this on every
/// initial value, parameters included (probed).
pub(crate) fn first_group_has_end_plus_one(group: &str) -> bool {
    group
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .contains("end+1")
}

/// Whether the first bracket group of a target holds `numel(base)+1`, `length(base)+1`
/// or `size(base,2)+1`, the size forms of the same growth (probed: `x(size(x,2)+1) = i`
/// in a loop is SAGROW).
pub(crate) fn first_group_has_size_plus_one(group: &str, base: &str) -> bool {
    let normalized: String = group.chars().filter(|c| !c.is_whitespace()).collect();
    normalized.contains(&format!("numel({base})+1"))
        || normalized.contains(&format!("length({base})+1"))
        || normalized.contains(&format!("size({base},2)+1"))
}

/// Whether `base` starts as a value the growth checks follow (undefined, `[]`,
/// `{}`, a number or a literal) before `target`. Probed: the size form
/// `x(numel(x)+1)` is silent after `zeros(1,0)` and `cell(1,0)`.
pub(crate) fn starts_growable(target: tree_sitter::Node, source: &str, base: &str) -> bool {
    is_growable_initial_value(reaching(target, source, base).binding, source)
}

/// Whether `base` is written by an indexed assignment before the outermost loop
/// (in the same function, or the file) that encloses `target`.
fn indexed_write_before_loop(target: tree_sitter::Node, source: &str, base: &str) -> bool {
    let mut outer = None;
    let mut scope = None;
    let mut current = target.parent();
    while let Some(p) = current {
        match p.kind() {
            "function_definition" => {
                scope = Some(p);
                break;
            }
            "for_statement" | "while_statement" => outer = Some(p),
            _ => {}
        }
        current = p.parent();
    }
    let before = outer.map_or(target.start_byte(), |l| l.start_byte());
    let mut stack = vec![scope.unwrap_or_else(|| {
        let mut root = target;
        while let Some(p) = root.parent() {
            root = p;
        }
        root
    })];
    while let Some(n) = stack.pop() {
        if n.start_byte() >= before {
            continue;
        }
        if n.kind() == "assignment" {
            if let Some(left) = n.child_by_field_name("left") {
                let text = node_text(left, source);
                // A plain field write (`x.f = []`) does not size `x` (probed: `x(i) = i`
                // after it still reports); only a write with a bracket counts.
                if left.kind() != "identifier"
                    && text.contains(['(', '{'])
                    && text
                        .split(['(', '{', '.'])
                        .next()
                        .is_some_and(|b| b.trim() == base)
                {
                    return true;
                }
            }
        }
        let mut cursor = n.walk();
        stack.extend(n.children(&mut cursor));
    }
    false
}

/// Whether `node` is inside a `parfor` loop: MATLAB does not report growth there
/// (probed: `a(i) = i` in `parfor`).
fn inside_parfor(node: tree_sitter::Node, source: &str) -> bool {
    let mut current = node.parent();
    while let Some(p) = current {
        if p.kind() == "for_statement" && node_text(p, source).trim_start().starts_with("parfor") {
            return true;
        }
        current = p.parent();
    }
    false
}

/// Whether `name` is bound inside a loop that encloses `node` (a loop variable,
/// or a whole assignment in the loop body): an index that stays the same on
/// every iteration does not grow the variable more than once. Probed: `x(k)`
/// with `k` counted up in the loop reports; a constant index does not.
fn changes_in_loop(node: tree_sitter::Node, source: &str, name: &str) -> bool {
    let mut current = node.parent();
    while let Some(p) = current {
        match p.kind() {
            "function_definition" => break,
            "for_statement" | "while_statement" if loop_binds(p, source, name) => {
                return true;
            }
            _ => {}
        }
        current = p.parent();
    }
    false
}

/// Whether the loop `lp` binds `name`: as its iterator, or in an assignment in its body.
fn loop_binds(lp: tree_sitter::Node, source: &str, name: &str) -> bool {
    let mut stack = vec![lp];
    while let Some(n) = stack.pop() {
        let is_iterator = n.kind() == "for_statement"
            && (0..n.named_child_count()).any(|i| {
                n.named_child(i).is_some_and(|c| {
                    let it = if c.kind() == "iterator" {
                        c.named_child(0)
                    } else {
                        Some(c)
                    };
                    it.is_some_and(|it| it.kind() == "identifier" && node_text(it, source) == name)
                })
            });
        if is_iterator {
            return true;
        }
        if n.kind() == "assignment"
            && n.child_by_field_name("left")
                .is_some_and(|l| l.kind() == "identifier" && node_text(l, source) == name)
        {
            return true;
        }
        let mut cursor = n.walk();
        stack.extend(n.children(&mut cursor));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_file};

    fn growth(src: &str) -> bool {
        let diags = lint_file(&*engine(), src);
        has_id(&diags, "SAGROW") || has_id(&diags, "AGROW")
    }

    #[test]
    fn growth_fires_on_indexed_write_into_undefined_or_empty() {
        // Probed in MATLAB: SAGROW in scripts.
        for src in [
            "for i = 1:5\n    x(i) = i;\nend\n",
            "x = [];\nfor i = 1:5\n    x(i) = i;\nend\n",
            "x = {};\nfor i = 1:5\n    x{i} = i;\nend\n",
            "x = 0;\nfor i = 1:5\n    x(i) = i;\nend\n",
            "for i = 1:5\n    c{i} = 'a';\nend\n",
            "x = [];\nfor i = 1:5\n    x(i+1) = i;\nend\n",
            "x = [];\nfor i = 1:5\n    x(2*i) = i;\nend\n",
            "x = [];\nn = 0;\nfor i = 1:5\n    n = n + 1;\n    x(n) = i;\nend\n",
            "x = [];\nk = 0;\nwhile k < 5\n    k = k + 1;\n    x(k) = k;\nend\n",
            "x = [];\nfor i = 1:5\n    x(i,:) = [i i];\nend\n",
            "x = [];\nfor i = 1:5\n    x(:,i) = [i; i];\nend\n",
            "x = [];\nfor i = 1:3\n    for j = 1:3\n        x(i,j) = i + j;\n    end\nend\n",
            "x = [];\nfor i = 1:5\n    if i > 2\n        x(i) = i;\n    end\nend\n",
            "x = [];\nfor i = 1:5\n    x = [];\n    x(i) = i;\nend\n",
            "for i = 1:5\n    x(i) = i;\nend\nx = zeros(1, 5);\n",
            "x = [];\nfor i = 1:3\n    x(i) = i;\nend\nfor j = 1:3\n    x(j) = j;\nend\n",
            "x = [];\nfor i = 1:3\n    x(i) = i;\n    x(i) = x(i) + 1;\nend\n",
        ] {
            assert!(growth(src), "{src}");
        }
    }

    #[test]
    fn growth_silent_on_preallocated_constant_or_call_values() {
        // Probed in MATLAB: silent for these initial values and indices.
        for src in [
            "x = zeros(1, 5);\nfor i = 1:5\n    x(i) = i;\nend\n",
            "x = zeros(0, 1);\nfor i = 1:5\n    x(i) = i;\nend\n",
            "x = [1 2 3];\nfor i = 1:5\n    x(i) = i;\nend\n",
            "x = {1, 2};\nfor i = 1:5\n    x{i} = i;\nend\n",
            "x = 'abc';\nfor i = 1:5\n    x(i) = 'x';\nend\n",
            "x = '';\nfor i = 1:5\n    x(i) = 'a';\nend\n",
            "x = -1;\nfor i = 1:5\n    x(i) = i;\nend\n",
            "x = 5*2;\nfor i = 1:5\n    x(i) = i;\nend\n",
            "x = nan;\nfor i = 1:5\n    x(i) = i;\nend\n",
            "c = cell(1, 0);\nfor i = 1:5\n    c{i} = i;\nend\n",
            "s = struct([]);\nfor i = 1:5\n    s(i).a = i;\nend\n",
            "x = foo();\nfor i = 1:5\n    x(i) = i;\nend\n",
            "x = [];\nx(3) = 1;\n",
            "x = [];\nfor i = 1:5\n    x(1) = i;\nend\n",
            "x = [];\nfor i = 1:5\n    x(end) = i;\nend\n",
            "x = [];\nfor i = 1:5\n    x(:) = i;\nend\n",
            "for i = 1:5\n    y.x(i) = i;\nend\n",
        ] {
            assert!(!growth(src), "{src}");
        }
    }

    #[test]
    fn growth_silent_on_function_parameter() {
        let src = "function r = f(r, n)\nfor i = 1:n\n    r(i) = i;\nend\nend\n";
        assert!(!growth(src), "{src}");
    }

    #[test]
    fn growth_fires_on_end_plus_one_for_any_initial_value() {
        // Probed in MATLAB: `x(end+1)`, `x{end+1}`, `x(end+1,:)`, `x(end+1).f` and
        // `x = [x i]` report whatever `x` holds, call results included.
        for src in [
            "x = zeros(1, 0);\nfor i = 1:3\n    x(end+1) = i;\nend\n",
            "x = cell(3, 1);\nfor i = 1:3\n    x{end+1} = i;\nend\n",
            "x = struct('f', {});\nfor i = 1:3\n    x(end+1).f = i;\nend\n",
            "x = ones(1, 3);\nfor i = 1:3\n    x(end+1, :) = [i i];\nend\n",
            "x = [];\nfor i = 1:3\n    x = [x i];\nend\n",
        ] {
            assert!(growth(src), "{src}");
        }
    }

    #[test]
    fn growth_silent_on_element_of_call_result_cell_or_struct() {
        // Probed in MATLAB: silent for `x{i}(end+1)`, `x(i).f(end+1)` and
        // `x{i} = [x{i} i]` when `x` holds a call result (`cell`, `struct`, `zeros`).
        for src in [
            "c = cell(3, 1);\nfor i = 1:3\n    c{i}(end+1) = i;\nend\n",
            "c = cell(1, 3);\nfor i = 1:3\n    c{mod(i, 2) + 1}(end+1) = i;\nend\n",
            "c = cell(3, 1);\nfor ci = 1:3\n    for cj = ci+1:3\n        c{ci}(end+1) = cj;\n    end\nend\n",
            "s = struct('f', {});\nfor i = 1:3\n    s(i).f(end+1) = i;\nend\n",
            "c = cell(3, 1);\nfor i = 1:3\n    c{i} = [c{i} i];\nend\n",
        ] {
            assert!(!growth(src), "{src}");
        }
    }

    #[test]
    fn growth_fires_on_element_of_undefined_or_empty_cell_or_struct() {
        // Probed in MATLAB: SAGROW for these with `c`/`s` undefined, `{}` or `[]`.
        for src in [
            "for i = 1:3\n    c{i}(end+1) = i;\nend\n",
            "c = {};\nfor i = 1:3\n    c{i}(end+1) = i;\nend\n",
            "c = [];\nfor i = 1:3\n    c{i}(end+1) = i;\nend\n",
            "s = [];\nfor i = 1:3\n    s(i).f(end+1) = i;\nend\n",
            "c = {};\nfor i = 1:3\n    c{i} = [c{i} i];\nend\n",
            "for ci = 1:3\n    for cj = ci+1:3\n        c{ci}(end+1) = cj;\n    end\nend\n",
        ] {
            assert!(growth(src), "{src}");
        }
    }

    #[test]
    fn growth_silent_on_constant_index_and_size_form_after_call_result() {
        // Probed in MATLAB: a constant index does not grow the variable on every
        // iteration; `x(numel(x)+1)` is silent after `zeros(1,0)` and `struct(...)`.
        for src in [
            "x = cell(3, 1);\nfor i = 1:3\n    x{3}(end+1) = i;\nend\n",
            "for i = 1:3\n    x{3}(end+1) = i;\nend\n",
            "x = struct('f', {});\nfor i = 1:3\n    x(2).f(end+1) = i;\nend\n",
            "x = zeros(1, 0);\nfor i = 1:3\n    x(numel(x)+1) = i;\nend\n",
            "x = struct('f', {});\nfor i = 1:3\n    x(numel(x)+1).f = i;\nend\n",
        ] {
            assert!(!growth(src), "{src}");
        }
    }

    #[test]
    fn growth_fires_on_size_form_after_empty_value() {
        // Probed in MATLAB: SAGROW for `x(numel(x)+1)` after `[]` or undefined `x`,
        // and for a struct element `x(numel(x)+1).f` after `[]`.
        for src in [
            "x = [];\nfor i = 1:3\n    x(numel(x)+1) = i;\nend\n",
            "for i = 1:3\n    x(numel(x)+1) = i;\nend\n",
            "x = [];\nfor i = 1:3\n    x(numel(x)+1).f = i;\nend\n",
        ] {
            assert!(growth(src), "{src}");
        }
    }

    #[test]
    fn growth_fires_after_a_plain_field_write_to_the_base() {
        // Probed in MATLAB: `x.f = []` before the loop does not size `x`, so
        // `x(i) = i`, `x{i} = [x{i} i]` and `x(i).f = i` still report.
        for src in [
            "x.f = [];\nfor i = 1:3\n    x(i) = i;\nend\n",
            "x.f = [];\nfor i = 1:3\n    x{i} = [x{i} i];\nend\n",
            "x.f = [];\nfor i = 1:3\n    x(i).f = i;\nend\n",
        ] {
            assert!(growth(src), "{src}");
        }
    }

    #[test]
    fn growth_on_parameter_follows_the_probes() {
        // Probed in MATLAB: `x(end+1)` on an input argument reports; the element
        // form `x{i}(end+1)` does not.
        let end = "function f(x)\nfor i = 1:3\n    x(end+1) = i;\nend\nend\n";
        assert!(has_id(&lint_file(&*engine(), end), "AGROW"));
        let deep = "function f(x)\nfor i = 1:3\n    x{i}(end+1) = i;\nend\nend\n";
        assert!(!growth(deep));
    }

    #[test]
    fn agrow_fires_on_struct_element_field_growth() {
        // Probed in MATLAB: AGROW for `x(end+1).f = v` in a function (so a
        // `%#ok<AGROW>` on it is used); SAGROW in a script.
        let src = "function f()\nx = struct([]);\nfor i = 1:3\n    x(end+1).f = i;\nend\nend\n";
        assert!(has_id(&lint_file(&*engine(), src), "AGROW"));
        let src = "x = struct([]);\nfor i = 1:3\n    x(end+1).f = i;\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(
            has_id(&diags, "SAGROW") && !has_id(&diags, "AGROW"),
            "{diags:?}"
        );
    }

    #[test]
    fn agrow_fires_in_function_on_undefined_target() {
        // Probed in MATLAB: AGROW inside a function.
        let src = "function r = f(n)\nfor i = 1:n\n    r(i) = i;\nend\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "AGROW"), "got: {diags:?}");
        let src = "function r = f(n)\nr = [];\nfor i = 1:n\n    r(i) = i;\nend\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "AGROW"), "got: {diags:?}");
    }
}

#[cfg(test)]
mod size_form_tests {
    use crate::test_util::{has_id, lint_file};

    // Probed: `x(size(x,2)+1) = i` in a loop is SAGROW (in a script), as the numel form is;
    // after `zeros(1,0)` it is silent.
    #[test]
    fn sagrow_fires_on_the_size_second_dimension_form() {
        let e = super::super::test_support::engine();
        assert!(has_id(
            &lint_file(&*e, "x = [];\nfor i = 1:3\n    x(size(x,2)+1) = i;\nend\n"),
            "SAGROW"
        ));
        assert!(!has_id(
            &lint_file(
                &*e,
                "x = zeros(1,0);\nfor i = 1:3\n    x(size(x,2)+1) = i;\nend\n"
            ),
            "SAGROW"
        ));
    }
}
