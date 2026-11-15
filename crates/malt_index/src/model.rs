//! The data model: what the index knows about a MATLAB path entry.
//!
//! An [`Entity`] is a function, class, property, enumeration, script, live script,
//! package (`+pkg`), class folder (`@cls`) or plain folder. Entities form a tree
//! through [`Entity::members`]. Every field is a fact read from the source: nothing
//! is evaluated. Types, defaults, validators and bases are [`Expr`]s (raw text with
//! its span), and docstrings are the comment block as text (see [`Docstring`]).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use crate::expr::Expr;

/// The kind of an entity. Names follow the MATLAB concepts, not maxx's `Kind` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A plain folder on the path (or below one), with its files and folders.
    Folder,
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
    /// A live script: a `.mlx` file, or a plain-text live code `.m` file.
    LiveScript,
    /// A property of a class.
    Property,
}

impl Kind {
    /// The kind name used in the dump and by the Python objects.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Folder => "folder",
            Kind::Namespace => "namespace",
            Kind::ClassFolder | Kind::Class => "class",
            Kind::Enumeration => "enumeration",
            Kind::Function => "function",
            Kind::Script => "script",
            Kind::LiveScript => "live_script",
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
    pub type_: Option<Expr>,
    /// The dimensions as written: numbers and `:` (spread operators).
    pub dimensions: Option<Vec<String>>,
    /// The validation functions as written, for example `{mustBePositive}`.
    pub validators: Option<Expr>,
    /// The default value as written.
    pub default: Option<Expr>,
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
    pub type_: Option<Expr>,
    /// The dimensions as written.
    pub dimensions: Option<Vec<String>>,
    /// The validation functions as written.
    pub validators: Option<Expr>,
    /// The default value as written.
    pub default: Option<Expr>,
    /// The `get.Name` method of the property, if declared in the class.
    pub getter: Option<Box<Entity>>,
    /// The `set.Name` method of the property, if declared in the class.
    pub setter: Option<Box<Entity>>,
}

/// The kind of a live script section: prose, or code (with its outputs left out).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionKind {
    /// Formatted text (a `%[text]` paragraph, or a paragraph in a `.mlx`).
    Text,
    /// Code.
    Code,
}

impl SectionKind {
    /// The kind name used by the Python objects: `text` or `code`.
    pub fn as_str(self) -> &'static str {
        match self {
            SectionKind::Text => "text",
            SectionKind::Code => "code",
        }
    }
}

/// One run of a live script: consecutive text paragraphs, or consecutive code lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveSection {
    /// Text or code.
    pub kind: SectionKind,
    /// The text of the run: the paragraphs' text joined by newlines (markers removed),
    /// or the code lines as written.
    pub content: String,
    /// First line (1-based). `None` for a `.mlx`, whose paragraphs have no lines.
    pub lineno: Option<usize>,
    /// Last line (1-based, inclusive). `None` for a `.mlx`.
    pub endlineno: Option<usize>,
}

/// What an entity is, beyond its common fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Detail {
    /// No extra fields (folders, packages, class folders, scripts).
    None,
    /// A classdef class: its superclasses, as written.
    Class {
        /// The superclasses, in order.
        bases: Vec<Expr>,
    },
    /// A function or method.
    Function(FunctionDetail),
    /// A property.
    Property(PropertyDetail),
    /// An enumeration member: its value as written.
    Enumeration {
        /// The value expression, as written (`(1)` gives `1`).
        value: Option<Expr>,
    },
    /// A live script: its sections, in order.
    LiveScript {
        /// The runs of text and code, in the order of the file.
        sections: Vec<LiveSection>,
    },
}

/// One entity of the MATLAB path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    /// The kind of entity.
    pub kind: Kind,
    /// The name: the file stem for files, the method or property name for members.
    pub name: String,
    /// The file that declares the entity (the folder for packages, class folders and
    /// plain folders). Members inherit the file of their class.
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
    /// for packages, class folders and plain folders their files and sub-packages.
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
    pub fn bases(&self) -> &[Expr] {
        match &self.detail {
            Detail::Class { bases } => bases,
            _ => &[],
        }
    }

    /// The sections, if this entity is a live script.
    pub fn sections(&self) -> &[LiveSection] {
        match &self.detail {
            Detail::LiveScript { sections } => sections,
            _ => &[],
        }
    }

    /// The enumeration value, if this entity is an enumeration member with one.
    pub fn enumeration_value(&self) -> Option<&Expr> {
        match &self.detail {
            Detail::Enumeration { value } => value.as_ref(),
            _ => None,
        }
    }
}
