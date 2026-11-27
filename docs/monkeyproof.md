---
icon: lucide/badge-check
---

# MonkeyProof coding standard

malt can check code against the [MonkeyProof Coding Standard for MATLAB](https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/index.html),
a published set of naming, layout, statement, design and security rules by MonkeyProof Solutions B.V. It is an extra set
on top of malt's own checks. It is not part of MATLAB's Code Analyzer, so it is off unless you switch it on, and it never
changes what malt reports under the default configuration.

The checks are malt's own reading of the standard's rules. They are not MonkeyProof's tool and they do not promise the
same findings as it. Where a rule's text leaves room, each page says what malt decided.

## Switching it on

Use the preset, which is the default (`mathworks`) configuration plus the standard:

```toml title=".malt.toml"
[lint]
preset = "monkeyproof"
```

```bash
malt --preset monkeyproof src/
```

Or switch the category on in any configuration, with a severity if you want to override the standard's own:

```toml title=".malt.toml"
[lint.categories]
monkeyproof = "warn"
```

The `all` preset keeps the standard off, because `all` stands for MATLAB's checks and malt's own. Individual checks are
switched off like any other:

```toml title=".malt.toml"
[lint]
preset = "monkeyproof"

[lint.rules]
MP_LAYOUT_5 = "off"                # no line length limit

[lint.rules.MP_LAYOUT_5]           # or keep it, with a different limit
max_length = 140
```

## What is covered

The standard has 102 rules. Its own tool checks 66 of them automatically. The other 36 need a person: spelling in
comments, descriptive names, single responsibility, repeated code. malt implements 63 of the 66 and leaves those 36 out.

| Chapter | Page | Checks |
| ------- | ---- | ------ |
| Naming conventions | [MonkeyProof naming](rules/monkeyproof-naming.md) | 7 |
| Layout and comments | [MonkeyProof layout](rules/monkeyproof-layout.md) | 13 |
| Statements and expressions | [MonkeyProof statements](rules/monkeyproof-statements.md) | 22 of 23 |
| Higher level rules | [MonkeyProof functions](rules/monkeyproof-functions.md) | 15 of 17 |
| Security | [MonkeyProof security](rules/monkeyproof-security.md) | 6 |

Three automatic rules are not implemented, because they need information a single file does not hold:

- `STAT-17`, every called function is known to MATLAB, needs the functions of MATLAB and its toolboxes and the user's path.
- `FCNS-23` and `FCNS-25`, no re-implemented properties and no changed method access compared with a superclass, need the
  superclass itself. The path index only knows whether a class is a handle class.

Some checks cover part of their rule for the same reason. Each page says which, and what the check does not catch.

## IDs and severities

A check ID is the standard's ID with its dash turned into an underscore: `NAMING-2` is `MP_NAMING_2`. That keeps it valid
in `%#ok<MP_NAMING_2>` directives and as an unquoted key in `.malt.toml`.

The standard gives each rule a severity level from 1 (certainly a programming error) to 10 (controversial or style).
malt maps the levels to its three severities:

| Standard level | malt severity |
| -------------- | ------------- |
| 1 and 2 | error |
| 3 to 5 | warning |
| 6 to 10 | info |

The mapping is a default like any other: `[lint.rules.MP_STAT_6] severity = "error"` overrides it for one check.

## Differences from the standard

- A rule with a threshold has it as a parameter of the check, with the standard's value as the default.
- A few rules are read strictly or loosely where the text is open. For example `MP_STAT_9` follows the text literally, so
  `a + b - c` counts as mixed operators, and `MP_SECURITY_5` flags only a literal value, because the standard's own
  recommended code assigns those names from a login dialog.
- Checks that look at other files (a mex function's platforms, the implementation of a declared method) read the folder of
  the file being checked and report nothing when the file is not on disk.
- The standard's own text, tables and examples are not reproduced here. The messages and the examples on the pages are
  malt's.
