// Smoke test for the built npm package: `node wasm-pkg/smoke-test.mjs <pkg-dir>`.
// Loads the wasm module in Node.js and checks linting, presets, and config errors.
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import { pathToFileURL } from 'node:url'

const pkg = path.resolve(process.argv[2] ?? 'crates/malt_wasm/pkg')
const mod = await import(pathToFileURL(path.join(pkg, 'malt_lib.js')).href)
await mod.default({ module_or_path: fs.readFileSync(path.join(pkg, 'malt_lib_bg.wasm')) })

const ids = (linter) => JSON.parse(linter.check('x = 1\n', null)).map((w) => w.rule_id)

assert.ok(ids(new mod.Linter({})).includes('NOPTS'), 'NOPTS fires by default')
assert.ok(
  !ids(mod.Linter.from_toml('[lint]\npreset = "recommended"\n')).includes('NOPTS'),
  'the recommended preset disables formatting',
)
assert.ok(ids(mod.Linter.from_toml('[lint]\npreset = "all"\n')).includes('EMSCR'), 'the all preset enables codegen')
assert.throws(() => mod.Linter.from_toml('[lint'), /Invalid config/, 'invalid TOML throws')
assert.match(mod.Linter.get_version(), /^\d+\.\d+\.\d+/)

console.log(`malt-wasm ${mod.Linter.get_version()} smoke test passed`)
