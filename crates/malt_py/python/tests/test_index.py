"""Tests for the ``malt_index`` module, run against the built wheel.

``project/`` is a small MATLAB project written for these tests. The golden test reads
the snapshot of malt-index's own sample project (``crates/malt_index/tests/golden``):
it is read from the repository, never copied.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import pytest
from malt_index import (
    Argument,
    Class,
    FilePathError,
    Function,
    NameResolutionError,
    ParserConfig,
    PathsCollection,
    Property,
    Script,
)

HERE = Path(__file__).resolve().parent
PROJECT = HERE / "project"
REPO = HERE.parents[3]
GOLDEN_PROJECT = REPO / "crates/malt_index/tests/project"
GOLDEN_JSON = REPO / "crates/malt_index/tests/golden/project.json"


@pytest.fixture
def paths() -> PathsCollection:
    return PathsCollection([PROJECT])


def test_members_are_the_top_level_names(paths: PathsCollection) -> None:
    names = set(paths.members)
    assert {
        "scale",
        "notes",
        "+shapes",
        "shapes.Shape",
        "shapes.Square",
        "Money",
    } <= names
    assert {"Money.plus"} <= names
    # `private` folders are never on the path.
    assert "hidden" not in names


def test_kinds_and_python_types(paths: PathsCollection) -> None:
    assert paths["scale"].kind == "function"
    assert isinstance(paths["scale"], Function)
    assert paths["shapes.Shape"].kind == "class"
    assert isinstance(paths["shapes.Shape"], Class)
    assert paths["Money"].kind == "class"
    assert paths["notes"].kind == "script"
    assert type(paths["notes"]) is Script
    assert paths.members["+shapes"].kind == "namespace"
    assert isinstance(paths["shapes.Shape"].members["Name"], Property)


def test_names_and_paths(paths: PathsCollection) -> None:
    square = paths["shapes.Square"]
    assert square.name == "Square"
    assert square.path == "shapes.Square"
    assert square.filepath is not None and square.filepath.endswith("Square.m")
    method = paths["Money.plus"]
    assert isinstance(method, Function)
    assert method.name == "plus"
    assert method.path == "Money.plus"


def test_function_arguments_and_returns(paths: PathsCollection) -> None:
    scale = paths["scale"]
    assert [a.name for a in scale.arguments] == ["x", "factor"]
    assert all(isinstance(a, Argument) for a in scale.arguments)
    assert [a.kind for a in scale.arguments] == ["positional-only", "optional"]
    x, factor = scale.arguments
    assert x.required is True
    assert factor.required is False
    assert factor.default == "2"
    assert x.type == "double"
    assert x.dimensions == ["1", "1"]
    assert [a.name for a in scale.returns] == ["y"]
    assert scale.is_getter is False and scale.is_setter is False
    assert scale.docstring is not None and scale.docstring.startswith(
        "SCALE Multiply x by a factor."
    )


def test_class_bases_mro_and_inherited_members(paths: PathsCollection) -> None:
    square = paths["shapes.Square"]
    assert square.bases == ["shapes.Shape"]
    # `handle` is not on the path: it is listed by name only.
    assert square.mro == ["Square", "shapes.Shape", "handle"]
    assert paths["shapes.Shape"].mro == ["Shape", "handle"]

    inherited = square.inherited_members
    assert {"Name", "Sides", "unit"} <= set(inherited)
    # `area` is overridden by Square, so it is its own member, not an inherited one.
    assert "area" not in inherited
    assert inherited["Name"].kind == "property"
    assert inherited["Name"].path == "shapes.Shape.Name"


def test_class_events_and_declared_constructor(tmp_path: Path) -> None:
    (tmp_path / "Sample.m").write_text(
        "classdef Sample < handle\n"
        "    events\n        Moved\n        Changed\n    end\n"
        "    methods\n        function obj = Sample()\n        end\n    end\n"
        "end\n",
        encoding="utf-8",
    )
    (tmp_path / "Plain.m").write_text("classdef Plain\nend\n", encoding="utf-8")
    collection = PathsCollection([tmp_path])
    sample = collection["Sample"]
    assert isinstance(sample, Class)
    assert sample.events == ["Changed", "Moved"]
    assert sample.declares_constructor is True
    plain = collection["Plain"]
    assert plain.events == []
    assert plain.declares_constructor is False


def test_class_folder_is_a_class(paths: PathsCollection) -> None:
    money = paths["Money"]
    assert money.kind == "class"
    assert money.bases == []
    assert money.mro == ["Money"]
    assert money.inherited_members == {}


def test_property_fields(paths: PathsCollection) -> None:
    name = paths["shapes.Shape"].members["Name"]
    assert name.type == "string"
    assert name.dimensions == ["1", "1"]
    assert name.validators is None
    assert name.default == '"shape"'


def test_docstrings_follow_maxx_placement(paths: PathsCollection) -> None:
    shape = paths["shapes.Shape"]
    assert shape.docstring == "Shape is the base of the planar shapes."
    # The raw block keeps its markers; the text does not.
    assert "% Shape is the base" in (shape.docstring_raw or "")
    assert "%" not in shape.docstring
    # By default a comment after a property documents the property before it.
    assert shape.members["Name"].docstring == "Number of sides."
    assert shape.members["Sides"].docstring is None


def test_docstring_span_lines(paths: PathsCollection) -> None:
    # Shape.m: the class comment is on line 2; the comment after `Name` is on line 7.
    shape = paths["shapes.Shape"]
    assert (shape.docstring_lineno, shape.docstring_endlineno) == (2, 2)
    assert shape.members["Name"].docstring_lineno == 7
    assert shape.members["Sides"].docstring_lineno is None
    assert shape.members["Sides"].docstring_endlineno is None


def test_parser_config_documents_before_properties() -> None:
    config = ParserConfig(docstring_before_properties=True)
    assert config.docstring_before_properties is True
    assert ParserConfig().docstring_before_arguments is False
    collection = PathsCollection([PROJECT], parser_config=config)
    members = collection["shapes.Shape"].members
    assert members["Name"].docstring == "Name of the shape."
    assert members["Sides"].docstring == "Number of sides."


def test_parent_links(paths: PathsCollection) -> None:
    shape = paths["shapes.Shape"]
    name = shape.members["Name"]
    assert name.parent is shape
    assert shape.parent is None
    assert paths.members["+shapes"].members["Square"].path == "shapes.Square"


def test_get_resolve_and_getitem(paths: PathsCollection) -> None:
    assert paths.get("nope") is None
    assert "nope" not in paths
    assert "scale" in paths
    with pytest.raises(KeyError):
        paths["nope"]
    with pytest.raises(NameResolutionError):
        paths.resolve("nope")
    with pytest.raises(NameResolutionError):
        paths.resolve("a..b")


def test_private_functions_resolve_from_their_folder(paths: PathsCollection) -> None:
    assert paths.get("hidden") is None
    with pytest.raises(NameResolutionError):
        paths.resolve("hidden")
    assert paths.resolve("hidden", from_dir=PROJECT).name == "hidden"


def test_relative_paths_use_the_working_directory() -> None:
    collection = PathsCollection(["project"], working_directory=HERE)
    assert "scale" in collection
    assert collection.search_path == [str(PROJECT.resolve())]


def test_addpath_rmpath_and_recursion(paths: PathsCollection, tmp_path: Path) -> None:
    lib = tmp_path / "lib"
    (lib / "sub").mkdir(parents=True)
    (lib / "tool.m").write_text("function y = tool(x)\ny = x;\nend\n")
    (lib / "sub" / "deep.m").write_text("function y = deep(x)\ny = x;\nend\n")

    paths.addpath(lib)
    assert paths.get("tool") is not None
    assert paths.get("deep") is None
    assert Path(paths.search_path[0]) == lib.resolve()

    # Adding again with recursion moves the folder and brings its subfolders in.
    paths.addpath(lib, recursive=True)
    assert paths.get("deep") is not None

    assert paths.rmpath(lib, recursive=True) is True
    assert paths.get("tool") is None
    assert paths.get("deep") is None
    assert paths.rmpath(lib) is False


def test_addpath_to_end_keeps_earlier_names(tmp_path: Path) -> None:
    (tmp_path / "scale.m").write_text("function y = scale(x)\ny = x;\nend\n")
    collection = PathsCollection([PROJECT])
    collection.addpath(tmp_path, to_end=True)
    # The project's own `scale` comes first and wins.
    assert collection["scale"].filepath == str(PROJECT.resolve() / "scale.m")


def test_missing_folder_raises_file_path_error(tmp_path: Path) -> None:
    with pytest.raises(FilePathError):
        PathsCollection([tmp_path / "missing"])
    collection = PathsCollection([])
    with pytest.raises(FilePathError):
        collection.addpath(tmp_path / "missing")


def _argument_plain(argument: Argument) -> dict[str, Any]:
    return {
        "name": argument.name,
        "kind": argument.kind,
        "type": argument.type,
        "dimensions": argument.dimensions,
        "validators": argument.validators,
        "default": argument.default,
        "docstring": argument.docstring,
    }


def _compare(obj: Any, golden: dict[str, Any], problems: list[str]) -> None:
    """Record each field of ``golden`` whose value differs from the object's."""
    for key, expected in golden.items():
        if key in ("path", "members"):
            continue
        if key in ("arguments", "returns"):
            actual: Any = [_argument_plain(a) for a in getattr(obj, key)]
        else:
            actual = getattr(obj, key)
        if actual != expected:
            problems.append(f"{obj.path}: {key}: {actual!r} != {expected!r}")
    members = golden.get("members", {})
    if set(members) != set(obj.members):
        problems.append(
            f"{obj.path}: members {sorted(obj.members)} != {sorted(members)}"
        )
    for name, child in members.items():
        if name in obj.members:
            _compare(obj.members[name], child, problems)


