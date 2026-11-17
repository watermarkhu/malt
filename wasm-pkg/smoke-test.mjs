// Smoke test for the built npm package: `node wasm-pkg/smoke-test.mjs <pkg-dir>`.
// Loads the wasm module in Node.js and checks linting, presets, and config errors.
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import { pathToFileURL } from 'node:url'

const pkg = path.resolve(process.argv[2] ?? 'crates/malt_wasm/pkg')
const mod = await import(pathToFileURL(path.join(pkg, 'malt_lib.js')).href)
await mod.default({ module_or_path: fs.readFileSync(path.join(pkg, 'malt_lib_bg.wasm')) })

const ids = (linter, source = 'x = 1\n') => JSON.parse(linter.check(source, null)).map((w) => w.rule_id)

assert.ok(ids(new mod.Linter({})).includes('NOPTS'), 'NOPTS fires by default')
assert.ok(
  !ids(mod.Linter.from_toml('[lint]\npreset = "recommended"\n')).includes('NOPTS'),
  'the recommended preset disables formatting',
)
// Codegen checks fire only in code marked `%#codegen`.
assert.ok(
  ids(mod.Linter.from_toml('[lint]\npreset = "all"\n'), '%#codegen\nx = 1\n').includes('EMSCR'),
  'the all preset enables codegen',
)
assert.throws(() => mod.Linter.from_toml('[lint'), /Invalid config/, 'invalid TOML throws')
// Realistic own-content cases with the IDs the native CLI reports for them (default
// `mathworks` preset), in line order. The wasm build must agree with the CLI.
const scaleAll = `function r = scale_all(v, k)
    r = [];
    for i = 1:numel(v)
        r(end+1) = v(i) * k
    end
    unused = 3;
end
`
const suppressed = `x = [];
for i = 1:3
    x(end+1) = i; %#ok<SAGROW>
end
disp(x)
`
const counter = `classdef Counter < handle
    properties
        Count = 0;
    end
    methods
        function obj = Counter()
        end
        function inc(obj, ~)
            obj.Count = obj.Count + 1;
        end
    end
end
`
const existCall = `result = zeros(1, 3);
for k = 1:3
    result(k) = k^2;
end
if exist('result') == 1, disp(result), end
`
const checkAs = (linter, source, file) => JSON.parse(linter.check(source, file)).map((w) => `${w.line}:${w.rule_id}`)
assert.deepEqual(checkAs(new mod.Linter({}), scaleAll, 'scale_all.m'), ['4:AGROW', '4:NOPRT', '6:NASGU'])
assert.deepEqual(checkAs(new mod.Linter({}), suppressed, 'suppressed.m'), [], '%#ok suppresses the check')
assert.deepEqual(checkAs(new mod.Linter({}), counter, 'Counter.m'), [])
assert.deepEqual(checkAs(new mod.Linter({}), existCall, 'existCall.m'), ['5:EXIST'])
// The file name matters (FNDEF): the same source under another name.
assert.deepEqual(checkAs(new mod.Linter({}), scaleAll, 'other.m').slice(0, 1), ['1:FNDEF'])

// `new Linter()` and `new Linter(null)` behave like `new Linter({})`, `%#ok` included.
for (const make of [() => new mod.Linter(), () => new mod.Linter(null), () => new mod.Linter({})]) {
  assert.deepEqual(checkAs(make(), suppressed, 'suppressed.m'), [], 'the default constructor honors %#ok')
}

// `fix` applies the safe fixes and leaves the rest.
assert.equal(
  new mod.Linter({}).fix(scaleAll, 'scale_all.m'),
  scaleAll.replace('v(i) * k\n', 'v(i) * k;\n'),
  'fix adds the missing semicolon',
)

// A deeply nested file is reported or analysed, never a trap (the wasm depth guard is 200).
const nested = 'x = ' + '('.repeat(5000) + '1' + ')'.repeat(5000) + ';\n'
assert.ok(Array.isArray(JSON.parse(new mod.Linter({}).check(nested, 'nested.m'))), 'deep nesting does not trap')

assert.match(mod.Linter.get_version(), /^\d+\.\d+\.\d+/)

console.log(`malt-wasm ${mod.Linter.get_version()} smoke test passed`)
