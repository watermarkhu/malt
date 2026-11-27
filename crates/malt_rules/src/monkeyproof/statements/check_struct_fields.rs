//! MP_STAT_13: fields added to a struct after it was used.

use std::collections::{HashMap, HashSet};

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::{
    call_arguments, call_identifier, statements_of, string_value, target_base, text, walk,
};
use crate::monkeyproof::finding;

/// What is known about a struct variable inside one block.
struct Tracked {
    /// The field names it is known to have.
    fields: HashSet<String>,
    /// Whether a statement already used the variable as a whole or read from it.
    used: bool,
}

/// The field names of `struct('a', 1, 'b', 2)`, or `None` when the call is not that or a
/// name is not a literal.
fn struct_fields(node: Node, source: &str) -> Option<HashSet<String>> {
    if node.kind() != "function_call" || call_identifier(node, source) != Some("struct") {
        return None;
    }
    let args = call_arguments(node);
    if !args.len().is_multiple_of(2) {
        return None;
    }
    args.iter()
        .step_by(2)
        .map(|arg| string_value(*arg, source).map(str::to_string))
        .collect()
}

/// Mark every tracked variable that `node` mentions as used.
fn mark_used(node: Node, source: &str, tracked: &mut HashMap<String, Tracked>) {
    if tracked.is_empty() {
        return;
    }
    walk(node, |inner, parent| {
        if inner.kind() != "identifier" {
            return;
        }
        if parent.is_some_and(|p| super::is_member(inner, p)) {
            return;
        }
        if let Some(entry) = tracked.get_mut(text(inner, source)) {
            entry.used = true;
        }
    });
}

/// Flag `s.new = ...` when `s` was created in the same block by `struct(...)` with literal
/// field names, and a statement used `s` in between. The finding covers the assignment
/// target.
///
/// The check follows one block at a time, in order. Any statement that mentions `s` (an
/// argument, a read of a field, a loop or branch that uses it) counts as a use. A struct
/// that comes from anywhere else, such as an input or a call, is not followed, and fields
/// added in a nested block are not reported.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    walk(ctx.tree.root_node(), |block, _| {
        if !matches!(block.kind(), "block" | "source_file") {
            return;
        }
        let mut tracked: HashMap<String, Tracked> = HashMap::new();
        for statement in statements_of(block) {
            if statement.kind() == "assignment" {
                let left = statement.child_by_field_name("left");
                let right = statement.child_by_field_name("right");
                if let (Some(left), Some(right)) = (left, right) {
                    if left.kind() == "identifier" {
                        mark_used(right, ctx.source, &mut tracked);
                        let name = text(left, ctx.source).to_string();
                        match struct_fields(right, ctx.source) {
                            Some(fields) => {
                                tracked.insert(
                                    name,
                                    Tracked {
                                        fields,
                                        used: false,
                                    },
                                );
                            }
                            None => {
                                tracked.remove(&name);
                            }
                        }
                        continue;
                    }
                    if let Some((name, field)) = simple_field_target(left, ctx.source) {
                        if let Some(entry) = tracked.get_mut(name) {
                            match field {
                                Some(field) => {
                                    if entry.used && !entry.fields.contains(field) {
                                        out.push(finding(
                                            ctx.source,
                                            "MP_STAT_13",
                                            left.byte_range(),
                                            format!(
                                                "Define the field {field} where the struct {name} is created."
                                            ),
                                        ));
                                    }
                                    entry.fields.insert(field.to_string());
                                }
                                None => {
                                    tracked.remove(name);
                                }
                            }
                        }
                        mark_used(right, ctx.source, &mut tracked);
                        continue;
                    }
                    // A more complex target (`s(2).a = ...`, `s.a(3) = ...`) is not followed.
                    if let Some(base) = target_base(left) {
                        tracked.remove(text(base, ctx.source));
                    }
                }
            }
            mark_used(statement, ctx.source, &mut tracked);
        }
    });
}

/// For `s.name` the struct name and the field name (`None` for a field that is not a
/// plain name, such as `s.(k)`). `None` when the target is not a dotted field of a name.
fn simple_field_target<'s>(left: Node, source: &'s str) -> Option<(&'s str, Option<&'s str>)> {
    if left.kind() != "field_expression" {
        return None;
    }
    let object = left.child_by_field_name("object")?;
    if object.kind() != "identifier" {
        return None;
    }
    let field = left.child_by_field_name("field")?;
    let field = (field.kind() == "identifier").then(|| text(field, source));
    Some((text(object, source), field))
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::statements::hits;

    #[test]
    fn flags_a_field_added_after_use() {
        let source = "s = struct('f', 2, 'g', 3);\ncomputeCost(s);\ns.h = 'new';\n";
        assert_eq!(hits("MP_STAT_13", source, ""), vec![(3, 1)]);
    }

    #[test]
    fn fields_added_before_use_are_fine() {
        let source = "s = struct();\ns.a = 1;\ns.b = 2;\ncomputeCost(s);\ns.a = 5;\n";
        assert!(hits("MP_STAT_13", source, "").is_empty());
    }

    #[test]
    fn a_field_given_to_struct_is_not_new() {
        let source = "s = struct('f', 2);\nuse(s);\ns.f = 3;\n";
        assert!(hits("MP_STAT_13", source, "").is_empty());
    }

    #[test]
    fn reading_a_field_counts_as_use() {
        let source = "s = struct('a', 1);\nx = s.a + 1;\ns.b = x;\n";
        assert_eq!(hits("MP_STAT_13", source, ""), vec![(3, 1)]);
    }

    #[test]
    fn structs_of_unknown_origin_are_not_followed() {
        let source =
            "s = load('x');\nuse(s);\ns.h = 1;\nfunction f(t)\n  use(t);\n  t.h = 1;\nend\n";
        assert!(hits("MP_STAT_13", source, "").is_empty());
    }

    #[test]
    fn a_reassigned_struct_starts_again() {
        let source = "s = struct('a', 1);\nuse(s);\ns = struct();\ns.b = 1;\n";
        assert!(hits("MP_STAT_13", source, "").is_empty());
    }

    #[test]
    fn survives_a_syntax_error() {
        let _ = hits("MP_STAT_13", "s = struct('a',\ns.b = (\n", "");
    }
}
