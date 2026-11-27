// Entry point of the action: load the WebAssembly build that sits next to this file and run.

import fs from 'node:fs'
import { fileURLToPath } from 'node:url'
import init, * as wasm from './vendor/malt_lib.js'
import { run } from './run.mjs'

try {
  const bytes = fs.readFileSync(fileURLToPath(new URL('./malt_lib_bg.wasm', import.meta.url)))
  await init({ module_or_path: bytes })
  const outcome = run({ wasm })
  if (outcome.failed) process.exitCode = 1
} catch (error) {
  process.stdout.write(`::error title=malt::${String(error?.message ?? error).replace(/\n/g, '%0A')}\n`)
  process.exitCode = 1
}
