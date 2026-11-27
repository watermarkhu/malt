//! `MP_LAYOUT_11`: blocks with equal attributes are combined and defaults are not spelled out.

use std::collections::HashMap;

use malt_core::{Config, Diagnostic, FileContext};
use tree_sitter::Node;

use super::text::{children, walk, Text};

/// Attributes of a `properties` block and their default values.
const PROPERTY_DEFAULTS: &[(&str, &str)] = &[
    ("Abstract", "false"),
    ("AbortSet", "false"),
    ("Access", "public"),
    ("Constant", "false"),
    ("Dependent", "false"),
    ("GetAccess", "public"),
    ("GetObservable", "false"),
    ("Hidden", "false"),
    ("NonCopyable", "false"),
    ("SetAccess", "public"),
    ("SetObservable", "false"),
    ("Transient", "false"),
];

/// Attributes of a `methods` block and their default values.
const METHOD_DEFAULTS: &[(&str, &str)] = &[
    ("Abstract", "false"),
    ("Access", "public"),
    ("Hidden", "false"),
    ("Sealed", "false"),
    ("Static", "false"),
];

/// Report an attribute set to its default (`Hidden = false`, `Access = public`), and a
/// `properties` or `methods` block of a class whose attributes, defaults left out and in any
/// order, equal those of an earlier block of the same kind. The finding for a duplicate sits on
/// its keyword and names the line of the first block.
///
/// Attribute values are compared as written (quotes around a name are ignored, `true` and
/// `false` are not case-sensitive), so `Access = ?A` and `Access = ?B` differ. A bare attribute
/// (`Constant`) means `Constant = true`.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let text = Text::new(ctx.source, ctx.tree.root_node());
    walk(ctx.tree.root_node(), |node, _| {
        if node.kind() != "class_definition" {
            return;
        }
        let mut seen: HashMap<(&str, String), usize> = HashMap::new();
        for block in children(node) {
            let kind = block.kind();
            if block.child_count() == 0 || !matches!(kind, "properties" | "methods") {
                continue;
            }
            let defaults = if kind == "properties" {
                PROPERTY_DEFAULTS
            } else {
                METHOD_DEFAULTS
            };
            let mut kept = Vec::new();
            if let Some(attributes) = children(block)
                .into_iter()
                .find(|c| c.kind() == "attributes")
            {
                for attribute in children(attributes)
                    .into_iter()
                    .filter(|c| c.kind() == "attribute")
                {
                    let (name, value) = split(&text, attribute);
                    if defaults.iter().any(|(n, v)| *n == name && *v == value) {
                        out.push(text.finding(
                            "MP_LAYOUT_11",
                            attribute.start_byte()..attribute.end_byte(),
                            format!(
                                "Leave out `{name}`, it is set to its default value `{value}`."
                            ),
                        ));
                    } else {
                        kept.push(format!("{name}={value}"));
                    }
                }
            }
            kept.sort();
            let line = text.line_of(block.start_byte());
            if let Some(first) = seen.get(&(kind, kept.join(","))) {
                let keyword = block.child(0).unwrap_or(block);
                out.push(text.finding(
                    "MP_LAYOUT_11",
                    keyword.start_byte()..keyword.end_byte(),
                    format!(
                        "Combine this {kind} block with the one on line {}, it has the same attributes.",
                        first + 1
                    ),
                ));
            } else {
                seen.insert((kind, kept.join(",")), line);
            }
        }
    });
}

/// The name and normalised value of an `attribute` node. A bare name is `true`.
fn split(text: &Text, attribute: Node) -> (String, String) {
    let source = &text.source[attribute.start_byte()..attribute.end_byte()];
    match source.split_once('=') {
        Some((name, value)) => {
            let value = value.trim().trim_matches(['\'', '"']);
            let value = if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false")
            {
                value.to_ascii_lowercase()
            } else {
                value.to_string()
            };
            (name.trim().to_string(), value)
        }
        None => (source.trim().to_string(), "true".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::layout::hits;

    fn at(source: &str) -> Vec<(usize, usize)> {
        hits("MP_LAYOUT_11", source, "")
            .iter()
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn defaults_and_duplicates_are_both_reported() {
        let source = "classdef Rocket\n    properties (Access = public, Hidden = false, Constant)\n        Fuel = 1\n    end\n\n    properties (Constant = true)\n        Capacity = 2\n    end\nend\n";
        let d = hits("MP_LAYOUT_11", source, "");
        let at: Vec<_> = d.iter().map(|d| (d.line, d.column)).collect();
        assert_eq!(at, [(2, 17), (2, 34), (6, 5)], "{d:?}");
        assert!(d[2].message.contains("line 2"), "{}", d[2].message);
    }

    #[test]
    fn attribute_order_does_not_matter() {
        let source = "classdef A\n    properties (Constant, Hidden)\n    end\n    properties (Hidden, Constant)\n    end\nend\n";
        assert_eq!(at(source), [(4, 5)]);
    }

    #[test]
    fn plain_blocks_are_duplicates_of_each_other() {
        let source = "classdef A\n    methods\n    end\n    methods\n    end\nend\n";
        assert_eq!(at(source), [(4, 5)]);
    }

    #[test]
    fn different_attributes_and_kinds_are_fine() {
        let source = "classdef A\n    properties\n    end\n    properties (Access = private)\n    end\n    methods\n    end\n    methods (Static)\n    end\n    methods (Access = ?B)\n    end\n    methods (Access = ?C)\n    end\nend\n";
        assert!(at(source).is_empty(), "{:?}", at(source));
    }

    #[test]
    fn non_default_values_are_kept() {
        let source =
            "classdef A\n    properties (Constant = true, Access = private)\n    end\nend\n";
        assert!(at(source).is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        let _ = at("classdef A\n    properties (Constant\n    end\n    properties (\n");
    }
}
