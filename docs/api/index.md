---
icon: lucide/folder-tree
---

# Path index

`malt-index` is a native index of the MATLAB path. It lists the functions, classes, scripts, live scripts, packages
(`+pkg`), class folders (`@cls`), plain folders and `private/` folders that MATLAB resolves names against, and it reads
their signatures, arguments, properties, enumerations, attributes and docstrings. Nothing is evaluated: every value is a
fact read from the source.

It is one library with three faces:

| Face | Name | For |
| ---- | ---- | --- |
| Rust crate | [`malt-index`](https://crates.io/crates/malt-index) on crates.io | Rust programs; the linter itself |
| Python module | `malt_index`, shipped as [`malt-index`](https://pypi.org/project/malt-index/) on PyPI | documentation generators such as [mkdocstrings-matlab](https://github.com/watermarkhu/mkdocstrings-matlab) |
| Linter feature | the `[index]` table of `.malt.toml` | checks that need to know about other files, see [Configuration](../configuration.md#path-index) |

The crate does not depend on the linter crates. The Python module is a separate wheel from `malt` (the CLI), because a
PyO3 extension cannot share a wheel with the `malt` binary.

The Python library that used to do this job was called maxx (MATLAB Language Tools). Its name was always a placeholder
for malt, and malt-index replaces it. If you come from there, read [Coming from maxx](maxx.md).

## Install

::: code-group

```sh [Python]
pip install malt-index
```

```sh [Rust]
cargo add malt-index
```

:::

The Python package ships type stubs (`malt_index/malt_index.pyi`) and the `py.typed` marker, so type checkers see the
whole API.

## A first look

::: code-group

```python [Python]
from malt_index import PathsCollection

paths = PathsCollection(["src", "lib"], working_directory="/path/to/project")

fn = paths["pkg.fn"]                 # KeyError when the name is not on the path
fn.kind, fn.filepath, fn.lineno      # "function", ".../pkg/fn.m", 1
[a.name for a in fn.arguments]       # the inputs, then the `arguments` block
fn.docstring                         # comment text, markers removed, not parsed

cls = paths.get("pkg.Widget")        # None when nothing matches
cls.bases, cls.mro                   # as written; mro in C3 order
cls.inherited_members["Size"].type   # members of the base classes
```

```rust [Rust]
use malt_index::{Index, ParserConfig};

let index = Index::scan(&[PathBuf::from("src")], &ParserConfig::default());
let fun = index.get("pkg.sub.fn")?;          // functions, classes, `+pkg.Cls`
let local = index.get_from("helper", Path::new("src"))?;  // siblings and private/ first
let chain = index.mro(&class)?;               // C3 order, the class first
```

:::

## What it is built on

- Files are parsed with tree-sitter on first use, and the result is cached. A lookup costs one parse per file it needs.
  Listing a folder reads no file at all.
- Syntax errors never stop a parse: tree-sitter recovers and the rest of the file is read.
- An index is `Send + Sync`: share it with `Arc` in Rust. The Python module logs through Python's `logging`.

## Where to go next

- [Python API](python.md): every class, attribute and function of the `malt_index` module.
- [Rust API](rust.md): the `Index`, the entities, the path configuration and the JSON dump.
- [How MATLAB is read](behaviour.md): folders on the path, precedence, parsing rules, live scripts, class hierarchy and
  docstrings.
- [Coming from maxx](maxx.md): what changed for users of the earlier library.
- [Development](development.md): building, tests and the oracle workflow.
