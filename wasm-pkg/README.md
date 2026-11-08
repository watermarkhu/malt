# malt-wasm

Fast MATLAB linter with 2,680+ Code Analyzer checks, compiled to WebAssembly.

## Installation

```bash
npm install malt-wasm
```

## Quick Start

```javascript
import init, { Linter } from 'malt-wasm';

// Initialize the WASM module
await init();

// Lint MATLAB content
const linter = new Linter({});
const warnings = JSON.parse(linter.check('x = 1'));

// Apply all auto-fixes
const fixed = linter.fix('x = 1');
```

## API Reference

### `init()`

Initialize the WASM module. Must be called before using other functions.

```javascript
await init();
```

### `new Linter(options)`

Create a linter with an optional configuration object.

```javascript
const linter = new Linter({
  disable: ['NOSEMI', 'AGROW'],
  exclude: ['vendor/**'],
});
```

### `Linter.from_toml(toml)`

Create a linter from a `.malt.toml` configuration string, with full config
fidelity (per-rule severity, per-category overrides, rule parameters,
`exclude`). Throws a string error when the TOML cannot be parsed.

```javascript
const linter = Linter.from_toml(`
[lint.rules]
NOSEMI = "off"

[lint.rules.CODEGEN_ENGINE]
severity = "error"
`);
```

### `linter.check(content, path?)`

Lint MATLAB content and return warnings as a JSON string.

```javascript
const warnings = JSON.parse(linter.check('x = 1'));
```

`path` is optional and defaults to `"untitled.m"`. When provided it is matched
against `exclude` patterns.

### `linter.fix(content, path?)`

Apply all available auto-fixes to the content.

```javascript
const fixed = linter.fix('x = 1');
```

### `Linter.get_version()`

Get the malt version.

```javascript
const version = Linter.get_version(); // e.g., "0.1.0"
```

### `Linter.get_available_rules()`

Get the list of available rule IDs as a JSON string.

```javascript
const rules = JSON.parse(Linter.get_available_rules());
// ["AGROW", "NOSEMI", ...]
```

### `LinterConfig`

| Field | Type | Description |
| ----- | ---- | ----------- |
| `disable` | `string[]` | Rule IDs to disable (e.g. `["NOSEMI"]`) |
| `enable` | `string[]` | If set, only these rule IDs are enabled |
| `extend_enable` | `string[]` | Additional rules to enable beyond the default set |
| `extend_disable` | `string[]` | Additional rules to disable beyond `disable` |
| `exclude` | `string[]` | File path glob patterns to skip (matched against `path`) |
| `inline_suppression` | `boolean` | Honor `%#ok<...>` directives (default `true`) |

## Warning Format

Each warning object contains:

```typescript
interface Warning {
  rule_id: string;      // Rule ID (e.g., "NOSEMI")
  message: string;      // Warning message
  severity: string;     // "Error", "Warning", or "Info"
  line: number;         // 1-indexed line number
  column: number;       // 1-indexed column number
  end_line: number;     // 1-indexed end line
  end_column: number;   // 1-indexed end column
  fix?: {               // Optional auto-fix (character offsets)
    start: number;
    end: number;
    replacement: string;
  };
}
```

## Browser Usage

### ES Module

```html
<script type="module">
  import init, { Linter } from './malt_lib.js';

  async function main() {
    await init();
    const linter = new Linter({});
    const warnings = JSON.parse(linter.check('x = 1'));
    console.log(warnings);
  }

  main();
</script>
```

### With CDN (jsDelivr)

```html
<script type="module">
  import init, { Linter } from 'https://cdn.jsdelivr.net/npm/malt-wasm/malt_lib.js';

  await init();
  const linter = new Linter({});
  console.log(JSON.parse(linter.check('x = 1')));
</script>
```

## Node.js Usage

```javascript
import init, { Linter } from 'malt-wasm';
import { readFile } from 'fs/promises';

await init();
const linter = new Linter({});

const content = await readFile('example.m', 'utf-8');
const warnings = JSON.parse(linter.check(content));

for (const w of warnings) {
  console.log(`${w.line}:${w.column} ${w.rule_id} ${w.message}`);
}
```

## Local Development

The playground docs page loads the wasm from the CDN by default. To build and
use a local build instead:

```bash
mise install
mise run build-wasm
mise run serve-local-wasm   # copies crates/malt_wasm/pkg -> docs/public/assets/malt-wasm
```

## License

MIT
