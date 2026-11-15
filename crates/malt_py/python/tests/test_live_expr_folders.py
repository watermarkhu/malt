"""Tests of the objects added on top of the path collection: expressions, enumerations,
folders, live scripts, lines, the object predicates, the logger and the errors.

Every project here is written under ``tmp_path``, so the tests never share files.
"""

from __future__ import annotations

import io
import logging
import zipfile
from pathlib import Path

import pytest
from malt_index import (
    AccessKind,
    ArgumentKind,
    Class,
    CyclicAliasError,
    Enumeration,
    Expr,
    FilePathError,
    Folder,
    Function,
    Kind,
    LinesCollection,
    LiveScript,
    LiveScriptSection,
    Namespace,
    ParserConfig,
    PathsCollection,
    Property,
    Script,
    SectionKind,
    parse_file,
    parse_live_script,
    parse_source,
)

DOCUMENT = (
    '<?xml version="1.0" encoding="UTF-8"?>'
    '<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">'
    "<w:body>"
    '<w:p><w:pPr><w:pStyle w:val="Title"/></w:pPr><w:r><w:t>Hello</w:t></w:r></w:p>'
    '<w:p><w:pPr><w:pStyle w:val="matlab-Code"/></w:pPr>'
    "<w:r><w:t>x = 1;</w:t></w:r></w:p>"
    '<w:p><w:pPr><w:pStyle w:val="matlab-Code"/></w:pPr>'
    "<w:r><w:t>y = 2;</w:t></w:r></w:p>"
    "<w:p><w:r><w:t>Tail</w:t></w:r></w:p>"
    "</w:body></w:document>"
)

TEXT_LIVE = (
    "%% Title\n"
    "%[text] # Head\n"
    "%[text] Some words.\n"
    "x = 1;\n"
    '%[output:abc]{"type":"text"}\n'
    '%   data: {"x":1}\n'
    "%---\n"
    '%[appendix]{"version":"1.0"}\n'
    "%---\n"
)


def write_mlx(path: Path, document: str = DOCUMENT) -> Path:
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w") as archive:
        archive.writestr("matlab/document.xml", document)
    path.write_bytes(buffer.getvalue())
    return path


@pytest.fixture
def project(tmp_path: Path) -> Path:
    (tmp_path / "Shape.m").write_text(
        "classdef Shape < handle & matlab.mixin.Copyable\n"
        "  properties (Access=private)\n"
        "    Units (1,:) char {mustBeMember(Units,{'m','cm'})} = 'm'\n"
        "  end\n"
        "  properties (Hidden)\n"
        "    Count double = 3\n"
        "  end\n"
        "  methods\n"
        "    function v = get.Count(obj)\n"
        "      v = 1;\n"
        "    end\n"
        "  end\n"
        "end\n"
    )
    (tmp_path / "Tint.m").write_text("classdef Tint < Shape\nend\n")
    (tmp_path / "Loop.m").write_text("classdef Loop < Loop\nend\n")
    (tmp_path / "Colour.m").write_text(
        "classdef Colour\n  enumeration\n    Red (1)\n    Blue\n  end\nend\n"
    )
    sub = tmp_path / "sub"
    sub.mkdir()
    (sub / "tool.m").write_text("function y = tool(x)\n% TOOL does it.\ny = x;\nend\n")
    (sub / "Contents.m").write_text("% Sub folder.\n")
    (tmp_path / "+pkg").mkdir()
    (tmp_path / "+pkg" / "Beta.m").write_text("function b = Beta()\nb = 2;\nend\n")
    write_mlx(tmp_path / "Shot.mlx")
    (tmp_path / "live.m").write_text(TEXT_LIVE)
    return tmp_path