def test_golden_snapshot_of_malt_index_project_matches() -> None:
    """Every field of the snapshot matches the object model, for every entity.

    The fields are kinds, spans, attributes, bases, arguments, docstrings and members of
    malt-index's own sample project."""
    golden = json.loads(GOLDEN_JSON.read_text())
    collection = PathsCollection([GOLDEN_PROJECT])
    problems: list[str] = []
    if set(collection.members) != set(golden["members"]):
        problems.append("top-level names differ")
    for name, entry in golden["members"].items():
        if name in collection.members:
            _compare(collection.members[name], entry, problems)
    assert problems == []


def test_golden_facts_by_identifier() -> None:
    collection = PathsCollection([GOLDEN_PROJECT])
    widget = collection["Widget"]
    assert widget.bases == ["handle", "matlab.mixin.Copyable"]
    assert "Sealed" in widget.attributes
    make = widget.members["make"]
    assert make.attributes == ["Static"]
    assert [a.name for a in make.arguments] == ["x"]
    shapes = collection["Shapes"]
    assert [a.kind for a in shapes.arguments] == [
        "positional-only",
        "optional",
        "varargin",
        "keyword-only",
        "keyword-only",
    ]
    circle = collection["geo.shape.Circle"]
    assert circle.mro == ["Circle", "geo.Point", "handle"]
