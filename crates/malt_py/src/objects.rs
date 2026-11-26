//! The objects a collection hands out: `Object` and its subclasses (`Folder`, `Namespace`,
//! `Script`, `LiveScript`, `Class`, `Function`, `Property`, `Enumeration`), `Argument`,
//! `LiveScriptSection` and `Expr`.
//!
//! Each object keeps a copy of its entity from the index, its dotted path, the collection
//! it came from (the hierarchy and the source lines are looked up there) and its parent
//! object. Nothing is evaluated: every value is a fact the index read from the source.

use std::path::{Component, Path};
use std::sync::Arc;

use malt_index::{
    Argument as IndexArgument, ArgumentKind, Detail, Entity, Expr as IndexExpr, Kind, LiveSection,
    Sources,
};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use crate::collection::PathsCollection;
use crate::{cyclic_alias_error, index_error};

/// A named thing on the search path. `kind` tells which subclass it is.
///
/// `is_alias` is true for a member a class inherits (its object is a copy of the base's
/// member); `target` is the object itself, so `target` works for every object.
#[pyclass(module = "malt_index", subclass, frozen)]
pub struct Object {
    entity: Entity,
    path: String,
    collection: Py<PathsCollection>,
    parent: Option<Py<PyAny>>,
    alias: bool,
}

