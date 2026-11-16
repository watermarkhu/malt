# malt-index

A native index of the MATLAB path: the functions, classes, scripts, live scripts, packages
(`+pkg`), class folders (`@cls`), plain folders and `private/` folders that MATLAB resolves
names against. It is the Rust core of the replacement for the Python library maxx. The Python
module is `crates/malt_py` (distribution `malt-index`). The linter uses the index only when `[index] enabled = true` or `--index` is given (off by default): see `docs/VERIFY.md`, 'Path index, stage 3.4', and `docs/configuration.md`.

The crate does not depend on the linter crates. It is `publish = false` for now.

Licence: maxx is GPL-3.0 and malt is MIT. The behaviour was read from maxx's sources; the
code here is written from scratch. No maxx code, tests, files or submodules are in this
repository.

## API

```rust
use malt_index::{Index, ParserConfig, parse_file, parse_source, parse_live_file, Entity, Expr};

let index = Index::scan(&[PathBuf::from("src")], &ParserConfig::default());
let fun: Option<Entity> = index.get("pkg.sub.fn")?;      // functions, classes, `+pkg.Cls`
let method = index.get("ClassFolder.method")?;            // class folder methods
let local = index.get_from("helper", Path::new("src"))?;  // siblings and private/ first
let names = index.member_names();                         // top-level names
let paths = index.contributor_paths("fn");                 // the files (folders) of a top-level name, in precedence order
let all = index.members()?;                               // every top-level entity
let folder = index.folder("src/sub")?;                    // a folder on the path, as an entity
let by_path = index.get_path("src/sub/fn.m")?;            // a file, or a folder, by path
index.addpath("lib", false, /* recursive */ true)?;      // front of the path
index.rmpath("lib", true);                                // with the folders below it
let file = parse_file(Path::new("src/x.m"), &ParserConfig::default())?;
let live = parse_live_file(Path::new("demo.mlx"))?;       // a live script
let chain = index.mro(&class)?;                            // C3 order, the class first
let inherited = index.inherited_members(&class)?;          // not the class's own members
```

- `Index::scan(paths, config)` lists the folders up front (no file is read). Paths that
  cannot be read are skipped, logged as a warning, and listed by `index.errors()`.
- Files are parsed on first use and the result is cached (`OnceLock`), so a lookup costs one
  parse per file it needs. `members()` and the dump parse everything.
- An `Index` is `Send + Sync`: share it with `Arc`.
- `Sources` (`malt_index::Sources`) reads the source lines of files, once per path, and
  caches them; it is `Send + Sync` too.

## Path configuration

The search path can be built from `.malt.toml` rather than listed by hand. `enabled` in the same `[index]` table turns the index on for the linter (default: off; see `docs/VERIFY.md`, 'Path index, stage 3.4'):

```rust
use malt_index::{Index, ParserConfig, PathConfig, resolve_paths};

let config = PathConfig::load(&cwd)?;                     // [index] paths, or empty
let (dirs, warnings) = resolve_paths(&config, &cwd, Some(file), &[]);
let (index, warnings) = Index::from_config(&config, &cwd, Some(file), &[], &ParserConfig::default());
```

