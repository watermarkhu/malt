// Build the action's dist: `bun action/build.mjs [path/to/malt-wasm/pkg]`.
//
// Bundles the Node script with the wasm glue (and the `ignore` package) into
// `action/dist/index.mjs`, and copies the wasm binary next to it. The result is
// self-contained: the release workflow commits it to the release tag only, never to main.

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.dirname(fileURLToPath(import.meta.url))
const pkg = path.resolve(process.argv[2] ?? path.join(root, '..', 'crates', 'malt_wasm', 'pkg'))
for (const file of ['malt_lib.js', 'malt_lib_bg.wasm']) {
  if (!fs.existsSync(path.join(pkg, file))) {
    console.error(`missing ${file} in ${pkg}: build the wasm package first (mise run build-wasm)`)
    process.exit(1)
  }
}

// The wasm glue is vendored for the bundler (and for the tests), and is not committed.
fs.mkdirSync(path.join(root, 'src', 'vendor'), { recursive: true })
fs.copyFileSync(path.join(pkg, 'malt_lib.js'), path.join(root, 'src', 'vendor', 'malt_lib.js'))

const dist = path.join(root, 'dist')
fs.rmSync(dist, { recursive: true, force: true })
const result = await Bun.build({
  entrypoints: [path.join(root, 'src', 'index.mjs')],
  outdir: dist,
  target: 'node',
  format: 'esm',
  naming: '[name].mjs',
  minify: false,
})
if (!result.success) {
  for (const log of result.logs) console.error(String(log))
  process.exit(1)
}
fs.copyFileSync(path.join(pkg, 'malt_lib_bg.wasm'), path.join(dist, 'malt_lib_bg.wasm'))
console.log(`built ${path.relative(process.cwd(), dist)}`)