#[pymethods]
impl Object {
    /// The kind: `folder`, `namespace`, `class`, `function`, `script`, `live_script`,
    /// `property` or `enumeration` (the strings of the `Kind` enum).
    #[getter]
    fn kind(&self) -> &'static str {
        self.entity.kind.as_str()
    }

    /// The name: the file's name, or the identifier of a member.
    #[getter]
    fn name(&self) -> &str {
        &self.entity.name
    }

    /// The dotted identifier, as `get` takes it (`pkg.sub.fn`, `Cls.method`); a folder is
    /// `/name`.
    #[getter]
    fn path(&self) -> &str {
        &self.path
    }

    /// The file that declares the object (a folder for a folder, a package or a class
    /// folder), or `None`.
    #[getter]
    fn filepath(&self) -> Option<String> {
        self.entity.path.as_ref().map(|p| p.display().to_string())
    }

    /// The first line of the declaration (1-based), or `None` when it has no span.
    #[getter]
    fn lineno(&self) -> Option<usize> {
        self.entity.lineno
    }

    /// The last line of the declaration (1-based, inclusive), or `None`.
    #[getter]
    fn endlineno(&self) -> Option<usize> {
        self.entity.endlineno
    }

    /// The comment text next to the declaration: the comment markers removed and the
    /// indentation taken off, with no section parsing (griffe and the like do that).
    #[getter]
    fn docstring(&self) -> Option<String> {
        self.entity.docstring.as_ref().map(|d| d.value.clone())
    }

    /// The exact source text of the comment block, markers included.
    #[getter]
    fn docstring_raw(&self) -> Option<String> {
        self.entity.docstring.as_ref().map(|d| d.raw.clone())
    }

    /// The first line of the comment block (1-based), or `None` without a docstring.
    #[getter]
    fn docstring_lineno(&self) -> Option<usize> {
        self.entity.docstring.as_ref().map(|d| d.lineno)
    }

    /// The last line of the comment block (1-based, inclusive), or `None` without a docstring.
    #[getter]
    fn docstring_endlineno(&self) -> Option<usize> {
        self.entity.docstring.as_ref().map(|d| d.endlineno)
    }

    /// The attributes, sorted, as `Name` or `Name=value` (for example `Access=private`).
    #[getter]
    fn attributes(&self) -> Vec<String> {
        self.entity.attributes.iter().cloned().collect()
    }

    /// The access level: the `Access` attribute's value, else `public`.
    #[getter]
    fn access(&self) -> String {
        self.entity
            .attributes
            .iter()
            .find_map(|a| a.strip_prefix("Access=").map(str::to_string))
            .unwrap_or_else(|| "public".to_string())
    }

    /// Whether the object is private: its file is in a `private` folder, or it has a
    /// non-public `Access` attribute (maxx's rule).
    #[getter]
    fn is_private(&self) -> bool {
        let in_private = self.entity.path.as_deref().is_some_and(in_private_folder);
        let access = self.access();
        in_private || !matches!(access.as_str(), "public" | "immutable")
    }

    /// Whether the object is hidden: its file is in a `+internal` package, or it has the
    /// `Hidden` attribute.
    #[getter]
    fn is_hidden(&self) -> bool {
        self.is_internal() || self.entity.attributes.contains("Hidden")
    }

    /// Whether this object is an alias: a member inherited from a base class.
    #[getter]
    fn is_alias(&self) -> bool {
        self.alias
    }

    /// The object this one stands for: itself (see the class docs).
    #[getter]
    fn target(slf: &Bound<'_, Self>) -> Py<PyAny> {
        slf.clone().into_any().unbind()
    }

    #[getter]
    fn is_namespace(&self) -> bool {
        self.entity.kind == Kind::Namespace
    }

    #[getter]
    fn is_folder(&self) -> bool {
        self.entity.kind == Kind::Folder
    }

    #[getter]
    fn is_class(&self) -> bool {
        matches!(self.entity.kind, Kind::Class | Kind::ClassFolder)
    }

    #[getter]
    fn is_function(&self) -> bool {
        self.entity.kind == Kind::Function
    }

    #[getter]
    fn is_script(&self) -> bool {
        self.entity.kind == Kind::Script
    }

    #[getter]
    fn is_live_script(&self) -> bool {
        self.entity.kind == Kind::LiveScript
    }

    #[getter]
    fn is_property(&self) -> bool {
        self.entity.kind == Kind::Property
    }

    #[getter]
    fn is_enumeration(&self) -> bool {
        self.entity.kind == Kind::Enumeration
    }

    /// The collection the object came from: its hierarchy and its source lines are there.
    #[getter]
    fn lines_collection(&self, py: Python<'_>) -> PyResult<Py<crate::lines::LinesCollection>> {
        crate::lines::wrap(py, self.sources(py))
    }

    /// The source lines of the declaration (`lineno` to `endlineno`), as read from its file.
    /// Empty when the object has no span or its file cannot be read.
    #[getter]
    fn lines(&self, py: Python<'_>) -> Vec<String> {
        let (Some(path), Some(first), Some(last)) =
            (&self.entity.path, self.entity.lineno, self.entity.endlineno)
        else {
            return Vec::new();
        };
        let Ok(lines) = self.sources(py).lines(path) else {
            return Vec::new();
        };
        if first == 0 || last < first {
            return Vec::new();
        }
        lines
            .get(first - 1..last)
            .map(<[String]>::to_vec)
            .unwrap_or_default()
    }

    /// The source of the declaration: `lines` joined, with the common indentation removed.
    #[getter]
    fn source(&self, py: Python<'_>) -> String {
        dedent(&self.lines(py).join("\n"))
    }

    /// The members, by name: properties, methods and enumeration members of a class; local
    /// and nested functions of a function; the files and sub-packages of a package or folder.
    #[getter]
    fn members<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        let py = slf.py();
        let this = slf.borrow();
        let parent: Py<PyAny> = slf.clone().into_any().unbind();
        let dict = PyDict::new(py);
        for (name, member) in &this.entity.members {
            let path = if this.entity.kind == Kind::Folder {
                format!("{}/{}", this.path.trim_end_matches('/'), name)
            } else {
                format!("{}.{}", this.path, name)
            };
            let object = wrap(
                py,
                member.clone(),
                path,
                this.collection.clone_ref(py),
                Some(parent.clone_ref(py)),
                false,
            )?;
            dict.set_item(name, object)?;
        }
        Ok(dict)
    }

    /// The object this one is a member of, or `None` at the top level.
    #[getter]
    fn parent(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.parent.as_ref().map(|p| p.clone_ref(py))
    }

    /// The dotted name without the folder it was found in: `path` of a file in a folder
    /// is `/folder/name`, `canonical_path` is `name`. Equal to `path` for a folder.
    #[getter]
    fn canonical_path(&self) -> &str {
        if self.entity.kind == Kind::Folder {
            &self.path
        } else {
            self.path.rsplit('/').next().unwrap_or(&self.path)
        }
    }

    /// Whether the file is in a `+internal` package.
    #[getter]
    fn is_internal(&self) -> bool {
        self.entity
            .path
            .as_deref()
            .is_some_and(|p| path_has_part(p, "+internal"))
    }

    /// The members and, for a class, the inherited members before them, by name.
    #[getter]
    fn all_members<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        let py = slf.py();
        let all = PyDict::new(py);
        if matches!(slf.borrow().entity.kind, Kind::Class | Kind::ClassFolder) {
            all.update(
                slf.getattr("inherited_members")?
                    .cast::<PyDict>()?
                    .as_mapping(),
            )?;
        }
        all.update(slf.getattr("members")?.cast::<PyDict>()?.as_mapping())?;
        Ok(all)
    }

    /// The folders among `all_members`.
    #[getter]
    fn folders<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        members_of_kind(slf, "folder")
    }

    /// The packages among `all_members`.
    #[getter]
    fn namespaces<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        members_of_kind(slf, "namespace")
    }

    /// The scripts among `all_members`.
    #[getter]
    fn scripts<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        members_of_kind(slf, "script")
    }

    /// The classes among `all_members`.
    #[getter]
    fn classes<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        members_of_kind(slf, "class")
    }

    /// The functions (methods too) among `all_members`.
    #[getter]
    fn functions<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        members_of_kind(slf, "function")
    }

    /// The properties among `all_members`.
    #[getter]
    fn properties<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        members_of_kind(slf, "property")
    }

    /// The enumeration members among `all_members`.
    #[getter]
    fn enumerations<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        members_of_kind(slf, "enumeration")
    }
}

