//! The objects a collection hands out: `Object`, its subclasses `Class`, `Function` and
//! `Property`, and `Argument`.
//!
//! Each object keeps a copy of its entity from the index, its dotted path, the collection
//! it came from (the class hierarchy is looked up there) and its parent object. Nothing is
//! evaluated: every value is a fact the index read from the source.

use malt_index::{Argument as IndexArgument, ArgumentKind, Detail, Entity, Kind};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::collection::PathsCollection;
use crate::index_error;

/// A named thing on the search path: a namespace, script, class folder, enumeration, or
/// any entity without a more specific class. `kind` tells which (see `kind`).
#[pyclass(module = "malt_index", subclass, frozen)]
pub struct Object {
    entity: Entity,
    path: String,
    collection: Py<PathsCollection>,
    parent: Option<Py<PyAny>>,
}

#[pymethods]
impl Object {
    /// The kind: `namespace`, `class`, `function`, `script`, `property`, or `enumeration`.
    #[getter]
    fn kind(&self) -> &'static str {
        self.entity.kind.as_str()
    }

    /// The name: the file's name, or the identifier of a member.
    #[getter]
    fn name(&self) -> &str {
        &self.entity.name
    }

    /// The dotted identifier, as `get` takes it (`pkg.sub.fn`, `Cls.method`).
    #[getter]
    fn path(&self) -> &str {
        &self.path
    }

    /// The file that declares the object, or `None` (packages and folders have a folder).
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

    /// The attributes, sorted, as `Name` or `Name=value` (for example `Access=private`).
    #[getter]
    fn attributes(&self) -> Vec<String> {
        self.entity.attributes.iter().cloned().collect()
    }

    /// The members, by name: properties, methods and enumeration members of a class; local
    /// and nested functions of a function; the files and sub-packages of a package.
    #[getter]
    fn members<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        let py = slf.py();
        let this = slf.borrow();
        let parent: Py<PyAny> = slf.clone().into_any().unbind();
        let dict = PyDict::new(py);
        for (name, member) in &this.entity.members {
            let path = format!("{}.{}", this.path, name);
            let object = wrap(
                py,
                member.clone(),
                path,
                this.collection.clone_ref(py),
                Some(parent.clone_ref(py)),
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
}

/// A classdef, or a class folder (`@cls`).
#[pyclass(module = "malt_index", extends = Object, subclass, frozen)]
pub struct Class;

#[pymethods]
impl Class {
    /// The superclasses, as written in the classdef.
    #[getter]
    fn bases(slf: PyRef<'_, Self>) -> Vec<String> {
        slf.as_super().entity.bases().to_vec()
    }

    /// The names of the classes in method resolution order (C3), this class first. A base
    /// that is not on the search path is listed by its name only.
    #[getter]
    fn mro(slf: &Bound<'_, Self>) -> PyResult<Vec<String>> {
        let py = slf.py();
        let this = slf.borrow();
        let object = this.as_super();
        let collection = object.collection.borrow(py);
        let ancestors = collection
            .index()
            .mro(&object.entity)
            .map_err(index_error)?;
        Ok(ancestors.into_iter().map(|a| a.name).collect())
    }

    /// The members the class inherits, by name, from its bases in method resolution order.
    /// A member the class declares itself is not inherited. Each is an object whose `path`
    /// is `base.name`.
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
            .map_err(index_error)?;
        let dict = PyDict::new(py);
        for item in inherited {
            let path = format!("{}.{}", item.from, item.name);
            let member = wrap(py, item.entity, path, object.collection.clone_ref(py), None)?;
            dict.set_item(item.name, member)?;
        }
        Ok(dict)
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

/// A property of a class.
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
            .and_then(|p| p.type_.clone())
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
            .and_then(|p| p.validators.clone())
    }

    /// The default value, as written, or `None`.
    #[getter]
    fn default(slf: PyRef<'_, Self>) -> Option<String> {
        slf.as_super()
            .entity
            .property()
            .and_then(|p| p.default.clone())
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
        self.inner.type_.clone()
    }

    #[getter]
    fn dimensions(&self) -> Option<Vec<String>> {
        self.inner.dimensions.clone()
    }

    #[getter]
    fn validators(&self) -> Option<String> {
        self.inner.validators.clone()
    }

    #[getter]
    fn default(&self) -> Option<String> {
        self.inner.default.clone()
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

/// The Python object for `entity`: `Class` for a classdef or class folder, `Function`,
/// `Property`, and `Object` for the rest. `path` is its dotted identifier and `parent` the
/// object it is a member of.
pub(crate) fn wrap(
    py: Python<'_>,
    entity: Entity,
    path: String,
    collection: Py<PathsCollection>,
    parent: Option<Py<PyAny>>,
) -> PyResult<Py<PyAny>> {
    let kind = entity.kind;
    let has_function = matches!(entity.detail, Detail::Function(_));
    let object = Object {
        entity,
        path,
        collection,
        parent,
    };
    Ok(match kind {
        Kind::Class | Kind::ClassFolder => Py::new(py, (Class, object))?.into_any(),
        Kind::Function if has_function => Py::new(py, (Function, object))?.into_any(),
        Kind::Property => Py::new(py, (Property, object))?.into_any(),
        _ => Py::new(py, object)?.into_any(),
    })
}
