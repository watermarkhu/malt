// Entry point of the action: load the WebAssembly build that sits next to this file and run.

import fs from 'node:fs'
import { fileURLToPath } from 'node:url'
import init, * as wasm from './vendor/malt_lib.js'
import { runShell } from './shell.mjs'

try {
  const bytes = fs.readFileSync(fileURLToPath(new URL('./malt_lib_bg.wasm', import.meta.url)))
  await init({ module_or_path: bytes })
  if (runShell({ wasm }).failed) process.exitCode = 1
} catch (error) {
  const message = String(error?.message ?? error).replace(/%/g, '%25').replace(/\r?\n/g, '%0A')
  process.stdout.write(`::error title=malt::${message}\n`)
  process.exitCode = 1
}