/// The entries of `all_members` whose kind is `kind`.
fn members_of_kind<'py>(slf: &Bound<'py, Object>, kind: &str) -> PyResult<Bound<'py, PyDict>> {
    let filtered = PyDict::new(slf.py());
    for (name, member) in slf.getattr("all_members")?.cast::<PyDict>()?.iter() {
        if member.getattr("kind")?.extract::<String>()? == kind {
            filtered.set_item(name, member)?;
        }
    }
    Ok(filtered)
}

impl Object {
    /// The source lines of the collection this object came from.
    fn sources(&self, py: Python<'_>) -> Arc<Sources> {
        self.collection.borrow(py).sources()
    }
}

/// A classdef, or a class folder (`@cls`).
#[pyclass(module = "malt_index", extends = Object, subclass, frozen)]
pub struct Class;

#[pymethods]
impl Class {
    /// The superclasses, as written in the classdef.
    #[getter]
    fn bases(slf: PyRef<'_, Self>) -> Vec<String> {
        slf.as_super()
            .entity
            .bases()
            .iter()
            .map(|b| b.text.clone())
            .collect()
    }

    /// The superclasses as expressions (text and span).
    #[getter]
    fn bases_expr(slf: PyRef<'_, Self>) -> Vec<Expr> {
        slf.as_super()
            .entity
            .bases()
            .iter()
            .cloned()
            .map(Expr::new)
            .collect()
    }

    /// The names of the entries of the class's `events` blocks, sorted. Names only: the
    /// attributes of an event are not recorded.
    #[getter]
    fn events(slf: PyRef<'_, Self>) -> Vec<String> {
        let mut names = slf.as_super().entity.events().to_vec();
        names.sort();
        names
    }