def test_expressions_are_text_with_a_span(project: Path) -> None:
    shape = parse_file(project / "Shape.m")
    units = shape.members["Units"]
    assert isinstance(units, Property)
    assert isinstance(units.type_expr, Expr)
    assert (
        units.type == "char"
        and units.type_expr == "char"
        and str(units.type_expr) == "char"
    )
    assert units.type_expr.lineno == 3 and units.type_expr.column == 16
    assert units.validators_expr.text == "{mustBeMember(Units,{'m','cm'})}"
    assert units.validators_expr.column == 21
    assert units.default_expr.text == "'m'"
    assert hash(units.type_expr) == hash("char")
    assert repr(units.type_expr) == 'Expr("char")'
    # Plain strings stay available, and a bases expression keeps its span.
    assert shape.bases == ["handle", "matlab.mixin.Copyable"]
    assert [str(b) for b in shape.bases_expr] == ["handle", "matlab.mixin.Copyable"]
    assert shape.bases_expr[0].column == 17


def test_enumeration_members_have_values_as_expressions(project: Path) -> None:
    colour = parse_file(project / "Colour.m")
    red = colour.members["Red"]
    assert isinstance(red, Enumeration)
    assert red.kind == "enumeration"
    assert red.value == "1" and red.value_expr == "1"
    assert colour.members["Blue"].value is None


def test_kind_and_argument_enums_are_strings(project: Path) -> None:
    assert Kind.FOLDER == "folder"
    assert Kind.LIVE_SCRIPT.value == "live_script"
    assert isinstance(Kind.CLASS, str)
    assert ArgumentKind.varargin == "varargin"
    assert ArgumentKind.keyword_only == "keyword-only"
    assert AccessKind.private == "private"
    assert SectionKind.TEXT == "text" and SectionKind.CODE == "code"


def test_subclasses_are_python_classes(project: Path) -> None:
    assert isinstance(parse_file(project / "Shape.m"), Class)
    assert isinstance(parse_file(project / "Shot.mlx"), LiveScript)
    assert isinstance(parse_file(project / "Shot.mlx"), Script) is False
    script = parse_source("y = 1;\n", project / "notes.m")
    assert isinstance(script, Script) and script.kind == "script"
    paths = PathsCollection(["."], working_directory=project)
    assert isinstance(paths["pkg.Beta"], Function)
    assert isinstance(paths["Tint"], Class) and paths["Tint"].is_class


def test_parse_file_and_parse_source(project: Path) -> None:
    fn = parse_file(project / "sub" / "tool.m")
    assert isinstance(fn, Function) and fn.parent is None
    assert fn.docstring == "TOOL does it."
    assert [a.name for a in fn.arguments] == ["x"]
    from_text = parse_source("function y = f(x)\ny = x;\nend\n", project / "f.m")
    assert from_text.name == "f" and from_text.lineno == 1
    with pytest.raises(FilePathError):
        parse_file(project / "missing.m")


def test_a_cyclic_superclass_raises(project: Path) -> None:
    loop = parse_file(project / "Loop.m")
    with pytest.raises(CyclicAliasError):
        _ = loop.mro
    with pytest.raises(CyclicAliasError):
        _ = PathsCollection(["."], working_directory=project)["Loop"].inherited_members


def test_inherited_members_are_aliases_with_a_target(project: Path) -> None:
    paths = PathsCollection(["."], working_directory=project)
    tint = paths["Tint"]
    inherited = tint.inherited_members
    assert "Units" in inherited
    units = inherited["Units"]
    assert units.is_alias is True and units.target is units
    assert paths["Shape"].members["Units"].is_alias is False


def test_getters_are_linked_to_their_property(project: Path) -> None:
    shape = parse_file(project / "Shape.m")
    count = shape.members["Count"]
    assert count.getter is not None and count.getter.is_getter
    assert count.setter is None
    assert count.getter.parent is count


def test_predicates_for_access_and_hidden(project: Path) -> None:
    shape = parse_file(project / "Shape.m")
    units = shape.members["Units"]
    assert units.access == "private" and units.is_private and not units.is_hidden
    count = shape.members["Count"]
    assert count.is_hidden and not count.is_private
    assert count.is_property and not count.is_function


def test_folders_by_path(project: Path) -> None:
    paths = PathsCollection(["."], working_directory=project)
    # A folder is a path lookup, so the identifier has a slash (as in maxx).
    assert "sub" not in paths
    sub = paths["./sub"]
    assert isinstance(sub, Folder)
    assert sub.kind == "folder" and sub.is_folder
    assert sub.docstring == "Sub folder."
    assert sorted(sub.members) == ["tool"]
    tool = sub.members["tool"]
    assert tool.path == "/sub/tool" and isinstance(tool, Function)
    assert paths["sub/tool.m"].name == "tool"
    assert "nowhere" not in paths
    with pytest.raises(KeyError):
        paths["nowhere/x.m"]
    # A folder given by an absolute path works the same way.
    assert isinstance(paths[str(project / "sub")], Folder)


