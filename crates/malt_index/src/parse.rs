//! Parsing one MATLAB file into entities, with tree-sitter.
//!
//! The file kind follows the first declaration: a file whose first statement is a
//! `function` is a function file, one starting with `classdef` is a class file,
//! and anything else is a script. Syntax errors never stop the parse: tree-sitter
//! recovers, and nodes that are not understood are skipped.

use std::cell::RefCell;
use std::path::Path;

use tree_sitter::{Node, Parser, Tree};

use crate::comments::{comment_docstring, node_text};
use crate::config::ParserConfig;
use crate::error::IndexError;
use crate::model::{
    Argument, ArgumentKind, Detail, Docstring, Entity, FunctionDetail, Kind, PropertyDetail,
};

thread_local! {
    static PARSER: RefCell<Option<Parser>> = const { RefCell::new(None) };
}

/// Parse a file from disk. The text is decoded lossily; a UTF-8 BOM is ignored.
pub fn parse_file(path: &Path, config: &ParserConfig) -> Result<Entity, IndexError> {
    let bytes = std::fs::read(path).map_err(|e| IndexError::io(path, &e))?;
    parse_source(path, &bytes, config)
}

/// Parse MATLAB source bytes that belong to `path`.
pub fn parse_source(
    path: &Path,
    bytes: &[u8],
    config: &ParserConfig,
) -> Result<Entity, IndexError> {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    // Text that is not UTF-8 is read as Latin-1 (maxx detects the encoding; the
    // Latin-1 reading matches it for the Western files MATLAB users write).
    let transcoded: Vec<u8>;
    let src: &[u8] = if std::str::from_utf8(bytes).is_ok() {
        bytes
    } else {
        transcoded = bytes
            .iter()
            .flat_map(|b| char::from(*b).to_string().into_bytes())
            .collect();
        &transcoded
    };
    let tree = parse_tree(src)?;
    let ctx = Ctx {
        src,
        path,
        stem: path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        config,
    };
    Ok(ctx.file(&tree))
}

fn parse_tree(src: &[u8]) -> Result<Tree, IndexError> {
    PARSER.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            let mut parser = Parser::new();
            parser
                .set_language(&tree_sitter_matlab::LANGUAGE.into())
                .map_err(|e| IndexError::Grammar(e.to_string()))?;
            *slot = Some(parser);
        }
        let parser = slot
            .as_mut()
            .ok_or_else(|| IndexError::Grammar("parser unavailable".into()))?;
        parser
            .parse(src, None)
            .ok_or_else(|| IndexError::Grammar("parse was cancelled".into()))
    })
}

/// The named children of a node, in order.
pub(crate) fn named(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

/// All children of a node (named and anonymous tokens), in order.
fn all(node: Node) -> Vec<Node> {
    (0..node.child_count())
        .filter_map(|i| node.child(i))
        .collect()
}

/// The first named child of a given kind.
fn named_of_kind<'t>(node: Node<'t>, kind: &str) -> Option<Node<'t>> {
    named(node).into_iter().find(|n| n.kind() == kind)
}

/// Start and end line of a node, 1-based and inclusive.
fn span(node: Node) -> (usize, usize) {
    (node.start_position().row + 1, node.end_position().row + 1)
}

/// The value of an attribute: `Name` alone is `true`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AttrValue {
    Flag(bool),
    Text(String),
}

impl AttrValue {
    fn truthy(&self) -> bool {
        match self {
            AttrValue::Flag(b) => *b,
            AttrValue::Text(s) => !s.is_empty(),
        }
    }
}

/// The parse context: the source of one file.
pub(crate) struct Ctx<'a> {
    src: &'a [u8],
    path: &'a Path,
    stem: String,
    config: &'a ParserConfig,
}