    /// Whether the class file declares a constructor (a method named as the class). An
    /// implicit constructor is not declared.
    #[getter]
    fn declares_constructor(slf: PyRef<'_, Self>) -> bool {
        slf.as_super().entity.declares_constructor()
    }

    /// The names of the classes in method resolution order (C3), this class first. A base
    /// that is not on the search path is listed by its name only. Raises `CyclicAliasError`
    /// when the superclasses lead back to the class.
    #[getter]
    fn mro(slf: &Bound<'_, Self>) -> PyResult<Vec<String>> {
        let py = slf.py();
        let this = slf.borrow();
        let object = this.as_super();
        let collection = object.collection.borrow(py);
        let ancestors = collection
            .index()
            .mro(&object.entity)
            .map_err(cyclic_or_index_error)?;
        Ok(ancestors.into_iter().map(|a| a.name).collect())
    }

    /// The members the class inherits, by name, from its bases in method resolution order.
    /// A member the class declares itself is not inherited. Each is an alias object whose
    /// `path` is `base.name`. Raises `CyclicAliasError` on a cycle of superclasses.
    #[getter]
    fn inherited_members<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        let py = slf.py();
        let this = slf.borrow();
        let object = this.as_super();
        let inherited = object
            .collection
            .borrow(py)
            .index()
            .inherited_members(&object.entity)
            .map_err(cyclic_or_index_error)?;
        let dict = PyDict::new(py);
        for item in inherited {
            let path = format!("{}.{}", item.from, item.name);
            let member = wrap(
                py,
                item.entity,
                path,
                object.collection.clone_ref(py),
                None,
                true,
            )?;
            dict.set_item(item.name, member)?;
        }
        Ok(dict)
    }

    /// The constructor: the member named as the class, when it is a function.
    #[getter]
    fn constructor(slf: &Bound<'_, Self>) -> PyResult<Option<Py<PyAny>>> {
        let name = slf.borrow().as_super().entity.name.clone();
        let members = slf.getattr("members")?;
        let Some(member) = members.cast::<PyDict>()?.get_item(&name)? else {
            return Ok(None);
        };
        let is_function = member.getattr("kind")?.extract::<String>()? == "function";
        Ok(is_function.then(|| member.unbind()))
    }

    /// The inputs of the constructor, or an empty list without one.
    #[getter]
    fn arguments(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        match Self::constructor(slf)? {
            Some(constructor) => Ok(constructor.bind(py).getattr("arguments")?.unbind()),
            None => Ok(pyo3::types::PyList::empty(py).into_any().unbind()),
        }
    }

    /// The superclasses found on the search path, in the order written; a base that is not
    /// on it is left out.
    #[getter]
    fn resolved_bases(slf: &Bound<'_, Self>) -> PyResult<Vec<Py<PyAny>>> {
        let py = slf.py();
        let bases = slf
            .borrow()
            .as_super()
            .entity
            .bases()
            .iter()
            .map(|b| b.text.clone())
            .collect::<Vec<_>>();
        let collection = slf.borrow().as_super().collection.clone_ref(py);
        let mut found = Vec::new();
        for base in bases {
            let object = collection.bind(py).call_method1("get", (base,))?;
            if !object.is_none() {
                found.push(object.unbind());
            }
        }
        Ok(found)
    }
}

/// A plain folder on the search path (or below one): its files, packages, class folders
/// and subfolders. Path lookups of `/name` give one.
#[pyclass(module = "malt_index", extends = Object, subclass, frozen)]
pub struct Folder;

/// A package (`+pkg`): its files and sub-packages.
#[pyclass(module = "malt_index", extends = Object, subclass, frozen)]
pub struct Namespace;

/// A script file (one without a leading function or classdef).
#[pyclass(module = "malt_index", extends = Object, subclass, frozen)]
pub struct Script;