def test_live_scripts_are_on_the_path_only_with_the_flag(project: Path) -> None:
    off = PathsCollection(["."], working_directory=project)
    assert "Shot" not in off
    on = PathsCollection(["."], working_directory=project, parse_live_scripts=True)
    shot = on["Shot"]
    assert isinstance(shot, LiveScript) and shot.kind == "live_script"
    config = ParserConfig(parse_live_scripts=True)
    assert config.parse_live_scripts is True
    assert "Shot" in PathsCollection(
        ["."], working_directory=project, parser_config=config
    )


def test_live_script_sections_of_an_mlx_and_of_plain_text(project: Path) -> None:
    mlx = parse_live_script(project / "Shot.mlx")
    sections = mlx.sections
    assert [(s.kind, s.content) for s in sections] == [
        ("text", "Hello"),
        ("code", "x = 1;\ny = 2;"),
        ("text", "Tail"),
    ]
    assert all(isinstance(s, LiveScriptSection) for s in sections)
    assert sections[0].lineno is None

    text = parse_live_script(project / "live.m", plaintext=True)
    assert [(s.kind, s.content, s.lineno, s.endlineno) for s in text.sections] == [
        ("text", "# Head\nSome words.", 2, 3),
        ("code", "x = 1;", 4, 4),
    ]


def test_a_malformed_mlx_is_a_value_error(tmp_path: Path) -> None:
    bad = tmp_path / "Bad.mlx"
    bad.write_bytes(b"PK\x03\x04 not an archive")
    with pytest.raises(ValueError):
        parse_live_script(bad)
    truncated = tmp_path / "Cut.mlx"
    truncated.write_bytes(write_mlx(tmp_path / "Whole.mlx").read_bytes()[:40])
    with pytest.raises(ValueError):
        parse_live_script(truncated)


def test_lines_collection_reads_and_caches_source_lines(project: Path) -> None:
    paths = PathsCollection(["."], working_directory=project)
    lines = paths.lines_collection
    assert isinstance(lines, LinesCollection)
    path = str(project / "Colour.m")
    assert path not in lines
    text = lines[path]
    assert text[0] == "classdef Colour" and text[-1] == "end"
    assert path in lines and path in lines.keys()
    assert lines.lines(path) == text
    with pytest.raises(FilePathError):
        lines.lines(str(project / "missing.m"))


def test_object_lines_and_source_follow_the_span(project: Path) -> None:
    colour = parse_file(project / "Colour.m")
    red = colour.members["Red"]
    assert red.lines == ["    Red (1)"]
    assert colour.lines_collection is not None
    assert colour.source.startswith("classdef Colour")
    shot = parse_file(project / "Shot.mlx")
    assert shot.lines == []


def test_the_logger_forwards_parse_events(
    project: Path, caplog: pytest.LogCaptureFixture
) -> None:
    (project / "Broken.m").write_text("function y = f(x)\n  y = x + ;\nend\n")
    caplog.set_level(logging.DEBUG, logger="malt_index")
    parse_file(project / "Broken.m")
    records = [r for r in caplog.records if r.name == "malt_index"]
    assert any(
        r.levelno == logging.DEBUG and "parsing" in r.getMessage() for r in records
    )
    assert any(
        r.levelno == logging.WARNING and "syntax errors" in r.getMessage()
        for r in records
    )


def test_a_syntax_error_does_not_raise(project: Path) -> None:
    (project / "Odd.m").write_text("x = [1 2 ;\ny = (\n")
    assert parse_file(project / "Odd.m").name == "Odd"


def test_top_level_members_keep_their_python_classes(project: Path) -> None:
    paths = PathsCollection(["."], working_directory=project)
    assert isinstance(paths.members["+pkg"], Namespace)
    assert isinstance(paths.members["Colour"], Class)
    assert isinstance(paths.members["Tint"], Class)
