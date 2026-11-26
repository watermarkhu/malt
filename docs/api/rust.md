---
icon: lucide/box
---

# Rust API

The crate is [`malt-index`](https://crates.io/crates/malt-index) and the library is `malt_index`. The full item
documentation is on [docs.rs](https://docs.rs/malt-index). This page is the tour.

## The index

```rust
use malt_index::{Index, ParserConfig, parse_file, parse_source, parse_live_file, Entity, Expr};

let index = Index::scan(&[PathBuf::from("src")], &ParserConfig::default());
let fun: Option<Entity> = index.get("pkg.sub.fn")?;      // functions, classes, `+pkg.Cls`
let method = index.get("ClassFolder.method")?;            // class folder methods
let local = index.get_from("helper", Path::new("src"))?;  // siblings and private/ first
let names = index.member_names();                         // top-level names
let paths = index.contributor_paths("fn");                // the files (folders) of a top-level name, in precedence order
let all = index.members()?;                               // every top-level entity
let folder = index.folder("src/sub")?;                    // a folder on the path, as an entity
let by_path = index.get_path("src/sub/fn.m")?;            // a file, or a folder, by path
index.addpath("lib", false, /* recursive */ true)?;      // front of the path
index.rmpath("lib", true);                                // with the folders below it
let file = parse_file(Path::new("src/x.m"), &ParserConfig::default())?;
let live = parse_live_file(Path::new("demo.mlx"))?;       // a live script
let chain = index.mro(&class)?;                           // C3 order, the class first
let inherited = index.inherited_members(&class)?;         // not the class's own members
```

- `Index::scan(paths, config)` lists the folders up front and reads no file. Paths that cannot be read are skipped,
  logged as a warning, and listed by `index.errors()`.
- Files are parsed on first use and the result is cached (`OnceLock`), so a lookup costs one parse per file it needs.
  `members()` and the dump parse everything.
- An `Index` is `Send + Sync`: share it with `Arc`.
- `Sources` (`malt_index::Sources`) reads the source lines of files, once per path, and caches them. It is `Send + Sync`
  too.

## Path configuration

The search path can be built from `.malt.toml` rather than listed by hand. `enabled` in the same `[index]` table turns
the index on for the linter. See [Configuration](../configuration.md#index-section) for the table itself.

```rust
use malt_index::{Index, ParserConfig, PathConfig, resolve_paths};

let config = PathConfig::load(&cwd)?;                     // [index] paths, or empty
let (dirs, warnings) = resolve_paths(&config, &cwd, Some(file), &[]);
let (index, warnings) = Index::from_config(&config, &cwd, Some(file), &[], &ParserConfig::default());
```

- The default path is the working directory. `paths` replaces it. The linted file's folder and the extra folders always
  come first.
- `lib/**` is genpath: the folder and its plain subfolders, recursively. Other patterns (`lib`, `src/*`, `lib?`, `[ab]x`,
  `lib/**/deeper`) match folders without recursion. Relative patterns resolve against the working directory.
- `+pkg`, `@cls`, `private` and hidden folders are never on the path, nor anything below them. Naming one explicitly is a
  warning.
- The result is deduplicated and sorted: the file's folder, the extra folders in the order given, then the configured
  folders by path. Missing folders and unmatched patterns are warnings in the returned `Vec<String>`. They never panic.
  A `.malt.toml` that is not valid TOML, or an invalid `[index]` table, is `IndexError::Config`.
- Globs are matched without the `globset` or `walkdir` crates. Symbolic links are followed when a pattern names them, but
  not while recursing.

## Entities

`Entity` has `kind`, `name`, `path`, `lineno`, `endlineno`, `docstring`, `attributes` (a sorted set), `detail`
(kind-specific) and `members` (a `BTreeMap`).

| Kind | Name | Detail | Members |
| ---- | ---- | ------ | ------- |
| `Function` | file stem (the file's own function), or the identifier (methods, local and nested functions, getters and setters) | `Function(FunctionDetail { arguments, returns, is_getter, is_setter })` | local and nested functions |
| `Class` | file stem | `Class { bases: Vec<Expr>, events: Vec<String>, declares_constructor: bool }` | properties, methods, enumeration members |
| `Property` | identifier | `Property(PropertyDetail { type, dimensions, validators, default, getter, setter })` | none |
| `Enumeration` | identifier | `Enumeration { value: Option<Expr> }` | none |
| `Script` | file stem | `None` | local functions |
| `LiveScript` | file stem | `LiveScript { sections: Vec<LiveSection> }` | none |
| `Namespace` | `pkg` (the `+` is in the top-level key) | `None` | files and sub-packages |
| `ClassFolder` | `cls` | `None` | the class file's members and the folder's methods |
| `Folder` | the folder's name | `None` | its files, packages, class folders and subfolders |

`Argument { name, kind, type, dimensions, validators, default, docstring }`, with `kind` one of `positional-only`,
`optional`, `keyword-only`, `varargin`.

`Expr { text, lineno, column, endlineno, endcolumn }` is the raw text of a type, default, validator, base or enumeration
value, with its span (lines 1-based, columns 0-based in characters). It derefs to its text (`&*expr == "double"`),
compares with strings, and is written as its text in the dump. Its text joins the tokens with nothing between them
(`uint8 scalar` reads as `uint8scalar`).

`LiveSection { kind, content, lineno, endlineno }`: `kind` is `Text` or `Code`. Text `content` is the paragraphs' text
with the markers removed, and code `content` is the lines as written. `lineno` and `endlineno` are `None` for a `.mlx`,
whose paragraphs have no lines.

## Logging

The core logs through the `log` facade: `debug` for each file parsed, `info` for each live script, and `warn` for
recovered syntax errors, unreadable paths and an unreadable `document.xml`. The core does not depend on Python. The
Python module forwards the records to the `malt_index` logger, see the [Python API](python.md#logging).

## The dump

`cargo run -p malt-index --example dump -- [--live] PATH...` prints the whole index as JSON, with keys sorted and indent
2. It is the format the golden snapshot and the [oracle comparison](maxx.md#oracle-workflow) use. The root is
`{ "root", "members" }`.

Every entity has `kind`, `name`, `lineno`, `endlineno`, `docstring` (a string or null), `attributes` (a list), `members`,
and `path` when the entity has a file (relative to the first `PATH`, absolute outside it). Extra keys by kind:

- `class`: `bases` (as text), `events` and `declares_constructor`.
- `function`: `arguments` and `returns` (each `{name, kind, type, dimensions, validators, default, docstring}`),
  `is_getter`, `is_setter`.
- `property`: `type`, `dimensions`, `validators`, `default`.
- `live_script`: `sections` (each `{kind, content, lineno, endlineno}`).

`kind` is one of `folder`, `namespace`, `class`, `enumeration`, `function`, `script`, `live_script`, `property`.
