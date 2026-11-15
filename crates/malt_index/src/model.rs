//! The data model: what the index knows about a MATLAB path entry.
//!
//! An [`Entity`] is a function, class, property, enumeration, script, package
//! (`+pkg`), class folder (`@cls`) or plain folder. Entities form a tree through
//! [`Entity::members`]. Every field is a fact read from the source: nothing is
//! evaluated, and docstrings are the comment block as text (see [`Docstring`]).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The kind of an entity. Names follow the MATLAB concepts, not maxx's `Kind` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A package, a folder named `+pkg`.
    Namespace,
    /// A class folder, a folder named `@cls`, with the class file and methods.
    ClassFolder,
    /// A classdef class.
    Class,
    /// A member of an `enumeration` block.
    Enumeration,
    /// A function, a method, a getter or setter, or a local or nested function.
    Function,
    /// A script file (one without a leading function or classdef).
    Script,
    /// A property of a class.
    Property,
}

impl Kind {
    /// The kind name used in the dump (the schema of `maxx_dump.py`).
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Namespace => "namespace",
            Kind::ClassFolder | Kind::Class => "class",
            Kind::Enumeration => "enumeration",
            Kind::Function => "function",
            Kind::Script => "script",
            Kind::Property => "property",
        }
    }
}

/// A comment block next to a declaration.
///
/// `value` is the comment text with the comment markers removed and the common
/// indentation taken off, as maxx's docstring extraction does. There is no
/// section parsing here: `raw` is the exact source text of the block, and
/// `lineno`/`endlineno` are its span (1-based, inclusive).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Docstring {
    /// The comment text, markers removed.
    pub value: String,
    /// The exact source text of the comment block.
    pub raw: String,
    /// First line of the block (1-based).
    pub lineno: usize,
    /// Last line of the block (1-based, inclusive).
    pub endlineno: usize,
}

/// How an argument is passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArgumentKind {
    /// Passed by position, no default.
    #[serde(rename = "positional-only")]
    PositionalOnly,
    /// Passed by position, with a default in the arguments block.
    #[serde(rename = "optional")]
    Optional,
    /// Passed by name (`opts.Name` in an arguments block).
    #[serde(rename = "keyword-only")]
    KeywordOnly,
    /// The `varargin` input.
    #[serde(rename = "varargin")]
    Varargin,
}

impl ArgumentKind {
    /// The kind name used in the dump.
    pub fn as_str(self) -> &'static str {
        match self {
            ArgumentKind::PositionalOnly => "positional-only",
            ArgumentKind::Optional => "optional",
            ArgumentKind::KeywordOnly => "keyword-only",
            ArgumentKind::Varargin => "varargin",
        }
    }
}

/// An input or output of a function, from the signature and its arguments block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Argument {
    /// The name as written (`~` for an ignored input or output).
    pub name: String,
    /// How the argument is passed.
    pub kind: ArgumentKind,
    /// The declared class, as written (`double`, `Other.Prop` for `opts.?Other.Prop`).
    #[serde(rename = "type")]
    pub type_: Option<String>,
    /// The dimensions as written: numbers and `:` (spread operators).
    pub dimensions: Option<Vec<String>>,
    /// The validation functions as written, for example `{mustBePositive}`.
    pub validators: Option<String>,
    /// The default value as written.
    pub default: Option<String>,
    /// The comment block next to the argument.
    pub docstring: Option<Docstring>,
}

impl Argument {
    /// A new argument with only a name and a kind.
    pub fn new(name: &str, kind: ArgumentKind) -> Self {
        Argument {
            name: name.to_string(),
            kind,
            type_: None,
            dimensions: None,
            validators: None,
            default: None,
            docstring: None,
        }
    }
}

/// The part of an entity that only functions have.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionDetail {
    /// Inputs, in signature order, then name-value arguments.
    pub arguments: Vec<Argument>,
    /// Outputs, in signature order.
    pub returns: Vec<Argument>,
    /// `get.Name` method.
    pub is_getter: bool,
    /// `set.Name` method.
    pub is_setter: bool,
}

/// The part of an entity that only properties have.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyDetail {
    /// The declared class, as written.
    #[serde(rename = "type")]
    pub type_: Option<String>,
    /// The dimensions as written.
    pub dimensions: Option<Vec<String>>,
    /// The validation functions as written.
    pub validators: Option<String>,
    /// The default value as written.
    pub default: Option<String>,
    /// The `get.Name` method of the property, if declared in the class.
    pub getter: Option<Box<Entity>>,
    /// The `set.Name` method of the property, if declared in the class.
    pub setter: Option<Box<Entity>>,
}

/// What an entity is, beyond its common fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Detail {
    /// No extra fields (folders, packages, class folders, scripts).
    None,
    /// A classdef class: its superclasses, as written.
    Class {
        /// The superclasses, in order.
        bases: Vec<String>,
    },
    /// A function or method.
    Function(FunctionDetail),
    /// A property.
    Property(PropertyDetail),
    /// An enumeration member: its value as written.
    Enumeration {
        /// The value expression, as written (`(1)` gives `1`).
        value: Option<String>,
    },
}

/// One entity of the MATLAB path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    /// The kind of entity.
    pub kind: Kind,
    /// The name: the file stem for files, the method or property name for members.
    pub name: String,
    /// The file that declares the entity (the folder for packages and class folders).
    /// Members inherit the file of their class.
    pub path: Option<PathBuf>,
    /// First line (1-based). `None` when the entity has no span (maxx's convention for
    /// folders, packages, class folders and scripts).
    pub lineno: Option<usize>,
    /// Last line (1-based, inclusive).
    pub endlineno: Option<usize>,
    /// The comment block next to the declaration (or the file header, see the README).
    pub docstring: Option<Docstring>,
    /// The attributes, sorted, as `Name` or `Name=value` (for example `Access=private`).
    pub attributes: BTreeSet<String>,
    /// The kind-specific fields.
    pub detail: Detail,
    /// The members: properties, methods, enumeration members, local functions, and
    /// for packages and class folders their files and sub-packages.
    pub members: BTreeMap<String, Entity>,
}

impl Entity {
    /// A new entity with only the common fields set.
    pub fn new(kind: Kind, name: &str) -> Self {
        Entity {
            kind,
            name: name.to_string(),
            path: None,
            lineno: None,
            endlineno: None,
            docstring: None,
            attributes: BTreeSet::new(),
            detail: Detail::None,
            members: BTreeMap::new(),
        }
    }

    /// The function detail, if this entity is a function or method.
    pub fn function(&self) -> Option<&FunctionDetail> {
        match &self.detail {
            Detail::Function(f) => Some(f),
            _ => None,
        }
    }

    /// The property detail, if this entity is a property.
    pub fn property(&self) -> Option<&PropertyDetail> {
        match &self.detail {
            Detail::Property(p) => Some(p),
            _ => None,
        }
    }

    /// The superclasses, if this entity is a class.
    pub fn bases(&self) -> &[String] {
        match &self.detail {
            Detail::Class { bases } => bases,
            _ => &[],
        }
    }
}
