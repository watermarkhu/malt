# malt-index (Python)

The MATLAB search path as a Python module: the functions, classes, properties, arguments
and packages on the path, read by malt's native index (`crates/malt_index`). It is the
first part of the replacement for the Python library maxx, for `mkdocstrings-matlab`.
Nothing is evaluated: every value is a fact read from the source.

The distribution is `malt-index` and the module is `malt_index`. It is a separate wheel
from `malt` (the CLI): maturin builds one kind of artifact per project, so a PyO3 module
cannot share a wheel with the `malt` binary.

Licence: MIT. maxx is GPL-3.0; nothing here is copied from it (see malt-index's README).

## Use

```python
from malt_index import PathsCollection, ParserConfig

paths = PathsCollection(
    ["src", "lib"],                       # folders, relative to working_directory
    recursive=False,
    working_directory="/path/to/project",
    parser_config=ParserConfig(docstring_before_properties=True),
)

fn = paths["pkg.fn"]                      # KeyError when the name is not on the path
fn.kind, fn.filepath, fn.lineno           # "function", ".../pkg/fn.m", 1
[a.name for a in fn.arguments]            # inputs, then the `arguments` block
fn.docstring                              # comment text, markers removed, not parsed

cls = paths.get("pkg.Widget")             # None when nothing matches
cls.bases, cls.mro                        # as written; mro in C3 order
cls.inherited_members                     # {name: Object}, not the class's own members
cls.members["Size"].type                  # a Property

paths.addpath("extra", recursive=True)    # to the front, as MATLAB's addpath
paths.rmpath("extra", recursive=True)
paths.resolve("helper", from_dir="src")   # looks in src/ and src/private first
```

## API

| Name | What it is |
|---|---|
| `PathsCollection(paths, recursive=False, working_directory=None, parser_config=None)` | The search path. Raises `FilePathError` for a folder that cannot be read. |
| `.members` | `{name: object}`: the top-level names, as maxx's collection lists them. |
| `.get(identifier)`, `[identifier]`, `in` | The object, or `None` / `KeyError` / `False`. |
| `.resolve(identifier, from_dir=None)` | Like `get`, but a plain name looks next to `from_dir` first. Raises `NameResolutionError`. |
| `.addpath(path, *, to_end=False, recursive=False)`, `.rmpath(path, recursive=False)` | Change the path. `rmpath` returns whether the folder was on it. |
| `.search_path` | The folders, in precedence order. |
| `Object` | `kind`, `name`, `path` (dotted identifier), `filepath`, `lineno`, `endlineno`, `docstring`, `docstring_raw`, `attributes`, `members`, `parent`. |
| `Class(Object)` | `bases`, `mro`, `inherited_members`. |
| `Function(Object)` | `arguments`, `returns`, `is_getter`, `is_setter`. |
| `Property(Object)` | `type`, `dimensions`, `validators`, `default`. |
| `Argument` | `name`, `kind`, `required`, `type`, `dimensions`, `validators`, `default`, `docstring`, `docstring_raw`. |
| `ParserConfig(docstring_before_properties, docstring_before_arguments, docstring_before_enumerations)` | Which comment block documents a declaration (default: the one after, as maxx does). |
| `FilePathError`, `NameResolutionError`, `CyclicAliasError` | Exceptions. `CyclicAliasError` is never raised yet (no aliases). |

`kind` is a string: `namespace`, `class`, `function`, `script`, `property`, `enumeration`.
`Argument.kind` is `positional-only`, `optional`, `keyword-only` or `varargin`.
`Argument.required` is true for a positional input with no default.

### Docstrings

`docstring` is the comment block's text with the comment markers removed, as malt's
index keeps it. It is not parsed: no sections, no Google or NumPy style. `docstring_raw`
is the block exactly as written, markers included. The consumer (griffe in
mkdocstrings-matlab) parses the text.

## Not provided yet

Compared with maxx's surface, these are missing: `Expr` (expressions), live scripts
(`LiveScript`, `LiveScriptSection`, `LinesCollection`), aliases (`Alias`, so
`CyclicAliasError` is never raised), `Folder`, `Namespace` and `Script` as their own
classes (they are `Object` with a `kind`), the `Kind`, `AccessKind` and `ArgumentKind`
enums (plain strings here), `FileParser`, the logger configuration, and the property
getter and setter objects. Enumeration members are `Object` with kind `enumeration`
(maxx reports `class`).

## Build and test

```sh
pip install maturin
maturin build -m crates/malt_py/Cargo.toml --release -o dist    # the wheel
pip install dist/malt_index-*.whl pytest
pytest crates/malt_py/python/tests
```

`cargo test -p malt-py` checks the Rust side; the Python tests need the built wheel
(the module is a compiled extension). The sample project in `python/tests/project` is
written for these tests.
