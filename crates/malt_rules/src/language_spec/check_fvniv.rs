//! See the parent module `super` for the shared engine and helpers.

use super::*;
use std::collections::HashSet;

/// Identifiers in a default value (`= expr`), skipping quoted text and field names
/// (`d.x` names `d` only).
fn referenced_names(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut names = Vec::new();
    let mut string_quote: Option<char> = None;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match string_quote {
            Some(quote) if c == quote => string_quote = None,
            Some(_) => {}
            None if c == '\'' || c == '"' => string_quote = Some(c),
            None => {}
        }
        if string_quote.is_some() || c == '\'' || c == '"' || !(c.is_ascii_alphabetic() || c == '_')
        {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
            i += 1;
        }
        // Skip field names (`x` in `d.x`), exponents (`e` in `1e3`) and calls (`f(1)`).
        let is_call = chars.get(i) == Some(&'(');
        let skip = is_call
            || (start > 0 && (chars[start - 1] == '.' || chars[start - 1].is_ascii_digit()));
        if !skip {
            names.push(chars[start..i].iter().collect());
        }
    }
    names
}

/// Byte offset of the default-value `=` in an argument declaration: the first `=`
/// outside braces (validators) and not part of `==`, `~=`, `<=`, `>=`.
fn default_separator(text: &str) -> Option<usize> {
    let mut depth = 0i32;
    let bytes = text.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' | b'(' => depth += 1,
            b'}' | b')' => depth -= 1,
            b'=' if depth == 0 => {
                let prev = if i > 0 { bytes[i - 1] } else { b' ' };
                let next = bytes.get(i + 1).copied().unwrap_or(b' ');
                if !matches!(prev, b'~' | b'<' | b'>' | b'=') && next != b'=' {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Names assigned anywhere in the function that owns the argument block.
fn assigned_names(prop: tree_sitter::Node, source: &str) -> HashSet<String> {
    let mut names = HashSet::new();
    let mut function = prop.parent();
    while let Some(node) = function {
        if node.kind() == "function_definition" {
            collect_assigned(node, source, &mut names);
            break;
        }
        function = node.parent();
    }
    names
}

fn collect_assigned(node: tree_sitter::Node, source: &str, names: &mut HashSet<String>) {
    if node.kind() == "assignment" {
        if let Some(left) = node.named_child(0) {
            collect_identifiers(left, source, names);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_assigned(child, source, names);
    }
}

fn collect_identifiers(node: tree_sitter::Node, source: &str, names: &mut HashSet<String>) {
    if node.kind() == "identifier" {
        names.insert(node_text(node, source).to_string());
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_identifiers(child, source, names);
    }
}

impl LanguageSpecEngine {
    /// FVNIV: a default value in an arguments block names a variable that is not an
    /// input but is assigned in the function body (probed: `a double = q` with `q = 1;`
    /// below, `b double = a + tmp`, `a double = d.x` with `d.x = 1;`). Defaults that name
    /// no assigned variable (`pi`, `zeros(2)`, `myfun(1)`, `'abc'`, `true`, an input)
    /// are silent.
    pub(crate) fn check_fvniv(
        &self,
        func_meta: &FunctionMeta,
        input_props: &[tree_sitter::Node],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVNIV") {
            return;
        }
        let mut assigned: Option<HashSet<String>> = None;
        for prop in input_props {
            if prop_has_ignored(*prop) {
                continue;
            }
            let text = node_text(*prop, source);
            let Some(eq) = default_separator(text) else {
                continue;
            };
            let refs = referenced_names(&text[eq + 1..]);
            if refs.is_empty() {
                continue;
            }
            let names = assigned.get_or_insert_with(|| assigned_names(*prop, source));
            let fires = refs
                .iter()
                .any(|r| !func_meta.inputs.contains(r) && names.contains(r));
            if fires {
                self.push_diag(
                    *prop,
                    "FVNIV",
                    "This variable is not an input to the function and cannot be used in an arguments block.",
                    diagnostics,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvniv_fires_on_default_naming_a_later_local() {
        let source = "\
function f(a, b)
    arguments
        a double
        b double = a + tmp
    end
    tmp = 1;
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "FVNIV").len(), 1, "{diags:?}");
    }

    #[test]
    fn test_fvniv_fires_on_field_of_a_later_local() {
        let source = "\
function f(a)
    arguments
        a double = d.x
    end
    d.x = 1;
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "FVNIV").len(), 1, "{diags:?}");
    }

    #[test]
    fn test_fvniv_silent_on_builtins_inputs_and_unassigned_names() {
        let source = "\
function f(a, b)
    arguments
        a double = pi
        b double = a
    end
    c = 1;
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVNIV").is_empty(), "{diags:?}");

        let source = "\
function f(a)
    arguments
        a double = myfun(1)
        b double = zeros(2)
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVNIV").is_empty(), "{diags:?}");
    }

    #[test]
    fn test_fvniv_no_fire_declared_input() {
        let source = "\
function f(a)
    arguments
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVNIV").is_empty(),
            "FVNIV should NOT fire for a declared input"
        );
    }
}