/// A live script (`.mlx`, or plain-text live code): its sections, in order.
#[pyclass(module = "malt_index", extends = Object, subclass, frozen)]
pub struct LiveScript;

#[pymethods]
impl LiveScript {
    /// The sections: runs of text or code, in the order of the file.
    #[getter]
    fn sections(slf: PyRef<'_, Self>) -> Vec<LiveScriptSection> {
        slf.as_super()
            .entity
            .sections()
            .iter()
            .cloned()
            .map(LiveScriptSection::new)
            .collect()
    }
}

/// One run of a live script: `kind` is `text` or `code`, `content` the text (markers
/// removed) or the code lines as written.
#[pyclass(module = "malt_index", frozen)]
pub struct LiveScriptSection {
    inner: LiveSection,
}

impl LiveScriptSection {
    fn new(inner: LiveSection) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl LiveScriptSection {
    /// `text` or `code`.
    #[getter]
    fn kind(&self) -> &'static str {
        self.inner.kind.as_str()
    }

    #[getter]
    fn content(&self) -> &str {
        &self.inner.content
    }

    /// The first line (1-based), or `None` for a `.mlx` (its paragraphs have no lines).
    #[getter]
    fn lineno(&self) -> Option<usize> {
        self.inner.lineno
    }

    #[getter]
    fn endlineno(&self) -> Option<usize> {
        self.inner.endlineno
    }

    fn __repr__(&self) -> String {
        let preview: String = self.inner.content.chars().take(40).collect();
        format!(
            "LiveScriptSection(kind={:?}, content={:?})",
            self.inner.kind.as_str(),
            preview.replace('\n', "\\n")
        )
    }
}

/// A function, a method, a getter or setter, or a local or nested function.
#[pyclass(module = "malt_index", extends = Object, subclass, frozen)]
pub struct Function;

#[pymethods]
impl Function {
    /// The inputs, in signature order, then each `arguments` block's entries.
    #[getter]
    fn arguments(slf: PyRef<'_, Self>) -> Vec<Argument> {
        slf.as_super()
            .entity
            .function()
            .map(|f| f.arguments.iter().cloned().map(Argument::new).collect())
            .unwrap_or_default()
    }

    /// The outputs, in signature order.
    #[getter]
    fn returns(slf: PyRef<'_, Self>) -> Vec<Argument> {
        slf.as_super()
            .entity
            .function()
            .map(|f| f.returns.iter().cloned().map(Argument::new).collect())
            .unwrap_or_default()
    }

    /// Whether the function is a method: a member of a class.
    #[getter]
    fn is_method(slf: &Bound<'_, Self>) -> PyResult<bool> {
        let py = slf.py();
        match &slf.borrow().as_super().parent {
            Some(parent) => parent.bind(py).getattr("is_class")?.extract(),
            None => Ok(false),
        }
    }

    /// Whether the function is the constructor of its class: a method named as the class.
    #[getter]
    fn is_constructor_method(slf: &Bound<'_, Self>) -> PyResult<bool> {
        let py = slf.py();
        if !Self::is_method(slf)? {
            return Ok(false);
        }
        let this = slf.borrow();
        let Some(parent) = &this.as_super().parent else {
            return Ok(false);
        };
        let name: String = parent.bind(py).getattr("name")?.extract()?;
        Ok(this.as_super().entity.name == name)
    }

    /// Whether the function is a property getter (`get.Name`).
    #[getter]
    fn is_getter(slf: PyRef<'_, Self>) -> bool {
        slf.as_super()
            .entity
            .function()
            .is_some_and(|f| f.is_getter)
    }

    /// Whether the function is a property setter (`set.Name`).
    #[getter]
    fn is_setter(slf: PyRef<'_, Self>) -> bool {
        slf.as_super()
            .entity
            .function()
            .is_some_and(|f| f.is_setter)
    }
}

