# malt Rule Documentation Schema

Every rule module in `crates/malt_rules/src` documents itself through a
**standardized header docstring** written in markdown inside the Rust `//!`
module comment. `docs/scripts/gen_rules_docs.ts` reads these docstrings and
generates one documentation page per rule.

A *rule* is a module that self-registers with `inventory::submit!` — i.e. one
row in the "Rule" trait sense (e.g. `NOSEMI`, or a multi-check engine like
`BUGS_ENGINE`, `COMPAT`, `NAMING_ENGINE`). Each rule page anchors every check
ID the rule emits.

## Docstring Layout

Every rule docstring MUST follow this exact structure:

```text
//! # <META_ID>: <Human Title>
//!
//! ```malt
//! id = "<META_ID>"                    # required — config/registration ID
//! title = "<Human Title>"             # required — shown as the page H1
//! category = "<category-slug>"        # required — Category::as_str()
//! severity = "<error|warning|info>"   # required — default severity
//! fix = <true|false>                  # required — does it auto-fix?
//! icon = "lucide/<name>"              # optional — page icon
//! slug = "<url-slug>"                 # optional — page filename stem
//! data_file = "<name>.toml"           # optional — data-driven engine
//! note = "..."                        # optional — italic callout line
//! ```
//!
//! ## Rule
//!
//! <free-form markdown: what the rule does, why it matters>
//!
//! ## Fix
//!
//! <free-form markdown: what the auto-fix rewrites; OMIT this section
//!  entirely when `fix = false`>
//!
//! ## Examples
//!
//! <free-form markdown with `Correct` / `Incorrect` / `Fixed` fenced
//!  matlab blocks>
```

Rules producing multiple check IDs (engines) add an optional fourth section
between `## Rule` and `## Fix`:

```text
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | -------- | -------- | --- | ----------- |
//! | CTRUE    | error    | no  | Condition is always true |
//! | CMDAND   | error    | yes | `&` where `&&` intended |
```

Data-driven engines (`data_file = "..."`) instead reference a TOML table in
`crates/malt_rules/src/data/`; the generator builds the check-ID list from the
data file. See [Data-driven engines](#data-driven-engines).

## Front-matter keys (` ```malt ` block)

| Key | Required | Type | Description |
| --- | -------- | ---- | ----------- |
| `id` | yes | string | Registration ID (e.g. `NOSEMI`, `BUGS_ENGINE`, `COMPAT`, `NAMING_ENGINE`). |
| `title` | yes | string | Human-readable title; becomes the page H1. |
| `category` | yes | string | Category slug from `Category::as_str()` (e.g. `bugs`, `formatting`, `compatibility`). |
| `severity` | yes | string | Default severity: `error`, `warning`, or `info`. |
| `fix` | yes | bool | Whether any emitted diagnostic carries an auto-fix. |
| `icon` | no | string | VitePress icon name (`lucide/bug`). Defaults to a category-derived icon. |
| `slug` | no | string | Page filename stem (default: `id` lowercased, `_` → `-`). |
| `data_file` | no | string | TOML data file name for data-driven engines. |
| `generated` | no | string | `naming` — engine check IDs are a computed product (see below). |
| `note` | no | string | One-line italic callout rendered under the H1. |

## `## Rule`

The primary section. Explain what the rule detects, the category it belongs
to, and — for engines — that one engine dispatches many check IDs. Keep it
prose; this becomes the page's "What this rule does" content.

## `## Check IDs`

Required for multi-check engines; omitted for single-check rules.

- Markdown table with exactly four columns: `Check ID | Severity | Fix | Description`.
- `Severity` is `error`, `warning`, or `info` (per-check override of the rule default).
- `Fix` is `yes` or `no` (does THIS check emit a fix).
- `Description` is a one-line summary; keep `|` escaped as `\|` if needed.

The generator turns each row into an anchor (`#<check-id>`) on the rule page,
and each row also feeds the global rule index in `docs/rules.md`.

## `## Fix`

Required **iff** `fix = true`. Describes what the auto-fix rewrites. Omit the
whole section when `fix = false`.

## `## Examples`

Required. Use `Correct` / `Incorrect` / `Fixed` sub-headings with fenced
`matlab` blocks. For engines, show representative examples covering several
check IDs.

## Data-driven engines

When `data_file` is present, the generator reads
`crates/malt_rules/src/data/<data_file>` and builds the check-ID list from its
`[[checks]]` entries (`id`, `severity`, `message`, `category`, `function_name`).
The docstring omits the `## Check IDs` table; the generator substitutes an
anchored table grouped by the entries' `category` field.

The `naming` engine uses `generated = "naming"` instead: the generator computes
the 81 `naming.<entity>.<checkType>` check IDs as the Cartesian product of the
documented entity and check-type lists, so the docstring needs no table.

## Generator outputs

For each rule module the generator writes:

1. `docs/rules/<slug>.md` — the rule page. The `slug` is the front-matter
   `slug` key (or derived from `id`). Every check ID appears as an anchor so
   overviews can deep-link (`docs/rules/bugs.md#FNAN`).
2. A row (or rows, one per check ID) in the rule index table in
   `docs/rules.md`, linking to the rule page and anchors.

`--check` mode verifies every generated file is byte-identical to what the
sources produce, so the `hk` pre-commit hook fails on stale docs.
`--check-module <slug>` verifies a single rule page — useful while migrating
modules one at a time.

## Rules for writers

- Keep the `malt` front-matter block byte-parseable: simple `key = "value"`
  lines, double-quoted strings, no comments inside the block.
- The front-matter `severity`/`category` MUST match the `Rule` impl's
  `fn severity()` / `fn category()`.
- Section headings are exact: `## Rule`, `## Check IDs`, `## Fix`, `## Examples`.
  Do not rename or add required-section variants.
- `## Configuration` (TOML example) is optional and may follow `## Examples`.
- `# <META_ID>: <Title>` H1 must contain the meta ID; the front-matter `title`
  may differ in wording but should be consistent.