- The default path is the working directory. `paths` (in the `[index]` table of the
  working directory's `.malt.toml`) replaces it. The linted file's folder and the extra
  folders always come first.
- `lib/**` is genpath: the folder and its plain subfolders, recursively. Other patterns
  (`lib`, `src/*`, `lib?`, `[ab]x`, `lib/**/deeper`) match folders without recursion.
  Relative patterns resolve against the working directory.
- `+pkg`, `@cls`, `private` and hidden folders are never on the path, nor anything below
  them. Naming one explicitly is a warning.
- The result is deduplicated and sorted (the file's folder, the extra folders in the
  order given, then the configured folders by path). Missing folders and unmatched patterns
  are warnings in the returned `Vec<String>`; they never panic. A `.malt.toml` that is not
  valid TOML, or an invalid `[index]` table, is `IndexError::Config`.
- Globs are matched without the `globset` or `walkdir` crates (neither is a dependency).
  Symbolic links are followed when a pattern names them, but not while recursing.

## Cross-check against MATLAB

The integration test `crates/malt_cli/tests/matlab_namespace.rs` compares this crate with MATLAB on the same folders (names, the file each resolves to, `which -all`, signatures, class members). It runs `matlab -batch` and skips without MATLAB; CI runs it, required. See `docs/VERIFY.md`, 'Cross-check against MATLAB'.

## Entities

`Entity` has `kind`, `name`, `path`, `lineno`, `endlineno`, `docstring`, `attributes`
(a sorted set), `detail` (kind-specific) and `members` (a `BTreeMap`).

| Kind | Name | Detail | Members |
|---|---|---|---|
| `Function` | file stem (the file's own function), or the identifier (methods, local and nested functions, getters/setters) | `Function(FunctionDetail { arguments, returns, is_getter, is_setter })` | local and nested functions |
| `Class` | file stem | `Class { bases: Vec<Expr> }` | properties, methods, enumeration members |
| `Property` | identifier | `Property(PropertyDetail { type, dimensions, validators, default, getter, setter })` | none |
| `Enumeration` | identifier | `Enumeration { value: Option<Expr> }` | none |
| `Script` | file stem | `None` | local functions |
| `LiveScript` | file stem | `LiveScript { sections: Vec<LiveSection> }` | none |
| `Namespace` | `pkg` (the `+` is in the top-level key) | `None` | files and sub-packages |
| `ClassFolder` | `cls` | `None` | the class file's members and the folder's methods |
| `Folder` | the folder's name | `None` | its files, packages, class folders and subfolders |

`Argument { name, kind, type, dimensions, validators, default, docstring }`, with `kind`
one of `positional-only`, `optional`, `keyword-only`, `varargin`.

`Expr { text, lineno, column, endlineno, endcolumn }` is the raw text of a type, default,
validator, base or enumeration value, with its span (lines 1-based, columns 0-based in
characters). It derefs to its text (`&*expr == "double"`), compares with strings and is
written as its text in the dump. Its text joins the tokens with nothing between them, as
maxx does (`uint8 scalar` reads as `uint8scalar`).

`LiveSection { kind, content, lineno, endlineno }`: `kind` is `Text` or `Code`. Text
`content` is the paragraphs' text (markers removed); code `content` is the lines as written.
`lineno` and `endlineno` are `None` for a `.mlx`, whose paragraphs have no lines.

## The dump (schema)

`cargo run -p malt-index --example dump -- [--live] PATH...` prints JSON in the schema of
`docs/scripts/maxx_dump.py`: keys sorted, indent 2. The root is `{ "root", "members" }`.
Every entity has `kind`, `name`, `lineno`, `endlineno`, `docstring` (a string or null),
`attributes` (a list), `members`, and `path` when the entity has a file (relative to the
first PATH, absolute outside it). Extra keys only by kind:

- `class`: `bases` (as text).
- `function`: `arguments`, `returns` (each `{name, kind, type, dimensions, validators,
  default, docstring}`), `is_getter`, `is_setter`.
- `property`: `type`, `dimensions`, `validators`, `default`.
- `live_script`: `sections` (each `{kind, content, lineno, endlineno}`).

`docstring` is the comment text (see below), not parsed. `kind` is one of `folder`,
`namespace`, `class`, `enumeration`, `function`, `script`, `live_script`, `property`.

## Behaviour

### Folders on the path

- A folder contributes its `.m` files, except `Contents.m`, which documents the folder.
  With `ParserConfig::parse_live_scripts`, its `.mlx` files are live scripts too.
- `+pkg`: a package. Its files and sub-packages are its members (`pkg.fn`, `pkg.sub.fn`).
  Class folders inside a package work too.
- `@cls`: a class folder. `cls.m` is the class file (a classdef, or an old-style constructor
  function); the other `.m` files are methods, named after their file.
- `private/`: visible only to the files of its parent folder (`get_from(name, dir)`).
  Never on the path itself. Private folders of subfolders are not on the path with recursion.
- `recursive`: plain subfolders are scanned too and their files join the top level, as
  MATLAB's `genpath` does. Their `+` and `@` folders join the top level as well.

### Plain folders and path lookups

- `folder(dir)` is the folder as an entity, if `dir` is on the path or below a folder on it
  (through plain folders only: not `+`, `@`, `private` or hidden folders). Its members are its
  files, packages and class folders, as the index reads them, and its plain subfolders, each
  a folder too. It is read with a throwaway index of the folder, so it does not change the
  parse cache of the search path, and a lookup parses the files again.
- `get_path(path)`: the folder (as `folder`), or the entity of a file in such a folder (its
  member of that stem). Anything else is `None`.

### Precedence

The first path (the front of the search path) wins a name. Within one path, names are
taken in sorted order. `get_from` looks at the folder's own files and its `private/` first.
When several paths define one package, the package is the union of its members, the
earlier path winning a name clash.

### Top-level names (`members()`)

As maxx's collection lists them: each top-level name (`fn`, `Cls`, `+pkg`, `cls`), and the
members of a package under dotted identifiers (`pkg.fn`, `pkg.Cls`, `+pkg.sub`) and the
methods of a class folder as `cls.method`.

### Parsing

- A file whose first statement is `function` is a function file: its entity is named after
  the file. A `classdef` file is a class. Anything else is a script. A `.mlx` path is a live
  script (see below).
- Syntax errors never stop the parse: tree-sitter recovers and the rest is read. A UTF-8
  BOM is ignored. CRLF is read as usual. Text that is not UTF-8 is read as Latin-1.
- Arguments: the signature's inputs and outputs, then each `arguments` block. Only names of
  the signature are updated (maxx's rule). `opts.Name` and `opts.?Cls.prop` are keyword
  arguments. `arguments (Output)` puts its names into `returns`. `varargin` has the kind
  `varargin`. `~` is an argument named `~`.
- Methods: the first input (the object) is removed unless the method is `Static` or is the
  constructor (named after the file).
- Getters and setters (`get.X`, `set.X`) attach to the property `X` of the class (its
  `getter`/`setter` in `PropertyDetail`); without a property, they are members named `X`
  with `is_getter`/`is_setter` set.
- Attributes: `Access`, `GetAccess`, `SetAccess` (not `public` ones) as `Access=private`; a class list (`{?Cls}`) or one class (`?Cls`) as `Access=restricted`
  etc.; the boolean attributes (`Abstract`, `Hidden`, `Sealed`, `Static`, `Constant`,
  `Dependent`, `Transient`, ...) as their names. A class's `Abstract`, `Hidden` and `Sealed`
  are its attributes too.
- Enumerations: the members of each `enumeration` block, with their values as written.
- `events` blocks are not entities (maxx does not record them either).
- Local functions (after the file's own function) are members of that function; nested
  functions (in a function body) are members of the function. Scripts' local functions are
  members of the script.
- A warning (`log`, see below) is written when syntax errors were recovered.

### Live scripts

Read by `parse_live_file` (a `.mlx`, or a file that is not a zip archive), by
`parse_live_text_file` (plain-text live code, whatever the extension), or by `parse_file` for a
`.mlx`. On the path, a `.mlx` is a live script only with `ParserConfig::parse_live_scripts`
(off by default, as maxx). A plain-text live `.m` is never picked up from its name: it is a
script on the path, and it is read as live code when the caller knows it is one.

- `.mlx`: the zip archive's `matlab/document.xml` is read with `zip` and `quick-xml`. The
  paragraphs (`p` elements in `body`, whatever their namespace prefix) are text, or code when
  their style is `matlab-Code`. Consecutive paragraphs of one kind form a section. The
  document is capped at 64 MiB. An archive without a document is `IndexError::Format`; a
  document that is not well-formed XML has no sections, with a warning.
- Plain-text live code (R2025a): a `%%` line (alone, or followed by a title) starts a new
  cell, and it is not a section itself. A `%[text] ...` line is a text paragraph: the marker
  and its `{...}` attributes are removed. `%[text:table]` and similar markers with no text
  are skipped. Any other `%[...]` marker (`%[output...]`, `%[control...]`, `%[image...]`) and
  the `%`-prefixed lines after it are not sections. Every other line is code. Consecutive
  text paragraphs form one text section, consecutive code lines one code section; blank
  lines inside code belong to it, and leading and trailing blank lines are dropped. A
  `%[appendix]` line ends the content.
- Neither format is evaluated. A live script has no docstring.

### Class hierarchy

- `mro(class)` is the C3 order of the class and its bases, each as `Ancestor { name,
  entity }`. A base is resolved from the class file's folder, then the path (as `get_from`).
  A base that is not on the path keeps its name and has no entity. A cycle of superclasses
  (`A < B`, `B < A`, or `A < A`) is `IndexError::CyclicBase` with the chain (`A -> B -> A`).
  An inconsistent hierarchy (no C3 order) takes the first base left.
- `inherited_members(class)` lists the members of the bases in that order, as `Inherited
  { from, name, entity }`. A name the class declares, or an earlier base gave, is skipped.

### Docstrings

- A comment block next to a declaration: the run of comments on adjacent lines (a blank
  line ends the block), with the comment markers removed. `%%` lines are cell titles, `%{ %}`
  block comments are kept, `%#` pragmas and `--8<--` lines are dropped. The text is then
  normalised as `inspect.cleandoc` does, with trailing whitespace removed (as maxx and
  griffe keep it).
- Placement: a comment after a property, argument or enumeration member documents the one
  before it (maxx's default). The `docstring_before_properties`, `_arguments` and
  `_enumerations` options of `ParserConfig` make the block before it document it.
- A function's docstring is the comment block after its signature (before the arguments
  block). A class's is the block after `classdef`. A file's header comments (before its first
  statement) document the file's entity when it has none.
- `docstring.raw` keeps the exact source text and `lineno`/`endlineno` its span. There is no
  section parsing (Google or NumPy style is for the consumer).
- A folder's docstring is its `Contents.m` header, else its `README.md` text. A class folder
  uses its class file's docstring first.

### Source lines

`Sources::lines(path)` reads a file once and returns its lines: split on `\n`, with a `\r`
before it removed, and no empty line for a final newline (maxx keeps `\r` and that empty
line). Text that is not UTF-8 is read as Latin-1. `Sources::is_cached` says whether a path
has been read.

### Logging

The core logs through the `log` facade: `debug` for each file parsed, `info` for each live
script, `warn` for recovered syntax errors, unreadable paths and an unreadable `document.xml`.
The core does not depend on Python; `malt_py` forwards the records to Python's `logging`
logger `malt_index` (see its README).

## Differences from maxx

Each difference is listed in `oracle_allowlist.json` with its reason, and the oracle
reports it as accepted. The main groups:

1. **Bugs in maxx, not replicated**: the function query reads the first match only, so
   inputs and outputs of a function with several outputs or a nested function are dropped
   or taken from the nested function; `~` is dropped; the docstring of a function with nested
   functions is lost; the `Dependant` misspelling drops `Dependent`; class attributes are not
   in `attributes`; the constructor of a class folder is popped from its members; a class
   folder's script members are joined without a dot (`class2classbody`); a classdef with a
   parse error (`[1 2 3](end)`) is read as a script.
2. **Records maxx does not**: local and nested functions; line spans of properties and
   enumeration members (maxx gives null); the varargin kind; the `~` arguments; line spans
   of live script sections (for a plain-text live script).
3. **Bare names**: maxx lists the methods of a class folder without a class file under the
   bare name (`inv`, for `@double/inv.m`). The index lists them as `double.inv`.
4. **Enumeration kind**: maxx's `Kind.ENUMERATION` equals `CLASS` (`"class"`); the index
   says `enumeration`.
5. **Encoding**: maxx detects the encoding with charset-normalizer; the index uses UTF-8
   and falls back to Latin-1 (correct for the Western files in the corpus).
6. **Live scripts**: maxx classifies a whole `%%` cell as text when all its lines are
   comments, so a cell of `%[text]` paragraphs with code in it is one text section, and it
   keeps the `%[text]` markers. The index splits a cell into runs, strips the markers, and
   leaves outputs and the appendix out (`live-text` in the allowlist: 9 accepted differences,
   all of this kind). The `.mlx` sections agree with maxx exactly.
7. **Folder members**: maxx's folder lists only the subfolders it registered on the path; a
   folder entity here lists every plain subfolder below it.

## Oracle workflow

Compare with maxx on the same paths (read-only inputs, never copied):

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

`--live` reads the `.mlx` files on the path (maxx's `parse_live_scripts`). `--plaintext DIR`
reads each `.m` file of DIR as plain-text live code, keyed by stem. The sample used for the
`live-mlx` and `live-text` projects is in `tests/live_oracle` (`make_mlx.py` writes the `.mlx`
into `mlx/`).

The report groups every difference by field, with the entity, its kind, span, attributes,
bases, arguments, sections and docstring, and a summary table. The allowlist entries match on
`project`, `entity`, `field` and `detail` (globs); each has a `reason`. To add a project,
list its paths (for a recursive scan: the root and its plain subfolders, not `+`, `@` or
`private` folders) in the same way the dump takes them.

## Tests

`cargo test -p malt-index`:

- `src/comments.rs`: the docstring text rules. `src/expr.rs`, `src/live.rs`, `src/source.rs`:
  unit tests of the expression text, the cell and marker rules, and the line splitting.
- `tests/parse.rs`: the parser on small samples (every kind, argument forms, attributes,
  enumerations, placement, block comments, nested functions, BOM/CRLF/Latin-1, malformed input).
- `tests/expr.rs`: expressions (text, span, the joined type, bases, enumeration values, the
  `opts.?` class path).
- `tests/live.rs`: `.mlx` files built in the test with `zip` (paragraph runs, namespace-free
  XML, malformed XML, a file that is not a zip, a zip without `document.xml`, truncated and
  random archives), plain-text live code (cells, markers, outputs, the appendix, CRLF), and the
  `parse_live_scripts` flag on the path.
- `tests/index.rs`: scanning and lookup on `tests/project` (packages, class folders,
  `private/`, precedence, addpath/rmpath, recursion), and the golden snapshot
  `tests/golden/project.json`. To update it: `MALT_INDEX_UPDATE_GOLDEN=1 cargo test -p malt-index`,
  then review the diff.
- `tests/folder.rs`: folders (their members, subfolders, `Contents.m`), `get_path` for files and
  folders, and what is not a folder on the path (`+`, `@`, `private`, hidden, outside).
- `tests/paths.rs`: the path configuration on temporary projects (the `[index]` table, each
  glob rule, missing folders, deduplication, order, and config parse errors).
- `tests/hierarchy.rs`: C3 order (a diamond), unresolved bases, inherited members with
  overrides, class folders, and cycles (an error).
- `tests/sources.rs`: the line cache (read once, shared, missing files).
- `tests/logging.rs`: the records of parsing and of warnings, through a test logger.

The Python tests of the bindings are in `crates/malt_py/python/tests`.

The sample project under `tests/project` is written for these tests.

## Dependencies

- `zip` (2.6, `default-features = false`, feature `deflate`): reads `.mlx` archives. The
  `deflate` feature also pulls in `zopfli`, the compressor, which is not used: it is the
  smallest feature set that decompresses.
- `quick-xml` (0.37): reads `document.xml`; event-based, no tree.
- `log` (0.4): the logging facade, no dependencies.

## What remains

- No `Alias` class: `is_alias` and `target` exist only for inherited members (the object is
  a copy of the base's member, and `target` is itself). Imports are not modelled.
- Folder entities parse their files again on every lookup (no cache for folders). A folder
  listing is a full tree of its subfolders.
- The `%[...]` marker grammar of plain-text live code is taken from the R2025a documentation
  the repository's skill describes; it is not yet checked against MATLAB's own reader
  (`export` or `matlab.desktop.editor`). Unknown markers are treated as output blocks.
- No section-level line spans for a `.mlx`, and no docstrings for live scripts.
- `events` blocks are still not entities.

## Real `.mlx` files

MATLAB writes code paragraphs with the style `code` (text in CDATA); maxx's own test file uses
`matlab-Code`. The reader accepts both. Checked on the three real MATLAB-made `.mlx` files in the
maxx checkout (advanced-logger `GettingStarted.mlx` and `CustomizingLogger.mlx`): the sections
alternate text and code. maxx reads such a file as one text section (allowlist project
`live-real`).