/// A property of a class. `getter` and `setter` are the `get.Name` and `set.Name` methods
/// the class declares for it (`None` when it has none).
#[pyclass(module = "malt_index", extends = Object, subclass, frozen)]
pub struct Property;

#[pymethods]
impl Property {
    /// The declared class, as written (`double`, `string`), or `None`.
    #[getter]
    #[pyo3(name = "type")]
    fn type_(slf: PyRef<'_, Self>) -> Option<String> {
        slf.as_super()
            .entity
            .property()
            .and_then(|p| p.type_.as_ref().map(|t| t.text.clone()))
    }

    /// The declared class as an expression (text and span), or `None`.
    #[getter]
    fn type_expr(slf: PyRef<'_, Self>) -> Option<Expr> {
        slf.as_super()
            .entity
            .property()
            .and_then(|p| p.type_.clone())
            .map(Expr::new)
    }

    /// The declared size, one entry per dimension, as written (`1`, `:`), or `None`.
    #[getter]
    fn dimensions(slf: PyRef<'_, Self>) -> Option<Vec<String>> {
        slf.as_super()
            .entity
            .property()
            .and_then(|p| p.dimensions.clone())
    }

    /// The validators, as written (`mustBeNumeric`), or `None`.
    #[getter]
    fn validators(slf: PyRef<'_, Self>) -> Option<String> {
        slf.as_super()
            .entity
            .property()
            .and_then(|p| p.validators.as_ref().map(|t| t.text.clone()))
    }

    #[getter]
    fn validators_expr(slf: PyRef<'_, Self>) -> Option<Expr> {
        slf.as_super()
            .entity
            .property()
            .and_then(|p| p.validators.clone())
            .map(Expr::new)
    }

    /// The default value, as written, or `None`.
    #[getter]
    fn default(slf: PyRef<'_, Self>) -> Option<String> {
        slf.as_super()
            .entity
            .property()
            .and_then(|p| p.default.as_ref().map(|t| t.text.clone()))
    }

    #[getter]
    fn default_expr(slf: PyRef<'_, Self>) -> Option<Expr> {
        slf.as_super()
            .entity
            .property()
            .and_then(|p| p.default.clone())
            .map(Expr::new)
    }

    /// The `get.Name` method of the property, or `None`.
    #[getter]
    fn getter(slf: &Bound<'_, Self>) -> PyResult<Option<Py<PyAny>>> {
        accessor(slf, true)
    }

    /// The `set.Name` method of the property, or `None`.
    #[getter]
    fn setter(slf: &Bound<'_, Self>) -> PyResult<Option<Py<PyAny>>> {
        accessor(slf, false)
    }
}

/// The getter or setter of a property, as a function object whose parent is the property.
fn accessor(slf: &Bound<'_, Property>, getter: bool) -> PyResult<Option<Py<PyAny>>> {
    let py = slf.py();
    let this = slf.borrow();
    let object = this.as_super();
    let Some(property) = object.entity.property() else {
        return Ok(None);
    };
    let method = if getter {
        property.getter.as_deref()
    } else {
        property.setter.as_deref()
    };
    let parent: Py<PyAny> = slf.clone().into_any().unbind();
    method
        .map(|m| {
            wrap(
                py,
                m.clone(),
                object.path.clone(),
                object.collection.clone_ref(py),
                Some(parent),
                false,
            )
        })
        .transpose()
}

/// An enumeration member of an `enumeration` block.
#[pyclass(module = "malt_index", extends = Object, subclass, frozen)]
pub struct Enumeration;

#[pymethods]
impl Enumeration {
    /// The value as written (`1`, `'red'`), or `None` when the member has none.
    #[getter]
    fn value(slf: PyRef<'_, Self>) -> Option<String> {
        slf.as_super()
            .entity
            .enumeration_value()
            .map(|v| v.text.clone())
    }

