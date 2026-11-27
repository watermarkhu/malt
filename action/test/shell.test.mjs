// The shell against the real WebAssembly build: only its input and output. The behaviour of a
// run (files, levels, formats, failing) is tested in Rust, in crates/malt_wasm.
// Needs `mise run build-wasm` and `bun action/build.mjs` first; skipped when they are missing.
import { afterEach, describe, expect, test } from 'bun:test'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const glue = path.join(here, '..', 'src', 'vendor', 'malt_lib.js')
const binary = path.join(here, '..', 'dist', 'malt_lib_bg.wasm')
const ready = fs.existsSync(glue) && fs.existsSync(binary)

let wasm = null
let runShell = null
if (ready) {
  wasm = await import(glue)
  await wasm.default({ module_or_path: fs.readFileSync(binary) })
  runShell = (await import('../src/shell.mjs')).runShell
}

const dirs = []
afterEach(() => {
  for (const dir of dirs.splice(0)) fs.rmSync(dir, { recursive: true, force: true })
})

function workspace(files) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'malt-shell-'))
  dirs.push(dir)
  for (const [name, content] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(dir, name)), { recursive: true })
    fs.writeFileSync(path.join(dir, name), content)
  }
  return dir
}

function go(dir, inputs = {}) {
  const lines = []
  const env = {
    GITHUB_WORKSPACE: dir,
    GITHUB_STEP_SUMMARY: path.join(dir, '.summary.md'),
    GITHUB_OUTPUT: path.join(dir, '.output.txt'),
  }
  for (const [name, value] of Object.entries(inputs)) env[`INPUT_${name.toUpperCase()}`] = value
  const outcome = runShell({ wasm, env, cwd: dir, out: { write: (text) => lines.push(text) } })
  return { outcome, log: lines.join(''), env }
}

// NOPTS (info) on line 1, EOLPAR (error) on line 2.
const BAD = 'x = 1\nfunction f(\n'

describe.skipIf(!ready)('shell', () => {
  test('writes the log, the job summary, the outputs and reports the outcome', () => {
    const dir = workspace({ 'src/a.m': BAD })
    const { outcome, log, env } = go(dir)
    expect(log).toContain('::error file=src/a.m,line=2,col=11,title=EOLPAR::')
    expect(outcome).toMatchObject({ findings: 2, errors: 1, files: 1, failed: true })
    expect(fs.readFileSync(env.GITHUB_STEP_SUMMARY, 'utf8')).toStartWith('## malt')
    expect(fs.readFileSync(env.GITHUB_OUTPUT, 'utf8')).toContain('findings=2\nerrors=1\nwarnings=0\nresults-file=\n')
  })

  test('the inputs reach the run: paths, preset and level', () => {
    const dir = workspace({ 'src/a.m': BAD, 'other/b.m': BAD })
    expect(go(dir, { paths: 'src' }).outcome.files).toBe(1)
    expect(go(dir, { level: 'error' }).outcome.findings).toBe(2)
    expect(go(dir, { preset: 'monkeyproof' }).log).toContain('title=MP_')
  })

  test('the results file is written where output-json says', () => {
    const dir = workspace({ 'a.m': BAD })
    const { env } = go(dir, { 'output-json': 'out/results.json' })
    const results = JSON.parse(fs.readFileSync(path.join(dir, 'out', 'results.json'), 'utf8'))
    expect(results.malt_version).toBe(wasm.Linter.get_version())
    expect(fs.readFileSync(env.GITHUB_OUTPUT, 'utf8')).toContain(`results-file=${path.join(dir, 'out', 'results.json')}`)
  })

  test('an invalid input throws, for the entry point to report', () => {
    const dir = workspace({ 'a.m': 'x = 1;\n' })
    expect(() => go(dir, { level: 'loud' })).toThrow(/'level'/)
  })

  test('the file system callbacks skip links and read what the run asks for', () => {
    const dir = workspace({ 'a.m': 'x = 1;\n', 'real/r.m': 'x = 1;\n' })
    fs.symlinkSync(path.join(dir, 'real'), path.join(dir, 'link'))
    expect(go(dir).outcome.files).toBe(2)
  })
})
