---
icon: lucide/book-open
---

# How MATLAB is read

This page describes what the index makes of a MATLAB project. It applies to the Rust crate and the Python module alike.

## Folders on the path

- A folder contributes its `.m` files, except `Contents.m`, which documents the folder. With `parse_live_scripts`, its
  `.mlx` files are live scripts too.
- `+pkg` is a package. Its files and sub-packages are its members (`pkg.fn`, `pkg.sub.fn`). Class folders inside a
  package work too.
- `@cls` is a class folder. `cls.m` is the class file (a classdef, or an old-style constructor function). The other `.m`
  files are methods, named after their file.
- `private/` is visible only to the files of its parent folder (`get_from(name, dir)` in Rust, `resolve(name, from_dir)`
  in Python). It is never on the path itself, and the private folders of subfolders are not on the path with recursion.
- With `recursive`, plain subfolders are scanned too and their files join the top level, as MATLAB's `genpath` does.
  Their `+` and `@` folders join the top level as well.

### Plain folders and path lookups

- A folder is an entity if its directory is on the path or below a folder on it, through plain folders only: not `+`,
  `@`, `private` or hidden folders. Its members are its files, packages and class folders, and its plain subfolders,
  each a folder too. A folder is read with a throwaway index of the folder, so it does not change the parse cache of the
  search path, and a lookup parses the files again.
- Looking up a path gives the folder, or the entity of a file in such a folder (its member of that stem). Anything else
  is none.

## Precedence

The first path (the front of the search path) wins a name. Within one path, names are taken in sorted order. A lookup
from a folder looks at that folder's own files and its `private/` first. When several paths define one package, the
package is the union of its members, and the earlier path wins a name clash.

## Top-level names

Each top-level name is listed (`fn`, `Cls`, `+pkg`, `cls`). The members of a package are listed under dotted identifiers
(`pkg.fn`, `pkg.Cls`, `+pkg.sub`), and the methods of a class folder as `cls.method`.

The dotted key of a class folder method is a listing name only. MATLAB does not resolve `double.inv` (`which('double.inv')`
is empty; the file is `@double/inv`), so it is not a name to call. The same holds for a method of a class folder inside
a package (`mlog.Logger.addMessage`).

## Parsing

- A file whose first statement is `function` is a function file, and its entity is named after the file. A `classdef`
  file is a class. Anything else is a script. A `.mlx` path is a live script.
- Syntax errors never stop the parse: tree-sitter recovers and the rest is read. A UTF-8 BOM is ignored and CRLF is read
  as usual. Text that is not UTF-8 is read as Latin-1.
- Arguments are the signature's inputs and outputs, then each `arguments` block. Only names of the signature are updated.
  `opts.Name` and `opts.?Cls.prop` are keyword arguments. `arguments (Output)` puts its names into `returns`. `varargin`
  has the kind `varargin`. `~` is an argument named `~`.
- In a method, the first input (the object) is removed unless the method is `Static` or is the constructor (named after
  the file).
- Getters and setters (`get.X`, `set.X`) attach to the property `X` of the class, as its `getter` and `setter`. Without a
  property, they are members named `X` with `is_getter` or `is_setter` set.
- Attributes: `Access`, `GetAccess` and `SetAccess` (not `public` ones) become `Access=private`. A class list (`{?Cls}`)
  or one class (`?Cls`) becomes `Access=restricted`. The boolean attributes (`Abstract`, `Hidden`, `Sealed`, `Static`,
  `Constant`, `Dependent`, `Transient`, and the like) become their names. A class's `Abstract`, `Hidden` and `Sealed`
  are its attributes too.
- Enumerations: the members of each `enumeration` block, with their values as written.
- `events` blocks are not entities. The names of their entries are recorded on the class, in source order in the Rust
  entity and sorted in the dump.
- Local functions (after the file's own function) are members of that function, and nested functions are members of the
  function they are in. A script's local functions are members of the script.
- A warning is logged when syntax errors were recovered.

## Docstrings

- A docstring is the comment block next to a declaration: the run of comments on adjacent lines (a blank line ends the
  block), with the comment markers removed. `%%` lines are cell titles, `%{ %}` block comments are kept, and `%#`
  pragmas and `--8<--` lines are dropped. The text is then normalised as `inspect.cleandoc` does, with trailing
  whitespace removed.
- A comment after a property, argument or enumeration member documents the one before it. The
  `docstring_before_properties`, `docstring_before_arguments` and `docstring_before_enumerations` options of
  `ParserConfig` make the block before it document it.
- A function's docstring is the comment block after its signature (before the arguments block). A class's is the block
  after `classdef`. A file's header comments, before its first statement, document the file's entity when it has none.
- The raw text keeps the exact source and its span. There is no section parsing: Google or NumPy style is for the
  consumer.
- A folder's docstring is its `Contents.m` header, else its `README.md` text. A class folder uses its class file's
  docstring first.

## Live scripts

A `.mlx` is read as a live script, or a file that is not a zip archive is read as plain-text live code. On the path, a
`.mlx` is a live script only with `parse_live_scripts` (off by default). A plain-text live `.m` is never picked up from
its name. It is a script on the path, and it is read as live code when the caller knows it is one.

- `.mlx`: the zip archive's `matlab/document.xml` is read. The paragraphs (`p` elements in `body`, whatever their
  namespace prefix) are text, or code when their style is `code` or `matlab-Code`. Consecutive paragraphs of one kind
  form a section. The document is capped at 64 MiB. An archive without a document is an error, and a document that is
  not well-formed XML has no sections, with a warning.
- Plain-text live code (R2025a): a `%%` line (alone, or followed by a title) starts a new cell, and it is not a section
  itself. A `%[text] ...` line is a text paragraph, with the marker and its `{...}` attributes removed. `%[text:table]`
  and similar markers with no text are skipped. Any other `%[...]` marker (`%[output...]`, `%[control...]`,
  `%[image...]`) and the `%`-prefixed lines after it are not sections. Every other line is code. Consecutive text
  paragraphs form one text section and consecutive code lines one code section. Blank lines inside code belong to it,
  and leading and trailing blank lines are dropped. A `%[appendix]` line ends the content.
- Neither format is evaluated, and a live script has no docstring.

## Class hierarchy

- The method resolution order is the C3 order of the class and its bases. A base is resolved from the class file's
  folder, then the path. A base that is not on the path keeps its name and has no entity. A cycle of superclasses (`A <
  B`, `B < A`, or `A < A`) is an error that names the chain (`A -> B -> A`). An inconsistent hierarchy, with no C3
  order, takes the first base left.
- The inherited members are the members of the bases in that order. A name the class declares, or an earlier base gave,
  is skipped.

## Source lines

A file is read once and its lines are returned: split on `\n` with a `\r` before it removed, and no empty line for a
final newline. Text that is not UTF-8 is read as Latin-1.

## Known limits

- There is no separate alias class. `is_alias` and `target` exist only for inherited members: the object is a copy of the
  base's member, and its `target` is itself. Imports are not modelled.
- A folder entity parses its files again on every lookup, and a folder listing is a full tree of its subfolders.
- The `%[...]` marker grammar of plain-text live code comes from the R2025a documentation. It is not yet checked against
  MATLAB's own reader. Unknown markers are treated as output blocks.
- A `.mlx` has no section-level line spans, and a live script has no docstring.
- `events` blocks are not entities: only the names of their entries are recorded, on the class.