    /// The value as an expression (text and span), or `None`.
    #[getter]
    fn value_expr(slf: PyRef<'_, Self>) -> Option<Expr> {
        slf.as_super()
            .entity
            .enumeration_value()
            .cloned()
            .map(Expr::new)
    }
}

/// An argument of a function: an input, an output, or an `arguments` entry.
#[pyclass(module = "malt_index", frozen)]
pub struct Argument {
    inner: IndexArgument,
}

impl Argument {
    fn new(inner: IndexArgument) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl Argument {
    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    /// `positional-only`, `optional`, `keyword-only` or `varargin`.
    #[getter]
    fn kind(&self) -> &'static str {
        self.inner.kind.as_str()
    }

    /// Whether a call must pass the argument: passed by position, with no default.
    #[getter]
    fn required(&self) -> bool {
        matches!(self.inner.kind, ArgumentKind::PositionalOnly)
    }

    #[getter]
    #[pyo3(name = "type")]
    fn type_(&self) -> Option<String> {
        self.inner.type_.as_ref().map(|t| t.text.clone())
    }

    #[getter]
    fn type_expr(&self) -> Option<Expr> {
        self.inner.type_.clone().map(Expr::new)
    }

    #[getter]
    fn dimensions(&self) -> Option<Vec<String>> {
        self.inner.dimensions.clone()
    }

    #[getter]
    fn validators(&self) -> Option<String> {
        self.inner.validators.as_ref().map(|t| t.text.clone())
    }

    #[getter]
    fn validators_expr(&self) -> Option<Expr> {
        self.inner.validators.clone().map(Expr::new)
    }

    #[getter]
    fn default(&self) -> Option<String> {
        self.inner.default.as_ref().map(|t| t.text.clone())
    }

    #[getter]
    fn default_expr(&self) -> Option<Expr> {
        self.inner.default.clone().map(Expr::new)
    }

    /// The comment text, as `Object.docstring` (markers removed, no parsing).
    #[getter]
    fn docstring(&self) -> Option<String> {
        self.inner.docstring.as_ref().map(|d| d.value.clone())
    }

    /// The exact source text of the comment block.
    #[getter]
    fn docstring_raw(&self) -> Option<String> {
        self.inner.docstring.as_ref().map(|d| d.raw.clone())
    }
}

/// An expression: the source text of a type, default, validator or base, with its span.
///
/// It is raw text (not evaluated, not resolved). `str(expr)` and `expr.text` are the text;
/// `expr == "double"` compares the text. The lines are 1-based, the columns 0-based and
/// count characters.
#[pyclass(module = "malt_index", frozen)]
pub struct Expr {
    inner: IndexExpr,
}

impl Expr {
    /// Wrap an expression of the index.
    pub(crate) fn new(inner: IndexExpr) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl Expr {
    /// The text, as written.
    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    #[getter]
    fn lineno(&self) -> usize {
        self.inner.lineno
    }

    #[getter]
    fn column(&self) -> usize {
        self.inner.column
    }

    #[getter]
    fn endlineno(&self) -> usize {
        self.inner.endlineno
    }

    #[getter]
    fn endcolumn(&self) -> usize {
        self.inner.endcolumn
    }

    fn __str__(&self) -> &str {
        &self.inner.text
    }

    /// The text in pieces: identifiers, runs of blanks and single other characters, so a
    /// renderer can link each name on its own.
    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyIterator>> {
        pyo3::types::PyList::new(py, tokens(&self.inner.text))?.try_iter()
    }

    /// The URL of the MathWorks documentation of the first builtin name in the expression,
    /// or an empty string.
    #[getter]
    fn doc(&self) -> String {
        let builtins = builtins();
        tokens(&self.inner.text)
            .into_iter()
            .find_map(|token| builtins.get(token).map(|page| format!("{DOC_URL}/{page}")))
            .unwrap_or_default()
    }

    fn __repr__(&self) -> String {
        format!("Expr({:?})", self.inner.text)
    }

    /// Equal to another expression or a string with the same text.
    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(expr) = other.cast::<Expr>() {
            return expr.borrow().inner.text == self.inner.text;
        }
        other
            .extract::<String>()
            .is_ok_and(|s| s == self.inner.text)
    }

