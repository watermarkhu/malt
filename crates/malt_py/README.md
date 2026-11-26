# malt-index (Python)

The MATLAB search path as a Python module: the functions, classes, properties, enumerations,
arguments, packages, folders and live scripts on the path, read by malt's native index
(`crates/malt_index`). It is the first part of the replacement for the Python library maxx,
for `mkdocstrings-matlab`. Nothing is evaluated: every value is a fact read from the source.

The distribution is `malt-index` and the module is `malt_index`. It is a separate wheel
from `malt` (the CLI): maturin builds one kind of artifact per project, so a PyO3 module
cannot share a wheel with the `malt` binary.

Licence: MIT. maxx is GPL-3.0; nothing here is copied from it (see malt-index's README).

The package ships type stubs (`malt_index/malt_index.pyi`) and the `py.typed` marker, so type
checkers see the API above.

## Use

```python
from malt_index import PathsCollection, ParserConfig, parse_file, parse_source, parse_live_script

paths = PathsCollection(
    ["src", "lib"],                       # folders, relative to working_directory
    recursive=False,
    working_directory="/path/to/project",
    parser_config=ParserConfig(docstring_before_properties=True),
    parse_live_scripts=False,             # True: .mlx files on the path are live scripts
)

fn = paths["pkg.fn"]                      # KeyError when the name is not on the path
fn.kind, fn.filepath, fn.lineno           # "function", ".../pkg/fn.m", 1
[a.name for a in fn.arguments]            # inputs, then the `arguments` block
fn.docstring                              # comment text, markers removed, not parsed

cls = paths.get("pkg.Widget")             # None when nothing matches
cls.bases, cls.mro                        # as written; mro in C3 order
cls.bases_expr[0].lineno                  # the base as an Expr, with its span
cls.inherited_members["Size"].is_alias    # True: a copy of the base's member
cls.members["Size"].type                  # a Property; .type_expr for the Expr

paths["src/sub"]                          # a folder on the path (a path has a slash)
paths["src/sub/fn.m"]                     # a file in one
paths.addpath("extra", recursive=True)
paths.rmpath("extra", recursive=True)
paths.resolve("helper", from_dir="src")   # looks in src/ and src/private first

paths.lines_collection["src/x.m"]         # the source lines of a file (cached)
paths["pkg.fn"].source                    # the declaration's lines, dedented

live = parse_live_script("demo.mlx")      # or parse_live_script("demo.m", plaintext=True)
[(s.kind, s.content) for s in live.sections]

obj = parse_file("src/x.m")               # the object of one file, no parent
obj = parse_source(text, "src/x.m")       # the same, from text (the file is not read)
```

## API

| Name | What it is |
|---|---|
| `PathsCollection(paths, recursive=False, working_directory=None, parser_config=None, parse_live_scripts=False)` | The search path. Raises `FilePathError` for a folder that cannot be read. |
| `.members` | `{name: object}`: the top-level names, as maxx's collection lists them. |
| `.get(identifier)`, `[identifier]`, `in` | The object, or `None` / `KeyError` / `False`. A path (with a `/`) gives a folder or a file in one. |
| `.resolve(identifier, from_dir=None)` | Like `get`, but a plain name looks next to `from_dir` first. Raises `NameResolutionError`. |
| `.addpath(path, *, to_end=False, recursive=False)`, `.rmpath(path, recursive=False)` | Change the path. `rmpath` returns whether the folder was on it. |
| `.search_path` | The folders, in precedence order. |
| `.lines_collection` | `LinesCollection`: `lines(path)`, `[path]`, `path in lines`, `keys()`. |
| `parse_file(path, parser_config=None)` | The object of one file (its parent is `None`). Raises `FilePathError`, or `ValueError` for a `.mlx` that is not one. |
| `parse_source(source, path, parser_config=None)` | The object of MATLAB text, named after `path`. The file is not read. |
| `parse_live_script(path, plaintext=False)` | A `LiveScript`: a `.mlx` by its contents, or a plain-text live `.m` with `plaintext=True`. |
| `Object` | `kind`, `name`, `path` (dotted identifier; a folder's `/name`), `filepath`, `lineno`, `endlineno`, `docstring`, `docstring_raw`, `docstring_lineno`, `docstring_endlineno`, `attributes`, `access`, `is_private`, `is_hidden`, `is_internal` (the file is in a `+internal` package, the package itself included), `canonical_path` (the dotted name without the folder it was found in), `is_alias`, `target`, `members`, `parent`, `lines`, `source`, `lines_collection`, `is_folder`, `is_namespace`, `is_class`, `is_function`, `is_script`, `is_live_script`, `is_property`, `is_enumeration`, and the member views `all_members` (the inherited members of a class first), `folders`, `namespaces`, `scripts`, `classes`, `functions`, `properties`, `enumerations`. |
| `Folder(Object)`, `Namespace(Object)`, `Script(Object)` | A plain folder, a package, a script. |
| `LiveScript(Object)` | `sections`: a list of `LiveScriptSection` (`kind`, `content`, `lineno`, `endlineno`). |
| `Class(Object)` | `bases` (text), `bases_expr` (`Expr`s), `events` (the names of the `events` entries, sorted), `declares_constructor` (a method named as the class is declared), `mro`, `inherited_members`, `constructor`, `arguments` (those of the constructor), `resolved_bases` (the superclasses found on the path). |
| `Function(Object)` | `arguments`, `returns`, `is_method`, `is_constructor_method`, `is_getter`, `is_setter`. |
| `Property(Object)` | `type`, `dimensions`, `validators`, `default`, and `type_expr`, `validators_expr`, `default_expr`; `getter` and `setter` (`Function`, with `parent` the property). |
| `Enumeration(Object)` | `value`, `value_expr`. |
| `Argument` | `name`, `kind`, `required`, `type`, `dimensions`, `validators`, `default`, `docstring`, `docstring_raw`, and the `*_expr` versions of the type, validators and default. |
| `Expr` | `text`, `lineno`, `column`, `endlineno`, `endcolumn`, `doc` (the MathWorks documentation URL of the first builtin name, or `""`). Iterating gives the text in tokens: identifiers, blanks and single characters. `str(expr)` is the text; `expr == "double"` compares the text (and `==` another `Expr` compares the text); hashes as its text. |
| `ParserConfig(docstring_before_properties, docstring_before_arguments, docstring_before_enumerations, parse_live_scripts)` | Which comment block documents a declaration (default: the one after, as maxx does), and whether `.mlx` files are live scripts. |
| `Kind`, `ArgumentKind`, `AccessKind`, `SectionKind` | `str` enums. `Kind` has `folder`, `namespace`, `class`, `enumeration`, `function`, `script`, `live_script`, `property` (the values of `Object.kind`). |
| `FilePathError`, `NameResolutionError`, `CyclicAliasError` | Exceptions. `CyclicAliasError` is raised when the superclasses of a class lead back to it (from `mro` and `inherited_members`). |

`Object.kind` is a plain string, equal to the matching `Kind` member. `Argument.kind` is
`positional-only`, `optional`, `keyword-only` or `varargin`. `Argument.required` is true for a
positional input with no default.

### Docstrings

`docstring` is the comment block's text with the comment markers removed, as malt's
index keeps it. It is not parsed: no sections, no Google or NumPy style. `docstring_raw`
is the block exactly as written, markers included. The consumer (griffe in
mkdocstrings-matlab) parses the text.

### Expressions

A type, default, validator or base is a `str` (`type`, `default`, `bases`, ...), the same as
before. The matching `*_expr` attribute is the `Expr`, with its span, for a consumer that
needs the position. Expressions are raw text: `Expr` does not map a name to a documentation
URL.

### Live scripts

`LiveScriptSection.kind` is `text` or `code`. A text section's `content` is the paragraphs'
text with the markers removed; a code section's is the code lines as written. Line numbers are
`None` for a `.mlx`. The sections stop at `%[appendix]`; outputs are not sections.

### Lines

`LinesCollection.lines(path)` returns the lines of a file, read once and cached. Lines are
split on `\n` with a `\r` before it removed; a final newline adds no empty line. An object's
`lines` is its declaration's span (`[]` without one); `source` is those lines, dedented.

### Logging

The index logs through Python's `logging` logger `malt_index`, at the levels the record has:
`DEBUG` for each file parsed, `INFO` for each live script, `WARNING` for recovered syntax
errors, unreadable paths and unreadable `document.xml`. Nothing is configured by the module:
set a handler or a level on the `malt_index` logger.

```python
import logging
logging.getLogger("malt_index").setLevel(logging.WARNING)
```

The bridge is a `log` logger in `malt-index`'s binding (`src/logging.rs`): a record is passed
to Python as `logger.log(level, message)`, so Python's own filtering applies. The bridge is
installed when the module is imported; it is not installed if another Rust logger was set first.

## Not provided yet

Compared with maxx's surface, these are missing: `Alias` (inherited members carry
`is_alias` and `target` instead; imports are not modelled), `FileParser` and
`LiveScriptParser` as classes (the functions above replace them), `PathsCollection.get_path`, `get_member`,
`as_local_collection`, `Object.resolve`, `has_docstring`, `is_kind`, and the
maxx `logger.configure` function, and the
`Kind.ENUMERATION == "class"` alias (malt says `enumeration`). Enumeration members are
`Enumeration` objects with kind `enumeration` (maxx reports `class`).

## Build and test

```sh
pip install maturin
maturin build -m crates/malt_py/Cargo.toml --release -o dist    # the wheel
pip install dist/malt_index-*.whl pytest
pytest crates/malt_py/python/tests
```

`cargo test -p malt-py` checks the Rust side; the Python tests need the built wheel
(the module is a compiled extension). The sample project in `python/tests/project` is
written for `test_index.py`; `test_live_expr_folders.py` writes its own projects under
`tmp_path`, including a `.mlx` built with `zipfile`.