impl Ctx<'_> {
    fn text(&self, node: Node) -> String {
        node_text(self.src, node)
    }

    /// The file: its kind is decided by the first statement.
    fn file(&self, tree: &Tree) -> Entity {
        let root = tree.root_node();
        let children = named(root);
        let first = children.iter().position(|n| n.kind() != "comment");
        let header: Vec<Node> = children[..first.unwrap_or(children.len())].to_vec();
        let header_doc = comment_docstring(self.src, &header);
        let first_node = first.map(|i| children[i]);

        let mut entity = match first_node.map(|n| n.kind()) {
            Some("function_definition") => {
                let node = first_node.unwrap_or(root);
                let mut entity = self
                    .function(node, Role::File, &Default::default())
                    .unwrap_or_else(|| Entity::new(Kind::Function, &self.stem));
                entity.docstring = entity.docstring.or(header_doc);
                entity
            }
            Some("class_definition") => {
                let node = first_node.unwrap_or(root);
                let mut entity = self.class(node);
                entity.docstring = entity.docstring.or(header_doc);
                entity
            }
            _ => {
                let mut entity = Entity::new(Kind::Script, &self.stem);
                entity.path = Some(self.path.to_path_buf());
                entity.docstring = header_doc;
                entity
            }
        };
        // Local functions: every other top-level function of the file. A function
        // file's local functions are its members; a script's are too.
        if entity.kind != Kind::Class {
            for child in &children {
                if child.kind() == "function_definition" && Some(*child) != first_node {
                    if let Some(local) = self.function(*child, Role::Nested, &Default::default()) {
                        entity.members.insert(local.name.clone(), local);
                    }
                }
            }
        }
        entity
    }

    /// A classdef: bases, class attributes, enumerations, properties and methods.
    fn class(&self, node: Node) -> Entity {
        let children = named(node);
        let name = self.stem.clone();
        let bases: Vec<String> = children
            .iter()
            .filter(|c| c.kind() == "superclasses")
            .flat_map(|s| named(*s))
            .filter(|c| c.kind() == "property_name")
            .map(|c| self.text(c))
            .collect();
        let mut attributes = std::collections::BTreeSet::new();
        for attrs in children.iter().filter(|c| c.kind() == "attributes") {
            for attribute in named(*attrs)
                .into_iter()
                .filter(|c| c.kind() == "attribute")
            {
                if let Some((key, value)) = self.attribute(attribute) {
                    if matches!(key.as_str(), "Sealed" | "Abstract" | "Hidden") && value.truthy() {
                        attributes.insert(key);
                    }
                }
            }
        }
        let block_start = children
            .iter()
            .position(|c| {
                matches!(
                    c.kind(),
                    "properties" | "methods" | "enumeration" | "events"
                )
            })
            .unwrap_or(children.len());
        let leading: Vec<Node> = children[..block_start]
            .iter()
            .filter(|c| c.kind() == "comment")
            .copied()
            .collect();

        let mut entity = Entity::new(Kind::Class, &name);
        let (lineno, endlineno) = span(node);
        entity.lineno = Some(lineno);
        entity.endlineno = Some(endlineno);
        entity.path = Some(self.path.to_path_buf());
        entity.docstring = comment_docstring(self.src, &leading);
        entity.attributes = attributes;
        entity.detail = Detail::Class { bases };

        for block in children.iter().filter(|c| c.kind() == "enumeration") {
            for member in self.enumeration(*block) {
                entity.members.insert(member.name.clone(), member);
            }
        }
        for block in children.iter().filter(|c| c.kind() == "properties") {
            for property in self.properties(*block) {
                entity.members.insert(property.name.clone(), property);
            }
        }
        for block in children.iter().filter(|c| c.kind() == "methods") {
            let attrs = self.method_attributes(*block);
            for method in named(*block)
                .into_iter()
                .filter(|c| c.kind() == "function_definition")
            {
                if let Some(method) = self.function(method, Role::Method, &attrs) {
                    self.attach_method(&mut entity, method);
                }
            }
        }
        entity
    }

    /// A getter or setter attaches to the property of the same name, when the
    /// class declares it; otherwise it is an ordinary member. Like maxx, a
    /// getter or setter whose name is taken by a non-property member is dropped.
    fn attach_method(&self, class: &mut Entity, method: Entity) {
        let is_accessor = method.function().map(|f| (f.is_getter, f.is_setter));
        let (getter, setter) = is_accessor.unwrap_or((false, false));
        if getter || setter {
            if let Some(existing) = class.members.get_mut(&method.name) {
                if let Detail::Property(detail) = &mut existing.detail {
                    if getter {
                        detail.getter = Some(Box::new(method));
                    } else {
                        detail.setter = Some(Box::new(method));
                    }
                }
                return;
            }
        }
        class.members.insert(method.name.clone(), method);
    }

    /// The `attribute` node: its name and its value (`Name`, `Name = value`).
    pub(crate) fn attribute(&self, node: Node) -> Option<(String, AttrValue)> {
        let key_node = named_of_kind(node, "identifier")?;
        let key = self.text(key_node);
        let values: Vec<Node> = all(node)
            .into_iter()
            .filter(|c| c.id() != key_node.id() && c.kind() != "=" && c.is_named())
            .collect();
        let value = match values.as_slice() {
            [] => AttrValue::Flag(true),
            [single] if single.kind() == "boolean" => {
                let t = self.text(*single).to_lowercase();
                AttrValue::Flag(t == "true" || t == "1")
            }
            _ => AttrValue::Text(
                values
                    .iter()
                    .map(|v| self.text(*v))
                    .collect::<Vec<_>>()
                    .join(""),
            ),
        };
        Some((key, value))
    }

    /// Attributes of a `methods` block, as the set of attribute strings.
    fn method_attributes(&self, block: Node) -> std::collections::BTreeSet<String> {
        let mut set = std::collections::BTreeSet::new();
        for attrs in named(block)
            .into_iter()
            .filter(|c| c.kind() == "attributes")
        {
            for attribute in named(attrs).into_iter().filter(|c| c.kind() == "attribute") {
                let Some((key, value)) = self.attribute(attribute) else {
                    continue;
                };
                match key.as_str() {
                    "Abstract" | "Hidden" | "Sealed" | "Static" if value.truthy() => {
                        set.insert(key);
                    }
                    "Access" => {
                        let access = access_value(&value);
                        if access != "public" {
                            set.insert(format!("Access={access}"));
                        }
                    }
                    _ => {}
                }
            }
        }
        set
    }

    /// The members of a `properties` block, in order.
    fn properties(&self, block: Node) -> Vec<Entity> {
        let mut set = std::collections::BTreeSet::new();
        for attrs in named(block)
            .into_iter()
            .filter(|c| c.kind() == "attributes")
        {
            for attribute in named(attrs).into_iter().filter(|c| c.kind() == "attribute") {
                let Some((key, value)) = self.attribute(attribute) else {
                    continue;
                };
                match key.as_str() {
                    "AbortSet" | "Abstract" | "Constant" | "Dependent" | "GetObservable"
                    | "Hidden" | "NonCopyable" | "SetObservable" | "Transient" | "WeakHandle"
                        if value.truthy() =>
                    {
                        set.insert(key);
                    }
                    "Access" | "GetAccess" | "SetAccess" => {
                        let access = access_value(&value);
                        if access != "public" {
                            set.insert(format!("{key}={access}"));
                        }
                    }
                    _ => {}
                }
            }
        }

        let items: Vec<Node> = named(block)
            .into_iter()
            .filter(|c| matches!(c.kind(), "property" | "comment"))
            .collect();
        let mut properties: Vec<Entity> = Vec::new();
        let mut pending: Option<Docstring> = None;
        for item in items {
            if item.kind() == "comment" {
                let doc = comment_docstring(self.src, &[item]);
                if self.config.docstring_before_properties {
                    pending = doc;
                } else if let (Some(doc), Some(last)) = (doc, properties.last_mut()) {
                    last.docstring = Some(doc);
                }
                continue;
            }
            let (name, detail) = self.property_parts(item);
            let mut entity = Entity::new(Kind::Property, &name);
            let (lineno, endlineno) = span(item);
            entity.lineno = Some(lineno);
            entity.endlineno = Some(endlineno);
            entity.path = Some(self.path.to_path_buf());
            entity.attributes = set.clone();
            entity.detail = Detail::Property(detail);
            if self.config.docstring_before_properties {
                entity.docstring = pending.take();
            }
            properties.push(entity);
        }
        properties
    }

    /// The name, type, dimensions, validators and default of a property node.
    fn property_parts(&self, node: Node) -> (String, PropertyDetail) {
        let name_node = node.child_by_field_name("name");
        let name = match name_node {
            Some(n) if n.kind() == "property_name" => {
                named(n).last().map(|i| self.text(*i)).unwrap_or_default()
            }
            Some(n) => self.text(n),
            None => String::new(),
        };
        // The type is every identifier after the name (maxx joins them: `uint8 scalar`
        // is read as `uint8scalar`), up to the validators and the default.
        let mut detail = PropertyDetail::default();
        let mut type_parts: Vec<String> = Vec::new();
        for child in named(node) {
            if name_node.is_some_and(|n| n.id() == child.id()) {
                continue;
            }
            match child.kind() {
                "dimensions" => detail.dimensions = Some(self.dimensions(child)),
                "validation_functions" => detail.validators = Some(self.text(child)),
                "default_value" => detail.default = Some(self.default_text(child)),
                "identifier" | "property_name"
                    if detail.default.is_none() && detail.validators.is_none() =>
                {
                    type_parts.push(self.text(child));
                }
                _ => {}
            }
        }
        if !type_parts.is_empty() {
            detail.type_ = Some(type_parts.concat());
        }
        (name, detail)
    }

    /// The dimensions as written: the numbers and `:` (spread operators) of a `dimensions` node.
    fn dimensions(&self, node: Node) -> Vec<String> {
        named(node)
            .into_iter()
            .filter(|c| matches!(c.kind(), "number" | "spread_operator"))
            .map(|c| self.text(c))
            .collect()
    }

    /// The default value as written: the nodes after `=`, joined.
    fn default_text(&self, node: Node) -> String {
        all(node)
            .into_iter()
            .filter(|c| c.kind() != "=")
            .map(|c| self.text(c))
            .collect::<Vec<_>>()
            .join("")
    }

    /// The enumeration members of an `enumeration` block.
    fn enumeration(&self, block: Node) -> Vec<Entity> {
        // The block's content in source order: each member name (the first
        // identifier of an `enum` node), its value nodes, and the comments.
        enum Item<'t> {
            Name(Node<'t>, String),
            Value(Node<'t>),
            Comment(Node<'t>),
        }
        let mut content: Vec<(usize, Item)> = Vec::new();
        for child in named(block) {
            match child.kind() {
                "enum" => {
                    for (i, part) in named(child).into_iter().enumerate() {
                        if i == 0 && part.kind() == "identifier" {
                            content.push((part.start_byte(), Item::Name(child, self.text(part))));
                        } else {
                            content.push((part.start_byte(), Item::Value(part)));
                        }
                    }
                }
                "comment" => content.push((child.start_byte(), Item::Comment(child))),
                _ => {}
            }
        }
        content.sort_by_key(|(start, _)| *start);

        let before_mode = self.config.docstring_before_enumerations;
        let mut members: Vec<Entity> = Vec::new();
        let mut current: Option<(Node, String)> = None;
        let mut values: Vec<Node> = Vec::new();
        let mut after: Vec<Node> = Vec::new();
        let mut before: Vec<Node> = Vec::new();
        let mut flush = |current: &Option<(Node, String)>, comments: &[Node], values: &[Node]| {
            if let Some((enum_node, name)) = current {
                let mut entity = Entity::new(Kind::Enumeration, name);
                let (lineno, endlineno) = span(*enum_node);
                entity.lineno = Some(lineno);
                entity.endlineno = Some(endlineno);
                entity.path = Some(self.path.to_path_buf());
                entity.docstring = comment_docstring(self.src, comments);
                let value: String = values.iter().map(|v| self.text(*v)).collect();
                entity.detail = Detail::Enumeration {
                    value: if values.is_empty() { None } else { Some(value) },
                };
                members.push(entity);
            }
        };
        for (_, item) in content {
            match item {
                Item::Name(enum_node, name) => {
                    let comments = if before_mode { &before } else { &after };
                    flush(&current, comments, &values);
                    before = std::mem::take(&mut after);
                    values.clear();
                    current = Some((enum_node, name));
                }
                Item::Comment(node) => after.push(node),
                Item::Value(node) => values.push(node),
            }
        }
        let comments = if before_mode { &before } else { &after };
        flush(&current, comments, &values);
        members
    }

    /// Parse a function node. Returns `None` when the name is missing (a
    /// syntax error in a method).
    pub(crate) fn function(
        &self,
        node: Node,
        role: Role,
        attrs: &std::collections::BTreeSet<String>,
    ) -> Option<Entity> {
        let is_method = role == Role::Method;
        let name = match role {
            Role::File => self.stem.clone(),
            Role::Method | Role::Nested => self.text(node.child_by_field_name("name")?),
        };
        if name.is_empty() {
            return None;
        }
        let children = named(node);
        let is_getter = all(node)
            .iter()
            .any(|c| !c.is_named() && self.text(*c).trim() == "get.");
        let is_setter = all(node)
            .iter()
            .any(|c| !c.is_named() && self.text(*c).trim() == "set.");

        let block_start = children
            .iter()
            .position(|c| matches!(c.kind(), "arguments_statement" | "block"))
            .unwrap_or(children.len());
        let leading: Vec<Node> = children[..block_start]
            .iter()
            .filter(|c| c.kind() == "comment")
            .copied()
            .collect();

        let mut inputs: Vec<Argument> = Vec::new();
        if let Some(args) = children.iter().find(|c| c.kind() == "function_arguments") {
            for child in named(*args) {
                if let Some(arg) = self.signature_name(child) {
                    inputs.push(Argument::new(&arg, ArgumentKind::PositionalOnly));
                }
            }
        }
        let mut returns: Vec<Argument> = Vec::new();
        if let Some(output) = children.iter().find(|c| c.kind() == "function_output") {
            for child in named(*output) {
                if child.kind() == "multioutput_variable" {
                    for part in named(child) {
                        if let Some(arg) = self.signature_name(part) {
                            returns.push(Argument::new(&arg, ArgumentKind::PositionalOnly));
                        }
                    }
                } else if let Some(arg) = self.signature_name(child) {
                    returns.push(Argument::new(&arg, ArgumentKind::PositionalOnly));
                }
            }
        }

        for block in children
            .iter()
            .filter(|c| c.kind() == "arguments_statement")
        {
            self.arguments_block(*block, &mut inputs, &mut returns);
        }
        for input in inputs.iter_mut() {
            if input.name == "varargin" {
                input.kind = ArgumentKind::Varargin;
            }
        }

        let is_static = attrs.contains("Static");
        if is_method && name != self.stem && !is_static && !inputs.is_empty() {
            // The first input of a method is the object itself.
            inputs.remove(0);
        }

        let (lineno, endlineno) = span(node);
        let mut entity = Entity::new(Kind::Function, &name);
        entity.lineno = Some(lineno);
        entity.endlineno = Some(endlineno);
        entity.path = Some(self.path.to_path_buf());
        entity.docstring = comment_docstring(self.src, &leading);
        entity.attributes = attrs.clone();
        entity.detail = Detail::Function(FunctionDetail {
            arguments: inputs,
            returns,
            is_getter,
            is_setter,
        });

        // Local and nested functions: the functions in the body of this one.
        if let Some(body) = children.iter().find(|c| c.kind() == "block") {
            for child in named(*body) {
                if child.kind() == "function_definition" {
                    if let Some(nested) = self.function(child, Role::Nested, &Default::default()) {
                        entity.members.insert(nested.name.clone(), nested);
                    }
                }
            }
        }
        Some(entity)
    }

    /// The name of a signature item: an identifier, or `~` for an ignored one.
    fn signature_name(&self, node: Node) -> Option<String> {
        match node.kind() {
            "identifier" => Some(self.text(node)),
            "ignored_argument" => Some("~".to_string()),
            _ => None,
        }
    }

    /// Apply one `arguments` block to the inputs and outputs of a function, as maxx
    /// does: only names in the signature are updated; a name-value argument
    /// (`opts.Name`) is added as a keyword-only argument.
    fn arguments_block(
        &self,
        block: Node,
        inputs: &mut Vec<Argument>,
        returns: &mut Vec<Argument>,
    ) {
        let attrs: Vec<String> = named_of_kind(block, "attributes")
            .map(|a| {
                named(a)
                    .into_iter()
                    .filter(|c| c.kind() == "identifier")
                    .map(|c| self.text(c))
                    .collect()
            })
            .unwrap_or_default();
        let is_input = attrs.iter().any(|a| a == "Input") || !attrs.iter().any(|a| a == "Output");
        let before_mode = self.config.docstring_before_arguments;

        // Where the next comment attaches: the input or output named here, or nothing.
        let mut current: Option<(Slot, String)> = None;
        let mut pending: Option<Docstring> = None;
        for item in named(block) {
            match item.kind() {
                "comment" => {
                    let doc = comment_docstring(self.src, &[item]);
                    if before_mode {
                        pending = doc;
                    } else if let (Some(doc), Some((slot, name))) = (doc, current.clone()) {
                        if let Some(arg) = find_mut(inputs, returns, slot, &name) {
                            arg.docstring = Some(doc);
                        }
                    }
                }
                "class_property" => {
                    // `opts.?Class.path`: a keyword argument whose class is a class path.
                    let text = self.text(item);
                    let (opts, rest) = match text.split_once(".?") {
                        Some(parts) => parts,
                        None => continue,
                    };
                    let class_path = rest.trim().trim_end_matches(';').trim();
                    if class_path.is_empty() {
                        continue;
                    }
                    let mut arg = Argument::new(opts.trim(), ArgumentKind::KeywordOnly);
                    arg.type_ = Some(class_path.to_string());
                    if before_mode {
                        arg.docstring = pending.take();
                    }
                    upsert(inputs, arg);
                    current = Some((Slot::Input, opts.trim().to_string()));
                }
                "property" => {
                    let (name, detail) = self.property_parts(item);
                    let name_node = item.child_by_field_name("name");
                    let is_keyword = name_node.is_some_and(|n| n.kind() == "property_name");
                    let has_default = detail.default.is_some();
                    let doc = if before_mode { pending.take() } else { None };
                    if is_keyword {
                        let options = name_node
                            .and_then(|n| named(n).first().map(|i| self.text(*i)))
                            .unwrap_or_default();
                        inputs.retain(|a| a.name != options);
                        let mut arg = Argument::new(&name, ArgumentKind::KeywordOnly);
                        fill_argument(&mut arg, &detail);
                        arg.docstring = doc;
                        upsert(inputs, arg);
                        current = Some((Slot::Input, name));
                    } else {
                        let kind = if has_default {
                            ArgumentKind::Optional
                        } else {
                            ArgumentKind::PositionalOnly
                        };
                        let (slot, list) = if is_input {
                            (Slot::Input, &mut *inputs)
                        } else {
                            (Slot::Output, &mut *returns)
                        };
                        if let Some(arg) = list.iter_mut().find(|a| a.name == name) {
                            arg.kind = kind;
                            fill_argument(arg, &detail);
                            arg.docstring = doc;
                            current = Some((slot, name));
                        } else {
                            // A name that is not in the signature is dropped, as maxx does.
                            current = Some((Slot::Detached, name));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// What a function node is: the file's own function (named after the file), a method
/// of a classdef, or a local or nested function (named by its identifier).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    File,
    Method,
    Nested,
}

/// The access level of an attribute value: one of the four known levels, otherwise private.
fn access_value(value: &AttrValue) -> String {
    match value {
        AttrValue::Text(s)
            if matches!(s.as_str(), "public" | "protected" | "private" | "immutable") =>
        {
            s.clone()
        }
        _ => "private".to_string(),
    }
}

/// Copy the dimensions, validators, type and default of a property into an argument.
fn fill_argument(arg: &mut Argument, detail: &PropertyDetail) {
    if detail.dimensions.is_some() {
        arg.dimensions = detail.dimensions.clone();
    }
    if detail.type_.is_some() {
        arg.type_ = detail.type_.clone();
    }
    if detail.validators.is_some() {
        arg.validators = detail.validators.clone();
    }
    if detail.default.is_some() {
        arg.default = detail.default.clone();
    }
}

/// Replace the argument of the same name in place, or append it.
fn upsert(list: &mut Vec<Argument>, arg: Argument) {
    match list.iter().position(|a| a.name == arg.name) {
        Some(i) => list[i] = arg,
        None => list.push(arg),
    }
}

/// The list an arguments-block comment attaches to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Slot {
    Input,
    Output,
    /// A name that is not in the signature: its comments go nowhere.
    Detached,
}

fn find_mut<'a>(
    inputs: &'a mut [Argument],
    returns: &'a mut [Argument],
    slot: Slot,
    name: &str,
) -> Option<&'a mut Argument> {
    match slot {
        Slot::Input => inputs.iter_mut().find(|a| a.name == name),
        Slot::Output => returns.iter_mut().find(|a| a.name == name),
        Slot::Detached => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Entity {
        parse_source(
            Path::new("/x/Sample.m"),
            src.as_bytes(),
            &ParserConfig::default(),
        )
        .unwrap()
    }

    #[test]
    fn function_file_takes_its_name_from_the_file() {
        let e = parse("function r = Sample(a)\nr = a;\nend\n");
        assert_eq!(e.kind, Kind::Function);
        assert_eq!(e.name, "Sample");
        assert_eq!(e.lineno, Some(1));
        assert_eq!(e.endlineno, Some(3));
    }
}
