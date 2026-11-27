//! Methods declared in a classdef file and where their implementation is, for FCNS-18
//! and FCNS-19.
//!
//! A `methods` block that is not abstract and holds only a signature promises an
//! implementation elsewhere: a function of the same name in the class folder
//! (`@Class/name.m`), or, as the standard's examples treat it, a function of that name in
//! the same file after the class.

use std::path::Path;

use malt_core::FileContext;
use tree_sitter::Node;

use super::common::{function_name, has_true_attribute, inputs, name_range, outputs, text};

/// The inputs and outputs of a method, as written.
#[derive(Debug, Clone)]
pub(crate) struct Signature {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

impl Signature {
    fn of(node: Node, source: &str) -> Self {
        let names = |nodes: Vec<Node>| nodes.iter().map(|n| text(*n, source).to_string()).collect();
        Self {
            inputs: names(inputs(node)),
            outputs: names(outputs(node)),
        }
    }

    /// Whether two signatures list the same names in the same order. A `~` matches any
    /// name.
    pub(crate) fn matches(&self, other: &Self) -> bool {
        let same = |a: &[String], b: &[String]| {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x == y || x == "~" || y == "~")
        };
        same(&self.inputs, &other.inputs) && same(&self.outputs, &other.outputs)
    }

    /// The signature as `[outputs] = (inputs)` for messages.
    pub(crate) fn describe(&self) -> String {
        format!(
            "[{}] = ({})",
            self.outputs.join(", "),
            self.inputs.join(", ")
        )
    }
}

/// Where the implementation of a declared method is.
pub(crate) enum Implementation {
    /// Found, with its signature.
    Found(Signature),
    /// Not in the file and not in the class folder.
    Missing,
    /// Cannot be told: the file is not on disk, or the class folder holds something that is
    /// not a plain `.m` file.
    Unknown,
}

/// A method declared by signature in a non-abstract `methods` block.
pub(crate) struct Declaration {
    /// The method name.
    pub name: String,
    /// Range of the name in the declaration.
    pub range: std::ops::Range<usize>,
    /// What the declaration lists.
    pub declared: Signature,
    /// Where the implementation is.
    pub implementation: Implementation,
}

/// The declarations of the first class in the file.
pub(crate) fn declarations(ctx: &FileContext) -> Vec<Declaration> {
    let root = ctx.tree.root_node();
    let source = ctx.source;
    let mut cursor = root.walk();
    let Some(class) = root
        .named_children(&mut cursor)
        .find(|c| c.kind() == "class_definition")
    else {
        return Vec::new();
    };
    let locals: Vec<Node> = {
        let mut c = root.walk();
        let found = root
            .named_children(&mut c)
            .filter(|n| n.kind() == "function_definition")
            .collect();
        found
    };
    let folder = std::fs::canonicalize(ctx.file_path)
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let mut out = Vec::new();
    let mut c = class.walk();
    for block in class.children(&mut c).filter(|n| n.kind() == "methods") {
        if has_true_attribute(block, "Abstract", source) {
            continue;
        }
        let mut inner = block.walk();
        for signature in block
            .named_children(&mut inner)
            .filter(|n| n.kind() == "function_signature")
        {
            let name = function_name(signature, source).to_string();
            let implementation =
                if let Some(local) = locals.iter().find(|f| function_name(**f, source) == name) {
                    Implementation::Found(Signature::of(*local, source))
                } else {
                    from_class_folder(folder.as_deref(), &name)
                };
            out.push(Declaration {
                range: name_range(signature),
                declared: Signature::of(signature, source),
                name,
                implementation,
            });
        }
    }
    out
}

fn from_class_folder(folder: Option<&Path>, name: &str) -> Implementation {
    let Some(folder) = folder else {
        return Implementation::Unknown;
    };
    let in_class_folder = folder
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with('@'));
    if !in_class_folder {
        return Implementation::Missing;
    }
    let file = folder.join(format!("{name}.m"));
    if !file.exists() {
        let other = ["p", "mlx", "mex"]
            .iter()
            .any(|ext| folder.join(format!("{name}.{ext}")).exists());
        return if other {
            Implementation::Unknown
        } else {
            Implementation::Missing
        };
    }
    let Ok(source) = std::fs::read_to_string(&file) else {
        return Implementation::Unknown;
    };
    // The grammar is not a dependency of this crate, so read the header line by hand.
    parse_header(&source).map_or(Implementation::Unknown, Implementation::Found)
}

/// The signature on the first `function` line of a file: `function [a, b] = name(c, d)`,
/// `function a = name(c)` or `function name(c)`. `None` when the header is split over
/// lines or not in a shape this reads.
fn parse_header(source: &str) -> Option<Signature> {
    let line = source
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('%'))?;
    let rest = line.strip_prefix("function")?;
    if !rest.starts_with([' ', '\t']) || rest.contains("...") {
        return None;
    }
    let rest = rest.split('%').next()?.trim();
    let list = |text: &str| -> Vec<String> {
        text.trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split([',', ' '])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect()
    };
    let (outputs, call) = match rest.split_once('=') {
        Some((outputs, call)) => (list(outputs), call.trim()),
        None => (Vec::new(), rest),
    };
    let args = match call.split_once('(') {
        Some((_, args)) => list(args.trim_end().trim_end_matches(';').trim_end_matches(')')),
        None => Vec::new(),
    };
    Some(Signature {
        inputs: args,
        outputs,
    })
}
