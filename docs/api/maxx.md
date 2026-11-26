---
icon: lucide/git-compare
---

# Coming from maxx

maxx (MATLAB Language Tools) was the Python library that mkdocstrings-matlab used to read MATLAB source. The `xx` was a
placeholder for malt, and `malt-index` is its replacement: the same job, written in Rust, with the same logo. This page
is for people who used maxx directly. mkdocstrings-matlab moves to malt-index in its own release, so a user of the handler has nothing to change.

## Switching

```sh
pip uninstall maxx
pip install malt-index
```

```python
# before
from maxx.collection import PathsCollection
# after
from malt_index import PathsCollection
```

The names follow maxx's where the first API had them (`PathsCollection`, `Object`, `Class`, `Function`, `Property`,
`Argument`, `Expr`, `ParserConfig`, `parse_file`).

| maxx | malt_index |
| ---- | ---------- |
| `Alias` | An `Object` with `is_alias` and `target`. Only inherited members are aliases. |
| `FileParser`, `LiveScriptParser` | `parse_file`, `parse_source`, `parse_live_script` |
| `PathsCollection.get_member`, `get_path` | `PathsCollection.get`, and `[...]` with a path |
| `Object.has_docstring`, `is_kind`, `resolve`, `as_local_collection` | Not provided. Test `docstring`, `kind` and use `resolve` on the collection. |
| `logger.configure` | The `malt_index` logger of Python's `logging`, see [Logging](python.md#logging) |
| `Kind.ENUMERATION == "class"` | `enumeration` is its own kind, and members are `Enumeration` objects |

## Differences in what is read

Each difference is listed in `crates/malt_index/oracle_allowlist.json` with its reason. The main groups:

1. Bugs in maxx, not replicated. The function query read the first match only, so the inputs and outputs of a function
   with several outputs or a nested function were dropped or taken from the nested function. `~` was dropped. The
   docstring of a function with nested functions was lost. The `Dependant` misspelling dropped `Dependent`. Class
   attributes were not in `attributes`. The constructor of a class folder was popped from its members, and a class
   folder's script members were joined without a dot (`class2classbody`). A classdef with a parse error
   (`[1 2 3](end)`) was read as a script.
2. Records maxx did not have: local and nested functions, line spans of properties and enumeration members, the
   `varargin` kind, the `~` arguments, and line spans of live script sections (for a plain-text live script).
3. Bare names. maxx listed the methods of a class folder without a class file under the bare name (`inv`, for
   `@double/inv.m`). The index lists them as `double.inv`.
4. Encoding. maxx detected the encoding with charset-normalizer. The index uses UTF-8 and falls back to Latin-1, which
   is right for the Western files in the corpus.
5. Live scripts. maxx classified a whole `%%` cell as text when all its lines were comments, so a cell of `%[text]`
   paragraphs with code in it was one text section, and it kept the `%[text]` markers. The index splits a cell into
   runs, strips the markers, and leaves outputs and the appendix out. The `.mlx` sections agree with maxx exactly. On
   the real MATLAB-made `.mlx` files checked, MATLAB writes code paragraphs with the style `code`, and maxx read such a
   file as one text section. The index reads both `code` and `matlab-Code`, and the sections alternate text and code.
6. Folder members. maxx's folder listed only the subfolders it had registered on the path. A folder entity here lists
   every plain subfolder below it.
7. Source lines. maxx kept the `\r` of a CRLF file and an empty last line. The index keeps neither.

The table of builtin documentation pages behind `Expr.doc` comes from maxx.

## Oracle workflow

While malt-index was written, its output was compared with maxx's on the same paths, read-only and never copied. The
scripts are in `docs/scripts`: `maxx_dump.py` runs inside a maxx checkout, and `compare_index.py` diffs the two JSON
dumps against the allowlist.

```sh
cd ../maxx && uv run python ../malt/docs/scripts/maxx_dump.py PATH... > /tmp/maxx.json
cd ../malt && cargo run -q -p malt-index --example dump -- PATH... > /tmp/rust.json
python3 -I docs/scripts/compare_index.py --maxx /tmp/maxx.json --rust /tmp/rust.json \
    --project NAME      # exit 1 when a difference is not in the allowlist
```

Live scripts have two more modes, on both sides:

```sh
cd ../maxx && uv run python ../malt/docs/scripts/maxx_dump.py --live PATH... > maxx.json
cd ../malt && cargo run -q -p malt-index --example dump -- --live PATH... > rust.json
cd ../maxx && uv run python ../malt/docs/scripts/maxx_dump.py --plaintext DIR > maxx.json
cd ../malt && cargo run -q -p malt-index --example dump -- --plaintext DIR > rust.json
```

`--live` reads the `.mlx` files on the path. `--plaintext DIR` reads each `.m` file of DIR as plain-text live code, keyed
by stem. The sample for the `live-mlx` and `live-text` projects is in `crates/malt_index/tests/live_oracle`.

The report groups every difference by field, with the entity, its kind, span, attributes, bases, arguments, sections and
docstring, and a summary table. The allowlist entries match on `project`, `entity`, `field` and `detail` (globs), and
each has a `reason`. To add a project, list its paths in the same way the dump takes them. For a recursive scan that
means the root and its plain subfolders, not `+`, `@` or `private` folders.