    /// The hash of the text, so that an expression and its string hash alike.
    fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        PyString::new(py, &self.inner.text).as_any().hash()
    }
}

/// The Python object for `entity`: the subclass its kind names. `path` is its dotted
/// identifier, `parent` the object it is a member of and `alias` whether it is inherited.
pub(crate) fn wrap(
    py: Python<'_>,
    entity: Entity,
    path: String,
    collection: Py<PathsCollection>,
    parent: Option<Py<PyAny>>,
    alias: bool,
) -> PyResult<Py<PyAny>> {
    let kind = entity.kind;
    let is_function = matches!(entity.detail, Detail::Function(_));
    let object = Object {
        entity,
        path,
        collection,
        parent,
        alias,
    };
    Ok(match kind {
        Kind::Folder => Py::new(py, (Folder, object))?.into_any(),
        Kind::Namespace => Py::new(py, (Namespace, object))?.into_any(),
        Kind::Script => Py::new(py, (Script, object))?.into_any(),
        Kind::LiveScript => Py::new(py, (LiveScript, object))?.into_any(),
        Kind::Class | Kind::ClassFolder => Py::new(py, (Class, object))?.into_any(),
        Kind::Function if is_function => Py::new(py, (Function, object))?.into_any(),
        Kind::Property => Py::new(py, (Property, object))?.into_any(),
        Kind::Enumeration => Py::new(py, (Enumeration, object))?.into_any(),
        Kind::Function => Py::new(py, object)?.into_any(),
    })
}

/// The MATLAB help the builtin pages are relative to.
const DOC_URL: &str = "https://www.mathworks.com/help/matlab";

/// Builtin name to its documentation page (the table is `matlab_builtins.json`).
fn builtins() -> &'static std::collections::HashMap<String, String> {
    static TABLE: std::sync::OnceLock<std::collections::HashMap<String, String>> =
        std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!("matlab_builtins.json")).unwrap_or_default()
    })
}

/// Split text into identifiers, runs of whitespace and single other characters.
fn tokens(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        let mut end = start + c.len_utf8();
        let same = |d: char| {
            if c.is_ascii_alphabetic() || c == '_' {
                d.is_ascii_alphanumeric() || d == '_'
            } else {
                c.is_whitespace() && d.is_whitespace()
            }
        };
        while let Some(&(i, d)) = chars.peek() {
            if !same(d) {
                break;
            }
            end = i + d.len_utf8();
            chars.next();
        }
        out.push(&text[start..end]);
    }
    out
}

/// Whether the file is directly in a `private` folder. Only the folder that holds the file
/// counts: a `private` further up (macOS temporary folders live under `/private`) does not
/// make the file private.
fn in_private_folder(path: &Path) -> bool {
    path.parent()
        .and_then(Path::file_name)
        .is_some_and(|name| name == "private")
}

/// Whether a path has a component called `part`.
fn path_has_part(path: &Path, part: &str) -> bool {
    path.components()
        .any(|c| matches!(c, Component::Normal(name) if name == part))
}

/// The text with the common indentation of its non-blank lines removed, as `textwrap.dedent`.
fn dedent(text: &str) -> String {
    let indent = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    text.lines()
        .map(|l| {
            if l.trim().is_empty() {
                ""
            } else {
                &l[indent.min(l.len())..]
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The error of a hierarchy lookup: a cycle is `CyclicAliasError`, others as usual.
fn cyclic_or_index_error(err: malt_index::IndexError) -> PyErr {
    match err {
        malt_index::IndexError::CyclicBase(_) => cyclic_alias_error(err),
        other => index_error(other),
    }
}
