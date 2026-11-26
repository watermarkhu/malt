---
icon: lucide/braces
---

# Python API

The module is `malt_index`; the distribution is `malt-index`. It needs Python 3.10 or later, and one wheel serves every
supported version (abi3). The API is typed: `malt_index/malt_index.pyi` ships in the wheel.

```python
from malt_index import PathsCollection, ParserConfig, parse_file, parse_source, parse_live_script
```

## The search path

`PathsCollection(paths, recursive=False, working_directory=None, parser_config=None, parse_live_scripts=False)` is the
MATLAB search path. `paths` are folders, relative to `working_directory`. A folder that cannot be read raises
`FilePathError`.

```python
paths = PathsCollection(
    ["src", "lib"],
    recursive=False,
    working_directory="/path/to/project",
    parser_config=ParserConfig(docstring_before_properties=True),
    parse_live_scripts=False,             # True: .mlx files on the path are live scripts
)
```

| Member | What it does |
| ------ | ------------ |
| `.members` | `{name: object}` of the top-level names. Package members are listed under dotted names (`pkg.fn`) and the methods of a class folder as `cls.method`. |
| `.get(identifier)`, `[identifier]`, `in` | The object, or `None`, `KeyError`, `False`. A path (with a `/`) gives a folder, or a file in one. |
| `.resolve(identifier, from_dir=None)` | Like `get`, but a plain name looks next to `from_dir` first. Raises `NameResolutionError`. |
| `.addpath(path, *, to_end=False, recursive=False)` | Adds a folder to the front of the path (the end with `to_end`). |
| `.rmpath(path, recursive=False)` | Removes a folder. Returns whether it was on the path. |
| `.search_path` | The folders, in precedence order. |
| `.lines_collection` | A [`LinesCollection`](#lines) with the source lines of every file. |

```python
fn = paths["pkg.fn"]
cls = paths.get("pkg.Widget")
paths["src/sub"]                          # a folder on the path
paths["src/sub/fn.m"]                     # a file in one
paths.resolve("helper", from_dir="src")   # looks in src/ and src/private first
```

## Parsing one file

| Function | Result |
| -------- | ------ |
| `parse_file(path, parser_config=None)` | The object of one file, with no parent. Raises `FilePathError`, or `ValueError` for a `.mlx` that is not one. |
| `parse_source(source, path, parser_config=None)` | The object of MATLAB text, named after `path`. The file is not read. |
| `parse_live_script(path, plaintext=False)` | A `LiveScript`: a `.mlx` by its contents, or a plain-text live `.m` with `plaintext=True`. |

## Objects

Everything the index finds is an `Object`. The subclass follows the kind.

### Object

| Attribute | Meaning |
| --------- | ------- |
| `kind`, `name` | The kind as a plain string equal to the matching `Kind` member, and the name. |
| `path` | The dotted identifier. A folder's is `/name`. |
| `canonical_path` | The dotted name without the folder it was found in. |
| `filepath`, `lineno`, `endlineno` | Where it is declared. Lines are 1-based, the end is inclusive. |
| `docstring`, `docstring_raw` | The comment text, and the block exactly as written. See [Docstrings](#docstrings). |
| `docstring_lineno`, `docstring_endlineno` | The span of the comment block. |
| `attributes`, `access` | The attributes of the declaration, and its `Access` value. |
| `is_private`, `is_hidden`, `is_internal` | `private` is a `private/` folder or a non-public access. `internal` is a file in a `+internal` package, the package itself included. `hidden` is internal or declared `Hidden`. |
| `is_alias`, `target` | True for a member inherited from a base class. Its `target` is the object itself. |
| `members`, `parent` | The members by name, and the object this one belongs to. |
| `all_members` | The members; for a class, the inherited members first. |
| `folders`, `namespaces`, `scripts`, `classes`, `functions`, `properties`, `enumerations` | The members of `all_members` of one kind. |
| `lines`, `source`, `lines_collection` | The declaration's lines, the same dedented, and the collection they come from. |
| `is_folder`, `is_namespace`, `is_class`, `is_function`, `is_script`, `is_live_script`, `is_property`, `is_enumeration` | Kind tests. |

### Folder, Namespace, Script

A plain folder, a package (`+pkg`) and a script. They add nothing to `Object`. A class folder (`@cls`) is a `Class`.

### Class

| Attribute | Meaning |
| --------- | ------- |
| `bases`, `bases_expr` | The superclasses as text, and as `Expr`s with their spans. |
| `resolved_bases` | The superclasses that are on the path. A base that is not on it is left out. |
| `mro` | The C3 order of the class and its bases. |
| `inherited_members` | The members of the bases, in `mro` order. A name the class declares is skipped. |
| `constructor`, `arguments` | The method named as the class, and its inputs. `None` and an empty list without one. |
| `events` | The names of the `events` entries, sorted. |
| `declares_constructor` | Whether a method named as the class is declared. |

A cycle of superclasses (`A < B`, `B < A`) raises `CyclicAliasError` from `mro` and `inherited_members`.

### Function

`arguments` and `returns` are lists of `Argument`. `is_method` is true for a member of a class, `is_constructor_method`
for the method named as the class, `is_getter` and `is_setter` for `get.X` and `set.X`.

### Property and Enumeration

| Attribute | Meaning |
| --------- | ------- |
| `type`, `dimensions`, `validators`, `default` | The declaration's parts, as text (`dimensions` is a list). |
| `type_expr`, `validators_expr`, `default_expr` | The same parts as `Expr`s. |
| `getter`, `setter` | `Function`s, with the property as their `parent`. |
| `value`, `value_expr` | On an `Enumeration` member: its value as written. |

### Argument

`name`, `kind`, `required`, `type`, `dimensions`, `validators`, `default`, `docstring`, `docstring_raw`, and the `*_expr`
versions of the type, validators and default.

`kind` is `positional-only`, `optional`, `keyword-only` or `varargin`. `required` is true for a positional input with no
default. `opts.Name` and `opts.?Cls.prop` are keyword arguments, and `~` is an argument named `~`.

### LiveScript

`sections` is a list of `LiveScriptSection` with `kind` (`text` or `code`), `content`, `lineno` and `endlineno`. See
[Live scripts](#live-scripts).

## Expressions

A type, default, validator or base is a `str` in `type`, `default` and `bases`. The matching `*_expr` attribute is an
`Expr`, for a consumer that needs the position.

| Member | Meaning |
| ------ | ------- |
| `text`, `str(expr)` | The expression as written. Its tokens join with nothing between them, so `uint8 scalar` reads as `uint8scalar`. |
| `lineno`, `column`, `endlineno`, `endcolumn` | The span. Lines are 1-based, columns 0-based in characters. |
| `iter(expr)` | The text in tokens: identifiers, runs of blanks and single characters. A renderer can link each name on its own. |
| `doc` | The MathWorks documentation URL of the first builtin name in the expression, or `""`. |
| `==`, `hash` | Compares with a string or another `Expr` by text, and hashes as the text. |

## Docstrings

`docstring` is the comment block next to a declaration with the comment markers removed. It is not parsed: no sections,
no Google or NumPy style. `docstring_raw` is the block exactly as written, markers included. Section parsing is for the
consumer, for example griffe in mkdocstrings-matlab.

Which comment block documents a declaration is set by `ParserConfig(docstring_before_properties,
docstring_before_arguments, docstring_before_enumerations, parse_live_scripts)`. The default is the block after it. The
rules are on [How MATLAB is read](behaviour.md#docstrings).

## Live scripts

`LiveScriptSection.kind` is `text` or `code`. A text section's `content` is the paragraphs' text with the markers
removed, and a code section's is the code lines as written. Line numbers are `None` for a `.mlx`. The sections stop at
`%[appendix]`, and outputs are not sections. A live script has no docstring.

## Lines

`LinesCollection` has `lines(path)`, `[path]`, `path in lines` and `keys()`. It reads a file once and caches it. Lines
are split on `\n` with a `\r` before it removed, and a final newline adds no empty line. An object's `lines` is its
declaration's span (`[]` without one), and `source` is those lines, dedented.

## Enumerations and errors

`Kind`, `ArgumentKind`, `AccessKind` and `SectionKind` are `str` enums. `Kind` has `folder`, `namespace`, `class`,
`enumeration`, `function`, `script`, `live_script` and `property`.

| Exception | Raised when |
| --------- | ----------- |
| `FilePathError` | A folder or file cannot be read. |
| `NameResolutionError` | `resolve` finds nothing. |
| `CyclicAliasError` | The superclasses of a class lead back to it. |

## Logging

The index logs through Python's `logging` logger `malt_index`, at the level the record has:

| Level | Records |
| ----- | ------- |
| `DEBUG` | Each file parsed. |
| `INFO` | Each live script. |
| `WARNING` | Recovered syntax errors, unreadable paths and an unreadable `document.xml`. |

Nothing is configured by the module. Set a handler or a level on the logger:

```python
import logging

logging.getLogger("malt_index").setLevel(logging.WARNING)
```

The bridge passes a record to Python as `logger.log(level, message)`, so Python's own filtering applies. It is installed
when the module is imported, and not installed if another Rust logger was set first.
